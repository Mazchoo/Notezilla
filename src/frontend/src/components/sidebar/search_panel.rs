use crate::components::file_io::open_note_at_path;
use crate::components::toast::show_error_toast;
use crate::mcp::tools::search_by_text;
use crate::models::note::NoteFile;
use crate::settings::DEFAULT_SEARCH_PREVIEW_CHARS;
use crate::state::AppState;
use leptos::prelude::*;
use leptos::task::spawn_local;

fn run_search(
    query: RwSignal<String>,
    results: RwSignal<Vec<NoteFile>>,
    session: RwSignal<Option<String>>,
    limit: RwSignal<usize>,
    offset: RwSignal<usize>,
    error_toast: RwSignal<Option<String>>,
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
    let offset = offset.get_untracked();
    spawn_local(async move {
        match search_by_text(&sid, &q, limit, offset).await {
            Ok(found) => results.set(found),
            Err(e) => {
                web_sys::console::error_1(&e.clone().into());
                show_error_toast(error_toast, e);
            }
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
    let current_path = state.current_path;
    let entries = state.entries;
    let error_toast = state.error_toast;
    let offset = RwSignal::new(0usize);

    let on_search = move |_| {
        offset.set(0);
        run_search(query, results, session, limit, offset, error_toast, true);
    };

    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() != "Enter" {
            return;
        }
        offset.set(0);
        run_search(query, results, session, limit, offset, error_toast, false);
    };

    let on_prev = move |_| {
        let page = limit.get_untracked();
        offset.update(|o| *o = o.saturating_sub(page));
        run_search(query, results, session, limit, offset, error_toast, true);
    };

    let on_next = move |_| {
        let page = limit.get_untracked();
        offset.update(|o| *o = o.saturating_add(page));
        run_search(query, results, session, limit, offset, error_toast, true);
    };

    let can_prev = move || offset.get() > 0;
    let can_next = move || results.get().len() == limit.get();

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
                    <button class="button is-small is-dark" on:click=on_search>
                        "Go"
                    </button>
                </div>
            </div>

            <div class="mt-3">
                <For
                    each=move || results.get()
                    key=|r: &NoteFile| r.filename.clone()
                    children=move |result: NoteFile| {
                        let path = result.filename.clone();
                        let file_name = result.file_name();
                        let text = result.text.clone();
                        let path_click = path.clone();
                        view! {
                            <div
                                class="py-1 px-2"
                                style="border-bottom:1px solid var(--border); cursor:pointer; font-size:0.82rem;"
                                on:click=move |_| {
                                    open_note_at_path(
                                        path_click.clone(),
                                        current_path,
                                        entries,
                                        session,
                                    );
                                }
                            >
                                <div style="color:var(--text);">{file_name}</div>
                                <div style="color:var(--text-muted); font-size:0.75rem; white-space:nowrap; overflow:hidden; text-overflow:ellipsis;">
                                    {NoteFile::snippet_text(&text, DEFAULT_SEARCH_PREVIEW_CHARS)}
                                </div>
                            </div>
                        }
                    }
                />
            </div>

            <Show when=move || can_prev() || can_next()>
                <div class="field has-addons mt-3">
                    <div class="control">
                        <button
                            class="button is-small"
                            prop:disabled=move || !can_prev()
                            on:click=on_prev
                        >
                            "Prev"
                        </button>
                    </div>
                    <div class="control">
                        <button
                            class="button is-small"
                            prop:disabled=move || !can_next()
                            on:click=on_next
                        >
                            "Next"
                        </button>
                    </div>
                </div>
            </Show>
        </div>
    }
}
