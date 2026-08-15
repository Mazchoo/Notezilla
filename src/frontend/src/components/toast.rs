use crate::settings::TOAST_DISMISS_MS;
use crate::state::AppState;
use leptos::prelude::*;
use std::cell::Cell;
use wasm_bindgen::prelude::*;

thread_local! {
    static WARNING_TOAST: Cell<Option<RwSignal<Option<String>>>> = const { Cell::new(None) };
}

/// Bind the warning toast signal so MCP responses can display `warnings` from async tasks.
pub fn bind_warning_toast(toast: RwSignal<Option<String>>) {
    WARNING_TOAST.set(Some(toast));
}

/// Show a short-lived toast message at the bottom of the viewport.
pub fn show_toast(toast: RwSignal<Option<String>>, message: impl Into<String>) {
    set_timed_toast(toast, message);
}

/// Show a short-lived error toast above the main toast.
pub fn show_error_toast(error_toast: RwSignal<Option<String>>, message: impl Into<String>) {
    set_timed_toast(error_toast, message);
}

/// Show MCP `warnings` when the list is non-empty. No-op if none are bound or all are empty.
pub fn show_mcp_warnings(warnings: &[String]) {
    let message = warnings
        .iter()
        .map(String::as_str)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    if message.is_empty() {
        return;
    }
    let Some(toast) = WARNING_TOAST.get() else {
        return;
    };
    set_timed_toast(toast, message);
}

fn set_timed_toast(toast: RwSignal<Option<String>>, message: impl Into<String>) {
    toast.set(Some(message.into()));
    let Some(window) = web_sys::window() else {
        return;
    };
    let closure = Closure::once(move || toast.set(None));
    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        closure.as_ref().unchecked_ref(),
        TOAST_DISMISS_MS,
    );
    closure.forget();
}

#[component]
pub fn Toast() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");
    let toast = state.toast;
    let warning_toast = state.warning_toast;
    let error_toast = state.error_toast;

    view! {
        <div class="toast-stack">
            {move || {
                error_toast.get().map(|msg| {
                    view! {
                        <div class="toast toast-error" role="alert" aria-live="assertive">
                            {msg}
                        </div>
                    }
                })
            }}
            {move || {
                warning_toast.get().map(|msg| {
                    view! {
                        <div class="toast toast-warning" role="status" aria-live="polite">
                            {msg}
                        </div>
                    }
                })
            }}
            {move || {
                toast.get().map(|msg| {
                    view! { <div class="toast" role="status" aria-live="polite">{msg}</div> }
                })
            }}
        </div>
    }
}
