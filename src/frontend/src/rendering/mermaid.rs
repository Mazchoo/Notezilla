use rusty_mermaid::{to_svg, Color, Theme};

/// Render a Mermaid diagram source string to an inline SVG string.
///
/// Uses the pure-Rust `rusty-mermaid` pipeline (parse → layout → SVG) so the
/// same code runs in WASM and on native, with no JavaScript runtime, no CDN
/// script, and no DOM post-processing step.
pub fn render_mermaid(src: &str) -> String {
    // The dark theme's text (#cdd6f4) matches the app's --text so labels read
    // correctly both on shape fills and on the diagram background. Setting
    // `background` to white suppresses the background <rect> in the SVG (the
    // renderer skips it for white), letting the editor surface show through.
    let theme = Theme {
        background: Color::WHITE,
        ..Theme::dark()
    };
    match to_svg(src, &theme) {
        Ok(svg) => svg,
        Err(e) => {
            let escaped = escape_html(src);
            format!(
                "<pre class=\"mermaid-error\"><code>{escaped}</code></pre>\
                 <!-- mermaid error: {e} -->"
            )
        }
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
