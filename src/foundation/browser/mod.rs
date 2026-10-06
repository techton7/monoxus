use dioxus::prelude::*;

use crate::foundation::{compose::MountedHandle, overlay::PresenceCloseCycleId};

#[allow(unused_imports)]
pub use oxidase::watcher::{
    DismissEvent, DocumentDismissEvent as DocumentDismissEventPayload, FloatingAutoUpdateEvent,
    FloatingAutoUpdatePayload, FormResetEvent, FormResetEventPayload, PresenceEvent,
    PresenceEventPayload, PresenceFallbackReason, PresenceMonitorFallback,
};




pub use oxidase::WatcherGuard;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PresenceMonitorEvent {
    Fallback {
        cycle_id: PresenceCloseCycleId,
        reason: PresenceMonitorFallback,
    },
    AnimationEnd {
        cycle_id: PresenceCloseCycleId,
        animation_name: String,
    },
    AnimationCancel {
        cycle_id: PresenceCloseCycleId,
        animation_name: String,
    },
    Stopped {
        cycle_id: PresenceCloseCycleId,
    },
}

pub(crate) fn start_presence_monitor(
    root_id: &str,
    cycle_id: PresenceCloseCycleId,
    mut on_event: impl FnMut(PresenceMonitorEvent) + 'static,
) -> WatcherGuard {
    oxidase::watcher::watch_presence(
        root_id,
        cycle_id.get(),
        move |payload: oxidase::watcher::PresenceEvent| {
            let event = match payload {
                oxidase::watcher::PresenceEvent::Fallback { cycle_id, reason } => {
                    PresenceMonitorEvent::Fallback {
                        cycle_id: PresenceCloseCycleId::from_raw(cycle_id),
                        reason,
                    }
                }
                oxidase::watcher::PresenceEvent::AnimationEnd {
                    cycle_id,
                    animation_name,
                } => PresenceMonitorEvent::AnimationEnd {
                    cycle_id: PresenceCloseCycleId::from_raw(cycle_id),
                    animation_name,
                },
                oxidase::watcher::PresenceEvent::AnimationCancel {
                    cycle_id,
                    animation_name,
                } => PresenceMonitorEvent::AnimationCancel {
                    cycle_id: PresenceCloseCycleId::from_raw(cycle_id),
                    animation_name,
                },
                oxidase::watcher::PresenceEvent::Stopped { cycle_id } => {
                    PresenceMonitorEvent::Stopped {
                        cycle_id: PresenceCloseCycleId::from_raw(cycle_id),
                    }
                }
            };
            on_event(event);
        },
    )
    .unwrap_or_else(|_| WatcherGuard::noop())
}

pub(crate) fn start_floating_auto_update_monitor(
    anchor_ids: &[&str],
    content_id: &str,
    mut on_event: impl FnMut(FloatingAutoUpdateEvent) + 'static,
) -> WatcherGuard {
    oxidase::watcher::watch_floating_auto_update(
        anchor_ids,
        content_id,
        move |payload: oxidase::watcher::FloatingAutoUpdateEvent| {
            let event = match payload {
                oxidase::watcher::FloatingAutoUpdateEvent::Scroll => {
                    FloatingAutoUpdateEvent::Scroll
                }
                oxidase::watcher::FloatingAutoUpdateEvent::Update => {
                    FloatingAutoUpdateEvent::Update
                }
            };
            on_event(event);
        },
    )
    .unwrap_or_else(|_| WatcherGuard::noop())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DocumentDismissEvent {
    PointerDown { path_ids: Vec<String> },
    FocusIn { path_ids: Vec<String> },
    Escape,
}

#[allow(dead_code)]
pub(crate) fn start_document_dismiss_monitor(
    on_event: impl FnMut(DocumentDismissEvent) + 'static,
) -> WatcherGuard {
    start_document_dismiss_monitor_with_boundaries(&[], on_event)
}

pub(crate) fn start_document_dismiss_monitor_with_boundaries(
    boundaries: &[&str],
    mut on_event: impl FnMut(DocumentDismissEvent) + 'static,
) -> WatcherGuard {
    oxidase::watcher::watch_document_dismiss(
        boundaries,
        move |payload: oxidase::watcher::DismissEvent| {
            let event = match payload {
                oxidase::watcher::DismissEvent::PointerDown { path_ids } => {
                    DocumentDismissEvent::PointerDown { path_ids }
                }
                oxidase::watcher::DismissEvent::FocusIn { path_ids } => {
                    DocumentDismissEvent::FocusIn { path_ids }
                }
                oxidase::watcher::DismissEvent::Escape => DocumentDismissEvent::Escape,
            };
            on_event(event);
        },
    )
    .unwrap_or_else(|_| WatcherGuard::noop())
}

#[allow(dead_code)]
pub(crate) const DEFAULT_FOCUSABLE_SELECTOR: &str = "a[href],button:not([disabled]),input:not([disabled]),select:not([disabled]),textarea:not([disabled]),[tabindex]:not([tabindex='-1'])";

pub(crate) fn focus_mounted_handle(handle: Option<MountedHandle>) -> bool {
    let Some(handle) = handle else {
        return false;
    };
    let el = oxidase::Element::from(handle);
    spawn(async move {
        let _ = el.focus().await;
    });
    true
}

pub(crate) async fn get_viewport_size() -> Result<[f64; 2], String> {
    let (width, height) = oxidase::window()
        .inner_size()
        .await
        .map_err(|e| e.to_string())?;
    Ok([width, height])
}

pub(crate) async fn active_element_matches_id(target_id: &str) -> bool {
    oxidase::document()
        .active_element()
        .and_then(|el| el.id())
        .map(|id| id == target_id)
        .unwrap_or(false)
}

pub(crate) async fn is_reference_hidden(anchor_ids: &[&str]) -> Result<bool, String> {
    oxidase::runtime::is_reference_hidden(anchor_ids)
        .await
        .map_err(|e| e.to_string())
}

pub(crate) fn focus_element_by_id(target_id: &str) {
    if let Some(el) = oxidase::document().element_by_id(target_id) {
        spawn(async move {
            let _ = el.focus().await;
        });
    }
    let tid = target_id.to_string();
    spawn(async move {
        let _ = oxidase::frame::next_frame().await;
        if let Some(el) = oxidase::document().element_by_id(&tid) {
            let _ = el.focus().await;
        }
    });
}

pub(crate) fn restore_focus_element_by_id(target_id: &str) {
    if let Some(el) = oxidase::document().element_by_id(target_id) {
        let options = oxidase::dom::FocusOptions::new(true);
        spawn(async move {
            let _ = el.focus_with_options(options).await;
        });
    }
    let tid = target_id.to_string();
    spawn(async move {
        let _ = oxidase::frame::next_frame().await;
        if let Some(el) = oxidase::document().element_by_id(&tid) {
            let options = oxidase::dom::FocusOptions::new(true);
            let _ = el.focus_with_options(options).await;
        }
    });
}

#[cfg(target_arch = "wasm32")]
mod scroll_lock {
    use std::{
        cell::RefCell,
        collections::HashSet,
        time::Duration,
    };

    struct ScrollLockState {
        locks: HashSet<String>,
        original_overflow: String,
        original_padding_right: String,
        cleanup_generation: u64,
    }

    thread_local! {
        static STATE: RefCell<ScrollLockState> = RefCell::new(ScrollLockState {
            locks: HashSet::new(),
            original_overflow: String::new(),
            original_padding_right: String::new(),
            cleanup_generation: 0,
        });
    }

    pub(crate) fn acquire(lock_id: &str) {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(document) = window.document() else {
            return;
        };
        let Some(body) = document.body() else {
            return;
        };

        STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.cleanup_generation = state.cleanup_generation.wrapping_add(1);

            if state.locks.is_empty() {
                let body_style = body.style();
                state.original_overflow = body_style
                    .get_property_value("overflow")
                    .unwrap_or_default();
                state.original_padding_right = body_style
                    .get_property_value("padding-right")
                    .unwrap_or_default();

                let doc_elem = document.document_element();
                let html_style = doc_elem
                    .as_ref()
                    .and_then(|el| window.get_computed_style(el).ok().flatten());
                let computed_body_style = window.get_computed_style(&body).ok().flatten();

                let has_stable_gutter = html_style
                    .and_then(|s| s.get_property_value("scrollbar-gutter").ok())
                    .unwrap_or_default()
                    .contains("stable")
                    || computed_body_style
                        .and_then(|s| s.get_property_value("scrollbar-gutter").ok())
                        .unwrap_or_default()
                        .contains("stable");

                let window_inner_width = window
                    .inner_width()
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as i32;
                let doc_client_width = doc_elem.map(|el| el.client_width()).unwrap_or(0);
                let scrollbar_width = window_inner_width - doc_client_width;

                if scrollbar_width > 0 && !has_stable_gutter {
                    let px_val = format!("{scrollbar_width}px");
                    let _ = body_style.set_property("padding-right", &px_val);
                    let _ = body_style.set_property("--scrollbar-width", &px_val);
                }
                let _ = body_style.set_property("overflow", "hidden");
            }

            state.locks.insert(lock_id.to_string());
        });
    }

    pub(crate) fn release(lock_id: &str, restore_delay: Option<u64>) {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(document) = window.document() else {
            return;
        };
        let Some(body) = document.body() else {
            return;
        };

        let (should_restore, token) = STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.locks.remove(lock_id);
            if !state.locks.is_empty() {
                return (false, 0);
            }
            state.cleanup_generation = state.cleanup_generation.wrapping_add(1);
            (true, state.cleanup_generation)
        });

        if !should_restore {
            return;
        }

        let do_restore = move || {
            STATE.with(|state| {
                let state = state.borrow();
                if state.cleanup_generation != token || !state.locks.is_empty() {
                    return;
                }
                let body_style = body.style();
                let _ = body_style.set_property("overflow", &state.original_overflow);
                let _ = body_style.set_property("padding-right", &state.original_padding_right);
                let _ = body_style.remove_property("--scrollbar-width");
            });
        };

        let delay_ms = restore_delay.unwrap_or(0);
        if delay_ms > 0 {
            dioxus::prelude::spawn(async move {
                futures_timer::Delay::new(Duration::from_millis(delay_ms)).await;
                do_restore();
            });
        } else {
            do_restore();
        }
    }
}

pub(crate) fn focus_first_focusable(content_id: &str, focusable_selector: Option<&str>) {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;

        let selector = focusable_selector.unwrap_or(DEFAULT_FOCUSABLE_SELECTOR);
        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(document) = window.document() else {
            return;
        };
        let Some(root) = document.get_element_by_id(content_id) else {
            return;
        };
        let Ok(root_html) = root.clone().dyn_into::<web_sys::HtmlElement>() else {
            return;
        };

        let candidate = if root.matches(selector).unwrap_or(false) {
            Some(root_html.clone())
        } else {
            root.query_selector(selector)
                .ok()
                .flatten()
                .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
        };

        if let Some(target) = candidate {
            if target.id() == root.id() && !root.has_attribute("tabindex") {
                let _ = root.set_attribute("tabindex", "-1");
            }
            let _ = target.focus();
            return;
        }

        if !root.has_attribute("tabindex") {
            let _ = root.set_attribute("tabindex", "-1");
        }
        let _ = root_html.focus();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = content_id;
        let _ = focusable_selector;
    }
}

pub(crate) fn acquire_scroll_lock(lock_id: &str) {
    #[cfg(target_arch = "wasm32")]
    scroll_lock::acquire(lock_id);
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = lock_id;
    }
}

pub(crate) fn release_scroll_lock(lock_id: &str, restore_delay: Option<u64>) {
    #[cfg(target_arch = "wasm32")]
    scroll_lock::release(lock_id, restore_delay);
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = lock_id;
        let _ = restore_delay;
    }
}

pub(crate) fn scroll_element_into_view_nearest(element_id: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(el) = document.get_element_by_id(element_id) {
                    el.scroll_into_view();
                }
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = element_id;
    }
}

pub(crate) fn set_body_user_select_none() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(body) = document.body() {
                    let style = body.style();
                    let _ = style.set_property("user-select", "none");
                    let _ = style.set_property("-webkit-user-select", "none");
                }
            }
        }
    }
}

pub(crate) fn restore_body_user_select() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(body) = document.body() {
                    let style = body.style();
                    let _ = style.remove_property("user-select");
                    let _ = style.remove_property("-webkit-user-select");
                }
            }
        }
    }
}

pub(crate) fn teleport_element_to_host(element_id: &str, host_id: Option<&str>) {
    #[cfg(target_arch = "wasm32")]
    {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(document) = window.document() else {
            return;
        };
        let Some(el) = document.get_element_by_id(element_id) else {
            return;
        };

        let host: Option<web_sys::Node> = match host_id.filter(|id| !id.is_empty()) {
            Some(id) => document.get_element_by_id(id).map(|el| el.into()),
            None => document
                .get_element_by_id("main")
                .map(|el| el.into())
                .or_else(|| document.body().map(|b| b.into())),
        };

        if let Some(host) = host {
            if el.parent_node() != Some(host.clone()) {
                let _ = host.append_child(&el);
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = element_id;
        let _ = host_id;
    }
}

pub(crate) fn remove_element_by_id(element_id: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(el) = document.get_element_by_id(element_id) {
                    el.remove();
                }
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = element_id;
    }
}


pub(crate) fn start_form_reset_monitor(
    element_id: &str,
    form_id: Option<&str>,
    on_reset: impl FnMut() + 'static,
) -> WatcherGuard {
    let mut on_reset = on_reset;
    oxidase::watcher::watch_form_reset(
        element_id,
        form_id,
        move |_sig: oxidase::watcher::FormResetEvent| {
            on_reset();
        },
    )
    .unwrap_or_else(|_| WatcherGuard::noop())
}

#[cfg(test)]
mod tests {
    use super::{
        DocumentDismissEventPayload, FloatingAutoUpdatePayload, FormResetEventPayload,
        PresenceEventPayload, PresenceMonitorFallback,
    };

    #[test]
    fn parses_document_dismiss_payload() {
        let json = r#"{"kind":"pointer_down","pathIds":["content","child"]}"#;
        let payload: DocumentDismissEventPayload = serde_json::from_str(json).unwrap();
        assert_eq!(
            payload,
            DocumentDismissEventPayload::PointerDown {
                path_ids: vec!["content".to_string(), "child".to_string()]
            }
        );

        let json = r#"{"kind":"focus_in","pathIds":["content"]}"#;
        let payload: DocumentDismissEventPayload = serde_json::from_str(json).unwrap();
        assert_eq!(
            payload,
            DocumentDismissEventPayload::FocusIn {
                path_ids: vec!["content".to_string()]
            }
        );

        let json = r#"{"kind":"escape"}"#;
        let payload: DocumentDismissEventPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload, DocumentDismissEventPayload::Escape);
    }

    #[test]
    fn parses_presence_event_payload() {
        let json = r#"{"kind":"fallback","cycleId":6,"reason":"missing"}"#;
        let payload: PresenceEventPayload = serde_json::from_str(json).unwrap();
        assert_eq!(
            payload,
            PresenceEventPayload::Fallback {
                cycle_id: 6,
                reason: PresenceMonitorFallback::Missing,
            }
        );

        let json = r#"{"kind":"animation_end","cycleId":9,"animationName":"fade-out"}"#;
        let payload: PresenceEventPayload = serde_json::from_str(json).unwrap();
        assert_eq!(
            payload,
            PresenceEventPayload::AnimationEnd {
                cycle_id: 9,
                animation_name: "fade-out".to_string(),
            }
        );

        let json = r#"{"kind":"stopped","cycleId":11}"#;
        let payload: PresenceEventPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload, PresenceEventPayload::Stopped { cycle_id: 11 });
    }

    #[test]
    fn parses_floating_auto_update_payload() {
        let json = r#"{"kind":"scroll"}"#;
        let payload: FloatingAutoUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload, FloatingAutoUpdatePayload::Scroll);

        let json = r#"{"kind":"update"}"#;
        let payload: FloatingAutoUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload, FloatingAutoUpdatePayload::Update);
    }

    #[test]
    fn parses_form_reset_payload() {
        let json = r#"{"kind":"reset"}"#;
        let payload: FormResetEventPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload, FormResetEventPayload::Reset);
    }
}
