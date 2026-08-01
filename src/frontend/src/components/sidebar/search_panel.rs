use crate::mcp::tools::search_by_text;
use crate::models::note::SearchResult;
use crate::state::AppState;
use leptos::prelude::*;
use leptos::task::spawn_local;

fn run_search(
    query: RwSignal<String>,
    results: RwSignal<Vec<SearchResult>>,
    session: RwSignal<Option<String>>,
    limit: RwSignal<usize>,
    warn_no_session: bool,
) {
    let q = query.get_untracked();
    if q.trim().is_empty() {
        return;
    }
    let sid = match session.get_untracked() {
        Some(s) => s,
        None => {
            if warn_no_session {
                web_sys::console::warn_1(&"MCP session not ready".into());
            }
            return;
        }
    };
    let limit = limit.get_untracked();
    spawn_local(async move {
        match search_by_text(&sid, &q, limit).await {
            Ok(found) => results.set(found),
            Err(e) => web_sys::console::error_1(&e.into()),
        }
    });
}

#[component]
pub fn SearchPanel() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");
    let query = state.search_query;
    let results = state.search_results;
    let session = state.session_id;
    let limit = state.number_results_per_page;

    let on_click = move |_| run_search(query, results, session, limit, true);

    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() != "Enter" {
            return;
        }
        run_search(query, results, session, limit, false);
    };

    view! {
        <div class="p-3">
            <p class="menu-label mt-2">"SEARCH"</p>
            <div class="field has-addons">
                <div class="control is-expanded">
                    <input
                        class="input is-small"
                        type="text"
                        placeholder="Search notes..."
                        prop:value=move || query.get()
                        on:input=move |ev| query.set(event_target_value(&ev))
                        on:keydown=on_keydown
                    />
                </div>
                <div class="control">
                    <button class="button is-small is-dark" on:click=on_click>
                        "Go"
                    </button>
                </div>
            </div>

            <div class="mt-3">
                <For
                    each=move || results.get()
                    key=|r: &SearchResult| r.path()
                    children=|result: SearchResult| {
                        let title = result.title();
                        let path  = result.path();
                        view! {
                            <div
                                class="py-1 px-2"
                                style="border-bottom:1px solid var(--border); cursor:pointer; font-size:0.82rem;"
                            >
                                <div style="color:var(--text);">{title}</div>
                                <div style="color:var(--text-muted); font-size:0.75rem;">{path}</div>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}
