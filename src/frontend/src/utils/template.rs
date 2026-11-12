//! `{{NAME}}` placeholder substitution.

/// Return `template` with each `{{NAME}}` listed in `values` replaced in one pass.
///
/// Unlisted placeholders are kept as written, and substituted text is never
/// rescanned, so a value containing `{{BODY}}` is inserted literally.
pub fn fill_placeholders(template: &str, values: &[(&str, &str)]) -> String {
    let extra: usize = values.iter().map(|(_, value)| value.len()).sum();
    let mut out = String::with_capacity(template.len() + extra);
    let mut rest = template;
    while let Some(open) = rest.find("{{") {
        let after = &rest[open + 2..];
        let Some(close) = after.find("}}") else {
            break;
        };
        let name = &after[..close];
        match values.iter().find(|(key, _)| *key == name) {
            Some((_, value)) => {
                out.push_str(&rest[..open]);
                out.push_str(value);
                rest = &after[close + 2..];
            }
            None => {
                out.push_str(&rest[..open + 2]);
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::fill_placeholders;

    #[test]
    /// Assert every listed placeholder is replaced, including repeats.
    fn fills_listed_placeholders() {
        let out = fill_placeholders("{{A}}-{{B}}-{{A}}", &[("A", "1"), ("B", "2")]);
        assert_eq!(out, "1-2-1");
    }

    #[test]
    /// Assert a name that prefixes another does not match it.
    fn prefix_names_do_not_collide() {
        let out = fill_placeholders(
            "{{THEME}} {{THEME_CSS}}",
            &[("THEME", "t"), ("THEME_CSS", "c")],
        );
        assert_eq!(out, "t c");
    }

    #[test]
    /// Assert unlisted and unterminated placeholders are kept verbatim.
    fn keeps_unknown_placeholders() {
        let out = fill_placeholders("{{X}} {{ {{A}} {{", &[("A", "1")]);
        assert_eq!(out, "{{X}} {{ 1 {{");
    }

    #[test]
    /// Assert substituted values are not scanned for further placeholders.
    fn does_not_rescan_values() {
        let out = fill_placeholders("{{A}}|{{B}}", &[("A", "{{B}}"), ("B", "2")]);
        assert_eq!(out, "{{B}}|2");
    }
}
