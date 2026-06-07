use leptos::*;
use crate::components::{
    activity_bar::ActivityBar,
    editor::Editor,
    sidebar::Sidebar,
    top_bar::TopBar,
};

#[component]
pub fn AppShell() -> impl IntoView {
    view! {
        <div id="app">
            <TopBar/>
            <div class="body-row">
                <ActivityBar/>
                <Sidebar/>
                <Editor/>
            </div>
        </div>
    }
}
