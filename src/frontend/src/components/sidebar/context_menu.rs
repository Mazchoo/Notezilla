use leptos::prelude::*;

/// Right-click context menu for file tree items.
#[component]
pub fn FileContextMenu<FOpen, FDelete>(
    visible: RwSignal<bool>,
    x: RwSignal<f64>,
    y: RwSignal<f64>,
    on_open: FOpen,
    on_delete: FDelete,
) -> impl IntoView
where
    FOpen: Fn() + 'static + Clone + Send + Sync,
    FDelete: Fn() + 'static + Clone + Send + Sync,
{
    let close = move |_| visible.set(false);

    view! {
        <Show when=move || visible.get()>
            <div class="context-menu-backdrop" on:click=close></div>
            <div
                class="file-context-menu menu"
                style=move || format!("left: {}px; top: {}px;", x.get(), y.get())
                on:click=|ev: web_sys::MouseEvent| ev.stop_propagation()
            >
                <ul class="menu-list">
                    <li>
                        <a
                            class="context-menu-item"
                            on:click={
                                let on_open = on_open.clone();
                                move |ev: web_sys::MouseEvent| {
                                    ev.prevent_default();
                                    on_open();
                                    visible.set(false);
                                }
                            }
                        >
                            "Open"
                        </a>
                    </li>
                    <li>
                        <a
                            class="context-menu-item"
                            on:click={
                                let on_delete = on_delete.clone();
                                move |ev: web_sys::MouseEvent| {
                                    ev.prevent_default();
                                    on_delete();
                                    visible.set(false);
                                }
                            }
                        >
                            "Delete"
                        </a>
                    </li>
                </ul>
            </div>
        </Show>
    }
}

/// Right-click context menu for folder tree items.
#[component]
pub fn FolderContextMenu<FDelete>(
    visible: RwSignal<bool>,
    x: RwSignal<f64>,
    y: RwSignal<f64>,
    on_delete: FDelete,
) -> impl IntoView
where
    FDelete: Fn() + 'static + Clone + Send + Sync,
{
    let close = move |_| visible.set(false);

    view! {
        <Show when=move || visible.get()>
            <div class="context-menu-backdrop" on:click=close></div>
            <div
                class="file-context-menu menu"
                style=move || format!("left: {}px; top: {}px;", x.get(), y.get())
                on:click=|ev: web_sys::MouseEvent| ev.stop_propagation()
            >
                <ul class="menu-list">
                    <li>
                        <a
                            class="context-menu-item"
                            on:click={
                                let on_delete = on_delete.clone();
                                move |ev: web_sys::MouseEvent| {
                                    ev.prevent_default();
                                    on_delete();
                                    visible.set(false);
                                }
                            }
                        >
                            "Delete"
                        </a>
                    </li>
                </ul>
            </div>
        </Show>
    }
}
