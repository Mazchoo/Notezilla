use crate::components::file_io::{
    display_note_path, fetch_dir_contents, normalize_note_path, open_note_at_path,
    path::{basename, is_invalid_move, join_path, rewrite_path_after_move},
};
use crate::components::sidebar::context_menu::{FileContextMenu, FolderContextMenu};
use crate::components::toast::{show_error_toast, show_toast};
use crate::mcp::tools::{delete_folder, delete_note, move_dir};
use crate::models::block::EditorEntry;
use crate::models::note::DirectoryContents;
use crate::state::AppState;
use icondata as id;
use leptos::either::Either;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;

const DRAG_MIME: &str = "text/plain";

/// Shared drag-and-drop signals for the file tree.
#[derive(Clone, Copy)]
struct FileTreeDnD {
    drag_src: RwSignal<Option<String>>,
    drop_target: RwSignal<Option<String>>,
}

fn clear_dnd(dnd: FileTreeDnD) {
    dnd.drag_src.set(None);
    dnd.drop_target.set(None);
}

fn accept_drag_over(ev: &web_sys::DragEvent, dnd: FileTreeDnD, target: &str) {
    let Some(src) = dnd.drag_src.get_untracked() else {
        return;
    };
    if is_invalid_move(&src, target) {
        return;
    }
    ev.prevent_default();
    ev.stop_propagation();
    if let Some(dt) = ev.data_transfer() {
        let _ = dt.set_drop_effect("move");
    }
    dnd.drop_target.set(Some(target.to_string()));
}

fn perform_move(state: &AppState, dnd: FileTreeDnD, src: String, dst: String) {
    clear_dnd(dnd);

    if is_invalid_move(&src, &dst) {
        return;
    }

    let sid = match state.session_id.get_untracked() {
        Some(s) => s,
        None => {
            web_sys::console::warn_1(&"MCP session not ready".into());
            return;
        }
    };

    let file_tree_epoch = state.file_tree_epoch;
    let current_path = state.current_path;
    let entries = state.entries;
    let toast = state.toast;
    let error_toast = state.error_toast;

    spawn_local(async move {
        match move_dir(&sid, &src, &dst).await {
            Ok(()) => {
                if let Some(cur) = current_path.get_untracked() {
                    if let Some(new_path) = rewrite_path_after_move(&cur, &src, &dst) {
                        current_path.set(Some(new_path));
                    }
                }
                entries.with_untracked(|list: &Vec<EditorEntry>| {
                    for entry in list {
                        let norm = normalize_note_path(&entry.title.path.get_untracked());
                        if let Some(new_path) = rewrite_path_after_move(&norm, &src, &dst) {
                            entry.title.path.set(display_note_path(&new_path));
                        }
                    }
                });
                file_tree_epoch.update(|n| *n = n.wrapping_add(1));
                let name = basename(&src);
                let dest_label = if dst.is_empty() {
                    "note folder root"
                } else {
                    dst.as_str()
                };
                show_toast(toast, format!("Moved {name} to {dest_label}"));
            }
            Err(e) => {
                web_sys::console::error_1(&format!("Move failed for {src} → {dst}: {e}").into());
                show_error_toast(error_toast, format!("Move failed for {src}: {e}"));
            }
        }
    });
}

fn on_drag_start(ev: web_sys::DragEvent, dnd: FileTreeDnD, path: &str) {
    if let Some(dt) = ev.data_transfer() {
        let _ = dt.set_data(DRAG_MIME, path);
        let _ = dt.set_effect_allowed("move");
    }
    dnd.drag_src.set(Some(path.to_string()));
}

fn on_drag_end(dnd: FileTreeDnD) {
    clear_dnd(dnd);
}

fn on_drop_at(ev: web_sys::DragEvent, state: &AppState, dnd: FileTreeDnD, dst: &str) {
    ev.prevent_default();
    ev.stop_propagation();
    let src = dnd
        .drag_src
        .get_untracked()
        .or_else(|| {
            ev.data_transfer()
                .and_then(|dt| dt.get_data(DRAG_MIME).ok())
                .filter(|s| !s.is_empty())
        });
    let Some(src) = src else {
        clear_dnd(dnd);
        return;
    };
    perform_move(state, dnd, src, dst.to_string());
}

/// File tree listing top-level note-folder entries from the MCP backend.
#[component]
pub fn FileTree() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");
    let session = state.session_id;
    let file_tree_epoch = state.file_tree_epoch;
    let dir_contents = RwSignal::new(None::<DirectoryContents>);
    let dnd = FileTreeDnD {
        drag_src: RwSignal::new(None::<String>),
        drop_target: RwSignal::new(None::<String>),
    };
    provide_context(dnd);

    Effect::new(move |_| {
        let _ = file_tree_epoch.get();
        fetch_dir_contents(session, "", dir_contents, || true);
    });

    let root_dragover = move |ev: web_sys::DragEvent| {
        accept_drag_over(&ev, dnd, "");
    };
    let root_drop = {
        let state = state.clone();
        move |ev: web_sys::DragEvent| {
            on_drop_at(ev, &state, dnd, "");
        }
    };
    let root_class = move || {
        if dnd.drop_target.get().as_deref() == Some("") {
            "p-2 file-tree-root drag-over"
        } else {
            "p-2 file-tree-root"
        }
    };

    view! {
        <div
            class=root_class
            on:dragover=root_dragover
            on:drop=root_drop
        >
            <p class="menu-label px-2 mt-2">"FILES"</p>
            <aside class="menu px-1">
                <ul class="menu-list">
                    <Show when=move || dir_contents.get().is_some()>
                        <For
                            each=move || {
                                dir_contents
                                    .get()
                                    .map(|c| c.folders)
                                    .unwrap_or_default()
                            }
                            key=|name| name.clone()
                            children=|name: String| {
                                view! { <TreeFolder name=name.clone() path=name/> }
                            }
                        />
                        <For
                            each=move || {
                                dir_contents
                                    .get()
                                    .map(|c| c.files)
                                    .unwrap_or_default()
                            }
                            key=|name| name.clone()
                            children=|name: String| {
                                view! { <TreeFile name=name.clone() path=name/> }
                            }
                        />
                    </Show>
                </ul>
            </aside>
        </div>
    }
}

#[component]
fn TreeFolder(name: String, path: String) -> AnyView {
    let state = use_context::<AppState>().expect("AppState not provided");
    let dnd = use_context::<FileTreeDnD>().expect("FileTreeDnD not provided");
    let session = state.session_id;
    let file_tree_epoch = state.file_tree_epoch;
    let toast = state.toast;
    let error_toast = state.error_toast;
    let open = RwSignal::new(false);
    let dir_contents = RwSignal::new(None::<DirectoryContents>);
    let menu_visible = RwSignal::new(false);
    let menu_x = RwSignal::new(0.0);
    let menu_y = RwSignal::new(0.0);
    let path_for_fetch = path.clone();
    let path_for_dnd = path.clone();

    // Fetch while open; re-fetch when file_tree_epoch bumps after a successful upsert/delete.
    Effect::new(move |_| {
        if !open.get() {
            dir_contents.set(None);
            return;
        }
        let _ = file_tree_epoch.get();
        fetch_dir_contents(session, path_for_fetch.clone(), dir_contents, move || {
            open.get_untracked()
        });
    });

    let toggle = move |_| {
        open.update(|is_open| *is_open = !*is_open);
    };

    let delete_dir = {
        let path = path.clone();
        move || {
            let sid = match session.get_untracked() {
                Some(s) => s,
                None => {
                    web_sys::console::warn_1(&"MCP session not ready".into());
                    return;
                }
            };
            let path = path.clone();
            spawn_local(async move {
                match delete_folder(&sid, &path).await {
                    Ok(()) => {
                        file_tree_epoch.update(|n| *n = n.wrapping_add(1));
                        show_toast(toast, format!("Deleted folder {path}"));
                    }
                    Err(e) => {
                        web_sys::console::error_1(
                            &format!("Delete folder failed for {path}: {e}").into(),
                        );
                        show_error_toast(
                            error_toast,
                            format!("Delete folder failed for {path}: {e}"),
                        );
                    }
                }
            });
        }
    };

    let on_contextmenu = move |ev: web_sys::MouseEvent| {
        ev.prevent_default();
        menu_x.set(ev.client_x() as f64);
        menu_y.set(ev.client_y() as f64);
        menu_visible.set(true);
    };

    let on_dragstart = {
        let path = path_for_dnd.clone();
        move |ev: web_sys::DragEvent| on_drag_start(ev, dnd, &path)
    };
    let on_dragend = move |_ev: web_sys::DragEvent| on_drag_end(dnd);
    let on_row_dragover = {
        let path = path_for_dnd.clone();
        move |ev: web_sys::DragEvent| accept_drag_over(&ev, dnd, &path)
    };
    let on_row_drop = {
        let path = path_for_dnd.clone();
        let state = state.clone();
        move |ev: web_sys::DragEvent| on_drop_at(ev, &state, dnd, &path)
    };
    let on_children_dragover = {
        let path = path_for_dnd.clone();
        move |ev: web_sys::DragEvent| accept_drag_over(&ev, dnd, &path)
    };
    let on_children_drop = {
        let path = path_for_dnd.clone();
        let state = state.clone();
        move |ev: web_sys::DragEvent| on_drop_at(ev, &state, dnd, &path)
    };

    let path_for_class = path.clone();
    let row_class = move || {
        let mut classes = String::new();
        if dnd.drag_src.get().as_deref() == Some(path_for_class.as_str()) {
            classes.push_str("dragging");
        }
        if dnd.drop_target.get().as_deref() == Some(path_for_class.as_str()) {
            if !classes.is_empty() {
                classes.push(' ');
            }
            classes.push_str("drag-over");
        }
        classes
    };

    view! {
        <li>
            <a
                class=row_class
                draggable="true"
                on:click=toggle
                on:contextmenu=on_contextmenu
                on:dragstart=on_dragstart
                on:dragend=on_dragend
                on:dragover=on_row_dragover
                on:drop=on_row_drop
            >
                {move || if open.get() {
                    Either::Left(view! { <Icon icon=id::LuFolderOpen/> })
                } else {
                    Either::Right(view! { <Icon icon=id::LuFolder/> })
                }}
                {name.clone()}
            </a>
            <FolderContextMenu
                visible=menu_visible
                x=menu_x
                y=menu_y
                on_delete=delete_dir
            />
            <ul
                class=move || if open.get() { "" } else { "is-hidden" }
                on:dragover=on_children_dragover
                on:drop=on_children_drop
            >
                <Show when=move || dir_contents.get().is_some()>
                    <For
                        each=move || {
                            dir_contents
                                .get()
                                .map(|c| c.folders)
                                .unwrap_or_default()
                        }
                        key=|name| name.clone()
                        children={
                            let folder_path = path.clone();
                            move |child_name: String| {
                                let child_path = join_path(&folder_path, &child_name);
                                view! { <TreeFolder name=child_name path=child_path/> }
                            }
                        }
                    />
                    <For
                        each=move || {
                            dir_contents
                                .get()
                                .map(|c| c.files)
                                .unwrap_or_default()
                        }
                        key=|name| name.clone()
                        children={
                            let folder_path = path.clone();
                            move |file_name: String| {
                                let file_path = join_path(&folder_path, &file_name);
                                view! { <TreeFile name=file_name.clone() path=file_path/> }
                            }
                        }
                    />
                </Show>
            </ul>
        </li>
    }
    .into_any()
}

#[component]
fn TreeFile(name: String, path: String) -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");
    let dnd = use_context::<FileTreeDnD>().expect("FileTreeDnD not provided");
    let current_path = state.current_path;
    let entries = state.entries;
    let session = state.session_id;
    let file_tree_epoch = state.file_tree_epoch;
    let toast = state.toast;
    let error_toast = state.error_toast;
    let path_for_active = path.clone();
    let path_for_dnd = path.clone();
    let menu_visible = RwSignal::new(false);
    let menu_x = RwSignal::new(0.0);
    let menu_y = RwSignal::new(0.0);

    let is_active = move || current_path.get().as_deref() == Some(path_for_active.as_str());

    let open_note = {
        let path = path.clone();
        move || open_note_at_path(path.clone(), current_path, entries, session)
    };

    let delete_file = {
        let path = path.clone();
        move || {
            let sid = match session.get_untracked() {
                Some(s) => s,
                None => {
                    web_sys::console::warn_1(&"MCP session not ready".into());
                    return;
                }
            };
            let path = path.clone();
            spawn_local(async move {
                match delete_note(&sid, &path).await {
                    Ok(()) => {
                        file_tree_epoch.update(|n| *n = n.wrapping_add(1));
                        show_toast(toast, format!("Deleted {path}"));
                    }
                    Err(e) => {
                        web_sys::console::error_1(&format!("Delete failed for {path}: {e}").into());
                        show_error_toast(error_toast, format!("Delete failed for {path}: {e}"));
                    }
                }
            });
        }
    };

    let on_click = {
        let open_note = open_note.clone();
        move |_| open_note()
    };

    let on_contextmenu = move |ev: web_sys::MouseEvent| {
        ev.prevent_default();
        menu_x.set(ev.client_x() as f64);
        menu_y.set(ev.client_y() as f64);
        menu_visible.set(true);
    };

    let on_dragstart = {
        let path = path_for_dnd.clone();
        move |ev: web_sys::DragEvent| on_drag_start(ev, dnd, &path)
    };
    let on_dragend = move |_ev: web_sys::DragEvent| on_drag_end(dnd);

    let path_for_class = path.clone();
    let row_class = move || {
        let mut classes = if is_active() {
            String::from("is-active")
        } else {
            String::new()
        };
        if dnd.drag_src.get().as_deref() == Some(path_for_class.as_str()) {
            if !classes.is_empty() {
                classes.push(' ');
            }
            classes.push_str("dragging");
        }
        classes
    };

    view! {
        <li>
            <a
                class=row_class
                draggable="true"
                on:click=on_click
                on:contextmenu=on_contextmenu
                on:dragstart=on_dragstart
                on:dragend=on_dragend
            >
                <Icon icon=id::LuFileText/>
                {name}
            </a>
            <FileContextMenu
                visible=menu_visible
                x=menu_x
                y=menu_y
                on_open=open_note
                on_delete=delete_file
            />
        </li>
    }
}
