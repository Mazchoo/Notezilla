use crate::components::file_io::{export_entries_as_html, load_markdown_file};
use crate::models::block::EditorEntry;
use crate::state::AppState;
use icondata as id;
use leptos::either::Either;
use leptos::prelude::*;
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

    let on_save = move |_| {
        for entry in state.entries.get().iter() {
            let file_name = entry.title.path.get_untracked();
            let body = entry.content.text.get_untracked();

            let full_content = match entry.front_matter.get_untracked() {
                Some(fm) => {
                    let raw = fm.raw.get_untracked();
                    format!("---\n{}\n---\n{}", raw, body)
                }
                None => body,
            };

            web_sys::console::log_1(&format!("file name: {}", file_name).into());
            web_sys::console::log_1(&format!("contents:\n{}", full_content).into());
        }
    };

    let on_export = move |_| {
        export_entries_as_html(&state.entries.get_untracked());
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
            // Save — logs full markdown to console.
            <button class="activity-btn" title="Save (Ctrl+S)" on:click=on_save>
                <Icon icon=id::LuSave/>
            </button>
            // Export — save each entry as a standalone HTML file.
            <button class="activity-btn" title="Export as HTML" on:click=on_export>
                <Icon icon=id::LuDownload/>
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
