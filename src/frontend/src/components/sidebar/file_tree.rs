use crate::components::file_io::{display_note_path, entry_from_note, open_note_in_editor};
use crate::mcp::tools::{get_dir_contents, get_note};
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

    let is_active = move || current_path.get().as_deref() == Some(path_for_active.as_str());

    let on_click = move |_| {
        current_path.set(Some(path.clone()));

        let sid = match session.get_untracked() {
            Some(s) => s,
            None => {
                web_sys::console::warn_1(&"MCP session not ready".into());
                return;
            }
        };

        let fetch_path = path.clone();
        spawn_local(async move {
            match get_note(&sid, &fetch_path).await {
                Ok(note) => {
                    let display_path = display_note_path(&fetch_path);
                    let mut metadata = std::collections::HashMap::new();
                    metadata.insert(
                        "filename".to_string(),
                        serde_json::json!(note.filename),
                    );
                    let entry = entry_from_note(display_path, &note.text, &metadata);
                    open_note_in_editor(entries, entry);
                }
                Err(e) => web_sys::console::error_1(&e.into()),
            }
        });
    };

    view! {
        <li>
            <a
                class=move || if is_active() { "is-active" } else { "" }
                on:click=on_click
            >
                <Icon icon=id::LuFileText/>
                {name}
            </a>
        </li>
    }
}
