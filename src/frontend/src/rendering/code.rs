/// Render a fenced code block with syntax highlighting using syntect.
///
/// Uses the `fancy-regex` backend so this compiles to both native and
/// `wasm32-unknown-unknown` targets without any JavaScript dependency.
/// Output is inline-styled HTML — no external stylesheet required.
pub fn highlight_code(lang: &str, src: &str) -> String {
    use syntect::easy::HighlightLines;
    use syntect::highlighting::ThemeSet;
    use syntect::html::{styled_line_to_highlighted_html, IncludeBackground};
    use syntect::parsing::SyntaxSet;

    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.dark"];

    let syntax = if lang.is_empty() {
        ss.find_syntax_plain_text()
    } else {
        ss.find_syntax_by_token(lang)
            .unwrap_or_else(|| ss.find_syntax_plain_text())
    };

    let mut h = HighlightLines::new(syntax, theme);
    let mut html = String::from("<pre class=\"code-block\"><code>");

    for line in syntect::util::LinesWithEndings::from(src) {
        match h.highlight_line(line, &ss) {
            Ok(ranges) => {
                match styled_line_to_highlighted_html(&ranges[..], IncludeBackground::No) {
                    Ok(line_html) => html.push_str(&line_html),
                    Err(_) => html.push_str(&escape_html(line)),
                }
            }
            Err(_) => html.push_str(&escape_html(line)),
        }
    }

    html.push_str("</code></pre>");
    html
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
