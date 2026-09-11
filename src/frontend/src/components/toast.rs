use crate::default_settings::TOAST_DISMISS_MS;
use crate::info_messages::{
    CLIPBOARD_COPY_FAILED_TOAST, ERROR_COPIED_TOAST, ERROR_TOAST_COPY_TITLE,
};
use crate::state::AppState;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::cell::Cell;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

thread_local! {
    static WARNING_TOAST: Cell<Option<RwSignal<Option<String>>>> = const { Cell::new(None) };
    static ERROR_DISMISS_TIMEOUT: Cell<Option<i32>> = const { Cell::new(None) };
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
    error_toast.set(Some(message.into()));
    schedule_error_dismiss(error_toast);
}

/// Return the status-toast text after copying an error toast.
fn error_toast_copy_feedback(succeeded: bool) -> &'static str {
    if succeeded {
        ERROR_COPIED_TOAST
    } else {
        CLIPBOARD_COPY_FAILED_TOAST
    }
}

/// Copy an error toast without replacing the error message on failure.
fn copy_error_toast_text(text: String, toast: RwSignal<Option<String>>) {
    let Some(window) = web_sys::window() else {
        web_sys::console::error_1(&"Clipboard unavailable: no window".into());
        show_toast(toast, error_toast_copy_feedback(false));
        return;
    };
    let clipboard = window.navigator().clipboard();
    spawn_local(async move {
        match JsFuture::from(clipboard.write_text(&text)).await {
            Ok(_) => show_toast(toast, error_toast_copy_feedback(true)),
            Err(e) => {
                web_sys::console::error_1(&e);
                show_toast(toast, error_toast_copy_feedback(false));
            }
        }
    });
}

/// Cancel the pending error-toast dismiss timeout.
fn clear_error_dismiss_timeout() {
    let Some(id) = ERROR_DISMISS_TIMEOUT.take() else {
        return;
    };
    let Some(window) = web_sys::window() else {
        return;
    };
    window.clear_timeout_with_handle(id);
}

/// Dismiss the error toast after the timeout, replacing any pending dismiss.
fn schedule_error_dismiss(error_toast: RwSignal<Option<String>>) {
    clear_error_dismiss_timeout();
    let Some(window) = web_sys::window() else {
        return;
    };
    let closure = Closure::once(move || error_toast.set(None));
    if let Ok(id) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        closure.as_ref().unchecked_ref(),
        TOAST_DISMISS_MS,
    ) {
        ERROR_DISMISS_TIMEOUT.set(Some(id));
    }
    closure.forget();
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

/// Show a toast message and clear it after the dismiss timeout.
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

/// Render stacked error, warning, and status toast messages.
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
                    let copy_msg = msg.clone();
                    view! {
                        <div
                            class="toast toast-error"
                            role="alert"
                            aria-live="assertive"
                            title=ERROR_TOAST_COPY_TITLE
                            on:click=move |_| copy_error_toast_text(copy_msg.clone(), toast)
                            on:mouseenter=move |_| clear_error_dismiss_timeout()
                            on:mouseleave=move |_| schedule_error_dismiss(error_toast)
                        >
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

#[cfg(test)]
mod tests {
    use super::{error_toast_copy_feedback, show_mcp_warnings};
    use crate::info_messages::{CLIPBOARD_COPY_FAILED_TOAST, ERROR_COPIED_TOAST};

    #[test]
    /// Assert unbound or empty warnings do not require a toast signal.
    fn show_mcp_warnings_is_noop_when_unbound_or_empty() {
        show_mcp_warnings(&[]);
        show_mcp_warnings(&[String::new(), String::new()]);
        show_mcp_warnings(&["warn".into()]);
    }

    #[test]
    /// Assert copying an error toast reports on the status toast and keeps the error.
    fn error_toast_copy_feedback_uses_status_copy() {
        assert_eq!(error_toast_copy_feedback(true), ERROR_COPIED_TOAST);
        assert_eq!(
            error_toast_copy_feedback(false),
            CLIPBOARD_COPY_FAILED_TOAST
        );
    }
}
