//! Fallback HTML for a render that could not process its source.

use crate::rendering::escape_html;

/// Return a fenced fallback block showing `source` and the failure cause.
///
/// `class` names the failing render, for example `graphviz-error`. The cause is
/// kept in an HTML comment so it stays out of the rendered page but remains
/// readable in exported HTML.
pub fn render_error_html(class: &str, source: &str, error: &str) -> String {
    let escaped = escape_html(source);
    format!("<pre class=\"{class}\"><code>{escaped}</code></pre><!-- {class}: {error} -->")
}

#[cfg(test)]
mod tests {
    use super::render_error_html;

    #[test]
    /// Assert the fallback block carries the class, source, and cause.
    fn fallback_block_carries_class_source_and_cause() {
        let html = render_error_html("mermaid-error", "graph LR", "bad token");
        assert_eq!(
            html,
            "<pre class=\"mermaid-error\"><code>graph LR</code></pre>\
             <!-- mermaid-error: bad token -->"
        );
    }

    #[test]
    /// Assert source markup is escaped so it cannot break out of the block.
    fn escapes_source_markup() {
        let html = render_error_html("graphviz-error", "<script>", "parse failed");
        assert!(html.contains("&lt;script&gt;"), "{html}");
        assert!(!html.contains("<script>"), "{html}");
    }
}
