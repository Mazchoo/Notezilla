use crate::settings::TOAST_DISMISS_MS;
use crate::state::AppState;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;

/// Show a short-lived toast message at the bottom of the viewport.
pub fn show_toast(toast: RwSignal<Option<String>>, message: impl Into<String>) {
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

    view! {
        {move || {
            toast
                .get()
                .map(|msg| view! { <div class="toast" role="status" aria-live="polite">{msg}</div> })
        }}
    }
}
