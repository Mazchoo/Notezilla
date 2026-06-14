use crate::components::file_io::load_markdown_file;
use crate::models::block::EditorEntry;
use crate::state::AppState;
use icondata as id;
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

    // Append a new empty entry (divider + title + blank markdown block) and focus it.
    let on_new_block = move |_| {
        entries.update(|list: &mut Vec<EditorEntry>| {
            let entry = EditorEntry::empty("./new_file.md");
            entry.content.focused.set(true);
            list.push(entry);
        });
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
            // New Block — appends a fresh empty entry.
            <button class="activity-btn top-bar-new-block" title="New Block" on:click=on_new_block>
                "＋"
            </button>
        </div>
    }
}
