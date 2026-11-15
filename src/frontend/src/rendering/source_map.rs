//! Maps rendered editor HTML back to offsets in the markdown source.
//!
//! The editor render tags elements with the byte offset of the source they
//! came from. A click on the rendered note reads the nearest tag and resolves
//! it to the caret position the textarea opens at.

use super::escape_html;
use crate::constants::{SOURCE_KIND_ATOM, SOURCE_KIND_ATTR, SOURCE_KIND_LINES, SOURCE_OFFSET_ATTR};
use std::ops::Range;

/// How the text of a tagged element relates to the source at its offset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceKind {
    /// The element text is the source text, byte for byte.
    Verbatim,
    /// Each line of the element text is the tail of one source line.
    Lines,
    /// The element maps to its offset as a whole.
    Atom,
}

impl SourceKind {
    /// Return the kind named by a `data-src-kind` value; none means verbatim.
    pub fn from_attr(value: Option<&str>) -> Self {
        match value {
            Some(SOURCE_KIND_LINES) => Self::Lines,
            Some(SOURCE_KIND_ATOM) => Self::Atom,
            _ => Self::Verbatim,
        }
    }

    /// Return the attributes that tag an element with `offset` and this kind.
    fn attrs(self, offset: usize) -> String {
        match self {
            Self::Verbatim => format!(r#" {SOURCE_OFFSET_ATTR}="{offset}""#),
            Self::Lines => format!(
                r#" {SOURCE_OFFSET_ATTR}="{offset}" {SOURCE_KIND_ATTR}="{SOURCE_KIND_LINES}""#
            ),
            Self::Atom => format!(
                r#" {SOURCE_OFFSET_ATTR}="{offset}" {SOURCE_KIND_ATTR}="{SOURCE_KIND_ATOM}""#
            ),
        }
    }
}

/// One stretch of substituted text: `out` replaced `src`.
#[derive(Debug)]
struct Edit {
    out: Range<usize>,
    src: Range<usize>,
}

/// Offsets of a substituted string mapped back to the string it came from.
#[derive(Debug, Default)]
pub struct OffsetMap {
    edits: Vec<Edit>,
}

impl OffsetMap {
    /// Record that output range `out` replaced source range `src`.
    ///
    /// Edits must be recorded in output order.
    pub fn record(&mut self, out: Range<usize>, src: Range<usize>) {
        self.edits.push(Edit { out, src });
    }

    /// Return the last edit starting at or before output offset `at`.
    fn edit_before(&self, at: usize) -> Option<&Edit> {
        let after = self.edits.partition_point(|edit| edit.out.start <= at);
        after.checked_sub(1).map(|index| &self.edits[index])
    }

    /// Return whether output offset `at` lies inside substituted text.
    pub fn is_replaced(&self, at: usize) -> bool {
        self.edit_before(at).is_some_and(|edit| at < edit.out.end)
    }

    /// Return the source offset of output offset `at`.
    ///
    /// An offset inside substituted text maps to the start of what it replaced.
    pub fn to_source(&self, at: usize) -> usize {
        match self.edit_before(at) {
            None => at,
            Some(edit) if at < edit.out.end => edit.src.start,
            Some(edit) => edit.src.end + (at - edit.out.end),
        }
    }
}

/// Tags editor HTML with offsets into the original markdown.
pub struct SourceMap<'a> {
    original: &'a str,
    offsets: OffsetMap,
}

impl<'a> SourceMap<'a> {
    /// Build a map from parser offsets to `original` through `offsets`.
    pub fn new(original: &'a str, offsets: OffsetMap) -> Self {
        Self { original, offsets }
    }

    /// Return `<tag>` holding `text`, tagged with where `text` sits in the source.
    ///
    /// `range` is the parser range of the event in `parsed`. `delimited` text
    /// (inline code) sits somewhere inside its range; other text must fill it,
    /// so an entity or escape maps as a whole. Return `None` for text inside
    /// substituted markup, which must stay untouched.
    pub fn text_element(
        &self,
        tag: &str,
        text: &str,
        parsed: &str,
        range: Range<usize>,
        delimited: bool,
    ) -> Option<String> {
        if self.offsets.is_replaced(range.start) {
            return None;
        }
        let within = match parsed.get(range.clone())?.find(text) {
            Some(within) if delimited || range.len() == text.len() => Some(within),
            _ => None,
        };
        let at = self.offsets.to_source(range.start + within.unwrap_or(0));
        let verbatim = within.is_some() && self.original.get(at..at + text.len()) == Some(text);
        let kind = if verbatim {
            SourceKind::Verbatim
        } else {
            SourceKind::Atom
        };
        Some(format!(
            "<{tag}{}>{}</{tag}>",
            kind.attrs(at),
            escape_html(text)
        ))
    }

    /// Tag a rendered fence or image that starts at parser offset `parsed_start`.
    ///
    /// A `<pre>` shows the fence body line for line, so it maps by line from
    /// the line after the opening fence. Anything else maps as a whole.
    pub fn annotate_block(&self, fragment: String, parsed_start: usize) -> String {
        let at = self.offsets.to_source(parsed_start);
        if !fragment.starts_with("<pre") {
            return annotate_first_tag(&fragment, at, SourceKind::Atom);
        }
        let len = self.original.len();
        let body = self
            .original
            .get(at..)
            .and_then(|rest| rest.find('\n'))
            .map_or(len, |newline| at + newline + 1);
        annotate_first_tag(&fragment, body, SourceKind::Lines)
    }
}

/// Return `fragment` with its first element tagged with `offset` and `kind`.
///
/// A fragment with no element is returned unchanged.
pub fn annotate_first_tag(fragment: &str, offset: usize, kind: SourceKind) -> String {
    let bytes = fragment.as_bytes();
    let Some(open) = (0..bytes.len().saturating_sub(1))
        .find(|&i| bytes[i] == b'<' && bytes[i + 1].is_ascii_alphabetic())
    else {
        return fragment.to_string();
    };
    let name_end = bytes[open + 1..]
        .iter()
        .position(|b| !b.is_ascii_alphanumeric() && *b != b'-')
        .map_or(bytes.len(), |n| open + 1 + n);
    let mut out = String::with_capacity(fragment.len() + 40);
    out.push_str(&fragment[..name_end]);
    out.push_str(&kind.attrs(offset));
    out.push_str(&fragment[name_end..]);
    out
}

/// Return the largest char boundary of `s` at or below `at`.
fn floor_char_boundary(s: &str, at: usize) -> usize {
    let mut at = at.min(s.len());
    while !s.is_char_boundary(at) {
        at -= 1;
    }
    at
}

/// Return the source offset of a caret inside a tagged element.
///
/// `offset` and `kind` come from the element's tag, `full_text` is the
/// element's text and `prefix` the part of it before the caret. When the
/// source no longer matches the rendered text, the element's offset is used.
pub fn resolve_source_offset(
    src: &str,
    offset: usize,
    kind: SourceKind,
    full_text: &str,
    prefix: &str,
) -> usize {
    let offset = floor_char_boundary(src, offset);
    match kind {
        SourceKind::Atom => offset,
        SourceKind::Verbatim => {
            let end = offset + prefix.len();
            if src.get(offset..end) == Some(prefix) {
                end
            } else {
                offset
            }
        }
        SourceKind::Lines => resolve_line_offset(src, offset, full_text, prefix),
    }
}

/// Return the source offset of a caret in line-mapped text starting at `start`.
///
/// The caret's line number selects the source line. Its column is counted
/// from where the rendered line begins in that source line, which skips
/// list or quote indentation.
fn resolve_line_offset(src: &str, start: usize, full_text: &str, prefix: &str) -> usize {
    let line_index = prefix.matches('\n').count();
    let column = prefix.rsplit('\n').next().unwrap_or_default();
    let rendered_line = full_text.split('\n').nth(line_index).unwrap_or(column);

    let mut line_start = start;
    for _ in 0..line_index {
        match src[line_start..].find('\n') {
            Some(newline) => line_start += newline + 1,
            None => return src.len(),
        }
    }
    let line_end = src[line_start..]
        .find('\n')
        .map_or(src.len(), |newline| line_start + newline);
    let line = src[line_start..line_end].trim_end_matches('\r');
    let indent = if line.ends_with(rendered_line) {
        line.len() - rendered_line.len()
    } else {
        0
    };
    floor_char_boundary(src, line_start + (indent + column.len()).min(line.len()))
}

/// Return the text a textarea holds before source byte `offset`.
///
/// A textarea reports `\r\n` as `\n`, so the `\r` of each pair is dropped.
/// The UTF-16 length of the result is the textarea selection offset.
pub fn textarea_text_before(src: &str, offset: usize) -> String {
    let end = floor_char_boundary(src, offset);
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(end);
    for (at, ch) in src[..end].char_indices() {
        if ch == '\r' && bytes.get(at + 1) == Some(&b'\n') {
            continue;
        }
        out.push(ch);
    }
    out
}

/// Return the source byte offset of textarea selection offset `units`.
///
/// The inverse of [`textarea_text_before`]: `units` counts UTF-16 units of
/// the textarea value, where each `\r\n` of `src` is a single `\n`.
pub fn source_offset_at_textarea(src: &str, units: u32) -> usize {
    let bytes = src.as_bytes();
    let mut seen = 0;
    for (at, ch) in src.char_indices() {
        if ch == '\r' && bytes.get(at + 1) == Some(&b'\n') {
            continue;
        }
        if seen >= units {
            return at;
        }
        seen += ch.len_utf16() as u32;
    }
    src.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Assert textarea offsets map back to source bytes across CRLF and wide chars.
    fn textarea_offsets_map_back_to_source() {
        let src = "a\r\né☕b";
        for offset in [0, 3, 5, 8, 9] {
            let units = textarea_text_before(src, offset).encode_utf16().count() as u32;
            assert_eq!(source_offset_at_textarea(src, units), offset, "{offset}");
        }
        assert_eq!(source_offset_at_textarea("ab", 99), 2);
    }

    #[test]
    /// Assert offsets before, inside and after an edit map back to the source.
    fn offset_map_maps_around_edits() {
        // source "ab$x$cd" -> output "ab<math>cd"
        let mut map = OffsetMap::default();
        map.record(2..8, 2..5);
        assert_eq!(map.to_source(0), 0);
        assert_eq!(map.to_source(2), 2);
        assert_eq!(map.to_source(5), 2);
        assert_eq!(map.to_source(8), 5);
        assert_eq!(map.to_source(9), 6);
        assert!(!map.is_replaced(1));
        assert!(map.is_replaced(2) && map.is_replaced(7));
        assert!(!map.is_replaced(8));
    }

    #[test]
    /// Assert an empty map leaves offsets unchanged.
    fn empty_offset_map_is_identity() {
        let map = OffsetMap::default();
        assert_eq!(map.to_source(7), 7);
        assert!(!map.is_replaced(7));
    }

    #[test]
    /// Assert the first element is tagged and leading text is kept.
    fn annotate_first_tag_tags_the_first_element() {
        assert_eq!(
            annotate_first_tag("<math><mi>x</mi></math>", 4, SourceKind::Atom),
            r#"<math data-src="4" data-src-kind="atom"><mi>x</mi></math>"#
        );
        assert_eq!(
            annotate_first_tag(r#"<pre class="c">x</pre>"#, 9, SourceKind::Lines),
            r#"<pre data-src="9" data-src-kind="lines" class="c">x</pre>"#
        );
        assert_eq!(
            annotate_first_tag("<!-- c --><svg/>", 1, SourceKind::Verbatim),
            r#"<!-- c --><svg data-src="1"/>"#
        );
        assert_eq!(annotate_first_tag("plain", 1, SourceKind::Atom), "plain");
    }

    #[test]
    /// Assert verbatim text maps to the exact character, multi-byte included.
    fn verbatim_resolves_to_the_exact_character() {
        let src = "# café bar";
        assert_eq!(
            resolve_source_offset(src, 2, SourceKind::Verbatim, "", "café "),
            8
        );
        assert_eq!(&src[8..], "bar");
    }

    #[test]
    /// Assert a stale tag falls back to the element offset instead of drifting.
    fn verbatim_mismatch_falls_back_to_element_start() {
        assert_eq!(
            resolve_source_offset("hello world", 6, SourceKind::Verbatim, "", "xyz"),
            6
        );
        assert_eq!(
            resolve_source_offset("short", 99, SourceKind::Verbatim, "", "x"),
            5
        );
    }

    #[test]
    /// Assert an atom maps to its own offset whatever the caret text is.
    fn atom_resolves_to_its_offset() {
        assert_eq!(
            resolve_source_offset("a $x$ b", 2, SourceKind::Atom, "x", "x"),
            2
        );
    }

    #[test]
    /// Assert a caret in a code block maps to the same line and column.
    fn lines_resolve_to_line_and_column() {
        let src = "```rust\nfn main() {\n    body();\n}\n```\n";
        let full = "fn main() {\n    body();\n}\n";
        let at = resolve_source_offset(src, 8, SourceKind::Lines, full, "fn main() {\n    bo");
        assert_eq!(&src[at..at + 5], "dy();");
    }

    #[test]
    /// Assert list indentation before a nested code line is skipped.
    fn lines_skip_container_indentation() {
        let src = "- item\n\n  ```\n  one\n  two\n  ```\n";
        let body = src.find("  one").expect("body line");
        let at = resolve_source_offset(src, body, SourceKind::Lines, "one\ntwo\n", "one\nt");
        assert_eq!(&src[at..at + 2], "wo");
    }

    #[test]
    /// Assert CRLF source lines map by line without counting the `\r`.
    fn lines_handle_crlf_source() {
        let src = "```\r\none\r\ntwo\r\n```\r\n";
        let at = resolve_source_offset(src, 5, SourceKind::Lines, "one\ntwo\n", "one\ntw");
        assert_eq!(&src[at..at + 1], "o");
    }

    #[test]
    /// Assert a caret past the last source line clamps to the source end.
    fn lines_past_the_end_clamp() {
        let src = "```\none";
        assert_eq!(
            resolve_source_offset(src, 4, SourceKind::Lines, "one\n\n", "one\n\n"),
            src.len()
        );
    }

    #[test]
    /// Assert the textarea prefix drops the `\r` of each CRLF pair.
    fn textarea_text_drops_carriage_returns() {
        assert_eq!(textarea_text_before("a\r\nb\r\nc", 6), "a\nb\n");
        assert_eq!(textarea_text_before("a\nb", 2), "a\n");
        assert_eq!(textarea_text_before("é", 1), "");
        assert_eq!(textarea_text_before("ab", 99), "ab");
    }

    #[test]
    /// Assert kind attribute values round-trip.
    fn kind_round_trips_through_its_attribute() {
        assert_eq!(SourceKind::from_attr(None), SourceKind::Verbatim);
        assert_eq!(
            SourceKind::from_attr(Some(SOURCE_KIND_LINES)),
            SourceKind::Lines
        );
        assert_eq!(
            SourceKind::from_attr(Some(SOURCE_KIND_ATOM)),
            SourceKind::Atom
        );
    }
}
