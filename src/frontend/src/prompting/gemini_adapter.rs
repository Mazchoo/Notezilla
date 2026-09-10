use crate::constants::{
    gemini_model_url, GEMINI_API_ORIGIN, GEMINI_GENERATE_CONTENT_ACTION, GEMINI_MODELS_PATH,
};
use gloo_net::http::Request;
use leptos::prelude::{Effect, Get, RwSignal, Set};
use leptos::task::spawn_local;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
struct GeminiErrorBody {
    error: Option<GeminiError>,
}

#[derive(Deserialize)]
struct GeminiError {
    message: Option<String>,
}

#[derive(Deserialize)]
struct GenerateResponse {
    error: Option<GeminiError>,
    candidates: Option<Vec<Candidate>>,
}

#[derive(Deserialize)]
struct Candidate {
    content: Option<Content>,
}

#[derive(Deserialize)]
struct Content {
    parts: Option<Vec<Part>>,
}

#[derive(Deserialize)]
struct Part {
    text: Option<String>,
}

#[derive(Deserialize)]
struct ModelMetadata {
    name: Option<String>,
}

/// Return the Gemini `generateContent` URL for `model` and `api_key`.
fn gemini_generate_url(model: &str, api_key: &str) -> String {
    format!(
        "{GEMINI_API_ORIGIN}{GEMINI_MODELS_PATH}/{model}:{GEMINI_GENERATE_CONTENT_ACTION}?key={api_key}"
    )
}

/// Return the JSON body for POST `{model}:generateContent`.
fn generate_request_body(prompt: &str) -> Value {
    json!({
        "contents": [{
            "parts": [{ "text": prompt }]
        }]
    })
}

/// Parse GET `/v1beta/models/{model}` and require a `name` field.
fn parse_model_metadata(body: &str) -> Result<(), String> {
    let parsed: ModelMetadata =
        serde_json::from_str(body).map_err(|e| format!("Parse error: {e}"))?;
    parsed
        .name
        .filter(|name| !name.is_empty())
        .map(|_| ())
        .ok_or_else(|| "Missing name in Gemini model body".to_string())
}

/// Parse a `generateContent` body and return concatenated candidate text.
fn parse_generate_response(body: &str) -> Result<String, String> {
    let parsed: GenerateResponse =
        serde_json::from_str(body).map_err(|e| format!("Parse error: {e}"))?;
    if let Some(message) = parsed
        .error
        .and_then(|e| e.message)
        .filter(|m| !m.is_empty())
    {
        return Err(message);
    }
    let text = parsed
        .candidates
        .into_iter()
        .flatten()
        .filter_map(|c| c.content)
        .filter_map(|c| c.parts)
        .flatten()
        .filter_map(|p| p.text)
        .collect::<Vec<_>>()
        .join("");
    if text.is_empty() {
        return Err("Missing text in Gemini body".to_string());
    }
    Ok(text)
}

/// GET `url` and return the response body when the status is 2xx.
async fn gemini_get(url: &str) -> Result<String, String> {
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
async fn gemini_post_json(url: &str, body: &Value) -> Result<String, String> {
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

/// Return an HTTP error string, preferring a Gemini `error.message` when present.
fn http_error(status: u16, body: &str) -> String {
    if let Ok(parsed) = serde_json::from_str::<GeminiErrorBody>(body) {
        if let Some(message) = parsed
            .error
            .and_then(|e| e.message)
            .filter(|m| !m.is_empty())
        {
            return message;
        }
    }
    format!("HTTP {status}")
}

/// GET model metadata for `model` and return when the body names a model.
async fn fetch_model_body(model: &str, api_key: &str) -> Result<String, String> {
    let body = gemini_get(&gemini_model_url(model, api_key)).await?;
    parse_model_metadata(&body)?;
    Ok(body)
}

/// Return the console message for a successful Gemini connection.
fn gemini_ready_log() -> String {
    format!("Gemini connection ready: {GEMINI_API_ORIGIN}")
}

/// Probe whether GET `/v1beta/models/{model}` accepts `api_key`.
async fn check_connection(model: &str, api_key: &str) -> Result<(), String> {
    fetch_model_body(model, api_key).await.map(|_| ())
}

/// Probe Gemini when the API key or model changes and store whether GET succeeded.
pub fn probe_gemini(
    api_key: RwSignal<String>,
    model: RwSignal<String>,
    available: RwSignal<bool>,
) {
    Effect::new(move |_| {
        let api_key = api_key.get();
        let model = model.get();
        spawn_local(async move {
            if api_key.trim().is_empty() || model.trim().is_empty() {
                available.set(false);
                return;
            }
            match check_connection(&model, &api_key).await {
                Ok(()) => {
                    web_sys::console::log_1(&gemini_ready_log().into());
                    available.set(true);
                }
                Err(e) => {
                    web_sys::console::warn_1(
                        &format!("Gemini init failed: {e}. Prompt send will be unavailable.")
                            .into(),
                    );
                    available.set(false);
                }
            }
        });
    });
}

/// Confirm the model, then send `prompt` via POST `{model}:generateContent`.
pub async fn send_gemini_prompt(
    api_key: &str,
    model: &str,
    prompt: &str,
) -> Result<String, String> {
    fetch_model_body(model, api_key).await?;
    let body = gemini_post_json(
        &gemini_generate_url(model, api_key),
        &generate_request_body(prompt),
    )
    .await?;
    parse_generate_response(&body)
}

#[cfg(test)]
mod tests {
    use super::{
        generate_request_body, gemini_generate_url, gemini_ready_log, parse_generate_response,
        parse_model_metadata,
    };
    use crate::constants::{
        gemini_model_url, GEMINI_API_ORIGIN, GEMINI_GENERATE_CONTENT_ACTION, GEMINI_MODELS_PATH,
    };
    use crate::default_settings::DEFAULT_GEMINI_MODEL;
    use serde_json::json;

    #[test]
    /// Assert a successful Gemini probe logs the Generative Language API origin.
    fn gemini_ready_log_names_the_api_origin() {
        assert_eq!(
            gemini_ready_log(),
            format!("Gemini connection ready: {GEMINI_API_ORIGIN}")
        );
    }

    #[test]
    /// Assert generate JSON wraps the prompt in `contents.parts`.
    fn generate_request_body_sets_contents_text() {
        assert_eq!(
            generate_request_body("Ask this"),
            json!({
                "contents": [{
                    "parts": [{ "text": "Ask this" }]
                }]
            })
        );
    }

    #[test]
    /// Assert generate and metadata URLs name the model, action, and key.
    fn gemini_urls_include_model_action_and_key() {
        let key = "AIzaSyExample";
        assert_eq!(
            gemini_model_url(DEFAULT_GEMINI_MODEL, key),
            format!("{GEMINI_API_ORIGIN}{GEMINI_MODELS_PATH}/{DEFAULT_GEMINI_MODEL}?key={key}")
        );
        assert_eq!(
            gemini_generate_url(DEFAULT_GEMINI_MODEL, key),
            format!(
                "{GEMINI_API_ORIGIN}{GEMINI_MODELS_PATH}/{DEFAULT_GEMINI_MODEL}:{GEMINI_GENERATE_CONTENT_ACTION}?key={key}"
            )
        );
    }

    #[test]
    /// Assert model metadata requires a non-empty `name`.
    fn parse_model_metadata_requires_name() {
        parse_model_metadata(r#"{"name":"models/gemini-2.5-flash"}"#).expect("name");
        let err = parse_model_metadata("{}").unwrap_err();
        assert!(err.contains("Missing name"), "{err}");
        let err = parse_model_metadata("not-json").unwrap_err();
        assert!(err.contains("Parse error"), "{err}");
    }

    #[test]
    /// Assert a generate body yields concatenated candidate `text` parts.
    fn parse_generate_response_reads_candidate_text() {
        let body = r#"{"candidates":[{"content":{"parts":[{"text":"hello"},{"text":" world"}]}}]}"#;
        assert_eq!(parse_generate_response(body).expect("text"), "hello world");
    }

    #[test]
    /// Assert a Gemini `error.message` is returned and missing text is an error.
    fn parse_generate_response_prefers_error_and_requires_text() {
        let err = parse_generate_response(r#"{"error":{"message":"API key not valid"}}"#)
            .unwrap_err();
        assert_eq!(err, "API key not valid");
        let err = parse_generate_response(r#"{"candidates":[]}"#).unwrap_err();
        assert!(err.contains("Missing text"), "{err}");
        let err = parse_generate_response("not-json").unwrap_err();
        assert!(err.contains("Parse error"), "{err}");
    }
}
