pub const DEFAULT_MARKDOWN_PATH: &str = "./example_folder/new_markdown.md";

pub const SIDEBAR_MIN_WIDTH: f64 = 160.0;
pub const SIDEBAR_MAX_WIDTH: f64 = 600.0;

/// Proxied by Trunk to http://127.0.0.1:8020 in development.
pub const MCP_URL: &str = "/mcp";
/// Proxied by Trunk to http://127.0.0.1:11434 in development.
pub const OLLAMA_URL: &str = "/ollama";
pub const OLLAMA_TAGS_PATH: &str = "/api/tags";
pub const OLLAMA_GENERATE_PATH: &str = "/api/generate";
pub const OLLAMA_TEMPERATURE_MIN: f64 = 0.0;
pub const OLLAMA_TEMPERATURE_MAX: f64 = 2.0;
pub const OLLAMA_TOP_P_MIN: f64 = 0.0;
pub const OLLAMA_TOP_P_MAX: f64 = 1.0;

pub const DRAG_MIME: &str = "text/plain";

pub const EXPORT_TEMPLATE: &str = include_str!("../templates/export.html");
pub const EXPORT_PDF_TEMPLATE: &str = include_str!("../templates/export-pdf.html");
pub const PROMPT_TEMPLATE: &str = include_str!("../templates/prompt.md");

/// Placeholder shown for markdown images until real image serving exists.
pub const IMAGE_MISSING_SVG: &str = include_str!("../templates/image-missing.svg");
/// Checkmark drawn on a checked settings checkbox. Served as a static file.
#[allow(dead_code)]
pub const CHECKBOX_CHECK_SVG_HREF: &str = "/checkbox-check.svg";

/// Palette for PDF export. Hex tokens match `templates/export-pdf.html` `:root`.
/// Tokens unused in the WASM binary are asserted against that stylesheet in tests.
#[allow(dead_code)]
pub const BG_0: &str = "#11111b";
#[allow(dead_code)]
pub const BG_1: &str = "#181825";
#[allow(dead_code)]
pub const BG_2: &str = "#1e1e2e";
pub const BG_3: &str = "#313244";
#[allow(dead_code)]
pub const BORDER: &str = "#45475a";
pub const TEXT: &str = "#cdd6f4";
#[allow(dead_code)]
pub const TEXT_MUTED: &str = "#6c7086";
/// Mermaid arrow fallback when a marker URL has no hex suffix (`--subtext0`).
pub const TEXT_SUBTLE: &str = "#a6adc8";
#[allow(dead_code)]
pub const ACCENT: &str = "#cba6f7";
#[allow(dead_code)]
pub const CODE: &str = "#f38ba8";

/// [`TEXT`] as PDF DeviceRGB fill. Ironpress math letters inherit this and
/// never emit `rg`; fraction rules emit hardcoded `0 0 0 rg`.
pub const TEXT_FILL: &[u8] = &crate::rendering::pdf_rgb_operator(TEXT, true);
pub const TEXT_STROKE: &[u8] = &crate::rendering::pdf_rgb_operator(TEXT, false);

pub const DEFAULT_FONT_SIZE: usize = 14;
/// Alphabetic baseline below the node center so Latin caps sit in the box.
pub const BASELINE_FROM_CENTER: f64 = 0.35;

/// rusty-mermaid measures labels as Intel One Mono (~0.6em). Ironpress maps
/// that family to Helvetica, so PDF uses Courier (standard 14 monospace).
pub const PDF_FONT_FAMILY: &str = "Courier, monospace";
/// Matches rusty_mermaid_core::constants::BASELINE_ASCENT_RATIO.
pub const BASELINE_ASCENT_RATIO: f64 = 0.3;
pub const ARROW_SIZE: f64 = 8.0;
/// Font family of flattened graphviz node labels.
pub const GRAPHVIZ_LABEL_FONT_FAMILY: &str = "Helvetica, Arial, sans-serif";
/// Extra mermaid padding so a stroke centered on the viewBox edge is not clipped.
pub const MERMAID_STROKE_SLOP: f64 = 1.0;

/// Syntect theme applied to fenced code blocks.
pub const CODE_THEME: &str = "base16-ocean.dark";
/// Item marker of an unordered list rewritten for PDF export.
pub const LIST_BULLET: &str = "•";

/// Classes of the fallback block a failing render emits. Styled by `index.html`.
pub const GRAPHVIZ_ERROR_CLASS: &str = "graphviz-error";
pub const MERMAID_ERROR_CLASS: &str = "mermaid-error";

/// XML declaration layout-rs prefixes to its SVG output.
pub const SVG_XML_DECLARATION: &str =
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>";

/// Header of every PDF ironpress produces.
pub const PDF_HEADER: &[u8] = b"%PDF-1.4\n";
/// Black DeviceRGB operators ironpress hardcodes for math, replaced by
/// [`TEXT_FILL`] and [`TEXT_STROKE`].
pub const PDF_BLACK_FILL: &[u8] = b"0 0 0 rg";
pub const PDF_BLACK_STROKE: &[u8] = b"0 0 0 RG";

pub const MATH_FONTS: [&[u8]; 3] = [b"/Helvetica-Oblique ", b"/Symbol ", b"/Helvetica "];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Assert the PDF export stylesheet matches the editor palette tokens.
    fn export_pdf_stylesheet_matches_palette() {
        for (token, hex) in [
            ("--bg-0", BG_0),
            ("--bg-1", BG_1),
            ("--bg-2", BG_2),
            ("--bg-3", BG_3),
            ("--border", BORDER),
            ("--text", TEXT),
            ("--text-muted", TEXT_MUTED),
            ("--accent", ACCENT),
            ("--code", CODE),
        ] {
            let decl = format!("{token}: {hex}");
            assert!(
                EXPORT_PDF_TEMPLATE.contains(&decl),
                "missing {decl} in {EXPORT_PDF_TEMPLATE}"
            );
        }
    }

    #[test]
    /// Assert inline code color uses the CODE token alias, not a raw hex.
    fn export_pdf_stylesheet_uses_code_alias() {
        assert!(
            EXPORT_PDF_TEMPLATE.contains("color: var(--code)"),
            "inline code must use var(--code): {EXPORT_PDF_TEMPLATE}"
        );
        assert!(
            !EXPORT_PDF_TEMPLATE.contains(&format!("color: {CODE}")),
            "inline code must not hardcode CODE hex: {EXPORT_PDF_TEMPLATE}"
        );
    }

    #[test]
    /// Assert the checkbox checkmark is a file URL, not an embedded SVG string.
    fn index_css_loads_checkbox_check_from_file() {
        const CSS: &str = include_str!("../index.css");
        assert!(
            !CSS.contains("data:image/svg+xml"),
            "CSS must not embed SVG as a data URI: {CSS}"
        );
        let url = format!("url(\"{CHECKBOX_CHECK_SVG_HREF}\")");
        assert!(
            CSS.contains(&url),
            "checked checkbox must load {CHECKBOX_CHECK_SVG_HREF}"
        );
        let svg = include_str!("../templates/checkbox-check.svg");
        assert!(svg.contains("<svg"), "{svg}");
        assert!(svg.contains("</svg>"), "{svg}");
    }
}
