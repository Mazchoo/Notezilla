use crate::constants::{OLLAMA_GENERATE_PATH, OLLAMA_TAGS_PATH, OLLAMA_URL};
use crate::default_settings::{
    DEFAULT_OLLAMA_NUM_CTX, DEFAULT_OLLAMA_NUM_PREDICT, DEFAULT_OLLAMA_TEMPERATURE,
    DEFAULT_OLLAMA_THINK, DEFAULT_OLLAMA_TOP_K, DEFAULT_OLLAMA_TOP_P,
};
use gloo_net::http::Request;
use serde::Deserialize;
use serde_json::{json, Value};

/// Generation controls sent with POST `/api/generate`.
#[derive(Clone, Debug, PartialEq)]
pub struct GenerateOptions {
    pub temperature: f64,
    pub num_predict: i32,
    pub num_ctx: u32,
    pub top_p: f64,
    pub top_k: u32,
    pub think: bool,
}

impl Default for GenerateOptions {
    fn default() -> Self {
        Self {
            temperature: DEFAULT_OLLAMA_TEMPERATURE,
            num_predict: DEFAULT_OLLAMA_NUM_PREDICT,
            num_ctx: DEFAULT_OLLAMA_NUM_CTX,
            top_p: DEFAULT_OLLAMA_TOP_P,
            top_k: DEFAULT_OLLAMA_TOP_K,
            think: DEFAULT_OLLAMA_THINK,
        }
    }
}

#[derive(Deserialize)]
struct TagsResponse {
    models: Vec<TagsModel>,
}

#[derive(Deserialize)]
struct TagsModel {
    name: String,
}

#[derive(Deserialize)]
struct GenerateResponse {
    response: Option<String>,
    error: Option<String>,
}

/// Return the same-origin Ollama origin. Trunk proxies this to 127.0.0.1.
pub fn ollama_base_url(_port: u16) -> String {
    OLLAMA_URL.to_string()
}

/// Return the same-origin Ollama URL for `path`.
fn ollama_url(port: u16, path: &str) -> String {
    format!("{}{path}", ollama_base_url(port))
}

/// Return the JSON body for POST `/api/generate` with streaming disabled.
fn generate_request_body(model: &str, prompt: &str, options: &GenerateOptions) -> Value {
    json!({
        "model": model,
        "prompt": prompt,
        "stream": false,
        "think": options.think,
        "options": {
            "temperature": options.temperature,
            "num_predict": options.num_predict,
            "num_ctx": options.num_ctx,
            "top_p": options.top_p,
            "top_k": options.top_k,
        }
    })
}

/// Parse `/api/tags` and return installed model names.
fn parse_model_names(body: &str) -> Result<Vec<String>, String> {
    let parsed: TagsResponse =
        serde_json::from_str(body).map_err(|e| format!("Parse error: {e}"))?;
    Ok(parsed.models.into_iter().map(|m| m.name).collect())
}

/// Parse a non-streaming `/api/generate` body and return the completion text.
fn parse_generate_response(body: &str) -> Result<String, String> {
    let parsed: GenerateResponse =
        serde_json::from_str(body).map_err(|e| format!("Parse error: {e}"))?;
    if let Some(error) = parsed.error.filter(|e| !e.is_empty()) {
        return Err(error);
    }
    parsed
        .response
        .ok_or_else(|| "Missing response in Ollama body".to_string())
}

/// GET `url` and return the response body when the status is 2xx.
async fn ollama_get(url: &str) -> Result<String, String> {
    let response = Request::get(url)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| format!("Body read error: {e}"))?;
    if !(200..300).contains(&status) {
        return Err(http_error(status, &body));
    }
    Ok(body)
}

/// POST JSON to `url` and return the response body when the status is 2xx.
async fn ollama_post_json(url: &str, body: &Value) -> Result<String, String> {
    let response = Request::post(url)
        .header("Content-Type", "application/json")
        .json(body)
        .map_err(|e| format!("Serialize error: {e}"))?
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| format!("Body read error: {e}"))?;
    if !(200..300).contains(&status) {
        return Err(http_error(status, &text));
    }
    Ok(text)
}

/// Return an HTTP error string, preferring an Ollama `error` field when present.
fn http_error(status: u16, body: &str) -> String {
    if let Ok(parsed) = serde_json::from_str::<GenerateResponse>(body) {
        if let Some(error) = parsed.error.filter(|e| !e.is_empty()) {
            return error;
        }
    }
    format!("HTTP {status}")
}

/// GET `/api/tags` on `port` and return the body when it is a tags payload.
async fn fetch_tags_body(port: u16) -> Result<String, String> {
    let body = ollama_get(&ollama_url(port, OLLAMA_TAGS_PATH)).await?;
    parse_model_names(&body)?;
    Ok(body)
}

/// Probe whether the Ollama HTTP API on `port` responds with `/api/tags`.
pub async fn check_connection(port: u16) -> Result<(), String> {
    fetch_tags_body(port).await.map(|_| ())
}

/// Confirm the API on `port`, then send `prompt` to `model` via POST `/api/generate`.
pub async fn send_prompt(
    port: u16,
    model: &str,
    prompt: &str,
    options: &GenerateOptions,
) -> Result<String, String> {
    fetch_tags_body(port).await?;
    let body = ollama_post_json(
        &ollama_url(port, OLLAMA_GENERATE_PATH),
        &generate_request_body(model, prompt, options),
    )
    .await?;
    parse_generate_response(&body)
}

#[cfg(test)]
mod tests {
    use super::{
        generate_request_body, ollama_base_url, ollama_url, parse_generate_response,
        parse_model_names, GenerateOptions,
    };
    use crate::constants::{OLLAMA_GENERATE_PATH, OLLAMA_TAGS_PATH, OLLAMA_URL};
    use crate::default_settings::DEFAULT_OLLAMA_PORT;
    use serde_json::json;

    #[test]
    /// Assert the origin is the same-origin proxy path, not a cross-origin loopback URL.
    fn ollama_base_url_uses_same_origin_proxy() {
        assert_eq!(ollama_base_url(DEFAULT_OLLAMA_PORT), OLLAMA_URL);
        assert_eq!(ollama_base_url(1), OLLAMA_URL);
        assert!(OLLAMA_URL.starts_with('/'));
        assert!(!OLLAMA_URL.contains("://"));
    }

    #[test]
    /// Assert API paths are appended to the origin.
    fn ollama_url_appends_api_paths() {
        assert_eq!(
            ollama_url(DEFAULT_OLLAMA_PORT, OLLAMA_TAGS_PATH),
            format!("{OLLAMA_URL}{OLLAMA_TAGS_PATH}")
        );
        assert_eq!(
            ollama_url(DEFAULT_OLLAMA_PORT, OLLAMA_GENERATE_PATH),
            format!("{OLLAMA_URL}{OLLAMA_GENERATE_PATH}")
        );
    }

    #[test]
    /// Assert generate JSON names the model, prompt, think flag, options, and disables streaming.
    fn generate_request_body_sets_model_prompt_options_and_disables_stream() {
        let options = GenerateOptions {
            temperature: 0.2,
            num_predict: 128,
            num_ctx: 4096,
            top_p: 0.5,
            top_k: 10,
            think: true,
        };
        let body = generate_request_body("the-model", "Ask this", &options);
        assert_eq!(
            body,
            json!({
                "model": "the-model",
                "prompt": "Ask this",
                "stream": false,
                "think": true,
                "options": {
                    "temperature": 0.2,
                    "num_predict": 128,
                    "num_ctx": 4096,
                    "top_p": 0.5,
                    "top_k": 10,
                }
            })
        );
    }

    #[test]
    /// Assert `/api/tags` model names are collected in order.
    fn parse_model_names_reads_name_fields() {
        let body = r#"{"models":[{"name":"a"},{"name":"b"}]}"#;
        assert_eq!(
            parse_model_names(body).expect("tags"),
            vec!["a".to_string(), "b".to_string()]
        );
        assert!(parse_model_names(r#"{"models":[]}"#)
            .expect("empty")
            .is_empty());
    }

    #[test]
    /// Assert a tags body without `models` is an error.
    fn parse_model_names_rejects_missing_models() {
        let err = parse_model_names("{}").unwrap_err();
        assert!(err.contains("Parse error"), "{err}");
        let err = parse_model_names("not-json").unwrap_err();
        assert!(err.contains("Parse error"), "{err}");
    }

    #[test]
    /// Assert a generate body yields the `response` text.
    fn parse_generate_response_reads_response_field() {
        let body = r#"{"response":"hello","done":true}"#;
        assert_eq!(parse_generate_response(body).expect("text"), "hello");
    }

    #[test]
    /// Assert an Ollama `error` field is returned and a missing `response` is an error.
    fn parse_generate_response_prefers_error_and_requires_response() {
        let err = parse_generate_response(r#"{"error":"model not found"}"#).unwrap_err();
        assert_eq!(err, "model not found");
        let err = parse_generate_response(r#"{"done":true}"#).unwrap_err();
        assert!(err.contains("Missing response"), "{err}");
        let err = parse_generate_response("not-json").unwrap_err();
        assert!(err.contains("Parse error"), "{err}");
    }
}
