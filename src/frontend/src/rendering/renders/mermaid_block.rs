//! Renders block diagrams with the shapes and labels rusty-mermaid loses.
//!
//! rusty-mermaid's block parser tests for a plain `[` before `[(`, `[[`,
//! `[/` and `[\`, so those nodes become rectangles whose label keeps the
//! inner brackets; `([` and `>` nodes also come out with broken ids. Node
//! lines are rewritten to the plain `id["label"]` form before parsing and
//! their real shapes restored afterwards. Labels the renderer then drops
//! for being too wide are put back, fitted to their shape.

use super::mermaid_fit::{fit_label, label_room, push_element, shape_bounds};
use rusty_mermaid::{Point, Primitive, Scene, TextStyle, Theme};
use rusty_mermaid_diagrams::block::ir::{BlockDiagram, BlockShape};
use rusty_mermaid_diagrams::block::parser::parse;
use rusty_mermaid_diagrams::block::to_scene;
use rusty_mermaid_diagrams::ParseError;
use std::borrow::Cow;
use std::collections::HashMap;
use std::f64::consts::FRAC_1_SQRT_2;

/// Node shape delimiters, longest opener first so `[(` wins over `[`.
const SHAPES: [(&str, &str, BlockShape); 13] = [
    ("[(", ")]", BlockShape::Cylinder),
    ("([", "])", BlockShape::Stadium),
    ("((", "))", BlockShape::Circle),
    ("{{", "}}", BlockShape::Hexagon),
    ("[[", "]]", BlockShape::Rect),
    ("[/", "/]", BlockShape::Rect),
    ("[/", "\\]", BlockShape::Rect),
    ("[\\", "\\]", BlockShape::Rect),
    ("[\\", "/]", BlockShape::Rect),
    ("(", ")", BlockShape::Round),
    ("{", "}", BlockShape::Diamond),
    ("[", "]", BlockShape::Rect),
    (">", "]", BlockShape::Rect),
];

/// Edge arrows; lines holding one are edges, not node definitions.
const ARROWS: [&str; 3] = ["-->", "==>", "-."];

/// Characters that open a node's shape after its id.
const SHAPE_OPENERS: [char; 4] = ['[', '(', '{', '>'];

/// Parse and lay out the block diagram `source` with its node shapes intact.
pub(super) fn render_block(source: &str, theme: &Theme) -> Result<Scene, ParseError> {
    Ok(to_scene(&parse_blocks(source)?, theme))
}

/// Return `scene` with every block label rusty-mermaid dropped put back.
///
/// Block shapes are matched to the blocks of `source` in order. The scene
/// is returned unchanged when `source` does not parse or the shapes and
/// blocks do not line up.
pub(super) fn label_blocks(scene: Scene, source: &str, theme: &Theme) -> Scene {
    let Ok(diagram) = parse_blocks(source) else {
        return scene;
    };
    let mut blocks = diagram
        .blocks
        .iter()
        .filter(|block| block.shape != BlockShape::Space);
    let elements = scene.elements();
    let shapes = elements
        .iter()
        .filter(|element| is_block_shape(&element.primitive))
        .count();
    if shapes != blocks.clone().count() {
        return scene;
    }
    let style = TextStyle {
        font_size: theme.font_size_node,
        fill: Some(theme.node_text),
        ..Default::default()
    };
    let mut labelled = Scene::new(scene.width, scene.height);
    for (index, element) in elements.iter().enumerate() {
        push_element(&mut labelled, element);
        if !is_block_shape(&element.primitive) {
            continue;
        }
        let Some(block) = blocks.next() else {
            continue;
        };
        let has_label = matches!(
            elements.get(index + 1).map(|next| &next.primitive),
            Some(Primitive::Text { .. })
        );
        if has_label || block.label.is_empty() {
            continue;
        }
        let Some(bbox) = shape_bounds(&element.primitive) else {
            continue;
        };
        let (width, height) = shape_room(block.shape, label_room(&bbox));
        let fitted = fit_label(&block.label, &style, width, height);
        labelled.push(fitted.at(Point::new(bbox.x, bbox.y)));
    }
    labelled
}

/// Return whether `primitive` is a block shape rather than an edge or label.
fn is_block_shape(primitive: &Primitive) -> bool {
    matches!(
        primitive,
        Primitive::Rect { .. } | Primitive::Circle { .. } | Primitive::Polygon { .. }
    )
}

/// Narrow the padded `(width, height)` room of a shape's box to its inside.
fn shape_room(shape: BlockShape, (width, height): (f64, f64)) -> (f64, f64) {
    match shape {
        BlockShape::Diamond => (width / 2.0, height / 2.0),
        BlockShape::Circle => (width * FRAC_1_SQRT_2, height * FRAC_1_SQRT_2),
        BlockShape::Stadium | BlockShape::Hexagon => (width - height / 2.0, height),
        _ => (width, height),
    }
}

/// Parse `source`, restoring the node shapes rusty-mermaid misreads.
fn parse_blocks(source: &str) -> Result<BlockDiagram, ParseError> {
    let mut shapes = HashMap::new();
    let mut diagram = parse(&rewrite_nodes(source, &mut shapes))?;
    for block in &mut diagram.blocks {
        if let Some(&shape) = shapes.get(&block.id) {
            block.shape = shape;
        }
    }
    Ok(diagram)
}

/// Rewrite node lines of `source` to `id["label"]`, recording each shape.
///
/// Borrows `source` when no line is a node with a shape.
fn rewrite_nodes<'a>(source: &'a str, shapes: &mut HashMap<String, BlockShape>) -> Cow<'a, str> {
    let mut body = String::with_capacity(source.len());
    let mut changed = false;
    for line in source.lines() {
        match parse_node(line.trim()) {
            Some(node) => {
                body.push_str(&node.id);
                body.push_str("[\"");
                body.push_str(node.label);
                body.push_str("\"]");
                if let Some(span) = node.span {
                    body.push(':');
                    body.push_str(span);
                }
                shapes.insert(node.id, node.shape);
                changed = true;
            }
            None => body.push_str(line),
        }
        body.push('\n');
    }
    if changed {
        Cow::Owned(body)
    } else {
        Cow::Borrowed(source)
    }
}

/// A block node definition with a shape.
struct Node<'a> {
    id: String,
    label: &'a str,
    shape: BlockShape,
    span: Option<&'a str>,
}

/// Parse `line` as `id<open>label<close>[:span]`.
///
/// Returns `None` for edges, keywords and nodes without a shape.
fn parse_node(line: &str) -> Option<Node<'_>> {
    if ARROWS.iter().any(|arrow| line.contains(arrow)) {
        return None;
    }
    let (main, span) = match line.rsplit_once(':') {
        Some((main, span)) if span.trim().parse::<usize>().is_ok() => (main, Some(span.trim())),
        _ => (line, None),
    };
    let start = main.find(SHAPE_OPENERS)?;
    let id = main[..start].trim();
    if id.is_empty() || id.contains(char::is_whitespace) {
        return None;
    }
    let rest = main[start..].trim_end();
    let (open, close, shape) = SHAPES.iter().find(|(open, close, _)| {
        rest.len() >= open.len() + close.len() && rest.starts_with(open) && rest.ends_with(close)
    })?;
    let inner = rest[open.len()..rest.len() - close.len()].trim();
    let label = inner
        .strip_prefix('"')
        .and_then(|label| label.strip_suffix('"'))
        .unwrap_or(inner);
    Some(Node {
        id: id.to_string(),
        label,
        shape: *shape,
        span,
    })
}

#[cfg(test)]
mod tests {
    use super::{label_blocks, parse_blocks, render_block};
    use rusty_mermaid::{Primitive, Theme};
    use rusty_mermaid_diagrams::block::ir::BlockShape;

    /// Block diagram whose wide cylinder label rusty-mermaid drops.
    const GATEWAY: &str = "block-beta\n    columns 4\n    \
                           Header[\"Distributed System Gateway\"]:4\n    \
                           A[\"Reverse Proxy\"]\n    B[\"WAF Firewall\"]\n    \
                           Database[(\"Distributed Clustered Database\")]:4\n    \
                           A --> B\n    B --> Database\n";

    /// Return every text label in the labelled scene of `source`.
    fn labels(source: &str) -> Vec<String> {
        let theme = Theme::dark();
        let scene = render_block(source, &theme).unwrap();
        label_blocks(scene, source, &theme)
            .elements()
            .iter()
            .filter_map(|element| match &element.primitive {
                Primitive::Text { content, .. } => Some(content.replace('\n', " ")),
                _ => None,
            })
            .collect()
    }

    #[test]
    /// Assert node shapes rusty-mermaid misreads are parsed with clean labels.
    fn shapes_keep_their_kind_and_label() {
        let source = "block-beta\n  a[(\"Db\")]\n  b([\"Pill\"])\n  c[/\"Lean\"/]\n  \
                      d>\"Flag\"]\n  e[[\"Sub\"]]\n  f((\"Ring\")):2\n  g\n";
        let diagram = parse_blocks(source).unwrap();
        let parsed: Vec<_> = diagram
            .blocks
            .iter()
            .map(|b| (b.id.as_str(), b.label.as_str(), b.shape, b.span))
            .collect();
        assert_eq!(
            parsed,
            [
                ("a", "Db", BlockShape::Cylinder, 1),
                ("b", "Pill", BlockShape::Stadium, 1),
                ("c", "Lean", BlockShape::Rect, 1),
                ("d", "Flag", BlockShape::Rect, 1),
                ("e", "Sub", BlockShape::Rect, 1),
                ("f", "Ring", BlockShape::Circle, 2),
                ("g", "g", BlockShape::Rect, 1),
            ]
        );
    }

    #[test]
    /// Assert edges still connect blocks whose definitions were rewritten.
    fn edges_survive_rewrite() {
        let diagram = parse_blocks(GATEWAY).unwrap();
        let edges: Vec<_> = diagram
            .edges
            .iter()
            .map(|e| (e.from.as_str(), e.to.as_str()))
            .collect();
        assert_eq!(edges, [("A", "B"), ("B", "Database")]);
    }

    #[test]
    /// Assert a label too wide for its box is restored once, without brackets.
    fn wide_label_is_restored() {
        let labels = labels(GATEWAY);
        let database = "Distributed Clustered Database";
        assert_eq!(
            labels.iter().filter(|l| *l == database).count(),
            1,
            "{labels:?}"
        );
        assert!(labels.contains(&"Reverse Proxy".to_string()), "{labels:?}");
    }

    #[test]
    /// Assert labels in every misread shape are drawn.
    fn misread_shape_labels_are_drawn() {
        let labels = labels("block-beta\n  a([\"Stadium quoted\"])\n  b>\"Flag quoted\"]\n");
        assert!(labels.contains(&"Stadium quoted".to_string()), "{labels:?}");
        assert!(labels.contains(&"Flag quoted".to_string()), "{labels:?}");
    }
}
