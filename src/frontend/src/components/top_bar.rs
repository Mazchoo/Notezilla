use crate::components::file_io::{
    entry_save_params, export_entries_as_html, export_entries_as_markdown, export_entries_as_pdf,
    load_markdown_file,
};
use crate::components::hotkeys::format_ctrl_hotkey;
use crate::components::sidebar::file_tree_backend::FileTreeBackend;
use crate::components::toast::{show_error_toast, show_toast};
use crate::info_messages::{
    format_save_summary, save_failed_toast, with_hotkey, EDIT_MAIN_TEXT_FROZEN_TITLE,
    EDIT_MAIN_TEXT_ON_TITLE, EXPORT_HTML_TITLE, EXPORT_MARKDOWN_TITLE, EXPORT_PDF_TITLE,
    IMPORT_MARKDOWN_TITLE, NEW_FILE_BUTTON, NEW_FILE_TITLE, SAVE_TITLE,
};
use crate::mcp::tools::upsert_note;
use crate::models::block::EditorEntry;
use crate::state::AppState;
use icondata as id;
use leptos::either::Either;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;
use web_sys::Event;

/// Upsert every open editor entry via the MCP backend.
pub fn save_all_entries(state: &AppState) {
    let sid = match state.session_id.get_untracked() {
        Some(s) => s,
        None => {
            web_sys::console::warn_1(&"MCP session not ready".into());
            return;
        }
    };

    let toast = state.toast;
    let error_toast = state.error_toast;
    let file_tree_epoch = state.file_tree_epoch;
    let items: Vec<_> = state
        .entries
        .get_untracked()
        .iter()
        .map(|entry| entry_save_params(*entry))
        .collect();

    spawn_local(async move {
        let mut created = 0usize;
        let mut updated = 0usize;
        let mut errors = Vec::new();

        for (path, contents, fields) in items {
            match upsert_note(&sid, &path, &contents, fields).await {
                Ok(result) => {
                    web_sys::console::log_1(&format!("Saved {path}").into());
                    if result.new_file_created {
                        created += 1;
                    } else {
                        updated += 1;
                    }
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Save failed for {path}: {e}").into());
                    errors.push(save_failed_toast(&path, e));
                }
            }
        }

        if created > 0 || updated > 0 {
            file_tree_epoch.update(|n| *n = n.wrapping_add(1));
            show_toast(toast, format_save_summary(created, updated));
        }
        if !errors.is_empty() {
            show_error_toast(error_toast, errors.join("\n"));
        }
    });
}

/// Append a new empty entry and focus its content when editing is enabled.
pub fn append_new_file(state: &AppState) {
    let editing_enabled = state.markdown_editing_enabled.get_untracked();
    state.entries.update(|list: &mut Vec<EditorEntry>| {
        let entry = EditorEntry::empty("./new_file.md");
        if editing_enabled {
            entry.content.focused.set(true);
        }
        FileTreeBackend::Notes.insert_in_editor(list, entry);
    });
}

/// Open the OS file picker for markdown import.
pub fn open_import_picker(state: &AppState) {
    if let Some(input) = state.import_file_input.get_untracked() {
        input.click();
    }
}

/// Export every open editor entry as a standalone HTML file.
pub fn export_all_as_html(state: &AppState) {
    export_entries_as_html(
        state.entries.get_untracked(),
        state.export_progress,
        state.error_toast,
    );
}

/// Convert each open entry's markdown to HTML, then to PDF, and download the files.
pub fn export_all_as_pdf(state: &AppState) {
    export_entries_as_pdf(
        state.entries.get_untracked(),
        state.export_progress,
        state.error_toast,
    );
}

/// Export every open editor entry as a markdown file.
pub fn export_all_as_markdown(state: &AppState) {
    export_entries_as_markdown(
        state.entries.get_untracked(),
        state.export_progress,
        state.error_toast,
    );
}

/// Toggle whether main markdown blocks can enter edit mode.
pub fn toggle_markdown_editing(state: &AppState) {
    state
        .markdown_editing_enabled
        .update(|enabled| *enabled = !*enabled);
}

/// Render the top-bar import, save, export, edit-toggle, and new-file actions.
#[component]
pub fn TopBar() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");
    let entries = state.entries;
    let toast = state.toast;
    let save_hotkey_key = state.save_hotkey_key;
    let new_file_hotkey_key = state.new_file_hotkey_key;
    let import_hotkey_key = state.import_hotkey_key;
    let export_hotkey_key = state.export_hotkey_key;
    let toggle_markdown_editing_hotkey_key = state.toggle_markdown_editing_hotkey_key;
    let file_input_ref = state.import_file_input;

    let state_import = state.clone();
    let on_import_click = move |_| {
        open_import_picker(&state_import);
    };

    // Delegate file reading + entry creation to file_io.
    let on_file_change = move |ev: Event| {
        load_markdown_file(ev, entries, toast);
    };

    let state_save = state.clone();
    let on_save = move |_| {
        save_all_entries(&state_save);
    };

    let state_export = state.clone();
    let on_export_html = move |_| {
        export_all_as_html(&state_export);
    };

    let state_export_pdf = state.clone();
    let on_export_pdf = move |_| {
        export_all_as_pdf(&state_export_pdf);
    };

    let state_export_markdown = state.clone();
    let on_export_markdown = move |_| {
        export_all_as_markdown(&state_export_markdown);
    };

    let state_new = state.clone();
    let on_new_block = move |_| {
        append_new_file(&state_new);
    };

    let state_toggle = state.clone();
    let on_toggle_markdown_editing = move |_| {
        toggle_markdown_editing(&state_toggle);
    };

    view! {
        <div class="top-bar">
            // Hidden file input — accepts markdown and plain-text files.
            <input
                type="file"
                accept=".md,.markdown,text/markdown,text/plain"
                style="display:none"
                node_ref=file_input_ref
                on:change=on_file_change
            />
            // Import button — opens the file picker.
            <button
                class="activity-btn"
                title=move || with_hotkey(IMPORT_MARKDOWN_TITLE, &format_ctrl_hotkey(import_hotkey_key.get()))
                on:click=on_import_click
            >
                <Icon icon=id::LuUpload/>
            </button>
            // Save — upserts each entry via the MCP backend.
            <button
                class="activity-btn"
                title=move || with_hotkey(SAVE_TITLE, &format_ctrl_hotkey(save_hotkey_key.get()))
                on:click=on_save
            >
                <Icon icon=id::LuSave/>
            </button>
            // Export — save each entry as a standalone HTML file.
            <button
                class="activity-btn"
                title=move || with_hotkey(EXPORT_HTML_TITLE, &format_ctrl_hotkey(export_hotkey_key.get()))
                on:click=on_export_html
            >
                <Icon icon=id::LuFileCode/>
            </button>
            // Export — markdown → HTML (including SVG drawings) → PDF.
            <button class="activity-btn" title=EXPORT_PDF_TITLE on:click=on_export_pdf>
                <Icon icon=id::LuFileDown/>
            </button>
            // Export — save each entry as a markdown file.
            <button class="activity-btn" title=EXPORT_MARKDOWN_TITLE on:click=on_export_markdown>
                <Icon icon=id::LuFileText/>
            </button>
            // Toggle main-text editing — off keeps rendered markdown selectable without opening the editor.
            <button
                class=move || AppState::activity_btn_class(state.markdown_editing_enabled.get())
                title=move || {
                    let hotkey = format_ctrl_hotkey(toggle_markdown_editing_hotkey_key.get());
                    if state.markdown_editing_enabled.get() {
                        with_hotkey(EDIT_MAIN_TEXT_ON_TITLE, &hotkey)
                    } else {
                        with_hotkey(EDIT_MAIN_TEXT_FROZEN_TITLE, &hotkey)
                    }
                }
                on:click=on_toggle_markdown_editing
            >
                {move || if state.markdown_editing_enabled.get() {
                    Either::Left(view! { <Icon icon=id::LuPencil/> })
                } else {
                    Either::Right(view! { <Icon icon=id::LuLock/> })
                }}
            </button>
            // New File — appends a fresh empty entry.
            <button
                class="activity-btn top-bar-new-block"
                title=move || with_hotkey(NEW_FILE_TITLE, &format_ctrl_hotkey(new_file_hotkey_key.get()))
                on:click=on_new_block
            >
                {NEW_FILE_BUTTON}
            </button>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::DEFAULT_MARKDOWN_PATH;
    use leptos::prelude::{GetUntracked, Owner, Set};

    #[test]
    /// Assert a new empty entry is appended and focused when editing is enabled.
    fn append_new_file_adds_an_empty_focused_entry() {
        let owner = Owner::new();
        owner.with(|| {
            let state = AppState::new();
            append_new_file(&state);
            let entries = state.entries.get_untracked();
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].title.path.get_untracked(), DEFAULT_MARKDOWN_PATH);
            let added = entries[1];
            assert_eq!(added.title.path.get_untracked(), "./new_file.md");
            assert_eq!(added.content.text.get_untracked(), "");
            assert!(added.content.focused.get_untracked());
        });
    }

    #[test]
    /// Assert a new entry is not focused when markdown editing is disabled.
    fn append_new_file_skips_focus_when_editing_disabled() {
        let owner = Owner::new();
        owner.with(|| {
            let state = AppState::new();
            state.markdown_editing_enabled.set(false);
            append_new_file(&state);
            let added = state.entries.get_untracked()[1];
            assert!(!added.content.focused.get_untracked());
        });
    }

    #[test]
    /// Assert the markdown-editing flag flips on each call.
    fn toggle_markdown_editing_flips_the_flag() {
        let owner = Owner::new();
        owner.with(|| {
            let state = AppState::new();
            assert!(state.markdown_editing_enabled.get_untracked());
            toggle_markdown_editing(&state);
            assert!(!state.markdown_editing_enabled.get_untracked());
            toggle_markdown_editing(&state);
            assert!(state.markdown_editing_enabled.get_untracked());
        });
    }
}
