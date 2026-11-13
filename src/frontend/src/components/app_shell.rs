use crate::components::{
    activity_bar::ActivityBar, diagram_modal::DiagramModal, editor::Editor, hotkeys::GlobalHotkeys,
    sidebar::Sidebar, toast::Toast, top_bar::TopBar,
};
use crate::info_messages::APP_STATE_MISSING_PANIC;
use crate::state::AppState;
use leptos::prelude::*;

/// Render the top-level layout: hotkeys, top bar, sidebar, editor, toasts, and diagram modal.
#[component]
pub fn AppShell() -> impl IntoView {
    let export_progress = use_context::<AppState>()
        .expect(APP_STATE_MISSING_PANIC)
        .export_progress;

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
            <DiagramModal/>
            <Show when=move || export_progress.get().is_some()>
                <div class="export-overlay" role="status" aria-live="polite" aria-busy="true">
                    <div class="export-spinner" aria-hidden="true"></div>
                    <p class="export-overlay-label">
                        {move || export_progress.get().unwrap_or_default()}
                    </p>
                </div>
            </Show>
        </div>
    }
}
