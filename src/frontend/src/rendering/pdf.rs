use crate::constants::{MATH_FONTS, TEXT_FILL, TEXT_STROKE};

/// Convert a complete HTML document string to PDF bytes.
pub fn html_to_pdf_bytes(html: &str) -> Result<Vec<u8>, String> {
    // Uncompressed page streams so math fill/stroke operators can be rewritten.
    let pdf = ironpress::HtmlConverter::new()
        .compress(false)
        .convert(html)
        .map_err(|e| e.to_string())?;
    recolor_math_operators(&pdf)
}

/// Recolor black math fill/stroke operators to the page text color.
fn recolor_math_operators(pdf: &[u8]) -> Result<Vec<u8>, String> {
    let Some(body) = pdf.strip_prefix(b"%PDF-1.4\n") else {
        return Ok(pdf.to_vec());
    };
    let xref_at = body
        .windows(6)
        .rposition(|w| w == b"\nxref\n")
        .map(|i| i + 1)
        .ok_or_else(|| "PDF xref table not found".to_string())?;
    let objects_bytes = &body[..xref_at];
    let trailer_bytes = &body[xref_at..];
    let catalog_id = parse_catalog_id(trailer_bytes)?;

    let objects = split_objects(objects_bytes)?;
    let processed: Vec<Vec<u8>> = objects.into_iter().map(recolor_object).collect();
    Ok(serialize_pdf(&processed, catalog_id))
}

/// Parse the catalog object id from a PDF trailer.
fn parse_catalog_id(trailer: &[u8]) -> Result<usize, String> {
    let text = String::from_utf8_lossy(trailer);
    let marker = "/Root ";
    let start = text
        .find(marker)
        .ok_or_else(|| "PDF trailer Root not found".to_string())?
        + marker.len();
    let id: usize = text[start..]
        .split_whitespace()
        .next()
        .ok_or_else(|| "PDF trailer Root id missing".to_string())?
        .parse()
        .map_err(|_| "PDF trailer Root id is not a number".to_string())?;
    Ok(id)
}

/// Split a PDF body into individual object byte slices.
fn split_objects(body: &[u8]) -> Result<Vec<Vec<u8>>, String> {
    let mut objects = Vec::new();
    let mut rest = body;
    while !rest.is_empty() {
        while rest.first().is_some_and(|b| b.is_ascii_whitespace()) {
            rest = &rest[1..];
        }
        if rest.is_empty() {
            break;
        }
        let (obj, next) = take_object(rest)?;
        objects.push(obj);
        rest = next;
    }
    Ok(objects)
}

/// Take the next PDF object from `input` and return the remainder.
fn take_object(input: &[u8]) -> Result<(Vec<u8>, &[u8]), String> {
    let obj_tag = input
        .windows(6)
        .position(|w| w == b" 0 obj")
        .ok_or_else(|| "PDF object header not found".to_string())?;
    let after_header = obj_tag + 6;
    let header_end = input[after_header..]
        .iter()
        .position(|&b| b == b'\n')
        .map(|i| after_header + i + 1)
        .ok_or_else(|| "PDF object header newline not found".to_string())?;

    if let Some(stream_rel) = find_subslice(&input[header_end..], b"\nstream\n") {
        let stream_kw = header_end + stream_rel;
        let dict = &input[header_end..stream_kw];
        let payload_start = stream_kw + b"\nstream\n".len();
        if find_subslice(dict, b"/Filter").is_some() {
            let len = parse_length(dict)
                .ok_or_else(|| "filtered PDF stream missing Length".to_string())?;
            let payload_end = payload_start + len;
            if payload_end > input.len() {
                return Err("filtered PDF stream Length overruns file".to_string());
            }
            let mut end = payload_end;
            if input.get(end) == Some(&b'\n') {
                end += 1;
            }
            if !input[end..].starts_with(b"endstream") {
                return Err("filtered PDF stream missing endstream".to_string());
            }
            end += b"endstream".len();
            if input.get(end) == Some(&b'\n') {
                end += 1;
            }
            if !input[end..].starts_with(b"endobj") {
                return Err("filtered PDF stream missing endobj".to_string());
            }
            end += b"endobj".len();
            if input.get(end) == Some(&b'\n') {
                end += 1;
            }
            return Ok((input[..end].to_vec(), &input[end..]));
        }
        let endstream = find_subslice(&input[payload_start..], b"\nendstream")
            .map(|i| payload_start + i)
            .ok_or_else(|| "PDF stream missing endstream".to_string())?;
        let mut end = endstream + b"\nendstream".len();
        if input.get(end) == Some(&b'\n') {
            end += 1;
        }
        if !input[end..].starts_with(b"endobj") {
            return Err("PDF stream missing endobj".to_string());
        }
        end += b"endobj".len();
        if input.get(end) == Some(&b'\n') {
            end += 1;
        }
        return Ok((input[..end].to_vec(), &input[end..]));
    }

    let endobj =
        find_subslice(input, b"\nendobj").ok_or_else(|| "PDF object missing endobj".to_string())?;
    let mut end = endobj + b"\nendobj".len();
    if input.get(end) == Some(&b'\n') {
        end += 1;
    }
    Ok((input[..end].to_vec(), &input[end..]))
}

/// Recolor an uncompressed content stream inside one PDF object.
fn recolor_object(obj: Vec<u8>) -> Vec<u8> {
    let Some(stream_at) = find_subslice(&obj, b"\nstream\n") else {
        return obj;
    };
    let dict = &obj[..stream_at];
    if find_subslice(dict, b"/Filter").is_some() {
        return obj;
    }
    let payload_start = stream_at + b"\nstream\n".len();
    let Some(endstream_rel) = find_subslice(&obj[payload_start..], b"\nendstream") else {
        return obj;
    };
    let payload_end = payload_start + endstream_rel;
    let mut content = obj[payload_start..payload_end].to_vec();
    if content.ends_with(b"\n") {
        content.pop();
    }
    let recolored = recolor_content(&content);
    let mut dict = dict.to_vec();
    rewrite_length(&mut dict, recolored.len());

    let mut out = dict;
    out.extend_from_slice(b"\nstream\n");
    out.extend_from_slice(&recolored);
    out.extend_from_slice(b"\nendstream\nendobj\n");
    out
}

/// Rewrite a stream dictionary `/Length` to `new_len`.
fn rewrite_length(dict: &mut Vec<u8>, new_len: usize) {
    let key = b"/Length ";
    let Some(at) = find_subslice(dict, key) else {
        return;
    };
    let digits_at = at + key.len();
    let digits_end = digits_at
        + dict[digits_at..]
            .iter()
            .take_while(|b| b.is_ascii_digit())
            .count();
    let mut new_dict = dict[..digits_at].to_vec();
    new_dict.extend_from_slice(new_len.to_string().as_bytes());
    new_dict.extend_from_slice(&dict[digits_end..]);
    *dict = new_dict;
}

/// Recolor black operators and inject fill after math `BT` operators.
fn recolor_content(content: &[u8]) -> Vec<u8> {
    let replaced = replace_black_operators(content);
    insert_fill_after_bt(&replaced)
}

/// Replace `0 0 0 rg` / `RG` with the page text color operators.
fn replace_black_operators(content: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(content.len());
    let mut i = 0;
    while i < content.len() {
        if i + 8 <= content.len()
            && is_pdf_operator_start(content, i)
            && (&content[i..i + 8] == b"0 0 0 rg" || &content[i..i + 8] == b"0 0 0 RG")
        {
            if content[i + 7] == b'g' {
                out.extend_from_slice(TEXT_FILL.trim_ascii_end());
            } else {
                out.extend_from_slice(TEXT_STROKE.trim_ascii_end());
            }
            i += 8;
            continue;
        }
        out.push(content[i]);
        i += 1;
    }
    out
}

/// Insert the page text fill immediately after math `BT` operators.
fn insert_fill_after_bt(content: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(content.len() + 64);
    let mut i = 0;
    while i < content.len() {
        let at_bt = is_pdf_operator_start(content, i)
            && content[i..].starts_with(b"BT")
            && (i + 2 == content.len() || matches!(content[i + 2], b'\n' | b' ' | b'\r'));
        if at_bt {
            let mut after = i + 2;
            while after < content.len() && matches!(content[after], b'\n' | b' ' | b'\r') {
                after += 1;
            }
            let is_math = MATH_FONTS
                .iter()
                .any(|font| content[after..].starts_with(font));
            if is_math {
                out.extend_from_slice(b"BT\n");
                if !content[after..].starts_with(TEXT_FILL)
                    && !content[after..].starts_with(TEXT_FILL.trim_ascii_end())
                {
                    out.extend_from_slice(TEXT_FILL);
                }
                i = after;
                continue;
            }
        }
        out.push(content[i]);
        i += 1;
    }
    out
}

/// Serialize PDF objects with a rebuilt xref table and trailer.
fn serialize_pdf(objects: &[Vec<u8>], catalog_id: usize) -> Vec<u8> {
    let mut out = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::with_capacity(objects.len());
    for obj in objects {
        offsets.push(out.len());
        out.extend_from_slice(obj);
        if !obj.ends_with(b"\n") {
            out.push(b'\n');
        }
    }
    let xref_offset = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
    out.extend_from_slice(b"0000000000 65535 f \n");
    for offset in offsets {
        out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root {catalog_id} 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n",
            objects.len() + 1,
        )
        .as_bytes(),
    );
    out
}

/// Parse `/Length` from a PDF stream dictionary.
fn parse_length(dict: &[u8]) -> Option<usize> {
    let key = b"/Length ";
    let at = find_subslice(dict, key)?;
    let digits = &dict[at + key.len()..];
    let n: String = digits
        .iter()
        .take_while(|b| b.is_ascii_digit())
        .map(|b| *b as char)
        .collect();
    n.parse().ok()
}

/// Return the index of `needle` in `haystack`.
fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

/// Return whether index `i` starts a PDF operator token.
fn is_pdf_operator_start(pdf: &[u8], i: usize) -> bool {
    i == 0 || matches!(pdf[i - 1], b'\n' | b'\r' | b' ' | b'\t')
}

#[cfg(test)]
mod tests {
    use super::html_to_pdf_bytes;
    use crate::rendering::render_markdown;

    /// Assert `bytes` look like a non-trivial PDF document.
    fn assert_pdf(bytes: &[u8]) {
        assert!(
            bytes.starts_with(b"%PDF"),
            "expected PDF header, got {} bytes starting {:?}",
            bytes.len(),
            bytes.get(..8)
        );
        assert!(bytes.len() > 100, "PDF too small: {} bytes", bytes.len());
        assert!(
            bytes.windows(6).any(|w| w == b"%%EOF\n") || bytes.ends_with(b"%%EOF"),
            "PDF missing EOF marker"
        );
    }

    #[test]
    /// Assert simple HTML converts to a valid PDF.
    fn simple_html_converts_to_pdf() {
        let pdf = html_to_pdf_bytes("<h1>Hello</h1><p>World</p>").unwrap();
        assert_pdf(&pdf);
    }

    #[test]
    /// Assert inline SVG HTML converts to a valid PDF.
    fn inline_svg_converts_to_pdf() {
        let html = r##"<h1>Diagram</h1>
<svg xmlns="http://www.w3.org/2000/svg" width="120" height="80" viewBox="0 0 120 80">
  <rect x="10" y="10" width="100" height="60" fill="#cba6f7"/>
  <text x="60" y="48" text-anchor="middle" font-size="16">SVG</text>
</svg>"##;
        let pdf = html_to_pdf_bytes(html).unwrap();
        assert_pdf(&pdf);
    }

    #[test]
    /// Assert graphviz markdown renders to SVG and converts to PDF.
    fn markdown_graphviz_svg_converts_to_pdf() {
        let body = render_markdown("```graphviz\ndigraph { A -> B }\n```\n");
        assert!(
            body.contains("<svg"),
            "graphviz HTML should contain SVG: {body}"
        );
        assert!(
            body.contains(&format!("fill=\"{}\"", crate::constants::TEXT))
                && body.contains("font-size=\"14px\""),
            "graphviz labels must set fill and font-size as attributes: {body}"
        );
        assert!(
            !body.contains("<tspan"),
            "graphviz labels must not use tspan: {body}"
        );
        let template = include_str!("../../templates/export-pdf.html");
        let pdf = html_to_pdf_bytes(
            &template
                .replace("{{TITLE}}", "t")
                .replace("{{BODY}}", &body),
        )
        .unwrap();
        assert_pdf(&pdf);
        let text = String::from_utf8_lossy(&pdf);
        use crate::constants::TEXT_FILL;
        let idx = text.find("(A)").expect("missing graphviz A");
        let bt = text[..idx].rfind("BT\n").expect("no BT before A");
        let fill_trim = String::from_utf8_lossy(TEXT_FILL).trim().to_string();
        assert!(
            text[bt..idx].contains(&fill_trim),
            "graphviz A must paint with page text fill: {}",
            &text[bt..idx]
        );
    }

    #[test]
    /// Assert mermaid markdown renders to SVG and converts to PDF.
    fn markdown_mermaid_svg_converts_to_pdf() {
        let body =
            render_markdown("```mermaid\ngraph LR\n    A[Square Rect] --> B((Circle))\n```\n");
        assert!(
            body.contains("<svg"),
            "mermaid HTML should contain SVG: {body}"
        );
        assert!(
            !body.contains("dominant-baseline") && body.contains("Courier"),
            "mermaid labels must use an alphabetic baseline and Courier: {body}"
        );
        assert!(
            body.contains("<polygon"),
            "mermaid arrows must be polygons: {body}"
        );
        let template = include_str!("../../templates/export-pdf.html");
        let pdf = html_to_pdf_bytes(
            &template
                .replace("{{TITLE}}", "t")
                .replace("{{BODY}}", &body),
        )
        .unwrap();
        assert_pdf(&pdf);
        let text = String::from_utf8_lossy(&pdf);
        assert!(
            text.contains("Courier") || text.contains("(A)") || text.contains("Square"),
            "PDF should contain mermaid label text"
        );
    }

    #[test]
    /// Assert the PDF export stylesheet sets the editor page background.
    fn pdf_export_stylesheet_sets_page_background() {
        use crate::constants::BG_2;
        let template = include_str!("../../templates/export-pdf.html");
        assert!(
            template.contains("background-color: var(--bg-2)"),
            "PDF export stylesheet must set the editor page background"
        );
        assert!(
            template.contains(&format!("--bg-2: {BG_2}")),
            "PDF export stylesheet must keep the editor background token"
        );
        let pdf = html_to_pdf_bytes(
            &template
                .replace("{{TITLE}}", "t")
                .replace("{{BODY}}", "<p>Hi</p>"),
        )
        .unwrap();
        assert_pdf(&pdf);
    }

    #[test]
    /// Assert PDF math uses `data-math` and converts with page-color fill.
    fn latex_markdown_for_pdf_uses_data_math_and_converts() {
        use crate::constants::{TEXT, TEXT_FILL};
        use crate::rendering::render_markdown_for_pdf;

        let src = "inline $x_i$ and\n\n$$\nx = \\frac{-b \\pm \\sqrt{b^2-4ac}}{2a}\n$$\n";
        let html = render_markdown_for_pdf(src);
        assert!(!html.contains("<math"), "{html}");
        assert!(html.contains("data-math="), "{html}");
        assert!(html.contains(&format!(r#"style="color:{TEXT}""#)), "{html}");
        assert!(
            html.contains(r#"\frac{-b \pm \sqrt{b^2-4ac}}{2a}"#),
            "{html}"
        );
        assert!(!html.contains("<em>"), "{html}");
        let template = include_str!("../../templates/export-pdf.html");
        let pdf = html_to_pdf_bytes(
            &template
                .replace("{{TITLE}}", "t")
                .replace("{{BODY}}", &html),
        )
        .unwrap();
        assert_pdf(&pdf);
        let text = String::from_utf8_lossy(&pdf);
        assert!(
            text.contains("(x)") && text.contains("(b)"),
            "PDF should contain math letters x and b"
        );
        assert!(
            text.contains("Helvetica-Oblique"),
            "italic math letters should use Helvetica-Oblique"
        );
        let fill_trim = String::from_utf8_lossy(TEXT_FILL).trim().to_string();
        for glyph in ["(x)", "(b)"] {
            let idx = text
                .find(glyph)
                .unwrap_or_else(|| panic!("missing {glyph}"));
            let bt = text[..idx]
                .rfind("BT\n")
                .unwrap_or_else(|| panic!("no BT before {glyph}"));
            let chunk = &text[bt..idx];
            assert!(
                chunk.contains(&fill_trim),
                "fill must be inside the text object before {glyph}: {chunk}"
            );
        }
    }

    #[test]
    /// Assert a list-item inline `$\\pi$` converts to PDF without `<li>`.
    fn list_item_inline_pi_converts_to_pdf() {
        use crate::rendering::render_markdown_for_pdf;

        let src = "- $x = \\pi$\n";
        let html = render_markdown_for_pdf(src);
        assert!(html.contains("pdf-li"), "{html}");
        assert!(
            !html.contains("<li>"),
            "PDF lists must not use <li> (ironpress drops inline math): {html}"
        );
        assert!(
            html.contains(r#"class="math-inline""#) && html.contains(r#"data-math="x = \pi""#),
            "{html}"
        );
        assert!(
            !html.contains("math-display"),
            "single-dollar math must stay inline: {html}"
        );
        let template = include_str!("../../templates/export-pdf.html");
        let pdf = html_to_pdf_bytes(
            &template
                .replace("{{TITLE}}", "t")
                .replace("{{BODY}}", &html),
        )
        .unwrap();
        assert_pdf(&pdf);
        let text = String::from_utf8_lossy(&pdf);
        assert!(
            text.contains("(x)"),
            "PDF should contain italic x from $x = \\pi$: {text}"
        );
        assert!(
            text.contains("/Symbol") && text.contains("(\\160)"),
            "PDF should contain Symbol-encoded pi from $x = \\pi$: {text}"
        );
    }
}
