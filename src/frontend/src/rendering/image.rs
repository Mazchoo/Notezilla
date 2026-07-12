use super::escape_html;

/// Placeholder shown for markdown images until real image serving exists.
const IMAGE_MISSING_SVG: &str = include_str!("../../templates/image-missing.svg");

/// Inline HTML for a missing/unavailable markdown image.
pub fn missing_image_html(alt: &str) -> String {
    let svg = IMAGE_MISSING_SVG
        .find("<svg")
        .map(|i| &IMAGE_MISSING_SVG[i..])
        .unwrap_or(IMAGE_MISSING_SVG)
        .trim();
    let alt_esc = escape_html(alt);
    format!(r#"<span class="md-image-missing" role="img" aria-label="{alt_esc}">{svg}</span>"#)
}
