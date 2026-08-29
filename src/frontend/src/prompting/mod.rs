pub mod create_prompt;
pub mod ollama_adapter;

pub use create_prompt::build_prompt;
pub use ollama_adapter::{check_connection, send_prompt, GenerateOptions};
