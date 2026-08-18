//! Palette for PDF export. Hex tokens match `templates/export-pdf.html` `:root`.

pub(crate) const BG_0: &str = "#11111b";
pub(crate) const BG_1: &str = "#181825";
pub(crate) const BG_2: &str = "#1e1e2e";
pub(crate) const BG_3: &str = "#313244";
pub(crate) const BORDER: &str = "#45475a";
pub(crate) const TEXT: &str = "#cdd6f4";
pub(crate) const TEXT_MUTED: &str = "#6c7086";
/// Mermaid arrow fallback when a marker URL has no hex suffix (`--subtext0`).
pub(crate) const TEXT_SUBTLE: &str = "#a6adc8";
pub(crate) const ACCENT: &str = "#cba6f7";
pub(crate) const CODE: &str = "#f38ba8";

/// [`TEXT`] as PDF DeviceRGB fill. Ironpress math letters inherit this and
/// never emit `rg`; fraction rules emit hardcoded `0 0 0 rg`.
pub(crate) const TEXT_FILL: &[u8] = b"0.8039 0.8392 0.9569 rg\n";
pub(crate) const TEXT_STROKE: &[u8] = b"0.8039 0.8392 0.9569 RG\n";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_pdf_stylesheet_matches_palette() {
        let template = include_str!("../../templates/export-pdf.html");
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
            assert!(template.contains(&decl), "missing {decl} in {template}");
        }
        assert!(
            template.contains(&format!("color: {CODE}")) || template.contains(CODE),
            "stylesheet must use CODE pink for inline code: {template}"
        );
    }
}
