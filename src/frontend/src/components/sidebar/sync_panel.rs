use crate::components::toast::show_error_toast;
use crate::info_messages::{
    APP_STATE_MISSING_PANIC, MCP_SESSION_NOT_READY_TOAST, SYNC_CHECKING_LABEL, SYNC_CHECK_TITLE,
    SYNC_DB_ONLY_HEADING, SYNC_DELETE_ALL_FILES_TITLE, SYNC_DELETE_ALL_FROM_DB_TITLE,
    SYNC_DELETE_FILE_TITLE, SYNC_DELETE_FROM_DB_TITLE, SYNC_DIR_ONLY_HEADING, SYNC_HEADING,
    SYNC_NONE_LABEL, SYNC_NOT_CHECKED_LABEL, SYNC_RESTORE_ALL_TO_DB_TITLE,
    SYNC_RESTORE_ALL_TO_DISK_TITLE, SYNC_RESTORE_TO_DB_TITLE, SYNC_RESTORE_TO_DISK_TITLE,
};
use crate::mcp::tools::{
    delete_db_only_note, delete_db_only_notes, delete_dir_only_notes, delete_note,
    get_database_sync_difference, restore_db_only_note, restore_db_only_notes,
    upsert_dir_only_note, upsert_dir_only_notes,
};
use crate::models::note::SyncDifference;
use crate::state::AppState;
use icondata as id;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;

/// Which side of the sync difference a list shows.
#[derive(Clone, Copy, PartialEq, Debug)]
enum SyncSide {
    /// In the database but not on disk.
    DbOnly,
    /// On disk but not in the database.
    DirOnly,
}

/// Action applied to one path or a whole list.
#[derive(Clone, Copy, PartialEq, Debug)]
enum SyncAction {
    Restore,
    Delete,
}

impl SyncSide {
    fn heading(self) -> &'static str {
        match self {
            Self::DbOnly => SYNC_DB_ONLY_HEADING,
            Self::DirOnly => SYNC_DIR_ONLY_HEADING,
        }
    }

    fn item_title(self, action: SyncAction) -> &'static str {
        match (self, action) {
            (Self::DbOnly, SyncAction::Restore) => SYNC_RESTORE_TO_DISK_TITLE,
            (Self::DbOnly, SyncAction::Delete) => SYNC_DELETE_FROM_DB_TITLE,
            (Self::DirOnly, SyncAction::Restore) => SYNC_RESTORE_TO_DB_TITLE,
            (Self::DirOnly, SyncAction::Delete) => SYNC_DELETE_FILE_TITLE,
        }
    }

    fn all_title(self, action: SyncAction) -> &'static str {
        match (self, action) {
            (Self::DbOnly, SyncAction::Restore) => SYNC_RESTORE_ALL_TO_DISK_TITLE,
            (Self::DbOnly, SyncAction::Delete) => SYNC_DELETE_ALL_FROM_DB_TITLE,
            (Self::DirOnly, SyncAction::Restore) => SYNC_RESTORE_ALL_TO_DB_TITLE,
            (Self::DirOnly, SyncAction::Delete) => SYNC_DELETE_ALL_FILES_TITLE,
        }
    }

    fn paths(self, diff: &SyncDifference) -> &Vec<String> {
        match self {
            Self::DbOnly => &diff.db_only,
            Self::DirOnly => &diff.dir_only,
        }
    }

    fn paths_mut(self, diff: &mut SyncDifference) -> &mut Vec<String> {
        match self {
            Self::DbOnly => &mut diff.db_only,
            Self::DirOnly => &mut diff.dir_only,
        }
    }

    /// Apply `action` to a single path on the MCP backend.
    async fn apply_one(self, action: SyncAction, sid: &str, path: &str) -> Result<(), String> {
        match (self, action) {
            (Self::DbOnly, SyncAction::Restore) => restore_db_only_note(sid, path).await,
            (Self::DbOnly, SyncAction::Delete) => delete_db_only_note(sid, path).await,
            (Self::DirOnly, SyncAction::Restore) => upsert_dir_only_note(sid, path).await,
            (Self::DirOnly, SyncAction::Delete) => delete_note(sid, path).await,
        }
    }

    /// Apply `action` to every path on this side on the MCP backend.
    async fn apply_all(self, action: SyncAction, sid: &str) -> Result<(), String> {
        match (self, action) {
            (Self::DbOnly, SyncAction::Restore) => restore_db_only_notes(sid).await,
            (Self::DbOnly, SyncAction::Delete) => delete_db_only_notes(sid).await,
            (Self::DirOnly, SyncAction::Restore) => upsert_dir_only_notes(sid).await,
            (Self::DirOnly, SyncAction::Delete) => delete_dir_only_notes(sid).await,
        }
    }
}

/// Signals shared by the sync panel and its lists.
#[derive(Clone, Copy)]
struct SyncCtx {
    session: RwSignal<Option<String>>,
    error_toast: RwSignal<Option<String>>,
    result: RwSignal<Option<SyncDifference>>,
    loading: RwSignal<bool>,
}

impl SyncCtx {
    /// Return the session id, or show a toast when the MCP session is not ready.
    fn session_id(self) -> Option<String> {
        let sid = self.session.get_untracked();
        if sid.is_none() {
            show_error_toast(self.error_toast, MCP_SESSION_NOT_READY_TOAST);
        }
        sid
    }

    /// Log an MCP error and show it in the error toast.
    fn report_error(self, e: String) {
        web_sys::console::error_1(&e.clone().into());
        show_error_toast(self.error_toast, e);
    }

    /// Request the database vs disk difference and store it in `result`.
    fn run_check(self) {
        if self.loading.get_untracked() {
            return;
        }
        let Some(sid) = self.session_id() else {
            return;
        };
        self.loading.set(true);
        spawn_local(async move {
            self.refresh(&sid).await;
            self.loading.set(false);
        });
    }

    /// Fetch the database vs disk difference into `result`, reporting any error.
    async fn refresh(self, sid: &str) {
        match get_database_sync_difference(sid).await {
            Ok(diff) => self.result.set(Some(diff)),
            Err(e) => self.report_error(e),
        }
    }

    /// Drop `path` from the `side` list after a successful item action.
    fn remove_path(self, side: SyncSide, path: &str) {
        self.result.update(|r| {
            if let Some(diff) = r {
                side.paths_mut(diff).retain(|p| p != path);
            }
        });
    }
}

/// Run `action` on one path, greying its row out while pending.
///
/// On success the row is removed and `pending` is left set, so the row never
/// becomes clickable again before it disappears. On failure `pending` is cleared.
fn run_item_action(
    ctx: SyncCtx,
    side: SyncSide,
    action: SyncAction,
    path: String,
    pending: RwSignal<bool>,
) {
    if pending.get_untracked() {
        return;
    }
    let Some(sid) = ctx.session_id() else {
        return;
    };
    pending.set(true);
    spawn_local(async move {
        match side.apply_one(action, &sid, &path).await {
            Ok(()) => ctx.remove_path(side, &path),
            Err(e) => {
                ctx.report_error(e);
                pending.set(false);
            }
        }
    });
}

/// Run `action` on a whole list, greying it out until the re-checked difference arrives.
fn run_all_action(ctx: SyncCtx, side: SyncSide, action: SyncAction, busy: RwSignal<bool>) {
    if busy.get_untracked() {
        return;
    }
    let Some(sid) = ctx.session_id() else {
        return;
    };
    busy.set(true);
    spawn_local(async move {
        if let Err(e) = side.apply_all(action, &sid).await {
            ctx.report_error(e);
        }
        ctx.refresh(&sid).await;
        busy.set(false);
    });
}

/// Render one titled list of paths with restore and delete buttons per item and for the list.
#[component]
fn SyncPathList(side: SyncSide, ctx: SyncCtx) -> impl IntoView {
    let busy = RwSignal::new(false);
    let paths = Memo::new(move |_| {
        ctx.result.with(|r| {
            r.as_ref()
                .map(|d| side.paths(d).clone())
                .unwrap_or_default()
        })
    });
    let has_paths = move || paths.with(|p| !p.is_empty());

    view! {
        <div class=move || if busy.get() { "sync-list mt-3 is-pending" } else { "sync-list mt-3" }>
            <div class="sync-list-heading">
                <span>{side.heading()}</span>
                <span class="sync-list-count">{move || paths.with(Vec::len)}</span>
                <Show when=has_paths>
                    <span class="sync-list-actions">
                        <button
                            class="sync-icon-btn"
                            type="button"
                            title=side.all_title(SyncAction::Restore)
                            prop:disabled=move || busy.get()
                            on:click=move |_| run_all_action(ctx, side, SyncAction::Restore, busy)
                        >
                            <Icon icon=id::LuRotateCcw/>
                        </button>
                        <button
                            class="sync-icon-btn is-danger"
                            type="button"
                            title=side.all_title(SyncAction::Delete)
                            prop:disabled=move || busy.get()
                            on:click=move |_| run_all_action(ctx, side, SyncAction::Delete, busy)
                        >
                            <Icon icon=id::LuTrash2/>
                        </button>
                    </span>
                </Show>
            </div>
            <Show
                when=has_paths
                fallback=|| view! { <p class="sync-list-empty">{SYNC_NONE_LABEL}</p> }
            >
                <ul>
                    <For
                        each=move || paths.get()
                        key=|path| path.clone()
                        children=move |path: String| {
                            let pending = RwSignal::new(false);
                            let is_pending = move || busy.get() || pending.get();
                            let restore_path = path.clone();
                            let delete_path = path.clone();
                            let title = path.clone();
                            view! {
                                <li
                                    class=move || if is_pending() { "sync-list-item is-pending" } else { "sync-list-item" }
                                    title=title
                                >
                                    <span class="sync-list-path">{path}</span>
                                    <button
                                        class="sync-icon-btn"
                                        type="button"
                                        title=side.item_title(SyncAction::Restore)
                                        prop:disabled=is_pending
                                        on:click=move |_| run_item_action(
                                            ctx,
                                            side,
                                            SyncAction::Restore,
                                            restore_path.clone(),
                                            pending,
                                        )
                                    >
                                        <Icon icon=id::LuRotateCcw/>
                                    </button>
                                    <button
                                        class="sync-icon-btn is-danger"
                                        type="button"
                                        title=side.item_title(SyncAction::Delete)
                                        prop:disabled=is_pending
                                        on:click=move |_| run_item_action(
                                            ctx,
                                            side,
                                            SyncAction::Delete,
                                            delete_path.clone(),
                                            pending,
                                        )
                                    >
                                        <Icon icon=id::LuTrash2/>
                                    </button>
                                </li>
                            }
                        }
                    />
                </ul>
            </Show>
        </div>
    }
}

/// Render the database sync panel: a check button and the db-only and disk-only path lists.
#[component]
pub fn SyncPanel() -> impl IntoView {
    let state = use_context::<AppState>().expect(APP_STATE_MISSING_PANIC);
    let ctx = SyncCtx {
        session: state.session_id,
        error_toast: state.error_toast,
        result: RwSignal::new(None),
        loading: RwSignal::new(false),
    };
    let loading = ctx.loading;
    let result = ctx.result;

    view! {
        <div class="p-3">
            <div class="file-tree-heading mt-2">
                <p class="menu-label">{SYNC_HEADING}</p>
                <button
                    class=move || if loading.get() { "sync-icon-btn is-spinning" } else { "sync-icon-btn" }
                    type="button"
                    title=SYNC_CHECK_TITLE
                    prop:disabled=move || loading.get()
                    on:click=move |_| ctx.run_check()
                >
                    <Icon icon=id::LuRefreshCw/>
                </button>
            </div>
            <Show when=move || loading.get()>
                <p class="sync-list-empty mt-2">{SYNC_CHECKING_LABEL}</p>
            </Show>
            <Show
                when=move || result.with(Option::is_some)
                fallback=move || view! {
                    <Show when=move || !loading.get()>
                        <p class="sync-list-empty mt-2">{SYNC_NOT_CHECKED_LABEL}</p>
                    </Show>
                }
            >
                <SyncPathList side=SyncSide::DbOnly ctx=ctx/>
                <SyncPathList side=SyncSide::DirOnly ctx=ctx/>
            </Show>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diff() -> SyncDifference {
        SyncDifference {
            db_only: vec!["a.md".into()],
            dir_only: vec!["b.md".into(), "c.md".into()],
        }
    }

    #[test]
    /// Assert each side reads and edits its own path list.
    fn sync_side_selects_matching_list() {
        let mut d = diff();
        assert_eq!(SyncSide::DbOnly.paths(&d), &vec!["a.md".to_string()]);
        assert_eq!(SyncSide::DirOnly.paths(&d).len(), 2);
        SyncSide::DirOnly.paths_mut(&mut d).retain(|p| p != "b.md");
        assert_eq!(d.dir_only, vec!["c.md".to_string()]);
        assert_eq!(d.db_only, vec!["a.md".to_string()]);
    }

    #[test]
    /// Assert item and list titles differ by side and action.
    fn sync_side_titles_match_side_and_action() {
        assert_eq!(
            SyncSide::DbOnly.item_title(SyncAction::Restore),
            SYNC_RESTORE_TO_DISK_TITLE
        );
        assert_eq!(
            SyncSide::DirOnly.item_title(SyncAction::Delete),
            SYNC_DELETE_FILE_TITLE
        );
        assert_eq!(
            SyncSide::DirOnly.all_title(SyncAction::Restore),
            SYNC_RESTORE_ALL_TO_DB_TITLE
        );
    }
}
