use crate::mcp::tools::get_dir_contents;
use crate::models::note::DirectoryContents;
use leptos::prelude::*;
use leptos::task::spawn_local;

/// Fetch a directory listing from the MCP backend into `into`.
///
/// Reads `session` reactively so callers inside an `Effect` re-run when the
/// session becomes ready. After the request completes, contents are written
/// only if `guard` returns true (e.g. ignore stale responses after collapse).
pub fn fetch_dir_contents(
    session: RwSignal<Option<String>>,
    path: impl Into<String>,
    into: RwSignal<Option<DirectoryContents>>,
    guard: impl Fn() -> bool + 'static,
) {
    let sid = match session.get() {
        Some(s) => s,
        None => return,
    };
    let path = path.into();
    spawn_local(async move {
        match get_dir_contents(&sid, &path).await {
            Ok(contents) => {
                if guard() {
                    into.set(Some(contents));
                }
            }
            Err(e) => web_sys::console::error_1(&e.into()),
        }
    });
}
