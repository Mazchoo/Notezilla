use crate::state::AppState;
use icondata as id;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_icons::Icon;

/// Dummy file tree — static placeholder until backend file listing is wired up.
#[component]
pub fn FileTree() -> impl IntoView {
    view! {
        <div class="p-2">
            <p class="menu-label px-2 mt-2">"FILES"</p>
            <aside class="menu px-1">
                <ul class="menu-list">
                    <TreeFolder name="2024">
                        <TreeFolder name="Projects">
                            <TreeFile name="notezilla.md" path="2024/Projects/notezilla.md"/>
                            <TreeFile name="roadmap.md" path="2024/Projects/roadmap.md"/>
                        </TreeFolder>
                        <TreeFile name="ideas.md" path="2024/ideas.md"/>
                        <TreeFile name="getting-started.md" path="2024/getting-started.md"/>
                    </TreeFolder>
                    <TreeFolder name="Archive">
                        <TreeFile name="old-notes.md" path="Archive/old-notes.md"/>
                    </TreeFolder>
                    <TreeFile name="readme.md" path="readme.md"/>
                </ul>
            </aside>
        </div>
    }
}

#[component]
fn TreeFolder(name: &'static str, children: Children) -> impl IntoView {
    let open = RwSignal::new(true);

    view! {
        <li>
            <a on:click=move |_| open.update(|o| *o = !*o)>
                // Show open/closed folder icon based on state
                {move || if open.get() {
                    Either::Left(view! { <Icon icon=id::LuFolderOpen/> })
                } else {
                    Either::Right(view! { <Icon icon=id::LuFolder/> })
                }}
                {name}
            </a>
            <ul class=move || if open.get() { "" } else { "is-hidden" }>
                {children()}
            </ul>
        </li>
    }
}

#[component]
fn TreeFile(name: &'static str, path: &'static str) -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");
    let current_path = state.current_path;

    let is_active = move || current_path.get().as_deref() == Some(path);

    view! {
        <li>
            <a
                class=move || if is_active() { "is-active" } else { "" }
                on:click=move |_| current_path.set(Some(path.to_string()))
            >
                <Icon icon=id::LuFileText/>
                {name}
            </a>
        </li>
    }
}
