use crate::components::sidebar::file_tree_backend::FileTreeBackend;
use crate::models::note::DirectoryContents;
use leptos::prelude::*;
use leptos::task::spawn_local;

/// Fetch a directory listing from the MCP backend into `into`.
pub fn fetch_dir_contents(
    session: RwSignal<Option<String>>,
    path: impl Into<String>,
    into: RwSignal<Option<DirectoryContents>>,
    guard: impl Fn() -> bool + 'static,
    backend: FileTreeBackend,
) {
    let sid = match session.get() {
        Some(s) => s,
        None => return,
    };
    let path = path.into();
    spawn_local(async move {
        match backend.get_dir_contents(&sid, &path).await {
            Ok(contents) => {
                if guard() {
                    into.set(Some(contents));
                }
            }
            Err(e) => web_sys::console::error_1(&e.into()),
        }
    });
}
