use leptos::*;
use leptos_icons::Icon;
use icondata as id;

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
                            <TreeFile name="notezilla.md"/>
                            <TreeFile name="roadmap.md"/>
                        </TreeFolder>
                        <TreeFile name="ideas.md"/>
                        <TreeFile name="getting-started.md"/>
                    </TreeFolder>
                    <TreeFolder name="Archive">
                        <TreeFile name="old-notes.md"/>
                    </TreeFolder>
                    <TreeFile name="readme.md"/>
                </ul>
            </aside>
        </div>
    }
}

#[component]
fn TreeFolder(name: &'static str, children: Children) -> impl IntoView {
    let open = create_rw_signal(true);

    view! {
        <li>
            <a on:click=move |_| open.update(|o| *o = !*o)>
                // Show open/closed folder icon based on state
                {move || if open.get() {
                    view! { <Icon icon=id::LuFolderOpen/> }.into_view()
                } else {
                    view! { <Icon icon=id::LuFolder/> }.into_view()
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
fn TreeFile(name: &'static str) -> impl IntoView {
    view! {
        <li>
            <a>
                <Icon icon=id::LuFileText/>
                {name}
            </a>
        </li>
    }
}
