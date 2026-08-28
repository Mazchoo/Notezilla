//! Identifies which sidebar panel is currently visible.

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ActivePanel {
    Files,
    Templates,
    Search,
    Prompt,
    Settings,
}
