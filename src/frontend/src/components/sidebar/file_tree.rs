use crate::mcp::tools::get_dir_contents;
use crate::models::note::DirectoryContents;
use crate::state::AppState;
use icondata as id;
use leptos::either::Either;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;

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
                Ok(contents) if contents.error.is_none() => dir_contents.set(Some(contents)),
                Ok(contents) => {
                    web_sys::console::warn_1(
                        &format!("get_dir_contents: {:?}", contents.error).into(),
                    );
                }
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
                            children=|name: String| view! { <TreeFolder name=name/> }
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
fn TreeFolder(
    name: String,
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    let open = RwSignal::new(false);

    view! {
        <li>
            <a on:click=move |_| open.update(|o| *o = !*o)>
                {move || if open.get() {
                    Either::Left(view! { <Icon icon=id::LuFolderOpen/> })
                } else {
                    Either::Right(view! { <Icon icon=id::LuFolder/> })
                }}
                {name.clone()}
            </a>
            <ul class=move || if open.get() { "" } else { "is-hidden" }>
                {children.map(|c| c()).unwrap_or_else(|| view! {}.into_any())}
            </ul>
        </li>
    }
}

#[component]
fn TreeFile(name: String, path: String) -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");
    let current_path = state.current_path;
    let path_for_active = path.clone();

    let is_active = move || current_path.get().as_deref() == Some(path_for_active.as_str());

    view! {
        <li>
            <a
                class=move || if is_active() { "is-active" } else { "" }
                on:click=move |_| current_path.set(Some(path.clone()))
            >
                <Icon icon=id::LuFileText/>
                {name}
            </a>
        </li>
    }
}
