//! Applies mermaid `%%{init: ...}%%` theme variables that rusty-mermaid ignores.
//!
//! rusty-mermaid skips directives without reading them, and only when the
//! whole directive sits on one line. This module removes every directive,
//! single- or multi-line, and maps the `themeVariables` colours it understands
//! onto the rusty-mermaid [`Theme`].

use rusty_mermaid::{Color, Theme};
use std::borrow::Cow;

/// Opening marker of a mermaid directive.
const DIRECTIVE_OPEN: &str = "%%{";
/// Closing marker of a mermaid directive.
const DIRECTIVE_CLOSE: &str = "}%%";

/// Sets one theme colour from a mermaid theme variable.
type ApplyColor = fn(&mut Theme, Color);

/// Mermaid theme variables and the theme fields they set.
///
/// General variables come before the specific ones that refine them, so
/// `nodeBorder` wins over `primaryBorderColor` whatever the source order.
const THEME_VARIABLES: &[(&str, ApplyColor)] = &[
    ("primaryColor", |t, c| t.node_fill = c),
    ("primaryTextColor", |t, c| t.node_text = c),
    ("primaryBorderColor", |t, c| t.node_stroke = c),
    ("lineColor", |t, c| t.edge_stroke = c),
    ("tertiaryColor", |t, c| t.subgraph_fill = c),
    ("textColor", |t, c| {
        t.edge_label_text = c;
        t.subgraph_label = c;
    }),
    ("mainBkg", |t, c| t.node_fill = c),
    ("nodeBorder", |t, c| t.node_stroke = c),
    ("nodeTextColor", |t, c| t.node_text = c),
    ("clusterBkg", |t, c| t.subgraph_fill = c),
    ("clusterBorder", |t, c| t.subgraph_stroke = c),
    ("titleColor", |t, c| t.subgraph_label = c),
    ("edgeLabelBackground", |t, c| t.edge_label_bg = c),
    ("noteBkgColor", |t, c| t.note_fill = c),
    ("noteBorderColor", |t, c| t.note_stroke = c),
    ("noteTextColor", |t, c| t.note_text = c),
];

/// Remove every `%%{ ... }%%` directive from `source` and apply its colours.
///
/// Returns the diagram text without directives, borrowing `source` when it
/// has none. Unknown variables and unparseable colours are ignored.
pub(super) fn apply_directives<'a>(source: &'a str, theme: &mut Theme) -> Cow<'a, str> {
    if !source.lines().any(is_directive_start) {
        return Cow::Borrowed(source);
    }
    let mut body = String::with_capacity(source.len());
    let mut directives = String::new();
    let mut in_directive = false;
    for line in source.lines() {
        if in_directive || is_directive_start(line) {
            directives.push_str(line);
            directives.push('\n');
            in_directive = !line.trim_end().ends_with(DIRECTIVE_CLOSE);
            continue;
        }
        body.push_str(line);
        body.push('\n');
    }
    apply_theme_variables(&directives, theme);
    Cow::Owned(body)
}

/// Return whether `line` opens a directive.
fn is_directive_start(line: &str) -> bool {
    line.trim_start().starts_with(DIRECTIVE_OPEN)
}

/// Apply every known colour variable in `directives` to `theme`.
fn apply_theme_variables(directives: &str, theme: &mut Theme) {
    let pairs = string_pairs(directives);
    for (name, apply) in THEME_VARIABLES {
        // The last occurrence wins, as it does in a JavaScript object literal.
        let value = pairs.iter().rev().find(|(key, _)| key == name);
        if let Some(color) = value.and_then(|(_, value)| Color::from_css(value)) {
            apply(theme, color);
        }
    }
}

/// Return each `key: 'value'` pair whose value is a quoted string.
///
/// Keys may be bare or single/double quoted, matching the relaxed
/// object-literal syntax mermaid accepts in `init` directives.
fn string_pairs(text: &str) -> Vec<(&str, &str)> {
    let mut pairs = Vec::new();
    let mut key: Option<&str> = None;
    let mut after_colon = false;
    let mut rest = text;
    while let Some(ch) = rest.chars().next() {
        if ch == '\'' || ch == '"' {
            let Some(end) = rest[1..].find(ch) else {
                break;
            };
            let quoted = &rest[1..=end];
            rest = &rest[end + 2..];
            match (after_colon, key) {
                (true, Some(name)) => {
                    pairs.push((name, quoted));
                    key = None;
                    after_colon = false;
                }
                _ => key = Some(quoted),
            }
            continue;
        }
        if ch.is_ascii_alphanumeric() || ch == '_' {
            let end = rest
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(rest.len());
            key = Some(&rest[..end]);
            after_colon = false;
            rest = &rest[end..];
            continue;
        }
        match ch {
            ':' => after_colon = key.is_some(),
            c if c.is_whitespace() => {}
            _ => {
                key = None;
                after_colon = false;
            }
        }
        rest = &rest[ch.len_utf8()..];
    }
    pairs
}

#[cfg(test)]
mod tests {
    use super::{apply_directives, string_pairs};
    use rusty_mermaid::{Color, Theme};
    use std::borrow::Cow;

    /// Multi-line `init` directive with base-theme colours.
    const MULTILINE: &str = "%%{init: {\n  'theme': 'base',\n  'themeVariables': {\n    \
                             'primaryColor': '#4f46e5',\n    'primaryTextColor': '#ffffff',\n    \
                             'primaryBorderColor': '#312e81',\n    'lineColor': '#f59e0b',\n    \
                             'tertiaryColor': '#f3f4f6'\n  }\n}}%%\nflowchart LR\n    A --> B\n";

    #[test]
    /// Assert a multi-line directive is removed and its colours applied.
    fn multiline_directive_sets_theme_colours() {
        let mut theme = Theme::dark();
        let body = apply_directives(MULTILINE, &mut theme);
        assert_eq!(body, "flowchart LR\n    A --> B\n");
        assert_eq!(theme.node_fill, Color::rgb(0x4f, 0x46, 0xe5));
        assert_eq!(theme.node_text, Color::WHITE);
        assert_eq!(theme.node_stroke, Color::rgb(0x31, 0x2e, 0x81));
        assert_eq!(theme.edge_stroke, Color::rgb(0xf5, 0x9e, 0x0b));
        assert_eq!(theme.subgraph_fill, Color::rgb(0xf3, 0xf4, 0xf6));
    }

    #[test]
    /// Assert a single-line directive with double quotes is applied.
    fn single_line_directive_sets_theme_colours() {
        let mut theme = Theme::dark();
        let source = "%%{init: {\"themeVariables\": {\"lineColor\": \"red\"}}}%%\ngraph LR\n";
        assert_eq!(apply_directives(source, &mut theme), "graph LR\n");
        assert_eq!(theme.edge_stroke, Color::rgb(255, 0, 0));
    }

    #[test]
    /// Assert a specific variable beats its general one in either order.
    fn specific_variable_overrides_general() {
        let mut theme = Theme::dark();
        let source = "%%{init: {'themeVariables': {'nodeBorder': '#111111', \
                      'primaryBorderColor': '#222222'}}}%%\ngraph LR\n";
        apply_directives(source, &mut theme);
        assert_eq!(theme.node_stroke, Color::rgb(0x11, 0x11, 0x11));
    }

    #[test]
    /// Assert unknown variables and bad colours leave the theme unchanged.
    fn unknown_and_invalid_values_are_ignored() {
        let mut theme = Theme::dark();
        let source = "%%{init: {'themeVariables': {'primaryColor': 'not-a-colour', \
                      'fontFamily': 'serif'}}}%%\ngraph LR\n";
        apply_directives(source, &mut theme);
        assert_eq!(theme.node_fill, Theme::dark().node_fill);
    }

    #[test]
    /// Assert source without directives is borrowed untouched.
    fn source_without_directive_is_borrowed() {
        let mut theme = Theme::dark();
        let source = "%% a comment\ngraph LR\n    A --> B\n";
        assert!(matches!(
            apply_directives(source, &mut theme),
            Cow::Borrowed(s) if s == source
        ));
    }

    #[test]
    /// Assert bare and quoted keys pair with quoted values only.
    fn pairs_accept_bare_and_quoted_keys() {
        let pairs = string_pairs("{init: {theme: 'base', \"a\": \"b\", n: 3, 'c': 'd'}}");
        assert_eq!(pairs, [("theme", "base"), ("a", "b"), ("c", "d")]);
    }
}
