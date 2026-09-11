//! Night/day color theme. Night is the default.

use crate::constants::{
    ACCENT, BG_0, BG_1, BG_2, BG_3, BORDER, CODE, CODE_THEME, DAY_ACCENT, DAY_BG_0, DAY_BG_1,
    DAY_BG_2, DAY_BG_3, DAY_BORDER, DAY_CODE, DAY_CODE_THEME, DAY_CSS, DAY_TEXT, DAY_TEXT_FILL,
    DAY_TEXT_MUTED, DAY_TEXT_STROKE, DAY_TEXT_SUBTLE, NIGHT_CSS, TEXT, TEXT_FILL, TEXT_MUTED,
    TEXT_STROKE, TEXT_SUBTLE,
};
use std::cell::Cell;

thread_local! {
    static CURRENT_THEME: Cell<ColorTheme> = const { Cell::new(ColorTheme::Night) };
}

/// Night (dark background) or day (light background) palette.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorTheme {
    Night,
    Day,
}

/// Hex tokens and PDF paint operators for one color theme.
///
/// Fields match `--bg-0`…`--bg-3` and the other tokens in `night.css` /
/// `day.css`. Rendering currently reads a subset; the rest stay so the
/// numbered scale is complete.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct Palette {
    pub bg_0: &'static str,
    pub bg_1: &'static str,
    pub bg_2: &'static str,
    pub bg_3: &'static str,
    pub border: &'static str,
    pub text: &'static str,
    pub text_muted: &'static str,
    pub text_subtle: &'static str,
    pub accent: &'static str,
    pub code: &'static str,
    pub text_fill: &'static [u8],
    pub text_stroke: &'static [u8],
}

const NIGHT_PALETTE: Palette = Palette {
    bg_0: BG_0,
    bg_1: BG_1,
    bg_2: BG_2,
    bg_3: BG_3,
    border: BORDER,
    text: TEXT,
    text_muted: TEXT_MUTED,
    text_subtle: TEXT_SUBTLE,
    accent: ACCENT,
    code: CODE,
    text_fill: TEXT_FILL,
    text_stroke: TEXT_STROKE,
};

const DAY_PALETTE: Palette = Palette {
    bg_0: DAY_BG_0,
    bg_1: DAY_BG_1,
    bg_2: DAY_BG_2,
    bg_3: DAY_BG_3,
    border: DAY_BORDER,
    text: DAY_TEXT,
    text_muted: DAY_TEXT_MUTED,
    text_subtle: DAY_TEXT_SUBTLE,
    accent: DAY_ACCENT,
    code: DAY_CODE,
    text_fill: DAY_TEXT_FILL,
    text_stroke: DAY_TEXT_STROKE,
};

impl ColorTheme {
    /// Return the `data-theme` attribute value (`dark`/`light` for Bulma).
    pub fn as_attr(self) -> &'static str {
        match self {
            Self::Night => "dark",
            Self::Day => "light",
        }
    }

    /// Return the hex palette for this theme.
    pub fn palette(self) -> Palette {
        match self {
            Self::Night => NIGHT_PALETTE,
            Self::Day => DAY_PALETTE,
        }
    }

    /// Return the palette stylesheet for this theme.
    pub fn css(self) -> &'static str {
        match self {
            Self::Night => NIGHT_CSS,
            Self::Day => DAY_CSS,
        }
    }

    /// Return the syntect theme name for fenced code blocks.
    pub fn code_theme(self) -> &'static str {
        match self {
            Self::Night => CODE_THEME,
            Self::Day => DAY_CODE_THEME,
        }
    }
}

/// Return the color theme used by rendering and export.
pub fn current_theme() -> ColorTheme {
    CURRENT_THEME.with(Cell::get)
}

/// Set the color theme used by rendering and export.
pub fn set_current_theme(theme: ColorTheme) {
    CURRENT_THEME.with(|cell| cell.set(theme));
}

/// Write `data-theme` onto the document element so CSS palettes apply.
pub fn apply_document_theme(theme: ColorTheme) {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let Some(root) = document.document_element() else {
        return;
    };
    let _ = root.set_attribute("data-theme", theme.as_attr());
}

/// Fill export-template placeholders for the current color theme.
///
/// `{{THEME_CSS}}` is replaced before `{{THEME}}` because the latter is a
/// prefix of the former.
pub fn fill_export_template(template: &str, title: &str, body_html: &str) -> String {
    let theme = current_theme();
    template
        .replace("{{THEME_CSS}}", theme.css())
        .replace("{{PAGE_BG}}", theme.palette().bg_2)
        .replace("{{THEME}}", theme.as_attr())
        .replace("{{TITLE}}", title)
        .replace("{{BODY}}", body_html)
}

/// Restore `current_theme` when dropped. Used by tests that switch theme.
#[cfg(test)]
pub struct ThemeGuard {
    previous: ColorTheme,
}

#[cfg(test)]
impl ThemeGuard {
    /// Set `theme` as current and restore the previous theme on drop.
    pub fn set(theme: ColorTheme) -> Self {
        let previous = current_theme();
        set_current_theme(theme);
        Self { previous }
    }
}

#[cfg(test)]
impl Drop for ThemeGuard {
    fn drop(&mut self) {
        set_current_theme(self.previous);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        current_theme, fill_export_template, ColorTheme, ThemeGuard, DAY_PALETTE, NIGHT_PALETTE,
    };
    use crate::constants::{BG_0, BG_1, BG_2, BG_3, DAY_BG_0, DAY_BG_1, DAY_BG_2, DAY_BG_3, DAY_TEXT, NIGHT_CSS};

    #[test]
    /// Assert night is the default and day uses dark text on a light page.
    fn night_is_default_and_day_is_dark_on_light() {
        assert_eq!(current_theme(), ColorTheme::Night);
        assert_eq!(ColorTheme::Night.as_attr(), "dark");
        let night = ColorTheme::Night.palette();
        assert_eq!([night.bg_0, night.bg_1, night.bg_2, night.bg_3], [BG_0, BG_1, BG_2, BG_3]);
        assert_eq!(night.bg_2, NIGHT_PALETTE.bg_2);
        assert_eq!(ColorTheme::Day.as_attr(), "light");
        let day = ColorTheme::Day.palette();
        assert_eq!([day.bg_0, day.bg_1, day.bg_2, day.bg_3], [DAY_BG_0, DAY_BG_1, DAY_BG_2, DAY_BG_3]);
        assert_eq!(day.text, DAY_TEXT);
        assert_eq!(day.bg_2, DAY_BG_2);
        assert_eq!(DAY_PALETTE.text, "#4c4f69");
        assert_eq!(DAY_PALETTE.bg_2, "#eff1f5");

        let _guard = ThemeGuard::set(ColorTheme::Day);
        let html = fill_export_template(
            r#"<html data-theme="{{THEME}}"><style>{{THEME_CSS}}</style>{{PAGE_BG}}"#,
            "t",
            "b",
        );
        assert!(html.contains(r#"data-theme="light""#), "{html}");
        assert!(html.contains(DAY_TEXT), "{html}");
        assert!(html.contains(DAY_BG_2), "{html}");
        assert!(!html.contains(NIGHT_CSS), "{html}");
    }
}
