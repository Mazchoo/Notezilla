use crate::state::AppState;
use leptos::prelude::*;

#[component]
pub fn SettingsPanel() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");
    let number_results_per_page = state.number_results_per_page;

    let on_input = move |ev| {
        let raw = event_target_value(&ev);
        let Ok(n) = raw.parse::<usize>() else {
            return;
        };
        if n == 0 {
            return;
        }
        number_results_per_page.set(n);
    };

    view! {
        <div class="p-3">
            <p class="menu-label mt-2">"SETTINGS"</p>
            <div class="field">
                <label class="label is-small" style="color:var(--text-muted);">
                    "Results per page"
                </label>
                <div class="control">
                    <input
                        class="input is-small"
                        type="number"
                        min="1"
                        prop:value=move || number_results_per_page.get().to_string()
                        on:input=on_input
                    />
                </div>
            </div>
        </div>
    }
}
