use super::pdf_colors::{BG_3 as NODE_FILL, TEXT as STROKE, TEXT as TEXT_FILL};
use layout::backends::svg::SVGWriter;
use layout::gv::{DotParser, GraphBuilder};
use std::collections::HashMap;

const DEFAULT_FONT_SIZE: usize = 14;
/// Alphabetic baseline below the node center so Latin caps sit in the box.
const BASELINE_FROM_CENTER: f64 = 0.35;

pub fn render_dot(dot: &str) -> Result<String, String> {
    let mut parser = DotParser::new(dot);
    let tree = parser.process().map_err(|e| format!("{e:?}"))?;
    let mut builder = GraphBuilder::new();
    builder.visit_graph(&tree);
    let mut vg = builder.get();
    let mut svg = SVGWriter::new();
    vg.do_it(false, false, false, &mut svg);
    Ok(prepare_graphviz_svg(&svg.finalize()))
}

/// layout-rs emits CSS classes, `tspan dy`, and `dominant-baseline` that
/// browsers honor and ironpress does not. Flatten labels to presentation
/// attributes and map default black/white paints onto the editor palette.
fn prepare_graphviz_svg(raw: &str) -> String {
    let without_xml = raw
        .trim_start()
        .strip_prefix("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>")
        .unwrap_or(raw)
        .trim_start();
    let fonts = parse_font_classes(without_xml);
    let labeled = rewrite_text_elements(without_xml, &fonts);
    labeled
        .replace("fill=\"#ffffffff\"", &format!("fill=\"{NODE_FILL}\""))
        .replace("stroke=\"#000000ff\"", &format!("stroke=\"{STROKE}\""))
        .replace("fill=\"context-stroke\"", &format!("fill=\"{STROKE}\""))
}

fn parse_font_classes(svg: &str) -> HashMap<String, usize> {
    let mut fonts = HashMap::new();
    let mut rest = svg;
    while let Some(at) = rest.find(".a") {
        rest = &rest[at + 2..];
        let size: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if size.is_empty() {
            continue;
        }
        if let Ok(px) = size.parse::<usize>() {
            fonts.insert(format!("a{size}"), px);
        }
    }
    fonts
}

fn rewrite_text_elements(svg: &str, fonts: &HashMap<String, usize>) -> String {
    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(start) = rest.find("<text") {
        out.push_str(&rest[..start]);
        let Some(end_rel) = rest[start..].find("</text>") else {
            out.push_str(rest);
            return out;
        };
        let end = start + end_rel + "</text>".len();
        let elem = &rest[start..end];
        if elem.contains("<textPath") {
            out.push_str(elem);
        } else {
            out.push_str(&flatten_node_label(elem, fonts));
        }
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

fn flatten_node_label(elem: &str, fonts: &HashMap<String, usize>) -> String {
    let gt = elem.find('>').unwrap_or(elem.len());
    let open = &elem[..gt];
    let x = attr(open, "x").and_then(|v| v.parse::<f64>().ok());
    let y = attr(open, "y").and_then(|v| v.parse::<f64>().ok());
    let (x, y) = match (x, y) {
        (Some(x), Some(y)) => (x, y),
        _ => return elem.to_string(),
    };
    let font_size = attr(open, "class")
        .and_then(|class| fonts.get(&class).copied())
        .unwrap_or(DEFAULT_FONT_SIZE) as f64;

    let lines = tspan_lines(&elem[gt + 1..]);
    if lines.is_empty() {
        return String::new();
    }
    let n = lines.len() as f64;
    let center_y = y + (n + 1.0) * font_size / 2.0;
    let mut out = String::new();
    for (i, (line_x, line)) in lines.into_iter().enumerate() {
        let x = line_x.unwrap_or(x);
        let baseline =
            center_y + (i as f64 - (n - 1.0) / 2.0) * font_size + font_size * BASELINE_FROM_CENTER;
        out.push_str(&format!(
            "<text text-anchor=\"middle\" x=\"{x}\" y=\"{baseline}\" font-size=\"{font_size}px\" \
             font-family=\"Helvetica, Arial, sans-serif\" fill=\"{TEXT_FILL}\">{}</text>",
            line
        ));
    }
    out
}

fn tspan_lines(inner: &str) -> Vec<(Option<f64>, String)> {
    let mut lines = Vec::new();
    let mut rest = inner;
    while let Some(start) = rest.find("<tspan") {
        let Some(open_end) = rest[start..].find('>') else {
            break;
        };
        let open = &rest[start..start + open_end];
        let content_at = start + open_end + 1;
        let Some(close_rel) = rest[content_at..].find("</tspan>") else {
            break;
        };
        let content = rest[content_at..content_at + close_rel].trim();
        if !content.is_empty() {
            let x = attr(open, "x").and_then(|v| v.parse::<f64>().ok());
            lines.push((x, content.to_string()));
        }
        rest = &rest[content_at + close_rel + "</tspan>".len()..];
    }
    if lines.is_empty() {
        let text = inner.trim().trim_end_matches("</text>").trim().to_string();
        if !text.is_empty() && !text.contains('<') {
            lines.push((None, text));
        }
    }
    lines
}

fn attr(tag: &str, name: &str) -> Option<String> {
    let bytes = tag.as_bytes();
    let name_b = name.as_bytes();
    let mut i = 0;
    while i + name_b.len() < bytes.len() {
        if bytes[i..].starts_with(name_b) && (i == 0 || !bytes[i - 1].is_ascii_alphanumeric()) {
            let mut j = i + name_b.len();
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'=' {
                j += 1;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j < bytes.len() && (bytes[j] == b'"' || bytes[j] == b'\'') {
                    let quote = bytes[j];
                    j += 1;
                    let start = j;
                    while j < bytes.len() && bytes[j] != quote {
                        j += 1;
                    }
                    return Some(tag[start..j].to_string());
                }
            }
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graphviz_svg_inlines_label_paint_and_baseline() {
        let svg = render_dot("digraph { A -> B }").unwrap();
        assert!(!svg.contains("<?xml"), "{svg}");
        assert!(
            !svg.contains("dominant-baseline") && !svg.contains("<tspan"),
            "labels must not rely on tspan/dominant-baseline: {svg}"
        );
        assert!(svg.contains(&format!("fill=\"{TEXT_FILL}\"")), "{svg}");
        assert!(svg.contains("font-size=\"14px\""), "{svg}");
        assert!(svg.contains(&format!("fill=\"{NODE_FILL}\"")), "{svg}");

        let cy: f64 = svg
            .split("cy=\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .and_then(|s| s.parse().ok())
            .expect("ellipse cy");
        let text_y: f64 = svg
            .split("<text ")
            .nth(1)
            .and_then(|s| s.split("y=\"").nth(1))
            .and_then(|s| s.split('"').next())
            .and_then(|s| s.parse().ok())
            .expect("text y");
        let expected = cy + 14.0 * BASELINE_FROM_CENTER;
        assert!(
            (text_y - expected).abs() < 0.6,
            "label baseline {text_y} should sit near ellipse cy {cy} (expected {expected}): {svg}"
        );
    }
}
