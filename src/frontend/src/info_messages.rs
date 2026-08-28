//! User-visible copy for the frontend. Import from here; do not inline UI text.

// --- Activity bar ---

pub const ACTIVITY_FILES_TITLE: &str = "Files";
pub const ACTIVITY_TEMPLATES_TITLE: &str = "Templates";
pub const ACTIVITY_SEARCH_TITLE: &str = "Search";
pub const ACTIVITY_PROMPT_TITLE: &str = "Send prompt";
pub const ACTIVITY_SETTINGS_TITLE: &str = "Settings";

// --- Top bar ---

pub const IMPORT_MARKDOWN_TITLE: &str = "Import Markdown";
pub const SAVE_TITLE: &str = "Save";
pub const EXPORT_HTML_TITLE: &str = "Export as HTML";
pub const EXPORT_PDF_TITLE: &str = "Export as PDF";
pub const EXPORT_MARKDOWN_TITLE: &str = "Export as Markdown";
pub const EDIT_MAIN_TEXT_ON_TITLE: &str = "Edit main text (on)";
pub const EDIT_MAIN_TEXT_FROZEN_TITLE: &str =
    "Main text frozen — select and copy without opening the editor";
pub const NEW_FILE_TITLE: &str = "New File";
pub const NEW_FILE_BUTTON: &str = "＋";

// --- File tree ---

pub const NOTES_HEADING: &str = "NOTES";
pub const TEMPLATES_HEADING: &str = "TEMPLATE MARKDOWN RESPONSES";
pub const NOTE_FOLDER_ROOT_LABEL: &str = "note folder root";
pub const TEMPLATE_FOLDER_ROOT_LABEL: &str = "template folder root";
pub const NEW_FOLDER_TITLE: &str = "New folder";
pub const CREATE_FOLDER_SEPARATORS_TOAST: &str =
    "Create folder failed: name cannot contain path separators";
pub const RENAME_SEPARATORS_TOAST: &str = "Rename failed: name cannot contain path separators";

// --- Context menus ---

pub const OPEN_MENU: &str = "Open";
pub const RENAME_MENU: &str = "Rename";
pub const DELETE_MENU: &str = "Delete";
pub const NEW_FOLDER_MENU: &str = "New Folder";

// --- Rename and new-folder modals ---

pub const RENAME_FILE_TITLE: &str = "Rename file";
pub const RENAME_FOLDER_TITLE: &str = "Rename folder";
pub const CANCEL_BUTTON: &str = "Cancel";
pub const RENAME_BUTTON: &str = "Rename";
pub const CREATE_BUTTON: &str = "Create";
pub const FOLDER_NAME_PLACEHOLDER: &str = "Folder name";

// --- Search ---

pub const SEARCH_HEADING: &str = "SEARCH";
pub const SEARCH_PLACEHOLDER: &str = "Search notes...";
pub const SEARCH_GO_BUTTON: &str = "Go";
pub const PATH_FILTER_LABEL: &str = "Path filter";
pub const PATH_FILTER_PLACEHOLDER: &str = "2026/02";
pub const FRONT_MATTER_FILTER_LABEL: &str = "Front matter filter";
pub const FRONT_MATTER_FILTER_PLACEHOLDER: &str = "tags: [hello]\ndate: 2025-01-01";
pub const PREV_BUTTON: &str = "Prev";
pub const NEXT_BUTTON: &str = "Next";

// --- Prompt ---

pub const PROMPT_HEADING: &str = "Send prompt";
pub const PROMPT_PLACEHOLDER: &str = "Write a prompt...";
pub const OUTPUT_FILE_LABEL: &str = "output file";
pub const SEND_BUTTON: &str = "Send";
pub const COPY_BUTTON: &str = "Copy";
pub const COPY_PROMPT_TITLE: &str = "Copy prompt";
pub const ENTER_PROMPT_TOAST: &str = "Enter a prompt";
pub const PROMPT_COPIED_TOAST: &str = "Copied prompt";
pub const CLIPBOARD_COPY_FAILED_TOAST: &str = "Clipboard copy failed";

// --- Settings ---

pub const SETTINGS_HEADING: &str = "SETTINGS";
pub const RESULTS_PER_PAGE_LABEL: &str = "Results per page";
pub const OLLAMA_PORT_LABEL: &str = "Ollama port";
pub const SAVE_HOTKEY_LABEL: &str = "Save hotkey";
pub const NEW_FILE_HOTKEY_LABEL: &str = "New file hotkey";
pub const IMPORT_HOTKEY_LABEL: &str = "Import hotkey";
pub const EXPORT_HOTKEY_LABEL: &str = "Export hotkey";
pub const TOGGLE_MARKDOWN_EDITING_HOTKEY_LABEL: &str = "Toggle markdown editing";

// --- Editor ---

pub const EXPAND_SECTION_TITLE: &str = "Expand section";
pub const COLLAPSE_SECTION_TITLE: &str = "Collapse section";
pub const ADD_FRONTMATTER_TITLE: &str = "Add frontmatter";
pub const DELETE_BLOCK_TITLE: &str = "Delete block";
pub const DELETE_FRONT_MATTER_TITLE: &str = "Delete front matter";

// --- Import ---

pub const FILE_READ_ERROR_TOAST: &str = "File cannot be read";

// --- Export ---

pub const EXPORT_METADATA_HEADING: &str = "Metadata";

// --- Hotkeys ---

pub const CTRL_HOTKEY_PREFIX: &str = "Ctrl+";

// --- Page ---

pub const PAGE_TITLE: &str = "Notezilla";

// --- Formatted messages ---

const FILE_SINGULAR: &str = "file";
const FILE_PLURAL: &str = "files";

/// Return `label` with a parenthetical hotkey for titles and settings.
pub fn with_hotkey(label: &str, hotkey: &str) -> String {
    format!("{label} ({hotkey})")
}

/// Return a count with the matching singular or plural file noun.
pub fn file_count_label(count: usize) -> String {
    if count == 1 {
        format!("1 {FILE_SINGULAR}")
    } else {
        format!("{count} {FILE_PLURAL}")
    }
}

/// Return the toast text after a multi-file save.
pub fn format_save_summary(created: usize, updated: usize) -> String {
    match (created, updated) {
        (0, 0) => String::new(),
        (c, 0) => format!("Created {}", file_count_label(c)),
        (0, u) => format!("Updated {}", file_count_label(u)),
        (c, u) => format!(
            "Created {} and updated {}",
            file_count_label(c),
            file_count_label(u),
        ),
    }
}

/// Return the toast text when a save fails for `path`.
pub fn save_failed_toast(path: &str, error: impl std::fmt::Display) -> String {
    format!("Save failed for {path}: {error}")
}

/// Return the toast text after moving an item.
pub fn moved_toast(name: &str, dest: &str) -> String {
    format!("Moved {name} to {dest}")
}

/// Return the toast text when a move fails.
pub fn move_failed_toast(src: &str, error: impl std::fmt::Display) -> String {
    format!("Move failed for {src}: {error}")
}

/// Return the toast text after creating a folder.
pub fn created_folder_toast(path: &str) -> String {
    format!("Created folder {path}")
}

/// Return the toast text when folder creation fails.
pub fn create_folder_failed_toast(path: &str, error: impl std::fmt::Display) -> String {
    format!("Create folder failed for {path}: {error}")
}

/// Return the toast text after a rename.
pub fn renamed_toast(resolved: &str) -> String {
    format!("Renamed to {resolved}")
}

/// Return the toast text when a rename fails.
pub fn rename_failed_toast(path: &str, error: impl std::fmt::Display) -> String {
    format!("Rename failed for {path}: {error}")
}

/// Return the toast text after deleting a folder.
pub fn deleted_folder_toast(path: &str) -> String {
    format!("Deleted folder {path}")
}

/// Return the toast text when folder deletion fails.
pub fn delete_folder_failed_toast(path: &str, error: impl std::fmt::Display) -> String {
    format!("Delete folder failed for {path}: {error}")
}

/// Return the toast text after deleting a file.
pub fn deleted_toast(path: &str) -> String {
    format!("Deleted {path}")
}

/// Return the toast text when file deletion fails.
pub fn delete_failed_toast(path: &str, error: impl std::fmt::Display) -> String {
    format!("Delete failed for {path}: {error}")
}

/// Return the overlay label for the file currently being generated.
pub fn export_progress_label(filename: &str, index: usize, total: usize) -> String {
    if total <= 1 {
        format!("Generating {filename}")
    } else {
        format!("Generating {filename} ({} of {total})", index + 1)
    }
}

/// Return the toast text when an HTML or markdown export download fails.
pub fn export_failed_toast(filename: &str, err: &str) -> String {
    format!("Export failed for {filename}: {err}")
}

/// Return the toast text when a PDF download fails.
pub fn pdf_export_failed_toast(filename: &str, err: &str) -> String {
    format!("PDF export failed for {filename}: {err}")
}

/// Return the toast text when HTML-to-PDF conversion fails.
pub fn pdf_conversion_failed_toast(filename: &str, error: impl std::fmt::Display) -> String {
    format!("PDF conversion failed for {filename}: {error}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Assert a count of one uses the singular noun and any other count uses the plural.
    fn file_count_label_picks_singular_or_plural() {
        assert_eq!(file_count_label(1), "1 file");
        assert_eq!(file_count_label(0), "0 files");
        assert_eq!(file_count_label(2), "2 files");
    }

    #[test]
    /// Assert the save toast names created and updated files, or is empty when none.
    fn format_save_summary_describes_created_and_updated() {
        assert_eq!(format_save_summary(0, 0), "");
        assert_eq!(format_save_summary(1, 0), "Created 1 file");
        assert_eq!(format_save_summary(0, 2), "Updated 2 files");
        assert_eq!(
            format_save_summary(1, 2),
            "Created 1 file and updated 2 files"
        );
    }

    #[test]
    /// Assert the spinner names the file being generated and the batch position.
    fn export_progress_label_names_the_file_and_index() {
        assert_eq!(
            export_progress_label("hello.pdf", 0, 1),
            "Generating hello.pdf"
        );
        assert_eq!(
            export_progress_label("hello.pdf", 0, 3),
            "Generating hello.pdf (1 of 3)"
        );
        assert_eq!(
            export_progress_label("note.html", 2, 3),
            "Generating note.html (3 of 3)"
        );
    }
}
