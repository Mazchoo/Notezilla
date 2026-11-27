//! Recognises clicks on the controls of a rendered quiz or flash card deck.

use crate::constants::STUDY_CONTROL_SELECTOR;
use wasm_bindgen::JsCast;
use web_sys::{Element, MouseEvent};

/// Return whether a click landed on a quiz answer, a reveal button, or a flash
/// card navigation button.
///
/// In edit mode such a click must only open the block for editing, so the
/// caller cancels the checkbox toggle.
pub(crate) fn clicked_study_control(ev: &MouseEvent) -> bool {
    ev.target()
        .and_then(|target| target.dyn_into::<Element>().ok())
        .and_then(|element| element.closest(STUDY_CONTROL_SELECTOR).ok().flatten())
        .is_some()
}
