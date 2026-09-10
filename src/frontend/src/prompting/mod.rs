pub mod create_prompt;
pub mod gemini_adapter;
pub mod ollama_adapter;

pub use create_prompt::build_prompt;
pub use gemini_adapter::{probe_gemini, send_gemini_prompt};
pub use ollama_adapter::{probe_ollama, send_prompt, GenerateOptions};
