use crate::components::file_io::{
    entry_save_params, export_entries_as_html, export_entries_as_markdown, load_markdown_file,
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

#[component]
pub fn TopBar() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");
    let entries = state.entries;

    let file_input_ref: NodeRef<leptos::html::Input> = NodeRef::new();

    // Open the OS file-picker when the Import button is clicked.
    let on_import_click = move |_| {
        if let Some(input) = file_input_ref.get() {
            input.click();
        }
    };

    // Delegate file reading + entry creation to file_io.
    let on_file_change = move |ev: Event| {
        load_markdown_file(ev, entries);
    };

    let session = state.session_id;
    let on_save = move |_| {
        let sid = match session.get_untracked() {
            Some(s) => s,
            None => {
                web_sys::console::warn_1(&"MCP session not ready".into());
                return;
            }
        };

        let items: Vec<_> = state
            .entries
            .get_untracked()
            .iter()
            .map(|entry| entry_save_params(*entry))
            .collect();

        spawn_local(async move {
            for (path, contents, fields) in items {
                match upsert_note(&sid, &path, &contents, fields).await {
                    Ok(()) => {
                        web_sys::console::log_1(&format!("Saved {path}").into());
                    }
                    Err(e) => {
                        web_sys::console::error_1(&format!("Save failed for {path}: {e}").into());
                    }
                }
            }
        });
    };

    let on_export_html = move |_| {
        export_entries_as_html(&state.entries.get_untracked());
    };

    let on_export_markdown = move |_| {
        export_entries_as_markdown(&state.entries.get_untracked());
    };

    // Append a new empty entry (divider + title + blank markdown block) and focus it.
    let on_new_block = move |_| {
        let editing_enabled = state.markdown_editing_enabled.get_untracked();
        entries.update(|list: &mut Vec<EditorEntry>| {
            let entry = EditorEntry::empty("./new_file.md");
            if editing_enabled {
                entry.content.focused.set(true);
            }
            list.push(entry);
        });
    };

    let markdown_editing_enabled = state.markdown_editing_enabled;
    let on_toggle_markdown_editing = move |_| {
        markdown_editing_enabled.update(|enabled| *enabled = !*enabled);
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
            <button class="activity-btn" title="Import Markdown" on:click=on_import_click>
                <Icon icon=id::LuUpload/>
            </button>
            // Save — upserts each entry via the MCP backend.
            <button class="activity-btn" title="Save (Ctrl+S)" on:click=on_save>
                <Icon icon=id::LuSave/>
            </button>
            // Export — save each entry as a standalone HTML file.
            <button class="activity-btn" title="Export as HTML" on:click=on_export_html>
                <Icon icon=id::LuDownload/>
            </button>
            // Export — save each entry as a markdown file.
            <button class="activity-btn" title="Export as Markdown" on:click=on_export_markdown>
                <Icon icon=id::LuFileText/>
            </button>
            // Toggle main-text editing — off keeps rendered markdown selectable without opening the editor.
            <button
                class=move || if markdown_editing_enabled.get() {
                    "activity-btn active"
                } else {
                    "activity-btn"
                }
                title=move || if markdown_editing_enabled.get() {
                    "Edit main text (on)"
                } else {
                    "Main text frozen — select and copy without opening the editor"
                }
                on:click=on_toggle_markdown_editing
            >
                {move || if markdown_editing_enabled.get() {
                    Either::Left(view! { <Icon icon=id::LuPencil/> })
                } else {
                    Either::Right(view! { <Icon icon=id::LuLock/> })
                }}
            </button>
            // New Block — appends a fresh empty entry.
            <button class="activity-btn top-bar-new-block" title="New Block" on:click=on_new_block>
                "＋"
            </button>
        </div>
    }
}
