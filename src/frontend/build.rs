use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let env_paths = [
        manifest_dir.join(".env"),
        manifest_dir.join("..").join("..").join(".env"),
    ];
    for path in &env_paths {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    println!("cargo:rerun-if-env-changed=GEMINI_API_KEY");

    if env::var("GEMINI_API_KEY").is_ok() {
        return;
    }
    for path in &env_paths {
        if let Some(value) = gemini_api_key_from_dotenv_path(path) {
            println!("cargo:rustc-env=GEMINI_API_KEY={value}");
            return;
        }
    }
}

/// Return `GEMINI_API_KEY` from a `.env` file when the file exists and the key is set.
fn gemini_api_key_from_dotenv_path(path: &Path) -> Option<String> {
    gemini_api_key_from_dotenv(&fs::read_to_string(path).ok()?)
}

/// Return a non-empty `GEMINI_API_KEY` assignment from dotenv contents.
fn gemini_api_key_from_dotenv(contents: &str) -> Option<String> {
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() != "GEMINI_API_KEY" {
            continue;
        }
        let value = unquote_env_value(value.trim());
        if value.is_empty() {
            return None;
        }
        return Some(value);
    }
    None
}

/// Strip one matching pair of single or double quotes from a dotenv value.
fn unquote_env_value(value: &str) -> String {
    let bytes = value.as_bytes();
    if bytes.len() >= 2 {
        let first = bytes[0];
        let last = bytes[bytes.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return value[1..value.len() - 1].to_string();
        }
    }
    value.to_string()
}
