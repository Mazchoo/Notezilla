use crate::components::hotkeys::normalize_hotkey_key;
use crate::components::toast::show_error_toast;
use crate::info_messages::{invalid_setting_toast, HOTKEY_CONSTRAINT};
use leptos::prelude::*;
use wasm_bindgen::JsCast;

/// Capture a printable key press into a hotkey settings signal.
pub(crate) fn on_hotkey_keydown(
    ev: web_sys::KeyboardEvent,
    target: RwSignal<char>,
    error_toast: RwSignal<Option<String>>,
    label: &'static str,
) {
    let key = ev.key();
    if matches!(
        key.as_str(),
        "Tab" | "Escape" | "Shift" | "Control" | "Alt" | "Meta"
    ) {
        return;
    }
    ev.prevent_default();
    match normalize_hotkey_key(&key) {
        Some(ch) => target.set(ch),
        None => show_error_toast(error_toast, invalid_setting_toast(label, HOTKEY_CONSTRAINT)),
    }
}

/// Parse a TCP port from a settings input, rejecting 0 and non-numeric values.
pub(crate) fn parse_ollama_port(raw: &str) -> Option<u16> {
    let n = raw.parse::<u16>().ok()?;
    (n != 0).then_some(n)
}

/// Return whether `raw` is a float still being typed, such as `0.` or `1e-`.
fn is_incomplete_float(raw: &str) -> bool {
    let raw = raw.trim();
    raw.is_empty()
        || raw == "-"
        || raw == "."
        || raw == "-."
        || raw.ends_with('.')
        || raw.ends_with(['e', 'E'])
        || raw.ends_with("e-")
        || raw.ends_with("E-")
        || raw.ends_with("e+")
        || raw.ends_with("E+")
}

/// Parse a finite float in `[min, max]` from a settings input.
pub(crate) fn parse_bounded_f64(raw: &str, min: f64, max: f64) -> Option<f64> {
    if is_incomplete_float(raw) {
        return None;
    }
    let n = raw.parse::<f64>().ok()?;
    (n.is_finite() && (min..=max).contains(&n)).then_some(n)
}

/// Parse Ollama `num_predict` from a settings input. `-1` is unlimited.
pub(crate) fn parse_ollama_num_predict(raw: &str) -> Option<i32> {
    let n = raw.parse::<i32>().ok()?;
    (n >= -1).then_some(n)
}

/// Parse a positive integer from a settings input.
pub(crate) fn parse_positive_u32(raw: &str) -> Option<u32> {
    raw.parse::<u32>().ok().filter(|&n| n > 0)
}

/// Parse results-per-page from a settings input, rejecting 0 and non-numeric values.
pub(crate) fn parse_results_per_page(raw: &str) -> Option<usize> {
    let n = raw.parse::<usize>().ok()?;
    (n > 0).then_some(n)
}

/// Parse a non-empty Ollama model name.
pub(crate) fn parse_ollama_model(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

/// Parse a Gemini API key. Empty is allowed; whitespace inside the key is not.
pub(crate) fn parse_gemini_api_key(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    (!trimmed.chars().any(char::is_whitespace)).then(|| trimmed.to_string())
}

/// Parse a Gemini model id for `/v1beta/models/{model}`. Reject empty values and `/`.
pub(crate) fn parse_gemini_model(raw: &str) -> Option<String> {
    let trimmed = parse_ollama_model(raw)?;
    (!trimmed.contains('/')).then_some(trimmed)
}

/// Return the parsed value, or the invalid-setting toast text.
pub(crate) fn validate_setting<T>(
    raw: &str,
    parse: impl FnOnce(&str) -> Option<T>,
    label: &str,
    constraint: &str,
) -> Result<T, String> {
    parse(raw).ok_or_else(|| invalid_setting_toast(label, constraint))
}

/// Restore an input to `value` after a rejected change.
fn restore_input_value(ev: &web_sys::Event, value: &str) {
    let Some(target) = ev.target() else {
        return;
    };
    let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() else {
        return;
    };
    input.set_value(value);
}

/// Apply a parsed setting, or toast and restore the input when `raw` is invalid.
pub(crate) fn commit_setting<T>(
    ev: web_sys::Event,
    parse: impl FnOnce(&str) -> Option<T>,
    target: RwSignal<T>,
    error_toast: RwSignal<Option<String>>,
    label: &'static str,
    constraint: &'static str,
) where
    T: Clone + ToString + Send + Sync + 'static,
{
    let raw = event_target_value(&ev);
    match validate_setting(&raw, parse, label, constraint) {
        Ok(value) => target.set(value),
        Err(message) => {
            show_error_toast(error_toast, message);
            restore_input_value(&ev, &target.get_untracked().to_string());
        }
    }
}

/// Apply a setting when `raw` is valid. Ignore incomplete values while typing.
pub(crate) fn apply_if_valid<T>(
    raw: &str,
    parse: impl FnOnce(&str) -> Option<T>,
    target: RwSignal<T>,
) where
    T: Clone + Send + Sync + 'static,
{
    if let Some(value) = parse(raw) {
        target.set(value);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        parse_bounded_f64, parse_gemini_api_key, parse_gemini_model, parse_ollama_model,
        parse_ollama_num_predict, parse_ollama_port, parse_positive_u32, parse_results_per_page,
        validate_setting,
    };
    use crate::components::hotkeys::normalize_hotkey_key;
    use crate::constants::{
        gemini_model_url, OLLAMA_TEMPERATURE_MAX, OLLAMA_TEMPERATURE_MIN, OLLAMA_TOP_P_MAX,
        OLLAMA_TOP_P_MIN,
    };
    use crate::default_settings::{
        DEFAULT_GEMINI_API_KEY, DEFAULT_GEMINI_MODEL, DEFAULT_OLLAMA_MODEL,
        DEFAULT_OLLAMA_NUM_CTX, DEFAULT_OLLAMA_NUM_PREDICT, DEFAULT_OLLAMA_PORT,
        DEFAULT_OLLAMA_TEMPERATURE, DEFAULT_OLLAMA_TOP_K, DEFAULT_OLLAMA_TOP_P,
    };
    use crate::info_messages::{
        invalid_setting_toast, HOTKEY_CONSTRAINT, OLLAMA_MODEL_CONSTRAINT, OLLAMA_MODEL_LABEL,
        OLLAMA_NUM_CTX_CONSTRAINT, OLLAMA_NUM_CTX_LABEL, OLLAMA_NUM_PREDICT_CONSTRAINT,
        OLLAMA_NUM_PREDICT_LABEL, OLLAMA_PORT_CONSTRAINT, OLLAMA_PORT_LABEL,
        OLLAMA_TEMPERATURE_CONSTRAINT, OLLAMA_TEMPERATURE_LABEL, OLLAMA_TOP_K_CONSTRAINT,
        OLLAMA_TOP_K_LABEL, OLLAMA_TOP_P_CONSTRAINT, OLLAMA_TOP_P_LABEL,
        RESULTS_PER_PAGE_CONSTRAINT, RESULTS_PER_PAGE_LABEL, SAVE_HOTKEY_LABEL,
    };

    #[test]
    /// Assert the Ollama port input accepts 1..=65535 and rejects 0 and non-numeric values.
    fn parse_ollama_port_accepts_valid_tcp_ports() {
        assert_eq!(parse_ollama_port("11434"), Some(DEFAULT_OLLAMA_PORT));
        assert_eq!(parse_ollama_port("1"), Some(1));
        assert_eq!(parse_ollama_port("65535"), Some(65535));
        assert_eq!(parse_ollama_port("0"), None);
        assert_eq!(parse_ollama_port(""), None);
        assert_eq!(parse_ollama_port("abc"), None);
        assert_eq!(parse_ollama_port("65536"), None);
    }

    #[test]
    /// Assert in-progress decimals such as `0.` are not stored as `0`.
    fn parse_bounded_f64_rejects_incomplete_decimals() {
        assert_eq!(parse_bounded_f64("0.", 0.0, 2.0), None);
        assert_eq!(parse_bounded_f64(".", 0.0, 2.0), None);
        assert_eq!(parse_bounded_f64("1e-", 0.0, 2.0), None);
        assert_eq!(parse_bounded_f64("0.5", 0.0, 2.0), Some(0.5));
        assert_eq!(
            parse_bounded_f64("0.8", 0.0, 2.0),
            Some(DEFAULT_OLLAMA_TEMPERATURE)
        );
        assert_eq!(
            parse_bounded_f64("0.9", 0.0, 1.0),
            Some(DEFAULT_OLLAMA_TOP_P)
        );
    }

    #[test]
    /// Assert each setting validator accepts a valid value and returns a toast for an invalid one.
    fn validate_setting_returns_toast_text_when_invalid() {
        assert_eq!(
            validate_setting(
                "20",
                parse_results_per_page,
                RESULTS_PER_PAGE_LABEL,
                RESULTS_PER_PAGE_CONSTRAINT
            ),
            Ok(20)
        );
        assert_eq!(
            validate_setting(
                "0",
                parse_results_per_page,
                RESULTS_PER_PAGE_LABEL,
                RESULTS_PER_PAGE_CONSTRAINT
            ),
            Err(invalid_setting_toast(
                RESULTS_PER_PAGE_LABEL,
                RESULTS_PER_PAGE_CONSTRAINT
            ))
        );
        assert_eq!(
            validate_setting(
                "11434",
                parse_ollama_port,
                OLLAMA_PORT_LABEL,
                OLLAMA_PORT_CONSTRAINT
            ),
            Ok(DEFAULT_OLLAMA_PORT)
        );
        assert_eq!(
            validate_setting(
                "0",
                parse_ollama_port,
                OLLAMA_PORT_LABEL,
                OLLAMA_PORT_CONSTRAINT
            ),
            Err(invalid_setting_toast(
                OLLAMA_PORT_LABEL,
                OLLAMA_PORT_CONSTRAINT
            ))
        );
        assert_eq!(
            validate_setting(
                DEFAULT_OLLAMA_MODEL,
                parse_ollama_model,
                OLLAMA_MODEL_LABEL,
                OLLAMA_MODEL_CONSTRAINT
            ),
            Ok(DEFAULT_OLLAMA_MODEL.to_string())
        );
        assert_eq!(
            validate_setting(
                "  ",
                parse_ollama_model,
                OLLAMA_MODEL_LABEL,
                OLLAMA_MODEL_CONSTRAINT
            ),
            Err(invalid_setting_toast(
                OLLAMA_MODEL_LABEL,
                OLLAMA_MODEL_CONSTRAINT
            ))
        );
        assert_eq!(
            validate_setting(
                "0.8",
                |s| parse_bounded_f64(s, OLLAMA_TEMPERATURE_MIN, OLLAMA_TEMPERATURE_MAX),
                OLLAMA_TEMPERATURE_LABEL,
                OLLAMA_TEMPERATURE_CONSTRAINT,
            ),
            Ok(DEFAULT_OLLAMA_TEMPERATURE)
        );
        assert_eq!(
            validate_setting(
                "3",
                |s| parse_bounded_f64(s, OLLAMA_TEMPERATURE_MIN, OLLAMA_TEMPERATURE_MAX),
                OLLAMA_TEMPERATURE_LABEL,
                OLLAMA_TEMPERATURE_CONSTRAINT,
            ),
            Err(invalid_setting_toast(
                OLLAMA_TEMPERATURE_LABEL,
                OLLAMA_TEMPERATURE_CONSTRAINT
            ))
        );
        assert_eq!(
            validate_setting(
                "-1",
                parse_ollama_num_predict,
                OLLAMA_NUM_PREDICT_LABEL,
                OLLAMA_NUM_PREDICT_CONSTRAINT
            ),
            Ok(DEFAULT_OLLAMA_NUM_PREDICT)
        );
        assert_eq!(
            validate_setting(
                "-2",
                parse_ollama_num_predict,
                OLLAMA_NUM_PREDICT_LABEL,
                OLLAMA_NUM_PREDICT_CONSTRAINT
            ),
            Err(invalid_setting_toast(
                OLLAMA_NUM_PREDICT_LABEL,
                OLLAMA_NUM_PREDICT_CONSTRAINT
            ))
        );
        assert_eq!(
            validate_setting(
                "2048",
                parse_positive_u32,
                OLLAMA_NUM_CTX_LABEL,
                OLLAMA_NUM_CTX_CONSTRAINT
            ),
            Ok(DEFAULT_OLLAMA_NUM_CTX)
        );
        assert_eq!(
            validate_setting(
                "0",
                parse_positive_u32,
                OLLAMA_NUM_CTX_LABEL,
                OLLAMA_NUM_CTX_CONSTRAINT
            ),
            Err(invalid_setting_toast(
                OLLAMA_NUM_CTX_LABEL,
                OLLAMA_NUM_CTX_CONSTRAINT
            ))
        );
        assert_eq!(
            validate_setting(
                "0.9",
                |s| parse_bounded_f64(s, OLLAMA_TOP_P_MIN, OLLAMA_TOP_P_MAX),
                OLLAMA_TOP_P_LABEL,
                OLLAMA_TOP_P_CONSTRAINT,
            ),
            Ok(DEFAULT_OLLAMA_TOP_P)
        );
        assert_eq!(
            validate_setting(
                "1.1",
                |s| parse_bounded_f64(s, OLLAMA_TOP_P_MIN, OLLAMA_TOP_P_MAX),
                OLLAMA_TOP_P_LABEL,
                OLLAMA_TOP_P_CONSTRAINT,
            ),
            Err(invalid_setting_toast(
                OLLAMA_TOP_P_LABEL,
                OLLAMA_TOP_P_CONSTRAINT
            ))
        );
        assert_eq!(
            validate_setting(
                "40",
                parse_positive_u32,
                OLLAMA_TOP_K_LABEL,
                OLLAMA_TOP_K_CONSTRAINT
            ),
            Ok(DEFAULT_OLLAMA_TOP_K)
        );
        assert_eq!(
            validate_setting(
                "0",
                parse_positive_u32,
                OLLAMA_TOP_K_LABEL,
                OLLAMA_TOP_K_CONSTRAINT
            ),
            Err(invalid_setting_toast(
                OLLAMA_TOP_K_LABEL,
                OLLAMA_TOP_K_CONSTRAINT
            ))
        );
        assert_eq!(
            validate_setting(
                "Enter",
                normalize_hotkey_key,
                SAVE_HOTKEY_LABEL,
                HOTKEY_CONSTRAINT
            ),
            Err(invalid_setting_toast(SAVE_HOTKEY_LABEL, HOTKEY_CONSTRAINT))
        );
    }

    #[test]
    /// Assert Gemini API key and model accept values that fit `/v1beta/models/{model}?key={api_key}`.
    fn parse_gemini_settings_accept_key_and_model_for_models_url() {
        assert_eq!(
            parse_gemini_api_key(DEFAULT_GEMINI_API_KEY),
            Some(DEFAULT_GEMINI_API_KEY.to_string())
        );
        assert_eq!(
            parse_gemini_api_key("  AIzaSyExample  "),
            Some("AIzaSyExample".to_string())
        );
        assert_eq!(parse_gemini_api_key("AIza Sy"), None);
        assert_eq!(
            parse_gemini_model(DEFAULT_GEMINI_MODEL),
            Some(DEFAULT_GEMINI_MODEL.to_string())
        );
        assert_eq!(parse_gemini_model("  "), None);
        assert_eq!(parse_gemini_model("models/gemini-2.5-flash"), None);
        let key = parse_gemini_api_key("AIzaSyExample").expect("valid Gemini API key");
        let model = parse_gemini_model(DEFAULT_GEMINI_MODEL).expect("valid Gemini model");
        assert_eq!(
            gemini_model_url(&model, &key),
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash?key=AIzaSyExample"
        );
    }
}
