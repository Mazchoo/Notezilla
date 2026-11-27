//! Renders a ` ```flashcards ` fence as a deck shown one card at a time.
//!
//! The editor HTML has no script and no element ids. Every button of a deck,
//! Previous, Reveal, Hide and Next on each card, is a radio wrapped in its
//! `<label>`, and all of a deck's radios form one group, so the one checked
//! radio is the deck's whole state: which card is shown and whether its answer
//! is revealed. Checking any radio unchecks the rest, which is what hides an
//! answer again once the reader moves to another card. `flashcards.css` reads
//! the state with `:has()` and sibling combinators: before any click the first
//! card shows, Next and Previous show the card beside theirs, and Reveal and
//! Hide keep their own card shown with or without its answer.
//!
//! Each deck is its own `<form>`, which scopes the radio group to the deck so
//! two decks on one page cannot interfere. `method="dialog"` makes an implicit
//! submission (Enter on a focused radio) do nothing instead of reloading the
//! page. PDF export cannot interact, so it lists every card with its answer.
//!
//! Source format, one card per blank-line separated paragraph:
//!
//! ```text
//! Question text
//! Answer text
//! ```

use super::render_error::render_error_html;
use super::{Render, RenderPdf};
use crate::constants::{FLASHCARDS_ERROR_CLASS, FLASHCARDS_ROOT_CLASS_ATTR, FLASHCARD_RADIO_NAME};
use crate::info_messages::{
    flashcard_position, FLASHCARDS_EMPTY_ERROR, FLASHCARD_EXTRA_LINE_ERROR, FLASHCARD_HIDE_BUTTON,
    FLASHCARD_NEXT_BUTTON, FLASHCARD_NO_ANSWER_ERROR, FLASHCARD_PREVIOUS_BUTTON,
    FLASHCARD_REVEAL_BUTTON,
};
use crate::rendering::escape_html;
use std::fmt::Write;

/// Flash card fence render.
pub struct FlashcardsRender;

impl Render for FlashcardsRender {
    /// Return the interactive deck HTML, or the source in an error block.
    fn render(&self, source: &str) -> String {
        match parse_flashcards(source) {
            Ok(cards) => deck_html(&cards),
            Err(error) => render_error_html(FLASHCARDS_ERROR_CLASS, source, error),
        }
    }
}

impl RenderPdf for FlashcardsRender {
    /// Return a static list: each numbered question with its answer below.
    fn render_pdf(&self, source: &str) -> String {
        match parse_flashcards(source) {
            Ok(cards) => card_list_html(&cards),
            Err(error) => render_error_html(FLASHCARDS_ERROR_CLASS, source, error),
        }
    }
}

/// One flash card, borrowed from the fence source.
#[derive(Debug, PartialEq)]
struct Flashcard<'a> {
    question: &'a str,
    answer: &'a str,
}

/// Parse the fence source into cards.
fn parse_flashcards(source: &str) -> Result<Vec<Flashcard<'_>>, &'static str> {
    let mut cards = Vec::new();
    // Question and answer lines of the paragraph being read.
    let mut current: Option<(&str, Option<&str>)> = None;

    for line in source.lines() {
        let trimmed = line.trim();
        current = match (current, trimmed.is_empty()) {
            (None, true) => None,
            (Some(pending), true) => {
                cards.push(finish_card(pending)?);
                None
            }
            (None, false) => Some((trimmed, None)),
            (Some((question, None)), false) => Some((question, Some(trimmed))),
            (Some((_, Some(_))), false) => return Err(FLASHCARD_EXTRA_LINE_ERROR),
        };
    }
    if let Some(pending) = current {
        cards.push(finish_card(pending)?);
    }
    if cards.is_empty() {
        return Err(FLASHCARDS_EMPTY_ERROR);
    }
    Ok(cards)
}

/// Reject a card that has a question but no answer.
fn finish_card<'a>(
    (question, answer): (&'a str, Option<&'a str>),
) -> Result<Flashcard<'a>, &'static str> {
    let answer = answer.ok_or(FLASHCARD_NO_ANSWER_ERROR)?;
    Ok(Flashcard { question, answer })
}

/// Return the interactive editor HTML for `cards`.
fn deck_html(cards: &[Flashcard<'_>]) -> String {
    let last = cards.len() - 1;
    let mut html = format!(r#"<form {FLASHCARDS_ROOT_CLASS_ATTR} method="dialog">"#);
    for (index, card) in cards.iter().enumerate() {
        let _ = write!(
            html,
            r#"<div class="flashcard"><p class="flashcard-position">{}</p><p class="flashcard-question">{}</p><p class="flashcard-answer">{}</p><div class="flashcard-buttons">"#,
            flashcard_position(index + 1, cards.len()),
            escape_html(card.question),
            escape_html(card.answer)
        );
        if index > 0 {
            push_button(&mut html, "flashcard-previous", FLASHCARD_PREVIOUS_BUTTON);
        }
        push_button(&mut html, "flashcard-reveal", FLASHCARD_REVEAL_BUTTON);
        push_button(&mut html, "flashcard-hide", FLASHCARD_HIDE_BUTTON);
        if index < last {
            push_button(&mut html, "flashcard-next", FLASHCARD_NEXT_BUTTON);
        }
        html.push_str("</div></div>");
    }
    html.push_str("</form>");
    html
}

/// Append one button: a radio of the deck's group wrapped in its `<label>`.
fn push_button(html: &mut String, class: &str, label: &str) {
    let _ = write!(
        html,
        r#"<label class="flashcard-button {class}"><input type="radio" name="{FLASHCARD_RADIO_NAME}" class="flashcard-input">{label}</label>"#
    );
}

/// Return the static PDF list for `cards`: the numbered question in bold with
/// its answer on the line below.
fn card_list_html(cards: &[Flashcard<'_>]) -> String {
    let mut html = format!("<div {FLASHCARDS_ROOT_CLASS_ATTR}>");
    for (number, card) in cards.iter().enumerate() {
        let _ = write!(
            html,
            "<p><strong>{}. {}</strong><br>{}</p>",
            number + 1,
            escape_html(card.question),
            escape_html(card.answer)
        );
    }
    html.push_str("</div>");
    html
}

#[cfg(test)]
mod tests {
    use super::{parse_flashcards, Flashcard, FlashcardsRender};
    use crate::constants::{FLASHCARDS_ERROR_CLASS, FLASHCARD_RADIO_NAME, QUIZ_HELP_MANUAL};
    use crate::info_messages::{
        flashcard_position, FLASHCARDS_EMPTY_ERROR, FLASHCARD_EXTRA_LINE_ERROR,
        FLASHCARD_NO_ANSWER_ERROR,
    };
    use crate::rendering::renders::{Render, RenderPdf};

    const SAMPLE: &str = "What is the capital of Peru?
Lima


  How many legs has a spider?
Eight
";

    /// Return the flash card fence bodies of `markdown`.
    fn flashcard_fences(markdown: &str) -> Vec<&str> {
        markdown
            .split("```flashcards\n")
            .skip(1)
            .map(|rest| &rest[..rest.find("```").expect("closing fence")])
            .collect()
    }

    #[test]
    /// Assert cards split on blank lines and their lines are trimmed.
    fn parses_cards() {
        assert_eq!(
            parse_flashcards(SAMPLE),
            Ok(vec![
                Flashcard {
                    question: "What is the capital of Peru?",
                    answer: "Lima",
                },
                Flashcard {
                    question: "How many legs has a spider?",
                    answer: "Eight",
                },
            ])
        );
    }

    #[test]
    /// Assert CRLF line endings parse like LF.
    fn accepts_crlf() {
        let cards = parse_flashcards("Q\r\nA\r\n\r\nQ2\r\nA2\r\n").expect("valid deck");
        assert_eq!(cards.len(), 2);
        assert_eq!(cards[1].answer, "A2");
    }

    #[test]
    /// Assert malformed sources report why they failed.
    fn rejects_malformed_sources() {
        for (source, error) in [
            ("", FLASHCARDS_EMPTY_ERROR),
            ("\n\n", FLASHCARDS_EMPTY_ERROR),
            ("Question only\n", FLASHCARD_NO_ANSWER_ERROR),
            ("Q\nA\n\nQuestion only\n", FLASHCARD_NO_ANSWER_ERROR),
            ("Q\nA\nExtra\n", FLASHCARD_EXTRA_LINE_ERROR),
        ] {
            assert_eq!(parse_flashcards(source), Err(error), "{source:?}");
        }
    }

    #[test]
    /// Assert every card has reveal and hide, and inner cards both neighbours.
    fn renders_cards_and_buttons() {
        let html = FlashcardsRender.render("A\n1\n\nB\n2\n\nC\n3\n");
        assert_eq!(html.matches(r#"<div class="flashcard">"#).count(), 3);
        for (class, count) in [
            ("flashcard-previous", 2),
            ("flashcard-reveal", 3),
            ("flashcard-hide", 3),
            ("flashcard-next", 2),
        ] {
            assert_eq!(
                html.matches(&format!(r#"class="flashcard-button {class}""#))
                    .count(),
                count,
                "{class}: {html}"
            );
        }
        assert!(
            html.starts_with(r#"<form class="flashcards" method="dialog"><div class="flashcard">"#),
            "the first card must be the deck's first child: {html}"
        );
        assert!(
            html.contains(&format!(
                r#"<p class="flashcard-position">{}</p>"#,
                flashcard_position(2, 3)
            )),
            "{html}"
        );
        assert!(html.contains(r#"<p class="flashcard-answer">3</p>"#));
        assert!(!html.contains("<script"), "{html}");
        assert!(!html.contains(" id="), "deck needs no element ids: {html}");
    }

    #[test]
    /// Assert every button is a radio of the one group the deck's form scopes.
    fn buttons_share_one_radio_group() {
        let html = FlashcardsRender.render("A\n1\n\nB\n2\n");
        let radio = format!(r#"<input type="radio" name="{FLASHCARD_RADIO_NAME}""#);
        assert_eq!(html.matches("<input").count(), 6, "{html}");
        assert_eq!(html.matches(&radio).count(), 6, "{html}");
        assert_eq!(html.matches("<form").count(), 1, "{html}");
    }

    #[test]
    /// Assert a single card renders without navigation buttons.
    fn single_card_has_no_navigation() {
        let html = FlashcardsRender.render("Q\nA\n");
        assert!(!html.contains("flashcard-previous"), "{html}");
        assert!(!html.contains("flashcard-next"), "{html}");
    }

    #[test]
    /// Assert card text is escaped so it cannot inject markup.
    fn escapes_card_text() {
        let html = FlashcardsRender.render("<b>Q</b>\n<script>\n");
        assert!(
            !html.contains("<b>") && !html.contains("<script>"),
            "{html}"
        );
        assert!(html.contains("&lt;script&gt;"), "{html}");
    }

    #[test]
    /// Assert a malformed deck falls back to the error block.
    fn malformed_deck_renders_error_block() {
        let html = FlashcardsRender.render("Question only\n");
        assert!(
            html.contains(&format!(r#"class="{FLASHCARDS_ERROR_CLASS}""#)),
            "{html}"
        );
    }

    #[test]
    /// Assert the PDF lists every card with its answer and no inputs.
    fn pdf_renders_static_list() {
        let html = FlashcardsRender.render_pdf(SAMPLE);
        assert!(!html.contains("<input"), "{html}");
        assert!(
            html.contains("<p><strong>1. What is the capital of Peru?</strong><br>Lima</p>"),
            "{html}"
        );
        assert!(
            html.contains("<p><strong>2. How many legs has a spider?</strong><br>Eight</p>"),
            "{html}"
        );
    }

    #[test]
    /// Assert every deck in the bundled manual renders without falling back.
    fn manual_flashcards_render() {
        let fences = flashcard_fences(QUIZ_HELP_MANUAL);
        assert!(!fences.is_empty(), "manual has no flashcards fence");
        for fence in fences {
            assert!(parse_flashcards(fence).is_ok(), "{fence}");
        }
    }
}
