//! Tracks list style while rewriting markdown lists for PDF export.

use crate::constants::LIST_BULLET;

/// Unordered list, or an ordered list with the next marker number.
pub enum PdfListKind {
    Ul,
    Ol { next: u64 },
}

impl PdfListKind {
    /// Build a list kind from a markdown list's first number, if it has one.
    pub fn new(first_number: Option<u64>) -> Self {
        match first_number {
            Some(next) => PdfListKind::Ol { next },
            None => PdfListKind::Ul,
        }
    }

    /// Return this list's next item marker, advancing an ordered list.
    pub fn next_marker(&mut self) -> String {
        match self {
            PdfListKind::Ul => LIST_BULLET.to_string(),
            PdfListKind::Ol { next } => {
                let number = *next;
                *next += 1;
                format!("{number}.")
            }
        }
    }
}

/// Return the opening HTML of a PDF list item carrying `marker`.
///
/// The marker and the item body are separate flex children so that inline math
/// in the body typesets on the marker's row.
pub fn pdf_list_item_open(marker: &str) -> String {
    format!(
        "<div class=\"pdf-li\" style=\"display:flex;align-items:center\">\
         <div class=\"pdf-li-mark\" style=\"flex:0 0 18pt\">{marker} </div>\
         <div class=\"pdf-li-body\"><div>"
    )
}

#[cfg(test)]
mod tests {
    use super::{PdfListKind, LIST_BULLET};

    #[test]
    /// Assert an unordered list marks every item with a bullet.
    fn unordered_list_marks_items_with_bullets() {
        let mut kind = PdfListKind::new(None);
        assert_eq!(kind.next_marker(), LIST_BULLET);
        assert_eq!(kind.next_marker(), LIST_BULLET);
    }

    #[test]
    /// Assert an ordered list numbers items from its first number.
    fn ordered_list_numbers_items_from_its_start() {
        let mut kind = PdfListKind::new(Some(3));
        assert_eq!(kind.next_marker(), "3.");
        assert_eq!(kind.next_marker(), "4.");
    }
}
