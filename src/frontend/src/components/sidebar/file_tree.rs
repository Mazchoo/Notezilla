use crate::components::file_io::open_note_at_path;
use crate::components::sidebar::context_menu::FileContextMenu;
use crate::mcp::tools::get_dir_contents;
use crate::models::note::DirectoryContents;
use crate::state::AppState;
use icondata as id;
use leptos::either::Either;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;

fn join_path(dir: &str, name: &str) -> String {
    if dir.is_empty() {
        name.to_string()
    } else {
        format!("{dir}/{name}")
    }
}

/// File tree listing top-level note-folder entries from the MCP backend.
#[component]
pub fn FileTree() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");
    let session = state.session_id;
    let dir_contents = RwSignal::new(None::<DirectoryContents>);

    Effect::new(move |_| {
        let sid = match session.get() {
            Some(s) => s,
            None => return,
        };

        spawn_local(async move {
            match get_dir_contents(&sid, "").await {
                Ok(contents) => dir_contents.set(Some(contents)),
                Err(e) => web_sys::console::error_1(&e.into()),
            }
        });
    });

    view! {
        <div class="p-2">
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
    let session = state.session_id;
    let open = RwSignal::new(false);
    let dir_contents = RwSignal::new(None::<DirectoryContents>);
    let path_for_fetch = path.clone();

    let toggle = move |_| {
        let will_open = !open.get_untracked();
        open.set(will_open);

        if will_open {
            let sid = match session.get_untracked() {
                Some(s) => s,
                None => return,
            };
            let fetch_path = path_for_fetch.clone();
            spawn_local(async move {
                match get_dir_contents(&sid, &fetch_path).await {
                    Ok(contents) => {
                        if open.get_untracked() {
                            dir_contents.set(Some(contents));
                        }
                    }
                    Err(e) => web_sys::console::error_1(&e.into()),
                }
            });
        } else {
            dir_contents.set(None);
        }
    };

    view! {
        <li>
            <a on:click=toggle>
                {move || if open.get() {
                    Either::Left(view! { <Icon icon=id::LuFolderOpen/> })
                } else {
                    Either::Right(view! { <Icon icon=id::LuFolder/> })
                }}
                {name.clone()}
            </a>
            <ul class=move || if open.get() { "" } else { "is-hidden" }>
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
    let current_path = state.current_path;
    let entries = state.entries;
    let session = state.session_id;
    let path_for_active = path.clone();
    let menu_visible = RwSignal::new(false);
    let menu_x = RwSignal::new(0.0);
    let menu_y = RwSignal::new(0.0);

    let is_active = move || current_path.get().as_deref() == Some(path_for_active.as_str());

    let open_note = {
        let path = path.clone();
        move || open_note_at_path(path.clone(), current_path, entries, session)
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

    view! {
        <li>
            <a
                class=move || if is_active() { "is-active" } else { "" }
                on:click=on_click
                on:contextmenu=on_contextmenu
            >
                <Icon icon=id::LuFileText/>
                {name}
            </a>
            <FileContextMenu
                visible=menu_visible
                x=menu_x
                y=menu_y
                on_open=open_note
            />
        </li>
    }
}
