use crate::components::toast::show_error_toast;
use crate::info_messages::{
    MCP_SESSION_NOT_READY_TOAST, SYNC_CHECKING_LABEL, SYNC_CHECK_TITLE, SYNC_DB_ONLY_HEADING,
    SYNC_DIR_ONLY_HEADING, SYNC_HEADING, SYNC_NONE_LABEL, SYNC_NOT_CHECKED_LABEL,
};
use crate::mcp::tools::get_database_sync_difference;
use crate::models::note::SyncDifference;
use crate::state::AppState;
use icondata as id;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;

/// Request the database vs note-folder difference and store it in `result`.
fn run_sync_check(
    session: RwSignal<Option<String>>,
    result: RwSignal<Option<SyncDifference>>,
    loading: RwSignal<bool>,
    error_toast: RwSignal<Option<String>>,
) {
    if loading.get_untracked() {
        return;
    }
    let Some(sid) = session.get_untracked() else {
        show_error_toast(error_toast, MCP_SESSION_NOT_READY_TOAST);
        return;
    };
    loading.set(true);
    spawn_local(async move {
        match get_database_sync_difference(&sid).await {
            Ok(diff) => result.set(Some(diff)),
            Err(e) => {
                web_sys::console::error_1(&e.clone().into());
                show_error_toast(error_toast, e);
            }
        }
        loading.set(false);
    });
}

/// Render one titled list of paths with a count, or a placeholder when empty.
#[component]
fn SyncPathList(heading: &'static str, paths: Signal<Vec<String>>) -> impl IntoView {
    view! {
        <div class="sync-list mt-3">
            <p class="sync-list-heading">
                <span>{heading}</span>
                <span class="sync-list-count">{move || paths.with(Vec::len)}</span>
            </p>
            <Show
                when=move || paths.with(|p| !p.is_empty())
                fallback=|| view! { <p class="sync-list-empty">{SYNC_NONE_LABEL}</p> }
            >
                <ul>
                    <For
                        each=move || paths.get()
                        key=|path| path.clone()
                        children=move |path: String| {
                            let title = path.clone();
                            view! { <li class="sync-list-item" title=title>{path}</li> }
                        }
                    />
                </ul>
            </Show>
        </div>
    }
}

/// Render the database sync panel: a check button and the db-only and folder-only path lists.
#[component]
pub fn SyncPanel() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");
    let session = state.session_id;
    let error_toast = state.error_toast;
    let result = RwSignal::new(None::<SyncDifference>);
    let loading = RwSignal::new(false);

    let on_check = move |_| run_sync_check(session, result, loading, error_toast);

    let db_only = Signal::derive(move || {
        result.with(|r| r.as_ref().map(|d| d.db_only.clone()).unwrap_or_default())
    });
    let dir_only = Signal::derive(move || {
        result.with(|r| r.as_ref().map(|d| d.dir_only.clone()).unwrap_or_default())
    });

    view! {
        <div class="p-3">
            <div class="file-tree-heading mt-2">
                <p class="menu-label">{SYNC_HEADING}</p>
                <button
                    class=move || if loading.get() { "sync-check-btn is-spinning" } else { "sync-check-btn" }
                    type="button"
                    title=SYNC_CHECK_TITLE
                    prop:disabled=move || loading.get()
                    on:click=on_check
                >
                    <Icon icon=id::LuRefreshCw/>
                </button>
            </div>
            <Show when=move || loading.get()>
                <p class="sync-list-empty mt-2">{SYNC_CHECKING_LABEL}</p>
            </Show>
            <Show
                when=move || result.with(Option::is_some)
                fallback=move || view! {
                    <Show when=move || !loading.get()>
                        <p class="sync-list-empty mt-2">{SYNC_NOT_CHECKED_LABEL}</p>
                    </Show>
                }
            >
                <SyncPathList heading=SYNC_DB_ONLY_HEADING paths=db_only/>
                <SyncPathList heading=SYNC_DIR_ONLY_HEADING paths=dir_only/>
            </Show>
        </div>
    }
}
