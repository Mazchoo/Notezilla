//! Applies mermaid `%%{init: ...}%%` theme variables that rusty-mermaid ignores.
//!
//! rusty-mermaid skips directives without reading them, and only when the
//! whole directive sits on one line. This module removes every directive,
//! single- or multi-line, and maps each mermaid `themeVariables` colour that
//! has a rusty-mermaid [`Theme`] counterpart onto that theme.
//!
//! Colours rusty-mermaid hardcodes (pie slices, git branches, gantt bars,
//! section palettes, C4 and packet fills) have no theme field and are not
//! overridable.

use rusty_mermaid::{detect, Color, DiagramKind, Theme};
use std::borrow::Cow;

/// Opening marker of a mermaid directive.
const DIRECTIVE_OPEN: &str = "%%{";
/// Closing marker of a mermaid directive.
const DIRECTIVE_CLOSE: &str = "}%%";
/// Object that holds the top-level mermaid theme variables.
const THEME_VARIABLES_KEY: &str = "themeVariables";

/// Sets one or more theme colours from a mermaid theme variable.
type ApplyColor = fn(&mut Theme, Color);

/// One mermaid theme variable and the theme fields it sets.
struct ThemeVariable {
    /// Object the variable sits in, e.g. `themeVariables` or `xyChart`.
    parent: &'static str,
    /// Variable name inside `parent`.
    name: &'static str,
    /// Diagrams the variable applies to; empty means every diagram.
    diagrams: &'static [DiagramKind],
    /// Writes the colour into the theme.
    apply: ApplyColor,
}

impl ThemeVariable {
    /// A `themeVariables` entry that applies to every diagram.
    const fn general(name: &'static str, apply: ApplyColor) -> Self {
        Self::only(&[], name, apply)
    }

    /// A `themeVariables` entry that applies to `diagrams` only.
    const fn only(diagrams: &'static [DiagramKind], name: &'static str, apply: ApplyColor) -> Self {
        Self::nested(THEME_VARIABLES_KEY, diagrams, name, apply)
    }

    /// An entry inside a nested object such as `themeVariables.xyChart`.
    const fn nested(
        parent: &'static str,
        diagrams: &'static [DiagramKind],
        name: &'static str,
        apply: ApplyColor,
    ) -> Self {
        Self {
            parent,
            name,
            diagrams,
            apply,
        }
    }

    /// Return whether this variable affects `diagram`; `None` is unrecognised.
    fn applies_to(&self, diagram: Option<DiagramKind>) -> bool {
        self.diagrams.is_empty() || diagram.is_some_and(|kind| self.diagrams.contains(&kind))
    }
}

/// Diagrams that draw notes from the theme note colours.
const NOTE_DIAGRAMS: &[DiagramKind] = &[DiagramKind::Sequence, DiagramKind::State];

/// Every supported mermaid theme variable.
///
/// Entries run from general to specific, and later entries win, so
/// `nodeBorder` overrides `primaryBorderColor` and `actorBkg` overrides
/// `mainBkg` whatever order the source lists them in. Variables that share a
/// target (e.g. every gantt text colour) share one theme field, because
/// rusty-mermaid draws those elements with the same colour.
const THEME_VARIABLES: &[ThemeVariable] = &[
    // Base palette, mirroring the derivations in mermaid's base theme.
    ThemeVariable::general("background", |t, c| t.background = c),
    ThemeVariable::general("primaryColor", |t, c| t.node_fill = c),
    ThemeVariable::general("primaryTextColor", |t, c| t.node_text = c),
    ThemeVariable::general("primaryBorderColor", |t, c| t.node_stroke = c),
    ThemeVariable::general("secondaryColor", |t, c| {
        t.edge_label_bg = c;
        t.activation_fill = c;
    }),
    ThemeVariable::general("tertiaryColor", |t, c| t.subgraph_fill = c),
    ThemeVariable::general("tertiaryBorderColor", |t, c| t.subgraph_stroke = c),
    ThemeVariable::general("tertiaryTextColor", |t, c| t.subgraph_label = c),
    ThemeVariable::general("lineColor", |t, c| t.edge_stroke = c),
    ThemeVariable::general("textColor", |t, c| {
        t.edge_label_text = c;
        t.muted_text = c;
    }),
    // Flowchart-named variables mermaid also uses as defaults elsewhere.
    ThemeVariable::general("mainBkg", |t, c| t.node_fill = c),
    ThemeVariable::general("nodeBkg", |t, c| t.node_fill = c),
    ThemeVariable::general("nodeBorder", |t, c| t.node_stroke = c),
    ThemeVariable::general("nodeTextColor", |t, c| t.node_text = c),
    ThemeVariable::general("clusterBkg", |t, c| t.subgraph_fill = c),
    ThemeVariable::general("clusterBorder", |t, c| t.subgraph_stroke = c),
    ThemeVariable::general("titleColor", |t, c| t.subgraph_label = c),
    ThemeVariable::general("defaultLinkColor", |t, c| t.edge_stroke = c),
    ThemeVariable::general("edgeLabelBackground", |t, c| t.edge_label_bg = c),
    // Notes.
    ThemeVariable::only(NOTE_DIAGRAMS, "noteBkgColor", |t, c| t.note_fill = c),
    ThemeVariable::only(NOTE_DIAGRAMS, "noteBorderColor", |t, c| t.note_stroke = c),
    ThemeVariable::only(NOTE_DIAGRAMS, "noteTextColor", |t, c| t.note_text = c),
    // Sequence.
    ThemeVariable::only(&[DiagramKind::Sequence], "actorBkg", |t, c| t.node_fill = c),
    ThemeVariable::only(&[DiagramKind::Sequence], "actorBorder", |t, c| {
        t.node_stroke = c;
    }),
    ThemeVariable::only(&[DiagramKind::Sequence], "actorTextColor", |t, c| {
        t.node_text = c;
    }),
    ThemeVariable::only(&[DiagramKind::Sequence], "actorLineColor", |t, c| {
        t.lifeline_stroke = c;
    }),
    ThemeVariable::only(&[DiagramKind::Sequence], "signalColor", |t, c| {
        t.edge_stroke = c;
    }),
    ThemeVariable::only(&[DiagramKind::Sequence], "signalTextColor", |t, c| {
        t.edge_label_text = c;
    }),
    // rusty-mermaid fills the loop/alt label tab with its border colour.
    ThemeVariable::only(&[DiagramKind::Sequence], "labelBoxBkgColor", |t, c| {
        t.subgraph_stroke = c;
    }),
    ThemeVariable::only(&[DiagramKind::Sequence], "labelBoxBorderColor", |t, c| {
        t.subgraph_stroke = c;
    }),
    ThemeVariable::only(&[DiagramKind::Sequence], "labelTextColor", |t, c| {
        t.subgraph_label = c;
    }),
    ThemeVariable::only(&[DiagramKind::Sequence], "loopTextColor", |t, c| {
        t.subgraph_label = c;
    }),
    ThemeVariable::only(&[DiagramKind::Sequence], "activationBkgColor", |t, c| {
        t.activation_fill = c;
    }),
    ThemeVariable::only(&[DiagramKind::Sequence], "activationBorderColor", |t, c| {
        t.activation_stroke = c;
    }),
    // State.
    ThemeVariable::only(&[DiagramKind::State], "stateBkg", |t, c| t.node_fill = c),
    ThemeVariable::only(&[DiagramKind::State], "stateBorder", |t, c| {
        t.node_stroke = c
    }),
    ThemeVariable::only(&[DiagramKind::State], "stateLabelColor", |t, c| {
        t.node_text = c;
        t.composite_label = c;
    }),
    ThemeVariable::only(&[DiagramKind::State], "transitionColor", |t, c| {
        t.edge_stroke = c;
    }),
    ThemeVariable::only(&[DiagramKind::State], "transitionLabelColor", |t, c| {
        t.edge_label_text = c;
    }),
    ThemeVariable::only(&[DiagramKind::State], "specialStateColor", |t, c| {
        t.start_fill = c;
        t.end_inner_fill = c;
    }),
    ThemeVariable::only(&[DiagramKind::State], "innerEndBackground", |t, c| {
        t.end_inner_fill = c;
    }),
    ThemeVariable::only(&[DiagramKind::State], "compositeBackground", |t, c| {
        t.composite_fill = c;
    }),
    ThemeVariable::only(&[DiagramKind::State], "compositeBorder", |t, c| {
        t.composite_stroke = c;
    }),
    // Class.
    ThemeVariable::only(&[DiagramKind::Class], "classText", |t, c| t.node_text = c),
    // Entity relationship: even attribute rows share the entity fill.
    ThemeVariable::only(
        &[DiagramKind::Er],
        "attributeBackgroundColorEven",
        |t, c| {
            t.node_fill = c;
        },
    ),
    ThemeVariable::only(&[DiagramKind::Er], "attributeBackgroundColorOdd", |t, c| {
        t.composite_fill = c;
    }),
    // Requirement.
    ThemeVariable::only(
        &[DiagramKind::Requirement],
        "requirementBackground",
        |t, c| {
            t.node_fill = c;
        },
    ),
    ThemeVariable::only(
        &[DiagramKind::Requirement],
        "requirementBorderColor",
        |t, c| {
            t.node_stroke = c;
        },
    ),
    ThemeVariable::only(
        &[DiagramKind::Requirement],
        "requirementTextColor",
        |t, c| {
            t.node_text = c;
        },
    ),
    ThemeVariable::only(&[DiagramKind::Requirement], "relationColor", |t, c| {
        t.edge_stroke = c;
    }),
    ThemeVariable::only(
        &[DiagramKind::Requirement],
        "relationLabelBackground",
        |t, c| {
            t.edge_label_bg = c;
        },
    ),
    ThemeVariable::only(&[DiagramKind::Requirement], "relationLabelColor", |t, c| {
        t.edge_label_text = c;
    }),
    // Gantt: title, axis and task text share one colour.
    ThemeVariable::only(&[DiagramKind::Gantt], "titleColor", |t, c| t.node_text = c),
    ThemeVariable::only(&[DiagramKind::Gantt], "taskTextColor", |t, c| {
        t.node_text = c
    }),
    ThemeVariable::only(&[DiagramKind::Gantt], "taskTextOutsideColor", |t, c| {
        t.node_text = c;
    }),
    ThemeVariable::only(&[DiagramKind::Gantt], "gridColor", |t, c| t.grid_stroke = c),
    // Git.
    ThemeVariable::only(&[DiagramKind::GitGraph], "commitLabelColor", |t, c| {
        t.muted_text = c
    }),
    ThemeVariable::only(&[DiagramKind::GitGraph], "tagLabelBackground", |t, c| {
        t.note_fill = c
    }),
    ThemeVariable::only(&[DiagramKind::GitGraph], "tagLabelColor", |t, c| {
        t.node_text = c
    }),
    // Journey.
    ThemeVariable::only(&[DiagramKind::UserJourney], "faceColor", |t, c| {
        t.face_fill = c
    }),
    // Pie: title and legend text share one colour.
    ThemeVariable::only(&[DiagramKind::Pie], "pieTitleTextColor", |t, c| {
        t.node_text = c
    }),
    ThemeVariable::only(&[DiagramKind::Pie], "pieLegendTextColor", |t, c| {
        t.node_text = c;
    }),
    // Quadrant: all text shares one colour, as do all borders.
    ThemeVariable::only(&[DiagramKind::Quadrant], "quadrantTitleFill", |t, c| {
        t.node_text = c;
    }),
    ThemeVariable::only(&[DiagramKind::Quadrant], "quadrantXAxisTextFill", |t, c| {
        t.node_text = c;
    }),
    ThemeVariable::only(&[DiagramKind::Quadrant], "quadrantYAxisTextFill", |t, c| {
        t.node_text = c;
    }),
    ThemeVariable::only(&[DiagramKind::Quadrant], "quadrantPointTextFill", |t, c| {
        t.node_text = c;
    }),
    ThemeVariable::only(
        &[DiagramKind::Quadrant],
        "quadrantInternalBorderStrokeFill",
        |t, c| t.grid_stroke = c,
    ),
    ThemeVariable::only(
        &[DiagramKind::Quadrant],
        "quadrantExternalBorderStrokeFill",
        |t, c| t.grid_stroke = c,
    ),
    // XY chart (`themeVariables.xyChart`): text, axis lines and ticks.
    ThemeVariable::nested(
        "xyChart",
        &[DiagramKind::XyChart],
        "backgroundColor",
        |t, c| {
            t.background = c;
        },
    ),
    ThemeVariable::nested("xyChart", &[DiagramKind::XyChart], "titleColor", |t, c| {
        t.node_text = c;
    }),
    ThemeVariable::nested(
        "xyChart",
        &[DiagramKind::XyChart],
        "xAxisLabelColor",
        |t, c| {
            t.node_text = c;
        },
    ),
    ThemeVariable::nested(
        "xyChart",
        &[DiagramKind::XyChart],
        "xAxisTitleColor",
        |t, c| {
            t.node_text = c;
        },
    ),
    ThemeVariable::nested(
        "xyChart",
        &[DiagramKind::XyChart],
        "yAxisLabelColor",
        |t, c| {
            t.node_text = c;
        },
    ),
    ThemeVariable::nested(
        "xyChart",
        &[DiagramKind::XyChart],
        "yAxisTitleColor",
        |t, c| {
            t.node_text = c;
        },
    ),
    ThemeVariable::nested(
        "xyChart",
        &[DiagramKind::XyChart],
        "xAxisLineColor",
        |t, c| {
            t.edge_stroke = c;
        },
    ),
    ThemeVariable::nested(
        "xyChart",
        &[DiagramKind::XyChart],
        "xAxisTickColor",
        |t, c| {
            t.edge_stroke = c;
        },
    ),
    ThemeVariable::nested(
        "xyChart",
        &[DiagramKind::XyChart],
        "yAxisLineColor",
        |t, c| {
            t.edge_stroke = c;
        },
    ),
    ThemeVariable::nested(
        "xyChart",
        &[DiagramKind::XyChart],
        "yAxisTickColor",
        |t, c| {
            t.edge_stroke = c;
        },
    ),
    // Radar (`themeVariables.radar`): axes and graticule share one colour.
    ThemeVariable::nested("radar", &[DiagramKind::Radar], "axisColor", |t, c| {
        t.grid_stroke = c;
    }),
    ThemeVariable::nested("radar", &[DiagramKind::Radar], "graticuleColor", |t, c| {
        t.grid_stroke = c;
    }),
];

/// Diagrams whose node labels rusty-mermaid recolours for contrast.
const CONTRAST_LABEL_DIAGRAMS: &[DiagramKind] = &[DiagramKind::Flowchart, DiagramKind::State];

/// Diagram source with its directives applied.
pub(super) struct Directives<'a> {
    /// Diagram text without directives.
    pub(super) body: Cow<'a, str>,
    /// Node label colour the directives set explicitly, when rusty-mermaid
    /// would otherwise replace it with white or black for contrast.
    pub(super) node_label_color: Option<Color>,
}

/// Remove every `%%{ ... }%%` directive from `source` and apply its colours.
///
/// The returned body borrows `source` when it has no directives. Unknown
/// variables and unparseable colours are ignored.
pub(super) fn apply_directives<'a>(source: &'a str, theme: &mut Theme) -> Directives<'a> {
    if !source.lines().any(is_directive_start) {
        return Directives {
            body: Cow::Borrowed(source),
            node_label_color: None,
        };
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
    // The same detection rusty-mermaid uses to pick the diagram renderer.
    let diagram = detect(&body);
    let default_node_text = theme.node_text;
    apply_theme_variables(&directives, diagram, theme);
    let node_label_color = (diagram.is_some_and(|kind| CONTRAST_LABEL_DIAGRAMS.contains(&kind))
        && theme.node_text != default_node_text)
        .then_some(theme.node_text);
    Directives {
        body: Cow::Owned(body),
        node_label_color,
    }
}

/// Return whether `line` opens a directive.
fn is_directive_start(line: &str) -> bool {
    line.trim_start().starts_with(DIRECTIVE_OPEN)
}

/// Apply every variable in `directives` that affects `diagram` to `theme`.
fn apply_theme_variables(directives: &str, diagram: Option<DiagramKind>, theme: &mut Theme) {
    let pairs = string_pairs(directives);
    for variable in THEME_VARIABLES {
        if !variable.applies_to(diagram) {
            continue;
        }
        // The last occurrence wins, as it does in a JavaScript object literal.
        let value = pairs
            .iter()
            .rev()
            .find(|pair| pair.parent == variable.parent && pair.key == variable.name);
        if let Some(color) = value.and_then(|pair| Color::from_css(pair.value)) {
            (variable.apply)(theme, color);
        }
    }
}

/// A `key: 'value'` pair and the key of the object that holds it.
#[derive(Debug, PartialEq, Eq)]
struct StringPair<'a> {
    /// Key of the enclosing object, or empty at the top level.
    parent: &'a str,
    key: &'a str,
    value: &'a str,
}

/// Return each `key: 'value'` pair whose value is a quoted string.
///
/// Keys may be bare or single/double quoted, matching the relaxed
/// object-literal syntax mermaid accepts in `init` directives.
fn string_pairs(text: &str) -> Vec<StringPair<'_>> {
    let mut pairs = Vec::new();
    let mut parents: Vec<&str> = Vec::new();
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
            match key.filter(|_| after_colon) {
                Some(name) => {
                    pairs.push(StringPair {
                        parent: parents.last().copied().unwrap_or_default(),
                        key: name,
                        value: quoted,
                    });
                    key = None;
                    after_colon = false;
                }
                None => key = Some(quoted),
            }
            continue;
        }
        if is_word_char(ch) {
            let end = rest.find(|c| !is_word_char(c)).unwrap_or(rest.len());
            key = Some(&rest[..end]);
            after_colon = false;
            rest = &rest[end..];
            continue;
        }
        match ch {
            ':' => after_colon = key.is_some(),
            '{' => {
                parents.push(key.filter(|_| after_colon).unwrap_or_default());
                key = None;
                after_colon = false;
            }
            '}' => {
                parents.pop();
                key = None;
                after_colon = false;
            }
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

/// Return whether `ch` can appear in a bare object key.
fn is_word_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

#[cfg(test)]
mod tests {
    use super::{apply_directives, string_pairs, StringPair};
    use rusty_mermaid::{Color, Theme};
    use std::borrow::Cow;

    /// Multi-line `init` directive with base-theme colours.
    const MULTILINE: &str = "%%{init: {\n  'theme': 'base',\n  'themeVariables': {\n    \
                             'primaryColor': '#4f46e5',\n    'primaryTextColor': '#ffffff',\n    \
                             'primaryBorderColor': '#312e81',\n    'lineColor': '#f59e0b',\n    \
                             'secondaryColor': '#ec4899',\n    \
                             'tertiaryColor': '#f3f4f6'\n  }\n}}%%\nflowchart LR\n    A --> B\n";

    /// Apply `variables` (the body of `themeVariables`) to a dark theme.
    fn themed(variables: &str, diagram: &str) -> Theme {
        let mut theme = Theme::dark();
        let source = format!("%%{{init: {{'themeVariables': {{{variables}}}}}}}%%\n{diagram}\n");
        apply_directives(&source, &mut theme);
        theme
    }

    /// Return the colour for `#rrggbb`.
    fn hex(css: &str) -> Color {
        Color::from_css(css).unwrap()
    }

    #[test]
    /// Assert a multi-line directive is removed and its colours applied.
    fn multiline_directive_sets_theme_colours() {
        let mut theme = Theme::dark();
        let directives = apply_directives(MULTILINE, &mut theme);
        assert_eq!(directives.body, "flowchart LR\n    A --> B\n");
        assert_eq!(directives.node_label_color, Some(Color::WHITE));
        assert_eq!(theme.node_fill, hex("#4f46e5"));
        assert_eq!(theme.node_text, Color::WHITE);
        assert_eq!(theme.node_stroke, hex("#312e81"));
        assert_eq!(theme.edge_stroke, hex("#f59e0b"));
        assert_eq!(theme.edge_label_bg, hex("#ec4899"));
        assert_eq!(theme.subgraph_fill, hex("#f3f4f6"));
    }

    #[test]
    /// Assert a single-line directive with double quotes is applied.
    fn single_line_directive_sets_theme_colours() {
        let mut theme = Theme::dark();
        let source = "%%{init: {\"themeVariables\": {\"lineColor\": \"red\"}}}%%\ngraph LR\n";
        let directives = apply_directives(source, &mut theme);
        assert_eq!(directives.body, "graph LR\n");
        assert_eq!(directives.node_label_color, None);
        assert_eq!(theme.edge_stroke, Color::rgb(255, 0, 0));
    }

    #[test]
    /// Assert a specific variable beats its general one in either order.
    fn specific_variable_overrides_general() {
        let theme = themed(
            "'nodeBorder': '#111111', 'primaryBorderColor': '#222222'",
            "graph LR",
        );
        assert_eq!(theme.node_stroke, hex("#111111"));
    }

    #[test]
    /// Assert diagram-specific variables only affect their own diagram.
    fn diagram_variables_are_scoped() {
        let sequence = themed("'actorBkg': '#123456'", "sequenceDiagram");
        assert_eq!(sequence.node_fill, hex("#123456"));
        let flowchart = themed("'actorBkg': '#123456'", "flowchart LR");
        assert_eq!(flowchart.node_fill, Theme::dark().node_fill);
    }

    #[test]
    /// Assert sequence variables reach actors, lifelines, messages and frames.
    fn sequence_variables_apply() {
        let theme = themed(
            "'actorBorder': '#000001', 'actorTextColor': '#000002', \
             'actorLineColor': '#000003', 'signalColor': '#000004', \
             'signalTextColor': '#000005', 'labelBoxBorderColor': '#000006', \
             'loopTextColor': '#000007', 'activationBkgColor': '#000008', \
             'activationBorderColor': '#000009', 'noteBkgColor': '#00000a'",
            "sequenceDiagram",
        );
        assert_eq!(theme.node_stroke, hex("#000001"));
        assert_eq!(theme.node_text, hex("#000002"));
        assert_eq!(theme.lifeline_stroke, hex("#000003"));
        assert_eq!(theme.edge_stroke, hex("#000004"));
        assert_eq!(theme.edge_label_text, hex("#000005"));
        assert_eq!(theme.subgraph_stroke, hex("#000006"));
        assert_eq!(theme.subgraph_label, hex("#000007"));
        assert_eq!(theme.activation_fill, hex("#000008"));
        assert_eq!(theme.activation_stroke, hex("#000009"));
        assert_eq!(theme.note_fill, hex("#00000a"));
    }

    #[test]
    /// Assert state variables reach states, transitions and composites.
    fn state_variables_apply() {
        let theme = themed(
            "'stateBkg': '#000001', 'transitionColor': '#000002', \
             'specialStateColor': '#000003', 'innerEndBackground': '#000004', \
             'compositeBackground': '#000005', 'compositeBorder': '#000006', \
             'stateLabelColor': '#000007'",
            "stateDiagram-v2",
        );
        assert_eq!(theme.node_fill, hex("#000001"));
        assert_eq!(theme.edge_stroke, hex("#000002"));
        assert_eq!(theme.start_fill, hex("#000003"));
        assert_eq!(theme.end_inner_fill, hex("#000004"));
        assert_eq!(theme.composite_fill, hex("#000005"));
        assert_eq!(theme.composite_stroke, hex("#000006"));
        assert_eq!(theme.composite_label, hex("#000007"));
    }

    #[test]
    /// Assert nested `xyChart` variables apply only inside that object.
    fn nested_xychart_variables_apply() {
        let theme = themed(
            "'xyChart': {'titleColor': '#000001', 'xAxisLineColor': '#000002'}",
            "xychart-beta",
        );
        assert_eq!(theme.node_text, hex("#000001"));
        assert_eq!(theme.edge_stroke, hex("#000002"));
        // A top-level `titleColor` is the flowchart subgraph title, not the chart.
        let top_level = themed("'titleColor': '#000003'", "xychart-beta");
        assert_eq!(top_level.node_text, Theme::dark().node_text);
    }

    #[test]
    /// Assert unknown variables and bad colours leave the theme unchanged.
    fn unknown_and_invalid_values_are_ignored() {
        let theme = themed(
            "'primaryColor': 'not-a-colour', 'fontFamily': 'serif'",
            "graph LR",
        );
        assert_eq!(theme.node_fill, Theme::dark().node_fill);
    }

    #[test]
    /// Assert source without directives is borrowed untouched.
    fn source_without_directive_is_borrowed() {
        let mut theme = Theme::dark();
        let source = "%% a comment\ngraph LR\n    A --> B\n";
        assert!(matches!(
            apply_directives(source, &mut theme).body,
            Cow::Borrowed(s) if s == source
        ));
    }

    #[test]
    /// Assert pairs record their enclosing object and skip non-string values.
    fn pairs_track_parent_objects() {
        let pairs = string_pairs("{init: {theme: 'base', v: {'a': \"b\", n: 3}, 'c': 'd'}}");
        let pair = |parent, key, value| StringPair { parent, key, value };
        assert_eq!(
            pairs,
            [
                pair("init", "theme", "base"),
                pair("v", "a", "b"),
                pair("init", "c", "d"),
            ]
        );
    }
}
