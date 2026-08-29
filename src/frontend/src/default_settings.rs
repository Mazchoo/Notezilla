pub const TOAST_DISMISS_MS: i32 = 2000;
pub const DEFAULT_NUMBER_RESULTS_PER_PAGE: usize = 20;
pub const DEFAULT_SEARCH_PREVIEW_CHARS: usize = 50;
/// Default key for the save hotkey (used with Ctrl/Meta).
pub const DEFAULT_SAVE_HOTKEY_KEY: char = 's';
/// Default key for the new-file hotkey (used with Ctrl/Meta).
pub const DEFAULT_NEW_FILE_HOTKEY_KEY: char = 'n';
/// Default key for the import hotkey (used with Ctrl/Meta).
pub const DEFAULT_IMPORT_HOTKEY_KEY: char = 'i';
/// Default key for the export hotkey (used with Ctrl/Meta).
pub const DEFAULT_EXPORT_HOTKEY_KEY: char = 'e';
/// Default key for the markdown-editing toggle hotkey (used with Ctrl/Meta).
pub const DEFAULT_TOGGLE_MARKDOWN_EDITING_HOTKEY_KEY: char = 'm';
/// Default path for the prompt response file in the Send prompt panel.
pub const DEFAULT_PROMPT_OUTPUT_PATH: &str = "./prompt_response.md";
/// Default TCP port for the local Ollama HTTP API.
pub const DEFAULT_OLLAMA_PORT: u16 = 11434;
/// Default Ollama model name shown in Settings.
pub const DEFAULT_OLLAMA_MODEL: &str = "qwen3:4b";
/// Default Ollama sampling temperature (`options.temperature`).
pub const DEFAULT_OLLAMA_TEMPERATURE: f64 = 0.8;
/// Default Ollama max output tokens (`options.num_predict`). `-1` is unlimited.
pub const DEFAULT_OLLAMA_NUM_PREDICT: i32 = -1;
/// Default Ollama context window size in tokens (`options.num_ctx`).
pub const DEFAULT_OLLAMA_NUM_CTX: u32 = 2048;
/// Default Ollama nucleus-sampling threshold (`options.top_p`).
pub const DEFAULT_OLLAMA_TOP_P: f64 = 0.9;
/// Default Ollama top-K sampling limit (`options.top_k`).
pub const DEFAULT_OLLAMA_TOP_K: u32 = 40;
/// Default Ollama thinking flag (`think`) for models that support it.
pub const DEFAULT_OLLAMA_THINK: bool = true;
/// Default sidebar panel width in CSS pixels.
pub const DEFAULT_SIDEBAR_WIDTH: f64 = 400.0;
