//! Decodes character entities in Mermaid labels.
//!
//! Mermaid lets labels spell awkward characters as HTML entities (`&lt;`) or
//! its own `#`-prefixed form (`#91;`, `#quot;`). rusty-mermaid shows them
//! verbatim, so they are decoded in the source before parsing, where node
//! boxes are still sized from the decoded text. A decoded `"` or `[` would end
//! a label early, and a `<` would be measured as the start of a tag, so the
//! [`MERMAID_PROTECTED_CHARS`] go through layout as stand-ins and are put back
//! in the finished scene.

use crate::constants::{
    MERMAID_NAMED_ENTITIES, MERMAID_PROTECTED_CHARS, MERMAID_STAND_IN_BASE, MERMAID_STYLE_PREFIXES,
};
use rusty_mermaid::{Primitive, Scene};
use std::borrow::Cow;

/// Return `source` with label entities decoded, protected characters as stand-ins.
///
/// Style statements and `%%` lines are left alone, since a colour such as
/// `fill:#333;` reads like a numeric entity. Borrows `source` when unchanged.
pub(super) fn decode_source_entities(source: &str) -> Cow<'_, str> {
    if !source.contains(';') {
        return Cow::Borrowed(source);
    }
    let mut body = String::with_capacity(source.len());
    for line in source.lines() {
        let statement = line.trim_start();
        if MERMAID_STYLE_PREFIXES
            .iter()
            .any(|prefix| statement.starts_with(prefix))
        {
            body.push_str(line);
        } else {
            push_decoded(&mut body, line);
        }
        body.push('\n');
    }
    Cow::Owned(body)
}

/// Return `scene` with every stand-in in its labels turned back into its character.
pub(super) fn restore_protected_chars(scene: Scene) -> Scene {
    let mut restored = Scene::new(scene.width, scene.height);
    for element in scene.elements() {
        let mut primitive = element.primitive.clone();
        restore_primitive(&mut primitive);
        match &element.id {
            Some(id) => restored.push_identified(primitive, id.clone()),
            None => restored.push(primitive),
        }
    }
    restored
}

/// Restore stand-ins in `primitive`'s text, or in the labels of a group.
fn restore_primitive(primitive: &mut Primitive) {
    match primitive {
        Primitive::Text { content, .. } => {
            if content.chars().any(|c| protected_of(c).is_some()) {
                *content = content
                    .chars()
                    .map(|c| protected_of(c).unwrap_or(c))
                    .collect();
            }
        }
        Primitive::Group { children, .. } => children.iter_mut().for_each(restore_primitive),
        _ => {}
    }
}

/// Return the protected character `c` stands in for, if it is a stand-in.
fn protected_of(c: char) -> Option<char> {
    let index = (c as u32).checked_sub(MERMAID_STAND_IN_BASE)?;
    MERMAID_PROTECTED_CHARS.get(index as usize).copied()
}

/// Return the stand-in for `c` when it is protected, else `c` itself.
fn stand_in_for(c: char) -> char {
    MERMAID_PROTECTED_CHARS
        .iter()
        .position(|&protected| protected == c)
        .and_then(|index| char::from_u32(MERMAID_STAND_IN_BASE + index as u32))
        .unwrap_or(c)
}

/// Append `text` to `out` with `&…;` and `#…;` entities decoded.
///
/// Numeric forms are decimal or `x`-prefixed hex; named forms use
/// [`MERMAID_NAMED_ENTITIES`]. Anything unrecognised is kept as written.
fn push_decoded(out: &mut String, text: &str) {
    let mut rest = text;
    while let Some(start) = rest.find(['&', '#']) {
        out.push_str(&rest[..start]);
        let after = &rest[start..];
        match entity_at(after) {
            Some((c, len)) => {
                out.push(stand_in_for(c));
                rest = &after[len..];
            }
            None => {
                out.push_str(&after[..1]);
                rest = &after[1..];
            }
        }
    }
    out.push_str(rest);
}

/// Decode the entity opening `s`, returning its character and byte length.
fn entity_at(s: &str) -> Option<(char, usize)> {
    let (numeric, body_start) = if s.starts_with("&#") {
        (true, 2)
    } else {
        (s.starts_with('#'), 1)
    };
    let end = s.find(';')?;
    let body = s.get(body_start..end)?;
    let c = if numeric && !body.is_empty() && body.chars().all(|c| c.is_ascii_digit()) {
        char::from_u32(body.parse().ok()?)?
    } else if let Some(hex) = body
        .strip_prefix('x')
        .or_else(|| body.strip_prefix('X'))
        .filter(|hex| numeric && !hex.is_empty())
    {
        char::from_u32(u32::from_str_radix(hex, 16).ok()?)?
    } else {
        MERMAID_NAMED_ENTITIES
            .iter()
            .find(|(name, _)| *name == body)
            .map(|(_, c)| *c)?
    };
    Some((c, end + 1))
}

#[cfg(test)]
mod tests {
    use super::{decode_source_entities, protected_of, stand_in_for};
    use crate::constants::{MERMAID_PROTECTED_CHARS, MERMAID_STAND_IN_BASE};

    /// Decode `source` and turn stand-ins back into their characters.
    fn decoded(source: &str) -> String {
        decode_source_entities(source)
            .chars()
            .map(|c| protected_of(c).unwrap_or(c))
            .collect()
    }

    #[test]
    /// Assert mermaid's `#code;` form decodes to its character.
    fn decodes_mermaid_numeric_entities() {
        assert_eq!(
            decoded("Raw Data #91;Unfiltered#93; #40;x#41;"),
            "Raw Data [Unfiltered] (x)\n"
        );
    }

    #[test]
    /// Assert HTML named entities decode to their characters.
    fn decodes_html_named_entities() {
        assert_eq!(
            decoded("Output &lt;JSON-Stream&gt; &quot;q&quot; &copy;"),
            "Output <JSON-Stream> \"q\" ©\n"
        );
    }

    #[test]
    /// Assert mermaid's `#name;` form decodes like its `&name;` twin.
    fn decodes_mermaid_named_entities() {
        assert_eq!(decoded("#quot;hi#quot;"), "\"hi\"\n");
    }

    #[test]
    /// Assert HTML numeric entities, decimal and hex, decode.
    fn decodes_html_numeric_entities() {
        assert_eq!(decoded("&#60;&#x3E;#x26;"), "<>&\n");
    }

    #[test]
    /// Assert text that only resembles an entity is kept as written.
    fn keeps_non_entities() {
        assert_eq!(
            decoded("#1 & #tag; a;b &bogus;"),
            "#1 & #tag; a;b &bogus;\n"
        );
    }

    #[test]
    /// Assert a decoded `&` is not decoded a second time.
    fn decodes_only_once() {
        assert_eq!(decoded("&amp;lt;"), "&lt;\n");
    }

    #[test]
    /// Assert decoded markup characters reach the parser only as stand-ins.
    fn protected_chars_become_stand_ins() {
        let source = decode_source_entities("A[\"#91;&lt;&quot;#59;\"]");
        assert_eq!(source.matches(['[', '<', '"', ';']).count(), 3, "{source}");
    }

    #[test]
    /// Assert colours in style statements are not read as numeric entities.
    fn style_colours_are_untouched() {
        let source = "style A fill:#333;\nclassDef x fill:#123;\nA[#35;]";
        assert_eq!(
            decoded(source),
            "style A fill:#333;\nclassDef x fill:#123;\nA[#]\n"
        );
    }

    #[test]
    /// Assert every protected character has an unassigned stand-in and maps back.
    fn stand_ins_round_trip() {
        assert!(MERMAID_STAND_IN_BASE + MERMAID_PROTECTED_CHARS.len() as u32 <= 0x20D0);
        for &c in MERMAID_PROTECTED_CHARS {
            assert_eq!(protected_of(stand_in_for(c)), Some(c));
        }
    }
}
