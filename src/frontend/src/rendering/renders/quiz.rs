//! Renders a ` ```quiz ` fence as a multiple-choice quiz.
//!
//! The editor HTML has no script: every answer and each question's reveal
//! button is a checkbox wrapped in its `<label>`, so no element ids are needed,
//! and `quiz.css` uses `:checked` and `:has()` to colour answers and show their
//! explanations once the reveal checkbox is ticked. PDF export cannot interact,
//! so it renders a static answer key instead.
//!
//! Source format, one question per blank-line separated paragraph:
//!
//! ```text
//! Question text
//! Answer text
//!   + Shown when the answer is selected, as expected
//!   Shown when it is not selected
//! Another answer
//!   - Shown when the answer is left unselected, as expected
//!   Shown when it is selected
//! ```

use super::render_error::render_error_html;
use super::{Render, RenderPdf};
use crate::constants::{
    QUIZ_CORRECT_MARK, QUIZ_ERROR_CLASS, QUIZ_EXPECT_SELECTED_MARKER,
    QUIZ_EXPECT_UNSELECTED_MARKER, QUIZ_INCORRECT_MARK,
};
use crate::info_messages::{
    QUIZ_EMPTY_ERROR, QUIZ_EXPLANATION_MARKER_ERROR, QUIZ_EXPLANATION_WITHOUT_ANSWER_ERROR,
    QUIZ_HIDE_BUTTON, QUIZ_PDF_CORRECT_LABEL, QUIZ_PDF_INCORRECT_LABEL,
    QUIZ_QUESTION_NO_ANSWERS_ERROR, QUIZ_REVEAL_BUTTON, QUIZ_TOO_MANY_EXPLANATIONS_ERROR,
};
use crate::rendering::escape_html;
use std::fmt::Write;

/// Quiz fence render.
pub struct QuizRender;

impl Render for QuizRender {
    /// Return the interactive quiz HTML, or the source in an error block.
    fn render(&self, source: &str) -> String {
        match parse_quiz(source) {
            Ok(questions) => quiz_html(&questions),
            Err(error) => render_error_html(QUIZ_ERROR_CLASS, source, error),
        }
    }
}

impl RenderPdf for QuizRender {
    /// Return a static answer key: each answer labelled correct or incorrect.
    fn render_pdf(&self, source: &str) -> String {
        match parse_quiz(source) {
            Ok(questions) => answer_key_html(&questions),
            Err(error) => render_error_html(QUIZ_ERROR_CLASS, source, error),
        }
    }
}

/// One answer of a quiz question, borrowed from the fence source.
#[derive(Debug, Default, PartialEq)]
struct QuizAnswer<'a> {
    text: &'a str,
    /// Whether the answer should be selected; unmarked answers should not.
    expect_selected: bool,
    /// Explanation shown when the reader's choice matches the expectation.
    matched: Option<&'a str>,
    /// Explanation shown when the reader's choice does not match.
    mismatched: Option<&'a str>,
    /// Number of indented explanation lines read so far.
    explanation_lines: u8,
}

impl<'a> QuizAnswer<'a> {
    /// Return the explanation shown while the answer is selected.
    fn explain_when_selected(&self) -> Option<&'a str> {
        if self.expect_selected {
            self.matched
        } else {
            self.mismatched
        }
    }

    /// Return the explanation shown while the answer is not selected.
    fn explain_when_unselected(&self) -> Option<&'a str> {
        if self.expect_selected {
            self.mismatched
        } else {
            self.matched
        }
    }

    /// Read one indented line: the marked first line, then the mismatch line.
    fn push_explanation(&mut self, line: &'a str) -> Result<(), &'static str> {
        match self.explanation_lines {
            0 => {
                let (expect_selected, rest) = split_marker(line)?;
                self.expect_selected = expect_selected;
                self.matched = non_empty(rest);
            }
            1 => self.mismatched = non_empty(line),
            _ => return Err(QUIZ_TOO_MANY_EXPLANATIONS_ERROR),
        }
        self.explanation_lines += 1;
        Ok(())
    }
}

/// One quiz question and its answers.
#[derive(Debug, PartialEq)]
struct QuizQuestion<'a> {
    prompt: &'a str,
    answers: Vec<QuizAnswer<'a>>,
}

/// Parse the fence source into questions.
fn parse_quiz(source: &str) -> Result<Vec<QuizQuestion<'_>>, &'static str> {
    let mut questions = Vec::new();
    let mut current: Option<QuizQuestion<'_>> = None;

    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if let Some(question) = current.take() {
                questions.push(finish_question(question)?);
            }
        } else if line.starts_with([' ', '\t']) {
            current
                .as_mut()
                .and_then(|question| question.answers.last_mut())
                .ok_or(QUIZ_EXPLANATION_WITHOUT_ANSWER_ERROR)?
                .push_explanation(trimmed)?;
        } else if let Some(question) = current.as_mut() {
            question.answers.push(QuizAnswer {
                text: trimmed,
                ..QuizAnswer::default()
            });
        } else {
            current = Some(QuizQuestion {
                prompt: trimmed,
                answers: Vec::new(),
            });
        }
    }
    if let Some(question) = current {
        questions.push(finish_question(question)?);
    }
    if questions.is_empty() {
        return Err(QUIZ_EMPTY_ERROR);
    }
    Ok(questions)
}

/// Reject a question that has no answers.
fn finish_question(question: QuizQuestion<'_>) -> Result<QuizQuestion<'_>, &'static str> {
    if question.answers.is_empty() {
        return Err(QUIZ_QUESTION_NO_ANSWERS_ERROR);
    }
    Ok(question)
}

/// Split the `+` / `-` marker off a first explanation line.
fn split_marker(line: &str) -> Result<(bool, &str), &'static str> {
    if let Some(rest) = line.strip_prefix(QUIZ_EXPECT_SELECTED_MARKER) {
        Ok((true, rest.trim()))
    } else if let Some(rest) = line.strip_prefix(QUIZ_EXPECT_UNSELECTED_MARKER) {
        Ok((false, rest.trim()))
    } else {
        Err(QUIZ_EXPLANATION_MARKER_ERROR)
    }
}

/// Return `text` unless it is empty.
fn non_empty(text: &str) -> Option<&str> {
    (!text.is_empty()).then_some(text)
}

/// Return the interactive editor HTML for `questions`.
fn quiz_html(questions: &[QuizQuestion<'_>]) -> String {
    let mut html = String::from(r#"<div class="quiz">"#);
    for question in questions {
        let _ = write!(
            html,
            r#"<div class="quiz-question"><p class="quiz-prompt">{}</p><div class="quiz-answers">"#,
            escape_html(question.prompt)
        );
        for answer in &question.answers {
            push_answer_html(&mut html, answer);
        }
        let _ = write!(
            html,
            r#"</div><label class="quiz-reveal"><input type="checkbox" class="quiz-reveal-input"><span class="quiz-reveal-show">{QUIZ_REVEAL_BUTTON}</span><span class="quiz-reveal-hide">{QUIZ_HIDE_BUTTON}</span></label></div>"#
        );
    }
    html.push_str("</div>");
    html
}

/// Append one answer row: its checkbox, result marks, and explanations.
fn push_answer_html(html: &mut String, answer: &QuizAnswer<'_>) {
    let expect = if answer.expect_selected {
        "quiz-expect-selected"
    } else {
        "quiz-expect-unselected"
    };
    let _ = write!(
        html,
        r#"<div class="quiz-answer {expect}"><label class="quiz-choice"><input type="checkbox" class="quiz-input"><span class="quiz-answer-text">{}</span><span class="quiz-mark quiz-mark-correct">{QUIZ_CORRECT_MARK}</span><span class="quiz-mark quiz-mark-incorrect">{QUIZ_INCORRECT_MARK}</span></label>"#,
        escape_html(answer.text)
    );
    for (class, explanation) in [
        ("quiz-explain-selected", answer.explain_when_selected()),
        ("quiz-explain-unselected", answer.explain_when_unselected()),
    ] {
        if let Some(text) = explanation {
            let _ = write!(
                html,
                r#"<p class="quiz-explain {class}">{}</p>"#,
                escape_html(text)
            );
        }
    }
    html.push_str("</div>");
}

/// Return the static PDF answer key for `questions`.
///
/// Each answer is printed in bold, with the verdict on the line below so it is
/// read after the answer has been thought about: whether the answer should be
/// selected, followed by the explanation for that expected choice.
fn answer_key_html(questions: &[QuizQuestion<'_>]) -> String {
    let mut html = String::from(r#"<div class="quiz">"#);
    for (number, question) in questions.iter().enumerate() {
        let _ = write!(
            html,
            "<p><strong>{}. {}</strong></p>",
            number + 1,
            escape_html(question.prompt)
        );
        for answer in &question.answers {
            let label = if answer.expect_selected {
                QUIZ_PDF_CORRECT_LABEL
            } else {
                QUIZ_PDF_INCORRECT_LABEL
            };
            let _ = write!(
                html,
                "<p><strong>{}</strong><br>{label}",
                escape_html(answer.text)
            );
            if let Some(text) = answer.matched {
                let _ = write!(html, " <em>{}</em>", escape_html(text));
            }
            html.push_str("</p>");
        }
    }
    html.push_str("</div>");
    html
}

#[cfg(test)]
mod tests {
    use super::{parse_quiz, QuizAnswer, QuizRender};
    use crate::constants::{QUIZ_ERROR_CLASS, QUIZ_HELP_MANUAL};
    use crate::info_messages::{
        QUIZ_EMPTY_ERROR, QUIZ_EXPLANATION_MARKER_ERROR, QUIZ_EXPLANATION_WITHOUT_ANSWER_ERROR,
        QUIZ_PDF_CORRECT_LABEL, QUIZ_PDF_INCORRECT_LABEL, QUIZ_QUESTION_NO_ANSWERS_ERROR,
        QUIZ_TOO_MANY_EXPLANATIONS_ERROR,
    };
    use crate::rendering::renders::{Render, RenderPdf};

    const SAMPLE: &str = "Pick the primes
Two
  + Two is prime
  Two is the only even prime
Four
  - Four is 2 x 2
  Four has divisors other than 1 and itself
Nine

Second question
Yes
  + Right
";

    /// Return the quiz fence bodies of `markdown`.
    fn quiz_fences(markdown: &str) -> Vec<&str> {
        markdown
            .split("```quiz\n")
            .skip(1)
            .map(|rest| &rest[..rest.find("```").expect("closing fence")])
            .collect()
    }

    #[test]
    /// Assert questions split on blank lines and answers read their markers.
    fn parses_questions_answers_and_explanations() {
        let questions = parse_quiz(SAMPLE).expect("valid quiz");
        assert_eq!(questions.len(), 2);
        assert_eq!(questions[0].prompt, "Pick the primes");
        assert_eq!(questions[0].answers.len(), 3);

        let two = &questions[0].answers[0];
        assert_eq!(two.text, "Two");
        assert!(two.expect_selected);
        assert_eq!(two.explain_when_selected(), Some("Two is prime"));
        assert_eq!(
            two.explain_when_unselected(),
            Some("Two is the only even prime")
        );

        let four = &questions[0].answers[1];
        assert!(!four.expect_selected);
        assert_eq!(four.explain_when_unselected(), Some("Four is 2 x 2"));
        assert_eq!(
            four.explain_when_selected(),
            Some("Four has divisors other than 1 and itself")
        );

        assert_eq!(
            questions[0].answers[2],
            QuizAnswer {
                text: "Nine",
                ..QuizAnswer::default()
            },
            "an unmarked answer should not be selected and has no explanation"
        );
        assert_eq!(questions[1].answers[0].explain_when_unselected(), None);
    }

    #[test]
    /// Assert tab indentation and CRLF line endings parse like spaces and LF.
    fn accepts_tabs_and_crlf() {
        let questions = parse_quiz("Q\r\nA\r\n\t+ yes\r\n").expect("valid quiz");
        assert_eq!(questions[0].answers[0].matched, Some("yes"));
    }

    #[test]
    /// Assert malformed sources report why they failed.
    fn rejects_malformed_sources() {
        for (source, error) in [
            ("", QUIZ_EMPTY_ERROR),
            ("\n\n", QUIZ_EMPTY_ERROR),
            ("Question only\n", QUIZ_QUESTION_NO_ANSWERS_ERROR),
            ("Q\n  + floating\n", QUIZ_EXPLANATION_WITHOUT_ANSWER_ERROR),
            ("Q\nA\n  no marker\n", QUIZ_EXPLANATION_MARKER_ERROR),
            ("Q\nA\n  + a\n  b\n  c\n", QUIZ_TOO_MANY_EXPLANATIONS_ERROR),
        ] {
            assert_eq!(parse_quiz(source), Err(error), "{source:?}");
        }
    }

    #[test]
    /// Assert the editor HTML has a checkbox per answer and a reveal per question.
    fn renders_checkboxes_and_reveal_buttons() {
        let html = QuizRender.render(SAMPLE);
        assert_eq!(html.matches(r#"class="quiz-input""#).count(), 4, "{html}");
        assert_eq!(
            html.matches(r#"class="quiz-reveal-input""#).count(),
            2,
            "{html}"
        );
        assert_eq!(html.matches("quiz-expect-selected").count(), 2, "{html}");
        assert!(
            html.contains(r#"<p class="quiz-explain quiz-explain-selected">Two is prime</p>"#),
            "{html}"
        );
        assert!(
            html.contains(r#"<p class="quiz-explain quiz-explain-unselected">Four is 2 x 2</p>"#),
            "{html}"
        );
        assert!(!html.contains("<script"), "{html}");
        assert!(!html.contains(" id="), "quiz needs no element ids: {html}");
    }

    #[test]
    /// Assert quiz text is escaped so it cannot inject markup.
    fn escapes_quiz_text() {
        let html = QuizRender.render("<b>Q</b>\n<i>A</i>\n  + <script>\n");
        assert!(
            !html.contains("<b>") && !html.contains("<script>"),
            "{html}"
        );
        assert!(html.contains("&lt;script&gt;"), "{html}");
    }

    #[test]
    /// Assert a malformed quiz falls back to the error block.
    fn malformed_quiz_renders_error_block() {
        let html = QuizRender.render("Question only\n");
        assert!(
            html.contains(&format!(r#"class="{QUIZ_ERROR_CLASS}""#)),
            "{html}"
        );
    }

    #[test]
    /// Assert the PDF answer key labels answers without interactive inputs.
    fn pdf_renders_static_answer_key() {
        let html = QuizRender.render_pdf(SAMPLE);
        assert!(!html.contains("<input"), "{html}");
        assert!(
            html.contains("<strong>1. Pick the primes</strong>"),
            "{html}"
        );
        assert!(
            html.contains(&format!(
                "<p><strong>Two</strong><br>{QUIZ_PDF_CORRECT_LABEL} <em>Two is prime</em></p>"
            )),
            "{html}"
        );
        assert!(
            html.contains(&format!(
                "<p><strong>Nine</strong><br>{QUIZ_PDF_INCORRECT_LABEL}</p>"
            )),
            "{html}"
        );
    }

    #[test]
    /// Assert every quiz in the bundled manual renders without falling back.
    fn manual_quizzes_render() {
        let fences = quiz_fences(QUIZ_HELP_MANUAL);
        assert!(!fences.is_empty(), "manual has no quiz fence");
        for fence in fences {
            assert!(parse_quiz(fence).is_ok(), "{fence}");
        }
    }
}
