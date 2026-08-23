use crate::components::file_io::path::basename;
use leptos::html::Input;
use leptos::prelude::*;

/// Shared controller for the file-tree rename modal.
#[derive(Clone, Copy)]
pub struct RenameModalCtrl {
    pub path: RwSignal<Option<String>>,
    pub is_file: RwSignal<bool>,
}

impl RenameModalCtrl {
    /// Create a closed rename-modal controller.
    pub fn new() -> Self {
        Self {
            path: RwSignal::new(None),
            is_file: RwSignal::new(true),
        }
    }

    /// Open the rename modal for `path`.
    pub fn open(self, path: String, is_file: bool) {
        self.is_file.set(is_file);
        self.path.set(Some(path));
    }

    /// Close the rename modal.
    pub fn close(self) {
        self.path.set(None);
    }
}

/// Prompt for a new basename for a file or folder.
#[component]
pub fn RenameModal<FConfirm>(ctrl: RenameModalCtrl, on_confirm: FConfirm) -> impl IntoView
where
    FConfirm: Fn(String, String, bool) + 'static + Clone + Send + Sync,
{
    let draft = RwSignal::new(String::new());
    let input_ref = NodeRef::<Input>::new();

    Effect::new(move |_| {
        if let Some(path) = ctrl.path.get() {
            draft.set(basename(&path).to_string());
            request_animation_frame(move || {
                if let Some(el) = input_ref.get_untracked() {
                    let _ = el.focus();
                    let _ = el.select();
                }
            });
        }
    });

    let on_input = move |ev: web_sys::Event| {
        draft.set(event_target_value(&ev));
    };

    view! {
        <Show when=move || ctrl.path.get().is_some()>
            <div class="rename-modal-backdrop" on:click=move |_| ctrl.close()>
                <div
                    class="rename-modal"
                    role="dialog"
                    aria-modal="true"
                    aria-label=move || {
                        if ctrl.is_file.get() {
                            "Rename file".to_string()
                        } else {
                            "Rename folder".to_string()
                        }
                    }
                    on:click=|ev: web_sys::MouseEvent| ev.stop_propagation()
                >
                    <p class="rename-modal-title">
                        {move || {
                            if ctrl.is_file.get() {
                                "Rename file"
                            } else {
                                "Rename folder"
                            }
                        }}
                    </p>
                    <input
                        node_ref=input_ref
                        class="input rename-modal-input"
                        type="text"
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
                                    let Some(path) = ctrl.path.get_untracked() else {
                                        return;
                                    };
                                    let is_file = ctrl.is_file.get_untracked();
                                    let new_name = draft.get_untracked();
                                    ctrl.close();
                                    on_confirm(path, new_name, is_file);
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
                                    let Some(path) = ctrl.path.get_untracked() else {
                                        return;
                                    };
                                    let is_file = ctrl.is_file.get_untracked();
                                    let new_name = draft.get_untracked();
                                    ctrl.close();
                                    on_confirm(path, new_name, is_file);
                                }
                            }
                        >
                            "Rename"
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
