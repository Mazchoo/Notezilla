//! Identifies which sidebar panel is currently visible.

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ActivePanel {
    Files,
    Templates,
    Search,
    Prompt,
    Sync,
    Settings,
}

impl ActivePanel {
    /// Whether the panel needs the MCP backend to be usable.
    pub fn needs_backend(self) -> bool {
        self != ActivePanel::Settings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Assert only the settings panel works without the backend.
    fn needs_backend_is_false_only_for_settings() {
        assert!(ActivePanel::Files.needs_backend());
        assert!(ActivePanel::Templates.needs_backend());
        assert!(ActivePanel::Search.needs_backend());
        assert!(ActivePanel::Prompt.needs_backend());
        assert!(ActivePanel::Sync.needs_backend());
        assert!(!ActivePanel::Settings.needs_backend());
    }
}
