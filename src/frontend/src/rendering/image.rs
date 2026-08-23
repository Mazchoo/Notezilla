use super::escape_html;
use crate::constants::IMAGE_MISSING_SVG;

/// Return inline HTML for a missing markdown image.
pub fn missing_image_html(alt: &str) -> String {
    let svg = IMAGE_MISSING_SVG
        .find("<svg")
        .map(|i| &IMAGE_MISSING_SVG[i..])
        .unwrap_or(IMAGE_MISSING_SVG)
        .trim();
    let alt_esc = escape_html(alt);
    format!(r#"<span class="md-image-missing" role="img" aria-label="{alt_esc}">{svg}</span>"#)
}
