//! Splits and reassembles the object list of an uncompressed PDF file.
//!
//! Ironpress writes a flat `N 0 obj … endobj` list followed by an xref table.
//! Rewriting a content stream changes its length, so the objects are taken
//! apart, edited, and written back with a rebuilt xref table and trailer.

use crate::constants::PDF_HEADER;
use memchr::{memchr_iter, memmem};

/// Parse the catalog object id from a PDF trailer.
pub fn parse_catalog_id(trailer: &[u8]) -> Result<usize, String> {
    let text = String::from_utf8_lossy(trailer);
    let marker = "/Root ";
    let start = text
        .find(marker)
        .ok_or_else(|| "PDF trailer Root not found".to_string())?
        + marker.len();
    text[start..]
        .split_whitespace()
        .next()
        .ok_or_else(|| "PDF trailer Root id missing".to_string())?
        .parse()
        .map_err(|_| "PDF trailer Root id is not a number".to_string())
}

/// Split a PDF body into slices of its individual objects.
pub fn split_objects(body: &[u8]) -> Result<Vec<&[u8]>, String> {
    let mut objects = Vec::new();
    let mut rest = body;
    while !rest.is_empty() {
        while rest.first().is_some_and(|b| b.is_ascii_whitespace()) {
            rest = &rest[1..];
        }
        if rest.is_empty() {
            break;
        }
        let (object, next) = take_object(rest)?;
        objects.push(object);
        rest = next;
    }
    Ok(objects)
}

/// Writes PDF objects into one buffer and records their xref offsets.
pub struct PdfWriter {
    out: Vec<u8>,
    offsets: Vec<usize>,
}

impl PdfWriter {
    /// Start a document whose buffer holds `capacity` bytes before growing.
    pub fn with_capacity(capacity: usize) -> Self {
        let mut out = Vec::with_capacity(capacity);
        out.extend_from_slice(PDF_HEADER);
        Self {
            out,
            offsets: Vec::new(),
        }
    }

    /// Append one object written by `write`, ending it with a newline.
    pub fn write_object(&mut self, write: impl FnOnce(&mut Vec<u8>)) {
        self.offsets.push(self.out.len());
        write(&mut self.out);
        if !self.out.ends_with(b"\n") {
            self.out.push(b'\n');
        }
    }

    /// Append the xref table and trailer and return the document bytes.
    pub fn finish(mut self, catalog_id: usize) -> Vec<u8> {
        let size = self.offsets.len() + 1;
        let xref_offset = self.out.len();
        self.out
            .extend_from_slice(format!("xref\n0 {size}\n").as_bytes());
        self.out.extend_from_slice(b"0000000000 65535 f \n");
        for offset in &self.offsets {
            self.out
                .extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
        }
        self.out.extend_from_slice(
            format!(
                "trailer\n<< /Size {size} /Root {catalog_id} 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n"
            )
            .as_bytes(),
        );
        self.out
    }
}

/// Serialize PDF objects with a rebuilt xref table and trailer.
#[cfg(test)]
pub fn serialize_pdf(objects: &[&[u8]], catalog_id: usize) -> Vec<u8> {
    let mut writer = PdfWriter::with_capacity(0);
    for object in objects {
        writer.write_object(|out| out.extend_from_slice(object));
    }
    writer.finish(catalog_id)
}

/// Return the offset of the xref table that follows the object list.
pub fn find_xref_offset(body: &[u8]) -> Result<usize, String> {
    body.windows(6)
        .rposition(|w| w == b"\nxref\n")
        .map(|at| at + 1)
        .ok_or_else(|| "PDF xref table not found".to_string())
}

/// Parse `/Length` from a PDF stream dictionary.
pub fn parse_length(dict: &[u8]) -> Option<usize> {
    let key = b"/Length ";
    let at = find_subslice(dict, key)?;
    let digits: String = dict[at + key.len()..]
        .iter()
        .take_while(|b| b.is_ascii_digit())
        .map(|b| *b as char)
        .collect();
    digits.parse().ok()
}

/// Write a stream dictionary to `out` with its `/Length` set to `new_len`.
///
/// A dictionary without `/Length` is written unchanged.
pub fn write_dict_with_length(dict: &[u8], new_len: usize, out: &mut Vec<u8>) {
    let key = b"/Length ";
    let Some(at) = find_subslice(dict, key) else {
        out.extend_from_slice(dict);
        return;
    };
    let digits_at = at + key.len();
    let digits_end = digits_at
        + dict[digits_at..]
            .iter()
            .take_while(|b| b.is_ascii_digit())
            .count();
    out.extend_from_slice(&dict[..digits_at]);
    out.extend_from_slice(new_len.to_string().as_bytes());
    out.extend_from_slice(&dict[digits_end..]);
}

/// Return the index of `needle` in `haystack`.
pub fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    memmem::find(haystack, needle)
}

/// Take the next PDF object from `input` and return the remainder.
fn take_object(input: &[u8]) -> Result<(&[u8], &[u8]), String> {
    let header_end = object_header_end(input)?;

    let Some(stream_keyword) = own_stream_keyword(input, header_end) else {
        let endobj = find_subslice(input, b"\nendobj")
            .ok_or_else(|| "PDF object missing endobj".to_string())?;
        let end = skip_newline(input, endobj + b"\nendobj".len());
        return Ok(input.split_at(end));
    };

    let dict = &input[header_end..stream_keyword];
    let payload_start = stream_keyword + b"\nstream\n".len();
    let end = if find_subslice(dict, b"/Filter").is_some() {
        filtered_stream_end(input, dict, payload_start)?
    } else {
        let endstream = find_subslice(&input[payload_start..], b"\nendstream")
            .map(|at| payload_start + at)
            .ok_or_else(|| "PDF stream missing endstream".to_string())?;
        let after_endstream = skip_newline(input, endstream + b"\nendstream".len());
        expect_keyword(
            input,
            after_endstream,
            b"endobj",
            "PDF stream missing endobj",
        )?
    };
    Ok(input.split_at(end))
}

/// Return the index of the `stream` keyword belonging to this object.
///
/// A later object's stream must not be claimed by this one, so a `stream`
/// keyword only counts when it precedes the first `endobj`. The keyword of a
/// stream object always precedes its own `endobj`, including when a compressed
/// payload happens to contain the `endobj` bytes. Stopping at whichever
/// keyword comes first keeps a plain object from scanning ahead to the next
/// stream, and a stream object from scanning its own payload.
fn own_stream_keyword(input: &[u8], header_end: usize) -> Option<usize> {
    memchr_iter(b'\n', &input[header_end..])
        .map(|at| header_end + at)
        .find_map(|at| {
            let rest = &input[at..];
            if rest.starts_with(b"\nstream\n") {
                Some(Some(at))
            } else if rest.starts_with(b"\nendobj") {
                Some(None)
            } else {
                None
            }
        })
        .flatten()
}

/// Return the index just past the `N 0 obj` header line of `input`.
fn object_header_end(input: &[u8]) -> Result<usize, String> {
    let obj_tag =
        find_subslice(input, b" 0 obj").ok_or_else(|| "PDF object header not found".to_string())?;
    let after_tag = obj_tag + b" 0 obj".len();
    input[after_tag..]
        .iter()
        .position(|&b| b == b'\n')
        .map(|at| after_tag + at + 1)
        .ok_or_else(|| "PDF object header newline not found".to_string())
}

/// Return the index just past an object whose stream payload is compressed.
///
/// A filtered payload can contain `endstream`, so its length is taken from the
/// dictionary rather than by scanning for the keyword.
fn filtered_stream_end(input: &[u8], dict: &[u8], payload_start: usize) -> Result<usize, String> {
    let len = parse_length(dict).ok_or_else(|| "filtered PDF stream missing Length".to_string())?;
    let payload_end = payload_start + len;
    if payload_end > input.len() {
        return Err("filtered PDF stream Length overruns file".to_string());
    }
    let after_payload = skip_newline(input, payload_end);
    let after_endstream = expect_keyword(
        input,
        after_payload,
        b"endstream",
        "filtered PDF stream missing endstream",
    )?;
    expect_keyword(
        input,
        after_endstream,
        b"endobj",
        "filtered PDF stream missing endobj",
    )
}

/// Return the index just past `keyword` at `at`, and past a trailing newline.
fn expect_keyword(input: &[u8], at: usize, keyword: &[u8], error: &str) -> Result<usize, String> {
    if !input[at..].starts_with(keyword) {
        return Err(error.to_string());
    }
    Ok(skip_newline(input, at + keyword.len()))
}

/// Return `at`, advanced past one newline if one is present.
fn skip_newline(input: &[u8], at: usize) -> usize {
    if input.get(at) == Some(&b'\n') {
        at + 1
    } else {
        at
    }
}

#[cfg(test)]
mod tests {
    use super::{
        find_subslice, find_xref_offset, parse_catalog_id, parse_length, serialize_pdf,
        split_objects, write_dict_with_length, PDF_HEADER,
    };

    #[test]
    /// Assert the catalog id is read from the trailer's `/Root` entry.
    fn parses_catalog_id_from_trailer() {
        let trailer = b"trailer\n<< /Size 5 /Root 3 0 R >>\nstartxref\n99\n%%EOF\n";
        assert_eq!(parse_catalog_id(trailer), Ok(3));
    }

    #[test]
    /// Assert a trailer without `/Root` is an error.
    fn trailer_without_root_is_an_error() {
        assert!(parse_catalog_id(b"trailer\n<< /Size 5 >>\n").is_err());
    }

    #[test]
    /// Assert a non-numeric `/Root` id is an error.
    fn non_numeric_root_id_is_an_error() {
        assert!(parse_catalog_id(b"trailer\n<< /Root x 0 R >>\n").is_err());
    }

    #[test]
    /// Assert the xref offset points at the `xref` keyword.
    fn finds_xref_offset() {
        let body = b"1 0 obj\n<< >>\nendobj\nxref\n0 2\n";
        let at = find_xref_offset(body).expect("xref offset");
        assert!(body[at..].starts_with(b"xref\n"), "{at}");
    }

    #[test]
    /// Assert a body without an xref table is an error.
    fn body_without_xref_is_an_error() {
        assert!(find_xref_offset(b"1 0 obj\n<< >>\nendobj\n").is_err());
    }

    #[test]
    /// Assert plain objects split on their `endobj` keywords.
    fn splits_plain_objects() {
        let body = b"1 0 obj\n<< /Type /Catalog >>\nendobj\n2 0 obj\n<< >>\nendobj\n";
        let objects = split_objects(body).expect("objects");
        assert_eq!(objects.len(), 2);
        assert!(objects[0].starts_with(b"1 0 obj"), "{:?}", objects[0]);
        assert!(objects[1].starts_with(b"2 0 obj"), "{:?}", objects[1]);
    }

    #[test]
    /// Assert an uncompressed stream object is taken whole.
    fn splits_uncompressed_stream_object() {
        let body = b"1 0 obj\n<< /Length 5 >>\nstream\nBT ET\nendstream\nendobj\n";
        let objects = split_objects(body).expect("objects");
        assert_eq!(objects.len(), 1);
        assert_eq!(objects[0], &body[..]);
    }

    #[test]
    /// Assert a filtered stream's length comes from its dictionary.
    fn splits_filtered_stream_object_by_length() {
        // The payload itself contains `endstream`, so scanning would stop early.
        let payload = b"\nendstream junk";
        let mut body = format!(
            "1 0 obj\n<< /Filter /FlateDecode /Length {} >>\nstream\n",
            payload.len()
        )
        .into_bytes();
        body.extend_from_slice(payload);
        body.extend_from_slice(b"\nendstream\nendobj\n2 0 obj\n<< >>\nendobj\n");
        let objects = split_objects(&body).expect("objects");
        assert_eq!(objects.len(), 2);
        assert!(objects[1].starts_with(b"2 0 obj"), "{:?}", objects[1]);
    }

    #[test]
    /// Assert a filtered stream whose `/Length` overruns the file is an error.
    fn filtered_stream_length_overrun_is_an_error() {
        let body = b"1 0 obj\n<< /Filter /Fl /Length 9999 >>\nstream\nx\nendstream\nendobj\n";
        assert!(split_objects(body).is_err());
    }

    #[test]
    /// Assert an object without `endobj` is an error.
    fn object_without_endobj_is_an_error() {
        assert!(split_objects(b"1 0 obj\n<< >>\n").is_err());
    }

    #[test]
    /// Assert a plain object does not absorb a later object's stream.
    fn plain_object_before_a_stream_object_stays_separate() {
        let body = b"1 0 obj\n<< /Type /Catalog >>\nendobj\n\
                     2 0 obj\n<< /Length 5 >>\nstream\nBT ET\nendstream\nendobj\n";
        let objects = split_objects(body).expect("objects");
        assert_eq!(objects.len(), 2);
        assert!(
            !objects[0].ends_with(b"endstream\nendobj\n"),
            "{:?}",
            objects[0]
        );
        assert!(objects[1].starts_with(b"2 0 obj"), "{:?}", objects[1]);
    }

    #[test]
    /// Assert `/Length` is read from a stream dictionary.
    fn parses_stream_length() {
        assert_eq!(parse_length(b"<< /Length 42 /Type /X >>"), Some(42));
        assert_eq!(parse_length(b"<< /Type /X >>"), None);
    }

    #[test]
    /// Assert `/Length` is rewritten, keeping the rest of the dictionary.
    fn rewrites_stream_length() {
        let mut out = Vec::new();
        write_dict_with_length(b"<< /Length 7 /Type /X >>", 1234, &mut out);
        assert_eq!(out, b"<< /Length 1234 /Type /X >>".to_vec());
    }

    #[test]
    /// Assert a dictionary without `/Length` is written unchanged.
    fn rewrite_without_length_is_a_no_op() {
        let mut out = Vec::new();
        write_dict_with_length(b"<< /Type /X >>", 10, &mut out);
        assert_eq!(out, b"<< /Type /X >>".to_vec());
    }

    #[test]
    /// Assert serialization writes a header, xref entry per object, and trailer.
    fn serializes_header_xref_and_trailer() {
        let objects: [&[u8]; 1] = [b"1 0 obj\n<< >>\nendobj\n"];
        let pdf = serialize_pdf(&objects, 1);
        assert!(pdf.starts_with(PDF_HEADER), "{pdf:?}");
        let text = String::from_utf8_lossy(&pdf);
        assert!(text.contains("xref\n0 2\n"), "{text}");
        assert!(text.contains("0000000000 65535 f \n"), "{text}");
        assert!(text.contains("/Size 2 /Root 1 0 R"), "{text}");
        assert!(text.ends_with("%%EOF\n"), "{text}");
    }

    #[test]
    /// Assert each xref entry holds the object's byte offset.
    fn xref_entries_hold_object_offsets() {
        let objects: [&[u8]; 2] = [b"1 0 obj\n<< >>\nendobj\n", b"2 0 obj\n<< >>\nendobj\n"];
        let pdf = serialize_pdf(&objects, 1);
        let text = String::from_utf8_lossy(&pdf);
        for (index, object) in objects.iter().enumerate() {
            let offset = find_subslice(&pdf, object).expect("object offset");
            assert!(
                text.contains(&format!("{offset:010} 00000 n \n")),
                "object {index} offset {offset} missing from xref: {text}"
            );
        }
    }

    #[test]
    /// Assert splitting and reserializing a document round-trips its objects.
    fn split_and_serialize_round_trip() {
        let objects: [&[u8]; 2] = [
            b"1 0 obj\n<< /Type /Catalog >>\nendobj\n",
            b"2 0 obj\n<< /Length 5 >>\nstream\nBT ET\nendstream\nendobj\n",
        ];
        let pdf = serialize_pdf(&objects, 1);
        let body = &pdf[PDF_HEADER.len()..find_xref_offset(&pdf).expect("xref")];
        assert_eq!(split_objects(body).expect("objects"), objects.to_vec());
    }

    #[test]
    /// Assert a subslice is located by its first occurrence.
    fn finds_first_subslice() {
        assert_eq!(find_subslice(b"abcabc", b"bc"), Some(1));
        assert_eq!(find_subslice(b"abc", b"z"), None);
    }
}
