use crate::components::{
    activity_bar::ActivityBar, editor::Editor, hotkeys::GlobalHotkeys, sidebar::Sidebar,
    toast::Toast, top_bar::TopBar,
};
use leptos::prelude::*;

#[component]
pub fn AppShell() -> impl IntoView {
    view! {
        <div id="app">
            <GlobalHotkeys/>
            <TopBar/>
            <div class="body-row">
                <ActivityBar/>
                <Sidebar/>
                <Editor/>
            </div>
            <Toast/>
        </div>
    }
}
