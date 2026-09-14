use crate::components::file_io::refresh::on_file_change_notify;
use crate::constants::FILE_CHANGE_EVENTS_URL;
use crate::state::AppState;
use std::cell::RefCell;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{EventSource, MessageEvent};

thread_local! {
    static FILE_CHANGE_SOURCE: RefCell<Option<(EventSource, Closure<dyn FnMut(MessageEvent)>)>> =
        RefCell::new(None);
}

/// Open a standing EventSource and refresh the GUI on each file-change ping.
pub fn listen_file_changes(state: AppState) {
    let Ok(source) = EventSource::new(FILE_CHANGE_EVENTS_URL) else {
        web_sys::console::warn_1(&"Could not open file-change event stream".into());
        return;
    };

    let owner = state.root_owner.clone();
    let on_message = Closure::<dyn FnMut(MessageEvent)>::new(move |_ev: MessageEvent| {
        owner.with(|| on_file_change_notify(&state));
    });
    source.set_onmessage(Some(on_message.as_ref().unchecked_ref()));

    FILE_CHANGE_SOURCE.with(|slot| {
        *slot.borrow_mut() = Some((source, on_message));
    });
}
