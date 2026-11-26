//! Recognises clicks on the checkboxes of a rendered quiz.

use crate::constants::QUIZ_CONTROL_SELECTOR;
use wasm_bindgen::JsCast;
use web_sys::{Element, MouseEvent};

/// Return whether a click landed on a quiz answer or reveal button.
///
/// In edit mode such a click must only open the block for editing, so the
/// caller cancels the checkbox toggle.
pub(crate) fn clicked_quiz_control(ev: &MouseEvent) -> bool {
    ev.target()
        .and_then(|target| target.dyn_into::<Element>().ok())
        .and_then(|element| element.closest(QUIZ_CONTROL_SELECTOR).ok().flatten())
        .is_some()
}
