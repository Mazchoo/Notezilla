//! Convert `#rrggbb` hex tokens to PDF DeviceRGB fill/stroke operators.

/// Build `r g b rg\n` or `r g b RG\n` from a `#rrggbb` hex color.
pub(crate) const fn pdf_rgb_operator(hex: &str, fill: bool) -> [u8; 24] {
    let bytes = hex.as_bytes();
    let r = device_rgb_digits(hex_channel(bytes, 1));
    let g = device_rgb_digits(hex_channel(bytes, 3));
    let b = device_rgb_digits(hex_channel(bytes, 5));
    [
        r[0],
        r[1],
        r[2],
        r[3],
        r[4],
        r[5],
        b' ',
        g[0],
        g[1],
        g[2],
        g[3],
        g[4],
        g[5],
        b' ',
        b[0],
        b[1],
        b[2],
        b[3],
        b[4],
        b[5],
        b' ',
        if fill { b'r' } else { b'R' },
        if fill { b'g' } else { b'G' },
        b'\n',
    ]
}

const fn hex_nibble(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        b'A'..=b'F' => byte - b'A' + 10,
        _ => 0,
    }
}

const fn hex_channel(hex: &[u8], offset: usize) -> u8 {
    hex_nibble(hex[offset]) * 16 + hex_nibble(hex[offset + 1])
}

/// Round `byte / 255` to four decimal places (ironpress/Skia DeviceRGB).
const fn device_rgb_10k(byte: u8) -> u16 {
    ((byte as u32 * 10_000 + 127) / 255) as u16
}

const fn device_rgb_digits(byte: u8) -> [u8; 6] {
    let n = device_rgb_10k(byte);
    [
        b'0',
        b'.',
        b'0' + (n / 1000) as u8,
        b'0' + ((n / 100) % 10) as u8,
        b'0' + ((n / 10) % 10) as u8,
        b'0' + (n % 10) as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::pdf_rgb_operator;
    use crate::constants::{ACCENT, TEXT};

    #[test]
    /// Assert `#cdd6f4` becomes the page-text DeviceRGB fill operator.
    fn text_hex_becomes_device_rgb_fill() {
        assert_eq!(TEXT, "#cdd6f4");
        assert_eq!(pdf_rgb_operator(TEXT, true), *b"0.8039 0.8392 0.9569 rg\n");
    }

    #[test]
    /// Assert `#cdd6f4` becomes the page-text DeviceRGB stroke operator.
    fn text_hex_becomes_device_rgb_stroke() {
        assert_eq!(pdf_rgb_operator(TEXT, false), *b"0.8039 0.8392 0.9569 RG\n");
    }

    #[test]
    /// Assert uppercase and lowercase hex produce the same operator.
    fn hex_case_does_not_change_operator() {
        assert_eq!(
            pdf_rgb_operator("#CDD6F4", true),
            pdf_rgb_operator("#cdd6f4", true)
        );
    }

    #[test]
    /// Assert accent `#cba6f7` uses the same four-decimal DeviceRGB rounding.
    fn accent_hex_becomes_device_rgb_fill() {
        assert_eq!(ACCENT, "#cba6f7");
        assert_eq!(
            pdf_rgb_operator(ACCENT, true),
            *b"0.7961 0.6510 0.9686 rg\n"
        );
    }

    #[test]
    /// Assert `#000000` is four-decimal DeviceRGB black.
    fn black_hex_becomes_device_rgb_fill() {
        assert_eq!(
            pdf_rgb_operator("#000000", true),
            *b"0.0000 0.0000 0.0000 rg\n"
        );
    }
}
