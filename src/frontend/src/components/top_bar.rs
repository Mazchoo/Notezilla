use crate::components::file_io::load_markdown_file;
use crate::state::AppState;
use icondata as id;
use leptos::*;
use leptos_icons::Icon;
use web_sys::Event;

#[component]
pub fn TopBar() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState not provided");
    let blocks = state.blocks;

    let file_input_ref = create_node_ref::<leptos::html::Input>();

    // Open the OS file-picker when the Import button is clicked.
    let on_import_click = move |_| {
        if let Some(input) = file_input_ref.get() {
            input.click();
        }
    };

    // Delegate file reading + block creation to file_io.
    let on_file_change = move |ev: Event| {
        load_markdown_file(ev, blocks);
    };

    let on_save = move |_| {
        let content = state
            .blocks
            .get()
            .iter()
            .map(|b| b.text.get_untracked())
            .collect::<Vec<_>>()
            .join("\n\n");
        web_sys::console::log_1(&content.into());
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
        </div>
    }
}
