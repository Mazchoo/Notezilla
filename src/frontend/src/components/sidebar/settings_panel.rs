use super::settings_validation::{
    apply_if_valid, commit_setting, on_hotkey_keydown, parse_bounded_f64, parse_gemini_api_key,
    parse_gemini_model, parse_ollama_model, parse_ollama_num_predict, parse_ollama_port,
    parse_positive_u32, parse_results_per_page,
};
use crate::components::hotkeys::format_ctrl_hotkey;
use crate::constants::{
    OLLAMA_TEMPERATURE_MAX, OLLAMA_TEMPERATURE_MIN, OLLAMA_TOP_P_MAX, OLLAMA_TOP_P_MIN,
};
use crate::info_messages::{
    with_hotkey, DISPLAY_SETTINGS_HEADING, EXPORT_HOTKEY_LABEL, GEMINI_API_KEY_CONSTRAINT,
    GEMINI_API_KEY_LABEL, GEMINI_API_KEY_PLACEHOLDER, GEMINI_API_KEY_TITLE, GEMINI_MODEL_CONSTRAINT,
    GEMINI_MODEL_LABEL, GEMINI_MODEL_PLACEHOLDER, GEMINI_MODEL_TITLE, GEMINI_SETTINGS_HEADING,
    HOTKEY_SETTINGS_HEADING, IMPORT_HOTKEY_LABEL, NEW_FILE_HOTKEY_LABEL, OLLAMA_MODEL_CONSTRAINT,
    OLLAMA_MODEL_LABEL, OLLAMA_MODEL_PLACEHOLDER, OLLAMA_MODEL_TITLE, OLLAMA_NUM_CTX_CONSTRAINT,
    OLLAMA_NUM_CTX_LABEL, OLLAMA_NUM_CTX_TITLE, OLLAMA_NUM_PREDICT_CONSTRAINT,
    OLLAMA_NUM_PREDICT_LABEL, OLLAMA_NUM_PREDICT_TITLE, OLLAMA_PORT_CONSTRAINT, OLLAMA_PORT_LABEL,
    OLLAMA_PORT_TITLE, OLLAMA_SETTINGS_HEADING, OLLAMA_TEMPERATURE_CONSTRAINT,
    OLLAMA_TEMPERATURE_LABEL, OLLAMA_TEMPERATURE_TITLE, OLLAMA_THINK_LABEL, OLLAMA_THINK_TITLE,
    OLLAMA_TOP_K_CONSTRAINT, OLLAMA_TOP_K_LABEL, OLLAMA_TOP_K_TITLE, OLLAMA_TOP_P_CONSTRAINT,
    OLLAMA_TOP_P_LABEL, OLLAMA_TOP_P_TITLE, RESULTS_PER_PAGE_CONSTRAINT, RESULTS_PER_PAGE_LABEL,
    SAVE_HOTKEY_LABEL, SETTINGS_HEADING, TOGGLE_MARKDOWN_EDITING_HOTKEY_LABEL,
};
use crate::state::AppState;
use leptos::prelude::*;

/// Render the settings form for display, Ollama, Gemini, and hotkeys.
#[component]
pub fn SettingsPanel() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");
    let error_toast = state.error_toast;
    let number_results_per_page = state.number_results_per_page;
    let ollama_port = state.ollama_port;
    let ollama_model = state.ollama_model;
    let ollama_temperature = state.ollama_temperature;
    let temperature_text = RwSignal::new(ollama_temperature.get_untracked().to_string());
    let ollama_num_predict = state.ollama_num_predict;
    let ollama_num_ctx = state.ollama_num_ctx;
    let ollama_top_p = state.ollama_top_p;
    let top_p_text = RwSignal::new(ollama_top_p.get_untracked().to_string());
    let ollama_top_k = state.ollama_top_k;
    let ollama_think = state.ollama_think;
    let gemini_api_key = state.gemini_api_key;
    let gemini_model = state.gemini_model;
    let save_hotkey_key = state.save_hotkey_key;
    let new_file_hotkey_key = state.new_file_hotkey_key;
    let import_hotkey_key = state.import_hotkey_key;
    let export_hotkey_key = state.export_hotkey_key;
    let toggle_markdown_editing_hotkey_key = state.toggle_markdown_editing_hotkey_key;

    view! {
        <div class="p-3">
            <p class="menu-label mt-2">{SETTINGS_HEADING}</p>
            <p class="menu-label mt-2">{DISPLAY_SETTINGS_HEADING}</p>
            <div class="field">
                <label class="label is-small" style="color:var(--text-muted);">
                    {RESULTS_PER_PAGE_LABEL}
                </label>
                <div class="control">
                    <input
                        class="input is-small"
                        type="number"
                        min="1"
                        prop:value=move || number_results_per_page.get().to_string()
                        on:input=move |ev| {
                            apply_if_valid(
                                &event_target_value(&ev),
                                parse_results_per_page,
                                number_results_per_page,
                            )
                        }
                        on:change=move |ev| {
                            commit_setting(
                                ev,
                                parse_results_per_page,
                                number_results_per_page,
                                error_toast,
                                RESULTS_PER_PAGE_LABEL,
                                RESULTS_PER_PAGE_CONSTRAINT,
                            )
                        }
                    />
                </div>
            </div>
            <hr class="settings-divider"/>
            <p class="menu-label mt-2">{OLLAMA_SETTINGS_HEADING}</p>
            <div class="field">
                <label class="label is-small" style="color:var(--text-muted);" title=OLLAMA_PORT_TITLE>
                    {OLLAMA_PORT_LABEL}
                </label>
                <div class="control">
                    <input
                        class="input is-small"
                        type="number"
                        min="1"
                        max="65535"
                        title=OLLAMA_PORT_TITLE
                        prop:value=move || ollama_port.get().to_string()
                        on:input=move |ev| {
                            apply_if_valid(&event_target_value(&ev), parse_ollama_port, ollama_port)
                        }
                        on:change=move |ev| {
                            commit_setting(
                                ev,
                                parse_ollama_port,
                                ollama_port,
                                error_toast,
                                OLLAMA_PORT_LABEL,
                                OLLAMA_PORT_CONSTRAINT,
                            )
                        }
                    />
                </div>
            </div>
            <div class="field">
                <label class="label is-small" style="color:var(--text-muted);" title=OLLAMA_MODEL_TITLE>
                    {OLLAMA_MODEL_LABEL}
                </label>
                <div class="control">
                    <input
                        class="input is-small"
                        type="text"
                        placeholder=OLLAMA_MODEL_PLACEHOLDER
                        title=OLLAMA_MODEL_TITLE
                        prop:value=move || ollama_model.get()
                        on:input=move |ev| {
                            apply_if_valid(&event_target_value(&ev), parse_ollama_model, ollama_model)
                        }
                        on:change=move |ev| {
                            commit_setting(
                                ev,
                                parse_ollama_model,
                                ollama_model,
                                error_toast,
                                OLLAMA_MODEL_LABEL,
                                OLLAMA_MODEL_CONSTRAINT,
                            )
                        }
                    />
                </div>
            </div>
            <div class="field">
                <label class="label is-small" style="color:var(--text-muted);" title=OLLAMA_TEMPERATURE_TITLE>
                    {OLLAMA_TEMPERATURE_LABEL}
                </label>
                <div class="control">
                    <input
                        class="input is-small"
                        type="text"
                        inputmode="decimal"
                        title=OLLAMA_TEMPERATURE_TITLE
                        prop:value=move || temperature_text.get()
                        on:input=move |ev| {
                            let raw = event_target_value(&ev);
                            temperature_text.set(raw.clone());
                            apply_if_valid(
                                &raw,
                                |raw| {
                                    parse_bounded_f64(
                                        raw,
                                        OLLAMA_TEMPERATURE_MIN,
                                        OLLAMA_TEMPERATURE_MAX,
                                    )
                                },
                                ollama_temperature,
                            )
                        }
                        on:change=move |ev| {
                            commit_setting(
                                ev,
                                |raw| {
                                    parse_bounded_f64(
                                        raw,
                                        OLLAMA_TEMPERATURE_MIN,
                                        OLLAMA_TEMPERATURE_MAX,
                                    )
                                },
                                ollama_temperature,
                                error_toast,
                                OLLAMA_TEMPERATURE_LABEL,
                                OLLAMA_TEMPERATURE_CONSTRAINT,
                            );
                            temperature_text.set(ollama_temperature.get_untracked().to_string());
                        }
                    />
                </div>
            </div>
            <div class="field">
                <label class="label is-small" style="color:var(--text-muted);" title=OLLAMA_NUM_PREDICT_TITLE>
                    {OLLAMA_NUM_PREDICT_LABEL}
                </label>
                <div class="control">
                    <input
                        class="input is-small"
                        type="number"
                        min="-1"
                        step="1"
                        title=OLLAMA_NUM_PREDICT_TITLE
                        prop:value=move || ollama_num_predict.get().to_string()
                        on:input=move |ev| {
                            apply_if_valid(
                                &event_target_value(&ev),
                                parse_ollama_num_predict,
                                ollama_num_predict,
                            )
                        }
                        on:change=move |ev| {
                            commit_setting(
                                ev,
                                parse_ollama_num_predict,
                                ollama_num_predict,
                                error_toast,
                                OLLAMA_NUM_PREDICT_LABEL,
                                OLLAMA_NUM_PREDICT_CONSTRAINT,
                            )
                        }
                    />
                </div>
            </div>
            <div class="field">
                <label class="label is-small" style="color:var(--text-muted);" title=OLLAMA_NUM_CTX_TITLE>
                    {OLLAMA_NUM_CTX_LABEL}
                </label>
                <div class="control">
                    <input
                        class="input is-small"
                        type="number"
                        min="1"
                        step="1"
                        title=OLLAMA_NUM_CTX_TITLE
                        prop:value=move || ollama_num_ctx.get().to_string()
                        on:input=move |ev| {
                            apply_if_valid(
                                &event_target_value(&ev),
                                parse_positive_u32,
                                ollama_num_ctx,
                            )
                        }
                        on:change=move |ev| {
                            commit_setting(
                                ev,
                                parse_positive_u32,
                                ollama_num_ctx,
                                error_toast,
                                OLLAMA_NUM_CTX_LABEL,
                                OLLAMA_NUM_CTX_CONSTRAINT,
                            )
                        }
                    />
                </div>
            </div>
            <div class="field">
                <label class="label is-small" style="color:var(--text-muted);" title=OLLAMA_TOP_P_TITLE>
                    {OLLAMA_TOP_P_LABEL}
                </label>
                <div class="control">
                    <input
                        class="input is-small"
                        type="text"
                        inputmode="decimal"
                        title=OLLAMA_TOP_P_TITLE
                        prop:value=move || top_p_text.get()
                        on:input=move |ev| {
                            let raw = event_target_value(&ev);
                            top_p_text.set(raw.clone());
                            apply_if_valid(
                                &raw,
                                |raw| parse_bounded_f64(raw, OLLAMA_TOP_P_MIN, OLLAMA_TOP_P_MAX),
                                ollama_top_p,
                            )
                        }
                        on:change=move |ev| {
                            commit_setting(
                                ev,
                                |raw| parse_bounded_f64(raw, OLLAMA_TOP_P_MIN, OLLAMA_TOP_P_MAX),
                                ollama_top_p,
                                error_toast,
                                OLLAMA_TOP_P_LABEL,
                                OLLAMA_TOP_P_CONSTRAINT,
                            );
                            top_p_text.set(ollama_top_p.get_untracked().to_string());
                        }
                    />
                </div>
            </div>
            <div class="field">
                <label class="label is-small" style="color:var(--text-muted);" title=OLLAMA_TOP_K_TITLE>
                    {OLLAMA_TOP_K_LABEL}
                </label>
                <div class="control">
                    <input
                        class="input is-small"
                        type="number"
                        min="1"
                        step="1"
                        title=OLLAMA_TOP_K_TITLE
                        prop:value=move || ollama_top_k.get().to_string()
                        on:input=move |ev| {
                            apply_if_valid(&event_target_value(&ev), parse_positive_u32, ollama_top_k)
                        }
                        on:change=move |ev| {
                            commit_setting(
                                ev,
                                parse_positive_u32,
                                ollama_top_k,
                                error_toast,
                                OLLAMA_TOP_K_LABEL,
                                OLLAMA_TOP_K_CONSTRAINT,
                            )
                        }
                    />
                </div>
            </div>
            <div class="field">
                <label class="settings-checkbox" title=OLLAMA_THINK_TITLE>
                    {OLLAMA_THINK_LABEL}
                    <input
                        type="checkbox"
                        prop:checked=move || ollama_think.get()
                        on:change=move |ev| ollama_think.set(event_target_checked(&ev))
                    />
                </label>
            </div>
            <hr class="settings-divider"/>
            <p class="menu-label mt-2">{GEMINI_SETTINGS_HEADING}</p>
            <div class="field">
                <label class="label is-small" style="color:var(--text-muted);" title=GEMINI_API_KEY_TITLE>
                    {GEMINI_API_KEY_LABEL}
                </label>
                <div class="control">
                    <input
                        class="input is-small"
                        type="text"
                        placeholder=GEMINI_API_KEY_PLACEHOLDER
                        title=GEMINI_API_KEY_TITLE
                        autocomplete="off"
                        prop:value=move || gemini_api_key.get()
                        on:input=move |ev| {
                            apply_if_valid(
                                &event_target_value(&ev),
                                parse_gemini_api_key,
                                gemini_api_key,
                            )
                        }
                        on:change=move |ev| {
                            commit_setting(
                                ev,
                                parse_gemini_api_key,
                                gemini_api_key,
                                error_toast,
                                GEMINI_API_KEY_LABEL,
                                GEMINI_API_KEY_CONSTRAINT,
                            )
                        }
                    />
                </div>
            </div>
            <div class="field">
                <label class="label is-small" style="color:var(--text-muted);" title=GEMINI_MODEL_TITLE>
                    {GEMINI_MODEL_LABEL}
                </label>
                <div class="control">
                    <input
                        class="input is-small"
                        type="text"
                        placeholder=GEMINI_MODEL_PLACEHOLDER
                        title=GEMINI_MODEL_TITLE
                        prop:value=move || gemini_model.get()
                        on:input=move |ev| {
                            apply_if_valid(
                                &event_target_value(&ev),
                                parse_gemini_model,
                                gemini_model,
                            )
                        }
                        on:change=move |ev| {
                            commit_setting(
                                ev,
                                parse_gemini_model,
                                gemini_model,
                                error_toast,
                                GEMINI_MODEL_LABEL,
                                GEMINI_MODEL_CONSTRAINT,
                            )
                        }
                    />
                </div>
            </div>
            <hr class="settings-divider"/>
            <p class="menu-label mt-2">{HOTKEY_SETTINGS_HEADING}</p>
            <div class="field">
                <label class="label is-small" style="color:var(--text-muted);">
                    {move || with_hotkey(SAVE_HOTKEY_LABEL, &format_ctrl_hotkey(save_hotkey_key.get()))}
                </label>
                <div class="control">
                    <input
                        class="input is-small"
                        type="text"
                        maxlength="1"
                        prop:value=move || save_hotkey_key.get().to_string()
                        on:keydown=move |ev| {
                            on_hotkey_keydown(ev, save_hotkey_key, error_toast, SAVE_HOTKEY_LABEL)
                        }
                    />
                </div>
            </div>
            <div class="field">
                <label class="label is-small" style="color:var(--text-muted);">
                    {move || with_hotkey(
                        NEW_FILE_HOTKEY_LABEL,
                        &format_ctrl_hotkey(new_file_hotkey_key.get()),
                    )}
                </label>
                <div class="control">
                    <input
                        class="input is-small"
                        type="text"
                        maxlength="1"
                        prop:value=move || new_file_hotkey_key.get().to_string()
                        on:keydown=move |ev| {
                            on_hotkey_keydown(ev, new_file_hotkey_key, error_toast, NEW_FILE_HOTKEY_LABEL)
                        }
                    />
                </div>
            </div>
            <div class="field">
                <label class="label is-small" style="color:var(--text-muted);">
                    {move || with_hotkey(
                        IMPORT_HOTKEY_LABEL,
                        &format_ctrl_hotkey(import_hotkey_key.get()),
                    )}
                </label>
                <div class="control">
                    <input
                        class="input is-small"
                        type="text"
                        maxlength="1"
                        prop:value=move || import_hotkey_key.get().to_string()
                        on:keydown=move |ev| {
                            on_hotkey_keydown(ev, import_hotkey_key, error_toast, IMPORT_HOTKEY_LABEL)
                        }
                    />
                </div>
            </div>
            <div class="field">
                <label class="label is-small" style="color:var(--text-muted);">
                    {move || with_hotkey(
                        EXPORT_HOTKEY_LABEL,
                        &format_ctrl_hotkey(export_hotkey_key.get()),
                    )}
                </label>
                <div class="control">
                    <input
                        class="input is-small"
                        type="text"
                        maxlength="1"
                        prop:value=move || export_hotkey_key.get().to_string()
                        on:keydown=move |ev| {
                            on_hotkey_keydown(ev, export_hotkey_key, error_toast, EXPORT_HOTKEY_LABEL)
                        }
                    />
                </div>
            </div>
            <div class="field">
                <label class="label is-small" style="color:var(--text-muted);">
                    {move || with_hotkey(
                        TOGGLE_MARKDOWN_EDITING_HOTKEY_LABEL,
                        &format_ctrl_hotkey(toggle_markdown_editing_hotkey_key.get()),
                    )}
                </label>
                <div class="control">
                    <input
                        class="input is-small"
                        type="text"
                        maxlength="1"
                        prop:value=move || toggle_markdown_editing_hotkey_key.get().to_string()
                        on:keydown=move |ev| {
                            on_hotkey_keydown(
                                ev,
                                toggle_markdown_editing_hotkey_key,
                                error_toast,
                                TOGGLE_MARKDOWN_EDITING_HOTKEY_LABEL,
                            )
                        }
                    />
                </div>
            </div>
        </div>
    }
}
