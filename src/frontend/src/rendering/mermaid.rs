/// Render a Mermaid diagram source string to an SVG string.
///
/// On native targets the `rusty-mermaid-svg` crate is used, which shells out
/// to the `mmdc` CLI bundled with `@mermaid-js/mermaid-cli`.
///
/// On WASM/browser targets we emit a `<div class="mermaid">` element instead;
/// the mermaid.js script loaded in `index.html` picks it up and renders it
/// client-side after the DOM is ready.
pub fn render_mermaid(src: &str) -> String {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use rusty_mermaid_svg::generate_svg_from_str;
        match generate_svg_from_str(src) {
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

    #[cfg(target_arch = "wasm32")]
    {
        // Escape the diagram source so it is safe to embed as text content.
        // mermaid.js reads the text content of `.mermaid` divs and replaces
        // them with the rendered SVG.
        let escaped = escape_html(src);
        format!("<div class=\"mermaid\">{escaped}</div>")
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
