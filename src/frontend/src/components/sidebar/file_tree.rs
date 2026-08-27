use crate::components::file_io::{
    display_note_path, fetch_dir_contents, normalize_note_path, open_file_at_path,
    path::{
        basename, is_invalid_move, join_path, resolved_rename_basename, rewrite_path_after_move,
        rewrite_path_after_rename,
    },
};
use crate::components::sidebar::context_menu::{FileContextMenu, FolderContextMenu};
use crate::components::sidebar::file_tree_backend::FileTreeBackend;
use crate::components::sidebar::new_folder_modal::{NewFolderModal, NewFolderModalCtrl};
use crate::components::sidebar::rename_modal::{RenameModal, RenameModalCtrl};
use crate::components::toast::{show_error_toast, show_toast};
use crate::constants::DRAG_MIME;
use crate::models::block::EditorEntry;
use crate::models::note::DirectoryContents;
use crate::state::AppState;
use icondata as id;
use leptos::either::Either;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;

#[derive(Clone, Copy)]
struct FileTreeCtx {
    backend: FileTreeBackend,
    epoch: RwSignal<u64>,
    root_label: &'static str,
}

/// Per-tree backend and UI handles. Passed as props; two trees cannot share
/// Leptos context by type without the second overwriting the first.
#[derive(Clone, Copy)]
struct FileTreeScope {
    ctx: FileTreeCtx,
    dnd: FileTreeDnD,
    rename_ctrl: RenameModalCtrl,
    new_folder_ctrl: NewFolderModalCtrl,
}

/// Shared drag-and-drop signals for the file tree.
#[derive(Clone, Copy)]
struct FileTreeDnD {
    drag_src: RwSignal<Option<String>>,
    drop_target: RwSignal<Option<String>>,
}

/// Clear drag-source and drop-target highlights.
fn clear_dnd(dnd: FileTreeDnD) {
    dnd.drag_src.set(None);
    dnd.drop_target.set(None);
}

/// Accept a drag-over event when the move into `target` is valid.
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

/// Move `src` into directory `dst` via the MCP backend.
fn perform_move(state: &AppState, ctx: FileTreeCtx, dnd: FileTreeDnD, src: String, dst: String) {
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

    let epoch = ctx.epoch;
    let current_path = state.current_path;
    let entries = state.entries;
    let toast = state.toast;
    let error_toast = state.error_toast;
    let backend = ctx.backend;
    let root_label = ctx.root_label;

    spawn_local(async move {
        match backend.move_item(&sid, &src, &dst).await {
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
                epoch.update(|n| *n = n.wrapping_add(1));
                let name = basename(&src);
                let dest_label = if dst.is_empty() {
                    root_label
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

/// Record the drag source when a tree item starts dragging.
fn on_drag_start(ev: web_sys::DragEvent, dnd: FileTreeDnD, path: &str) {
    if let Some(dt) = ev.data_transfer() {
        let _ = dt.set_data(DRAG_MIME, path);
        let _ = dt.set_effect_allowed("move");
    }
    dnd.drag_src.set(Some(path.to_string()));
}

/// Clear drag-and-drop state when a drag ends.
fn on_drag_end(dnd: FileTreeDnD) {
    clear_dnd(dnd);
}

/// Drop the dragged item into directory `dst`.
fn on_drop_at(
    ev: web_sys::DragEvent,
    state: &AppState,
    ctx: FileTreeCtx,
    dnd: FileTreeDnD,
    dst: &str,
) {
    ev.prevent_default();
    ev.stop_propagation();
    let src = dnd.drag_src.get_untracked().or_else(|| {
        ev.data_transfer()
            .and_then(|dt| dt.get_data(DRAG_MIME).ok())
            .filter(|s| !s.is_empty())
    });
    let Some(src) = src else {
        clear_dnd(dnd);
        return;
    };
    perform_move(state, ctx, dnd, src, dst.to_string());
}

/// Open the new-folder modal for a folder at the tree root.
fn open_root_new_folder(ctrl: NewFolderModalCtrl) {
    ctrl.open(String::new());
}

/// Create a new folder under `parent_path` via the MCP backend.
fn perform_new_folder(state: &AppState, ctx: FileTreeCtx, parent_path: String, name: String) {
    let name = name.trim().to_string();
    if name.is_empty() {
        return;
    }
    if name.contains('/') || name.contains('\\') {
        show_error_toast(
            state.error_toast,
            "Create folder failed: name cannot contain path separators",
        );
        return;
    }

    let sid = match state.session_id.get_untracked() {
        Some(s) => s,
        None => {
            web_sys::console::warn_1(&"MCP session not ready".into());
            return;
        }
    };

    let path = join_path(&parent_path, &name);
    let epoch = ctx.epoch;
    let toast = state.toast;
    let error_toast = state.error_toast;
    let backend = ctx.backend;

    spawn_local(async move {
        match backend.new_dir(&sid, &path).await {
            Ok(()) => {
                epoch.update(|n| *n = n.wrapping_add(1));
                show_toast(toast, format!("Created folder {path}"));
            }
            Err(e) => {
                web_sys::console::error_1(&format!("Create folder failed for {path}: {e}").into());
                show_error_toast(error_toast, format!("Create folder failed for {path}: {e}"));
            }
        }
    });
}

/// Rename a file or folder via the MCP backend.
fn perform_rename(
    state: &AppState,
    ctx: FileTreeCtx,
    path: String,
    new_name: String,
    is_file: bool,
) {
    let new_name = new_name.trim().to_string();
    if new_name.is_empty() {
        return;
    }
    if new_name.contains('/') || new_name.contains('\\') {
        show_error_toast(
            state.error_toast,
            "Rename failed: name cannot contain path separators",
        );
        return;
    }

    let current_name = basename(&path).to_string();
    let resolved = resolved_rename_basename(&path, &new_name, is_file);
    if resolved == current_name {
        return;
    }

    let sid = match state.session_id.get_untracked() {
        Some(s) => s,
        None => {
            web_sys::console::warn_1(&"MCP session not ready".into());
            return;
        }
    };

    let epoch = ctx.epoch;
    let current_path = state.current_path;
    let entries = state.entries;
    let toast = state.toast;
    let error_toast = state.error_toast;
    let backend = ctx.backend;

    spawn_local(async move {
        match backend.rename(&sid, &path, &new_name).await {
            Ok(()) => {
                if let Some(cur) = current_path.get_untracked() {
                    if let Some(new_path) = rewrite_path_after_rename(&cur, &path, &resolved) {
                        current_path.set(Some(new_path));
                    }
                }
                entries.with_untracked(|list: &Vec<EditorEntry>| {
                    for entry in list {
                        let norm = normalize_note_path(&entry.title.path.get_untracked());
                        if let Some(new_path) = rewrite_path_after_rename(&norm, &path, &resolved) {
                            entry.title.path.set(display_note_path(&new_path));
                        }
                    }
                });
                epoch.update(|n| *n = n.wrapping_add(1));
                show_toast(toast, format!("Renamed to {resolved}"));
            }
            Err(e) => {
                web_sys::console::error_1(
                    &format!("Rename failed for {path} → {new_name}: {e}").into(),
                );
                show_error_toast(error_toast, format!("Rename failed for {path}: {e}"));
            }
        }
    });
}

/// Render a file tree listing top-level entries from an MCP backend.
#[component]
pub fn FileTree(
    backend: FileTreeBackend,
    heading: &'static str,
    root_label: &'static str,
    epoch: RwSignal<u64>,
) -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");
    let session = state.session_id;
    let dir_contents = RwSignal::new(None::<DirectoryContents>);
    let dnd = FileTreeDnD {
        drag_src: RwSignal::new(None::<String>),
        drop_target: RwSignal::new(None::<String>),
    };
    let rename_ctrl = RenameModalCtrl::new();
    let new_folder_ctrl = NewFolderModalCtrl::new();
    let ctx = FileTreeCtx {
        backend,
        epoch,
        root_label,
    };
    let scope = FileTreeScope {
        ctx,
        dnd,
        rename_ctrl,
        new_folder_ctrl,
    };

    Effect::new(move |_| {
        let _ = epoch.get();
        fetch_dir_contents(session, "", dir_contents, || true, backend);
    });

    let root_dragover = move |ev: web_sys::DragEvent| {
        accept_drag_over(&ev, dnd, "");
    };
    let root_drop = {
        let state = state.clone();
        move |ev: web_sys::DragEvent| {
            on_drop_at(ev, &state, ctx, dnd, "");
        }
    };
    let root_class = move || {
        if dnd.drop_target.get().as_deref() == Some("") {
            "p-2 file-tree-root drag-over"
        } else {
            "p-2 file-tree-root"
        }
    };

    let on_rename_confirm = {
        let state = state.clone();
        move |path: String, new_name: String, is_file: bool| {
            perform_rename(&state, ctx, path, new_name, is_file);
        }
    };
    let on_new_folder_confirm = {
        let state = state.clone();
        move |parent_path: String, name: String| {
            perform_new_folder(&state, ctx, parent_path, name);
        }
    };
    let on_new_root_folder = move |ev: web_sys::MouseEvent| {
        ev.prevent_default();
        ev.stop_propagation();
        open_root_new_folder(new_folder_ctrl);
    };

    view! {
        <div
            class=root_class
            on:dragover=root_dragover
            on:drop=root_drop
        >
            <div class="file-tree-heading px-2 mt-2">
                <p class="menu-label">{heading}</p>
                <button
                    class="file-tree-new-folder"
                    type="button"
                    title="New folder"
                    on:click=on_new_root_folder
                >
                    <Icon icon=id::LuFolderPlus/>
                </button>
            </div>
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
                            children=move |name: String| {
                                view! { <TreeFolder name=name.clone() path=name scope=scope/> }
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
                            children=move |name: String| {
                                view! { <TreeFile name=name.clone() path=name scope=scope/> }
                            }
                        />
                    </Show>
                </ul>
            </aside>
            <RenameModal ctrl=rename_ctrl on_confirm=on_rename_confirm />
            <NewFolderModal ctrl=new_folder_ctrl on_confirm=on_new_folder_confirm />
        </div>
    }
}

/// Render an expandable folder row and its children.
#[component]
fn TreeFolder(name: String, path: String, scope: FileTreeScope) -> AnyView {
    let state = use_context::<AppState>().expect("AppState not provided");
    let ctx = scope.ctx;
    let dnd = scope.dnd;
    let rename_ctrl = scope.rename_ctrl;
    let new_folder_ctrl = scope.new_folder_ctrl;
    let session = state.session_id;
    let epoch = ctx.epoch;
    let toast = state.toast;
    let error_toast = state.error_toast;
    let open = RwSignal::new(false);
    let dir_contents = RwSignal::new(None::<DirectoryContents>);
    let menu_visible = RwSignal::new(false);
    let menu_x = RwSignal::new(0.0);
    let menu_y = RwSignal::new(0.0);
    let path_for_fetch = path.clone();
    let path_for_dnd = path.clone();
    let backend = ctx.backend;

    // Fetch while open; re-fetch when epoch bumps after a successful upsert/delete.
    Effect::new(move |_| {
        if !open.get() {
            dir_contents.set(None);
            return;
        }
        let _ = epoch.get();
        fetch_dir_contents(
            session,
            path_for_fetch.clone(),
            dir_contents,
            move || open.get_untracked(),
            backend,
        );
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
                match backend.delete_folder(&sid, &path).await {
                    Ok(()) => {
                        epoch.update(|n| *n = n.wrapping_add(1));
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

    let new_folder = {
        let path = path.clone();
        move || {
            open.set(true);
            new_folder_ctrl.open(path.clone());
        }
    };

    let rename_folder = {
        let path = path.clone();
        move || rename_ctrl.open(path.clone(), false)
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
        move |ev: web_sys::DragEvent| on_drop_at(ev, &state, ctx, dnd, &path)
    };
    let on_children_dragover = {
        let path = path_for_dnd.clone();
        move |ev: web_sys::DragEvent| accept_drag_over(&ev, dnd, &path)
    };
    let on_children_drop = {
        let path = path_for_dnd.clone();
        let state = state.clone();
        move |ev: web_sys::DragEvent| on_drop_at(ev, &state, ctx, dnd, &path)
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
                on_new_folder=new_folder
                on_rename=rename_folder
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
                                view! { <TreeFolder name=child_name path=child_path scope=scope/> }
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
                                view! { <TreeFile name=file_name.clone() path=file_path scope=scope/> }
                            }
                        }
                    />
                </Show>
            </ul>
        </li>
    }
    .into_any()
}

/// Render a file row that opens, renames, deletes, or drags the note.
#[component]
fn TreeFile(name: String, path: String, scope: FileTreeScope) -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");
    let ctx = scope.ctx;
    let dnd = scope.dnd;
    let rename_ctrl = scope.rename_ctrl;
    let current_path = state.current_path;
    let entries = state.entries;
    let session = state.session_id;
    let epoch = ctx.epoch;
    let toast = state.toast;
    let error_toast = state.error_toast;
    let path_for_active = path.clone();
    let path_for_dnd = path.clone();
    let menu_visible = RwSignal::new(false);
    let menu_x = RwSignal::new(0.0);
    let menu_y = RwSignal::new(0.0);
    let backend = ctx.backend;

    let is_active = move || current_path.get().as_deref() == Some(path_for_active.as_str());

    let open_file = {
        let path = path.clone();
        move || open_file_at_path(path.clone(), current_path, entries, session, backend)
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
                match backend.delete_file(&sid, &path).await {
                    Ok(()) => {
                        epoch.update(|n| *n = n.wrapping_add(1));
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

    let rename_file = {
        let path = path.clone();
        move || rename_ctrl.open(path.clone(), true)
    };

    let on_click = {
        let open_file = open_file.clone();
        move |_| open_file()
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
                on_open=open_file
                on_rename=rename_file
                on_delete=delete_file
            />
        </li>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptos::prelude::{GetUntracked, Owner};

    #[test]
    /// Assert the header new-folder action opens the modal at the tree root.
    fn header_new_folder_opens_modal_at_root() {
        let owner = Owner::new();
        owner.with(|| {
            let ctrl = NewFolderModalCtrl::new();
            open_root_new_folder(ctrl);
            assert_eq!(ctrl.parent_path.get_untracked().as_deref(), Some(""));
        });
    }
}
