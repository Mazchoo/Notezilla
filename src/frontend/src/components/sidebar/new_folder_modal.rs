use leptos::html::Input;
use leptos::prelude::*;

/// Shared controller for the file-tree new-folder modal.
#[derive(Clone, Copy)]
pub struct NewFolderModalCtrl {
    /// Parent folder path relative to the note folder; `Some("")` for root.
    pub parent_path: RwSignal<Option<String>>,
}

impl NewFolderModalCtrl {
    /// Create a closed new-folder modal controller.
    pub fn new() -> Self {
        Self {
            parent_path: RwSignal::new(None),
        }
    }

    /// Open the new-folder modal under `parent_path`.
    pub fn open(self, parent_path: String) {
        self.parent_path.set(Some(parent_path));
    }

    /// Close the new-folder modal.
    pub fn close(self) {
        self.parent_path.set(None);
    }
}

/// Prompt for a new folder name under a parent path.
#[component]
pub fn NewFolderModal<FConfirm>(ctrl: NewFolderModalCtrl, on_confirm: FConfirm) -> impl IntoView
where
    FConfirm: Fn(String, String) + 'static + Clone + Send + Sync,
{
    let draft = RwSignal::new(String::new());
    let input_ref = NodeRef::<Input>::new();

    Effect::new(move |_| {
        if ctrl.parent_path.get().is_some() {
            draft.set(String::new());
            request_animation_frame(move || {
                if let Some(el) = input_ref.get_untracked() {
                    let _ = el.focus();
                }
            });
        }
    });

    let on_input = move |ev: web_sys::Event| {
        draft.set(event_target_value(&ev));
    };

    view! {
        <Show when=move || ctrl.parent_path.get().is_some()>
            <div class="rename-modal-backdrop" on:click=move |_| ctrl.close()>
                <div
                    class="rename-modal"
                    role="dialog"
                    aria-modal="true"
                    aria-label="New folder"
                    on:click=|ev: web_sys::MouseEvent| ev.stop_propagation()
                >
                    <p class="rename-modal-title">"New folder"</p>
                    <input
                        node_ref=input_ref
                        class="input rename-modal-input"
                        type="text"
                        placeholder="Folder name"
                        prop:value=move || draft.get()
                        on:input=on_input
                        on:keydown={
                            let on_confirm = on_confirm.clone();
                            move |ev: web_sys::KeyboardEvent| {
                                if ev.key() == "Escape" {
                                    ev.prevent_default();
                                    ctrl.close();
                                } else if ev.key() == "Enter" {
                                    ev.prevent_default();
                                    let Some(parent) = ctrl.parent_path.get_untracked() else {
                                        return;
                                    };
                                    let name = draft.get_untracked();
                                    ctrl.close();
                                    on_confirm(parent, name);
                                }
                            }
                        }
                    />
                    <div class="rename-modal-actions">
                        <button
                            class="button is-dark is-small"
                            type="button"
                            on:click=move |_| ctrl.close()
                        >
                            "Cancel"
                        </button>
                        <button
                            class="button is-dark is-small rename-modal-confirm"
                            type="button"
                            on:click={
                                let on_confirm = on_confirm.clone();
                                move |_| {
                                    let Some(parent) = ctrl.parent_path.get_untracked() else {
                                        return;
                                    };
                                    let name = draft.get_untracked();
                                    ctrl.close();
                                    on_confirm(parent, name);
                                }
                            }
                        >
                            "Create"
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
