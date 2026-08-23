pub const DEFAULT_MARKDOWN_PATH: &str = "./example_folder/new_markdown.md";
pub const DEFAULT_MARKDOWN_TEMPLATE: &str = include_str!("../templates/new_markdown.md");

pub const SIDEBAR_MIN_WIDTH: f64 = 160.0;
pub const SIDEBAR_MAX_WIDTH: f64 = 600.0;

/// Proxied by Trunk to http://127.0.0.1:8020 in development.
pub const MCP_URL: &str = "/mcp";

pub const FILE_READ_ERROR_TOAST: &str = "File cannot be read";
pub const DRAG_MIME: &str = "text/plain";

pub const EXPORT_TEMPLATE: &str = include_str!("../templates/export.html");
pub const EXPORT_PDF_TEMPLATE: &str = include_str!("../templates/export-pdf.html");

/// Placeholder shown for markdown images until real image serving exists.
pub const IMAGE_MISSING_SVG: &str = include_str!("../templates/image-missing.svg");

/// Palette for PDF export. Hex tokens match `templates/export-pdf.html` `:root`.
pub const BG_0: &str = "#11111b";
pub const BG_1: &str = "#181825";
pub const BG_2: &str = "#1e1e2e";
pub const BG_3: &str = "#313244";
pub const BORDER: &str = "#45475a";
pub const TEXT: &str = "#cdd6f4";
pub const TEXT_MUTED: &str = "#6c7086";
/// Mermaid arrow fallback when a marker URL has no hex suffix (`--subtext0`).
pub const TEXT_SUBTLE: &str = "#a6adc8";
pub const ACCENT: &str = "#cba6f7";
pub const CODE: &str = "#f38ba8";

/// [`TEXT`] as PDF DeviceRGB fill. Ironpress math letters inherit this and
/// never emit `rg`; fraction rules emit hardcoded `0 0 0 rg`.
pub const TEXT_FILL: &[u8] = b"0.8039 0.8392 0.9569 rg\n";
pub const TEXT_STROKE: &[u8] = b"0.8039 0.8392 0.9569 RG\n";

pub const DEFAULT_FONT_SIZE: usize = 14;
/// Alphabetic baseline below the node center so Latin caps sit in the box.
pub const BASELINE_FROM_CENTER: f64 = 0.35;

/// rusty-mermaid measures labels as Intel One Mono (~0.6em). Ironpress maps
/// that family to Helvetica, so PDF uses Courier (standard 14 monospace).
pub const PDF_FONT_FAMILY: &str = "Courier, monospace";
/// Matches rusty_mermaid_core::constants::BASELINE_ASCENT_RATIO.
pub const BASELINE_ASCENT_RATIO: f64 = 0.3;
pub const ARROW_SIZE: f64 = 8.0;

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
        ] {
            let decl = format!("{token}: {hex}");
            assert!(
                EXPORT_PDF_TEMPLATE.contains(&decl),
                "missing {decl} in {EXPORT_PDF_TEMPLATE}"
            );
        }
        assert!(
            EXPORT_PDF_TEMPLATE.contains(&format!("color: {CODE}"))
                || EXPORT_PDF_TEMPLATE.contains(CODE),
            "stylesheet must use CODE pink for inline code: {EXPORT_PDF_TEMPLATE}"
        );
    }
}
