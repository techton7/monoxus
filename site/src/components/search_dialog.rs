use dioxus::prelude::keyboard_types::{Key, NamedKey};
use dioxus::prelude::*;
use monoxus::dialog::{
    DialogClose, DialogContent, DialogDescription, DialogOverlay, DialogPortal, DialogRoot,
    DialogTitle,
};
use monoxus::foundation::shared::Orientation;
use monoxus::scroll_area::{
    ScrollAreaRoot, ScrollAreaScrollbar, ScrollAreaThumb, ScrollAreaType, ScrollAreaViewport,
};

use crate::components::icons::{CloseIcon, FileTextIcon, SearchIcon};
use crate::docs_data::search_docs;

/// Global command/search dialog triggered by `Cmd+K`, `Ctrl+K`, `/`, or header button.
#[component]
pub fn SearchDialog(open: Signal<bool>) -> Element {
    let navigator = use_navigator();
    let mut query = use_signal(String::new);
    let mut active_idx = use_signal(|| 0usize);

    #[cfg(target_arch = "wasm32")]
    {
        use std::{cell::RefCell, rc::Rc};
        use wasm_bindgen::{JsCast, closure::Closure};

        let listener_slot = use_hook(|| Rc::new(RefCell::new(None::<Closure<dyn FnMut(web_sys::KeyboardEvent)>>)));
        if listener_slot.borrow().is_none() {
            let mut open_sig = open;
            let cb = Closure::wrap(Box::new(move |evt: web_sys::KeyboardEvent| {
                let key = evt.key();
                let is_cmd_k = (evt.meta_key() || evt.ctrl_key())
                    && !evt.alt_key()
                    && (key == "k" || key == "K");
                let is_slash = key == "/" && !evt.meta_key() && !evt.ctrl_key() && !evt.alt_key();

                if is_cmd_k {
                    evt.prevent_default();
                    let next = !open_sig.try_peek().map(|v| *v).unwrap_or(false);
                    let _ = open_sig.try_write().map(|mut v| *v = next);
                    return;
                }

                if is_slash && !is_editable_active_element() {
                    evt.prevent_default();
                    let _ = open_sig.try_write().map(|mut v| *v = true);
                }
            }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

            if let Some(window) = web_sys::window() {
                let _ = window.add_event_listener_with_callback(
                    "keydown",
                    cb.as_ref().unchecked_ref(),
                );
            }
            *listener_slot.borrow_mut() = Some(cb);
        }

        let cleanup_slot = listener_slot.clone();
        use_drop(move || {
            if let Some(cb) = cleanup_slot.borrow_mut().take() {
                if let Some(window) = web_sys::window() {
                    let _ = window.remove_event_listener_with_callback(
                        "keydown",
                        cb.as_ref().unchecked_ref(),
                    );
                }
            }
        });
    }

    use_effect(move || {
        if !open() {
            query.set(String::new());
            active_idx.set(0);
        }
    });

    let results = search_docs(&query());
    let result_count = results.len();
    let clamped_idx = if result_count == 0 {
        0
    } else {
        active_idx().min(result_count - 1)
    };

    let navigate_to = move |path: &'static str| {
        let mut open_sig = open;
        open_sig.set(false);
        navigator.push(path);
    };

    rsx! {
        // TODO(monoxus): remove once auto-ID & wrapper z-index hoisting land
        DialogRoot {
            id: Some("site-search-dialog".into()),
            open: Some(open),
            DialogPortal {
                DialogOverlay {
                    class: "fixed inset-0 z-50 bg-black/50 backdrop-blur-[2px] animate-fade-in",
                }
                DialogContent {
                    class: "fixed left-1/2 top-[12vh] z-50 w-[min(92vw,36rem)] -translate-x-1/2 overflow-hidden rounded-card border border-border-card bg-background shadow-popover outline-hidden animate-scale-in",
                    DialogTitle { class: "sr-only", "Search documentation" }
                    DialogDescription {
                        class: "sr-only",
                        "Search Monoxus overview guides, concepts, and component API docs."
                    }

                    div {
                        class: "flex items-center gap-3 border-b border-border-card px-4 py-3",
                        SearchIcon { class: "h-4 w-4 shrink-0 text-muted-foreground" }
                        input {
                            r#type: "text",
                            autofocus: true,
                            value: "{query}",
                            placeholder: "Search documentation and components...",
                            aria_label: "Search documentation",
                            class: "w-full bg-transparent text-sm text-foreground placeholder:text-muted-foreground focus:outline-hidden",
                            oninput: move |evt| {
                                query.set(evt.value());
                                active_idx.set(0);
                            },
                            onkeydown: move |evt| {
                                match evt.key() {
                                    Key::Named(NamedKey::ArrowDown) => {
                                        evt.prevent_default();
                                        if result_count > 0 {
                                            active_idx.set((clamped_idx + 1) % result_count);
                                        }
                                    }
                                    Key::Named(NamedKey::ArrowUp) => {
                                        evt.prevent_default();
                                        if result_count > 0 {
                                            active_idx.set((clamped_idx + result_count - 1) % result_count);
                                        }
                                    }
                                    Key::Named(NamedKey::Enter) => {
                                        if let Some(entry) = results.get(clamped_idx) {
                                            evt.prevent_default();
                                            let path = entry.path;
                                            let mut open_sig = open;
                                            open_sig.set(false);
                                            navigator.push(path);
                                        }
                                    }
                                    _ => {}
                                }
                            },
                        }
                        DialogClose {
                            class: "inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-button text-muted-foreground transition-colors hover:bg-muted hover:text-foreground",
                            span { class: "sr-only", "Close search" }
                            CloseIcon { class: "h-4 w-4" }
                        }
                    }

                    // TODO(monoxus): remove once auto-ID & wrapper z-index hoisting land
                    ScrollAreaRoot {
                        id: Some("site-search-scroll-area".into()),
                        r#type: ScrollAreaType::Auto,
                        class: "max-h-[360px] w-full overflow-hidden",
                        ScrollAreaViewport {
                            class: "max-h-[360px] w-full p-2",
                            if results.is_empty() {
                                div {
                                    class: "py-10 text-center text-sm text-muted-foreground",
                                    "No results found for \"{query}\"."
                                }
                            } else {
                                div {
                                    role: "listbox",
                                    class: "space-y-1",
                                    for (idx, entry) in results.iter().enumerate() {
                                        {
                                            let is_active = idx == clamped_idx;
                                            let row_cls = if is_active {
                                                "flex w-full items-start gap-3 rounded-card-sm bg-muted px-3 py-2.5 text-left transition-colors"
                                            } else {
                                                "flex w-full items-start gap-3 rounded-card-sm px-3 py-2.5 text-left transition-colors hover:bg-muted/60"
                                            };
                                            let target_path = entry.path;
                                            rsx! {
                                                button {
                                                    key: "{entry.path}",
                                                    r#type: "button",
                                                    role: "option",
                                                    aria_selected: "{is_active}",
                                                    class: "{row_cls}",
                                                    onmouseenter: move |_| active_idx.set(idx),
                                                    onclick: move |_| navigate_to(target_path),
                                                    FileTextIcon { class: "mt-0.5 h-4 w-4 shrink-0 text-foreground-alt" }
                                                    div {
                                                        class: "min-w-0 flex-1",
                                                        div {
                                                            class: "flex items-center justify-between gap-2",
                                                            span {
                                                                class: "truncate text-sm font-medium text-foreground",
                                                                "{entry.title}"
                                                            }
                                                            span {
                                                                class: "shrink-0 rounded-md border border-border-card bg-background px-1.5 py-0.5 text-[11px] font-medium text-foreground-alt",
                                                                "{entry.section}"
                                                            }
                                                        }
                                                        p {
                                                            class: "mt-0.5 truncate text-xs text-foreground-alt",
                                                            "{entry.description}"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        ScrollAreaScrollbar {
                            orientation: Orientation::Vertical,
                            class: "flex w-2 touch-none select-none p-0.5 transition-colors",
                            ScrollAreaThumb {
                                class: "relative flex-1 rounded-full bg-dark-10 hover:bg-dark-40",
                            }
                        }
                    }

                    div {
                        class: "flex items-center justify-between border-t border-border-card bg-muted/30 px-4 py-2 text-[11px] text-muted-foreground",
                        div {
                            class: "flex items-center gap-3",
                            span {
                                kbd { class: "rounded border border-border-input bg-background px-1 py-0.5 font-mono text-[10px] text-foreground shadow-kbd", "↑↓" }
                                " Navigate"
                            }
                            span {
                                kbd { class: "rounded border border-border-input bg-background px-1 py-0.5 font-mono text-[10px] text-foreground shadow-kbd", "↵" }
                                " Open"
                            }
                        }
                        span {
                            kbd { class: "rounded border border-border-input bg-background px-1 py-0.5 font-mono text-[10px] text-foreground shadow-kbd", "Esc" }
                            " Close"
                        }
                    }
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn is_editable_active_element() -> bool {
    use wasm_bindgen::JsCast;

    let Some(window) = web_sys::window() else {
        return false;
    };
    let Some(document) = window.document() else {
        return false;
    };
    let Some(el) = document.active_element() else {
        return false;
    };
    let tag = el.tag_name();
    if matches!(tag.as_str(), "INPUT" | "TEXTAREA" | "SELECT") {
        return true;
    }
    el.dyn_ref::<web_sys::HtmlElement>()
        .is_some_and(|html| html.is_content_editable())
}
