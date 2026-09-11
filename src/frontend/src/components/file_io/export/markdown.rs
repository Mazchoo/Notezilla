use super::super::path::path_to_markdown_filename;
use super::{download_text_file, log_export_error, run_export};
use crate::info_messages::export_failed_toast;
use crate::models::block::EditorEntry;
use leptos::prelude::*;

/// Prompt the browser to save each editor entry as a markdown file.
pub fn export_entries_as_markdown(
    entries: Vec<EditorEntry>,
    progress: RwSignal<Option<String>>,
    error_toast: RwSignal<Option<String>>,
) {
    run_export(
        entries,
        progress,
        error_toast,
        path_to_markdown_filename,
        export_one,
    );
}

/// Convert one editor entry to a markdown download. Return toast text on failure.
fn export_one(entry: EditorEntry, filename: &str) -> Option<String> {
    let content = entry_to_markdown(entry);
    download_text_file(filename, &content, "text/markdown;charset=utf-8")
        .err()
        .map(|err| log_export_error(export_failed_toast(filename, &format!("{err:?}"))))
}

/// Serialize an editor entry to markdown, including front matter when present.
fn entry_to_markdown(entry: EditorEntry) -> String {
    let body = entry.content.text.get_untracked();
    match entry.front_matter.get_untracked() {
        Some(fm) => {
            let raw = fm.raw.get_untracked();
            if raw.is_empty() {
                body
            } else {
                format!("---\n{raw}\n---\n{body}")
            }
        }
        None => body,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Assert markdown export wraps non-empty front matter in `---` delimiters.
    fn entry_to_markdown_includes_front_matter() {
        use crate::models::block::{EditorEntry, FrontMatterBlock};
        use leptos::prelude::{Owner, Set};

        let owner = Owner::new();
        owner.with(|| {
            let entry = EditorEntry::new("./a.md", "body");
            assert_eq!(entry_to_markdown(entry), "body");

            entry
                .front_matter
                .set(Some(FrontMatterBlock::new("title: x")));
            assert_eq!(entry_to_markdown(entry), "---\ntitle: x\n---\nbody");

            entry.front_matter.set(Some(FrontMatterBlock::new("")));
            assert_eq!(entry_to_markdown(entry), "body");
        });
    }
}
