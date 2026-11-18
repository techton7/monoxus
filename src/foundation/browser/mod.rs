use dioxus::prelude::*;

use crate::foundation::{compose::MountedHandle, overlay::PresenceCloseCycleId};

#[allow(unused_imports)]
pub use oxidase::watcher::{
    DismissEvent, DocumentDismissEvent as DocumentDismissEventPayload, FloatingAutoUpdateEvent,
    FloatingAutoUpdatePayload, FormResetEvent, FormResetEventPayload, PresenceEvent,
    PresenceEventPayload, PresenceFallbackReason, PresenceMonitorFallback,
};

pub use crate::foundation::overlay::{Point2D, Rect2D};
pub use oxidase::WatcherGuard;
#[allow(unused_imports)]
pub use oxidase::dom::{FocusOptions, ModalShieldGuard, ModalShieldMode, ScrollLockOptions};

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
    TransitionEnd {
        cycle_id: PresenceCloseCycleId,
        property_name: String,
    },
    TransitionCancel {
        cycle_id: PresenceCloseCycleId,
        property_name: String,
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
    let runtime = dioxus::core::Runtime::try_current();
    oxidase::watcher::watch_presence(
        root_id,
        cycle_id.get(),
        move |payload: oxidase::watcher::PresenceEvent| {
            let _guard = runtime
                .as_ref()
                .map(|rt| dioxus::core::RuntimeGuard::new(rt.clone()));
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
                oxidase::watcher::PresenceEvent::TransitionEnd {
                    cycle_id,
                    property_name,
                } => PresenceMonitorEvent::TransitionEnd {
                    cycle_id: PresenceCloseCycleId::from_raw(cycle_id),
                    property_name,
                },
                oxidase::watcher::PresenceEvent::TransitionCancel {
                    cycle_id,
                    property_name,
                } => PresenceMonitorEvent::TransitionCancel {
                    cycle_id: PresenceCloseCycleId::from_raw(cycle_id),
                    property_name,
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
    let runtime = dioxus::core::Runtime::try_current();
    let scope_id = runtime.as_ref().and_then(|rt| rt.try_current_scope_id());
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
            if let Some(rt) = &runtime {
                let scope = scope_id.unwrap_or(dioxus::core::ScopeId::ROOT);
                rt.in_scope(scope, || on_event(event));
            } else {
                on_event(event);
            }
        },
    )
    .unwrap_or_else(|_| WatcherGuard::noop())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DocumentDismissEvent {
    PointerDown { path_ids: Vec<String> },
    Click { path_ids: Vec<String> },
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
    let runtime = dioxus::core::Runtime::try_current();
    oxidase::watcher::watch_document_dismiss(
        boundaries,
        move |payload: oxidase::watcher::DismissEvent| {
            let _guard = runtime
                .as_ref()
                .map(|rt| dioxus::core::RuntimeGuard::new(rt.clone()));
            let event = match payload {
                oxidase::watcher::DismissEvent::PointerDown { path_ids } => {
                    DocumentDismissEvent::PointerDown { path_ids }
                }
                oxidase::watcher::DismissEvent::Click { path_ids } => {
                    DocumentDismissEvent::Click { path_ids }
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

pub fn spawn_root_task<F: std::future::Future<Output = ()> + 'static>(
    fut: F,
) -> dioxus::core::Task {
    dioxus::core::spawn_forever(fut)
}

pub(crate) fn focus_mounted_handle(handle: Option<MountedHandle>) -> bool {
    focus_mounted_handle_with_options(handle, FocusOptions::new(true))
}

#[allow(dead_code)]
pub(crate) fn focus_mounted_handle_with_options(
    handle: Option<MountedHandle>,
    options: FocusOptions,
) -> bool {
    let Some(handle) = handle else {
        return false;
    };
    let el = oxidase::Element::from(handle);
    spawn_root_task(async move {
        let _ = el.focus_with_options(options).await;
    });
    true
}

pub(crate) async fn get_viewport_size() -> Result<[f64; 2], String> {
    let (width, height) = oxidase::window()
        .inner_size()
        .await
        .map_err(|e| e.to_string())?;
    if width > 0.0 && height > 0.0 {
        Ok([width, height])
    } else {
        Err("Viewport dimensions are unavailable or non-positive".to_string())
    }
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
    focus_element_by_id_with_options(target_id, FocusOptions::new(true));
}

pub(crate) fn focus_element_by_id_with_options(target_id: &str, options: FocusOptions) {
    let tid = target_id.to_string();
    spawn_root_task(async move {
        for _ in 0..5 {
            let _ = oxidase::frame::next_frame().await;
            let doc = oxidase::dom::Document::current().unwrap_or_else(oxidase::document);
            if let Some(el) = doc.element_by_id(&tid)
                && el.focus_with_options(options).await.is_ok()
            {
                break;
            }
        }
    });
}

pub(crate) fn restore_focus_element_by_id(target_id: &str) {
    restore_focus_element_by_id_with_options(target_id, FocusOptions::new(true));
}

pub(crate) fn restore_focus_element_by_id_with_options(target_id: &str, options: FocusOptions) {
    let tid = target_id.to_string();
    spawn_root_task(async move {
        for _ in 0..5 {
            let _ = oxidase::frame::next_frame().await;
            let doc = oxidase::dom::Document::current().unwrap_or_else(oxidase::document);
            if let Some(el) = doc.element_by_id(&tid)
                && el.focus_with_options(options).await.is_ok()
            {
                break;
            }
        }
    });
}

pub(crate) fn is_element_focus_visible_by_id(target_id: &str) -> Option<bool> {
    let doc = oxidase::dom::Document::current()?;
    let el = doc.element_by_id(target_id)?;
    Some(el.matches_focus_visible())
}

pub(crate) fn is_element_focused_by_id(target_id: &str) -> bool {
    let Some(doc) = oxidase::dom::Document::current() else {
        return false;
    };
    let Some(el) = doc.element_by_id(target_id) else {
        return false;
    };
    el.has_focus()
}

pub(crate) fn focus_first_focusable(content_id: &str, focusable_selector: Option<&str>) {
    focus_first_focusable_with_options(content_id, focusable_selector, FocusOptions::new(true));
}

pub(crate) fn focus_first_focusable_with_options(
    content_id: &str,
    focusable_selector: Option<&str>,
    options: FocusOptions,
) {
    let doc = oxidase::dom::Document::current().unwrap_or_else(oxidase::dom::document);
    let selector = focusable_selector.unwrap_or(DEFAULT_FOCUSABLE_SELECTOR);

    if let Some(root) = doc.element_by_id(content_id) {
        let target = if root.matches(selector) {
            root.clone()
        } else {
            root.query_selector(selector).unwrap_or_else(|| root.clone())
        };
        if target.id() == root.id() && !root.has_attribute("tabindex") {
            root.set_attribute("tabindex", "-1");
        }
        let _ = target.focus_sync_with_options(options);
        return;
    }

    let selector = selector.to_string();
    let cid = content_id.to_string();
    spawn_root_task(async move {
        let doc = oxidase::dom::Document::current().unwrap_or_else(oxidase::dom::document);
        if let Some(root) = doc.element_by_id(&cid) {
            let target = if root.matches(&selector) {
                root.clone()
            } else {
                root.query_selector(&selector).unwrap_or_else(|| root.clone())
            };
            if target.id() == root.id() && !root.has_attribute("tabindex") {
                root.set_attribute("tabindex", "-1");
            }
            let _ = target.focus_with_options(options).await;
        }
    });
}

pub(crate) fn suppress_external_dom(keep_ids: &[&str]) -> ModalShieldGuard {
    oxidase::dom::suppress_others(keep_ids)
}

#[allow(dead_code)]
pub(crate) fn acquire_scroll_lock(lock_id: &str) {
    oxidase::dom::acquire_scroll_lock(lock_id);
}

pub(crate) fn acquire_scroll_lock_with_options(
    lock_id: &str,
    prevent_body_pointer_events: bool,
) {
    oxidase::dom::acquire_scroll_lock_with_options(
        lock_id,
        ScrollLockOptions {
            prevent_body_pointer_events,
        },
    );
}

pub(crate) fn is_body_pointer_events_locked() -> bool {
    oxidase::dom::is_body_pointer_events_locked()
}

pub(crate) fn release_scroll_lock(lock_id: &str, restore_delay: Option<u64>) {
    oxidase::dom::release_scroll_lock(lock_id, restore_delay);
}

#[allow(dead_code)]
pub(crate) fn scroll_element_into_view_nearest(element_id: &str) {
    let doc = oxidase::dom::document();
    if let Some(el) = doc.element_by_id(element_id) {
        let _ = el.scroll_into_view_sync_with_options(oxidase::dom::ScrollIntoViewOptions {
            behavior: oxidase::dom::ScrollBehavior::Auto,
            block: oxidase::dom::ScrollLogicalPosition::Nearest,
            inline: oxidase::dom::ScrollLogicalPosition::Nearest,
        });
    }
}

pub(crate) fn set_body_user_select_none() {
    let doc = oxidase::dom::document();
    doc.set_body_user_select_none();
}

pub(crate) fn restore_body_user_select() {
    let doc = oxidase::dom::document();
    doc.restore_body_user_select();
}

pub(crate) fn teleport_element_to_host(element_id: &str, host_id: Option<&str>) {
    let doc = oxidase::dom::document();
    doc.teleport_portal_to_host(element_id, host_id);
}

pub(crate) fn remove_element_by_id(element_id: &str) {
    let doc = oxidase::dom::document();
    doc.remove_portal_element_by_id(element_id);
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

pub struct DocumentPointerStreamGuard {
    guard: Option<WatcherGuard>,
}

impl DocumentPointerStreamGuard {
    pub const fn noop() -> Self {
        Self { guard: None }
    }

    #[allow(dead_code)]
    pub fn stop(&mut self) {
        if let Some(guard) = self.guard.take() {
            guard.stop();
        }
    }
}

impl Default for DocumentPointerStreamGuard {
    fn default() -> Self {
        Self::noop()
    }
}

impl std::fmt::Debug for DocumentPointerStreamGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DocumentPointerStreamGuard").finish()
    }
}

pub fn start_document_pointer_stream(
    mut on_pointer_move: impl FnMut(Point2D) + 'static,
) -> DocumentPointerStreamGuard {
    let runtime = dioxus::core::Runtime::try_current();
    let scope_id = runtime.as_ref().and_then(|rt| rt.try_current_scope_id());
    match oxidase::dom::observer::observe_pointer_move(move |x, y| {
        let point = Point2D::new(x, y);
        if let Some(rt) = &runtime {
            let scope = scope_id.unwrap_or(dioxus::core::ScopeId::ROOT);
            rt.in_scope(scope, || on_pointer_move(point));
        } else {
            on_pointer_move(point);
        }
    }) {
        Ok(guard) => DocumentPointerStreamGuard { guard: Some(guard) },
        Err(_) => DocumentPointerStreamGuard::noop(),
    }
}

pub fn get_element_client_rect(id: &str) -> Option<Rect2D> {
    let doc = oxidase::dom::document();
    doc.measure_rect(id)
        .map(|rect| Rect2D::new(rect.x, rect.y, rect.width, rect.height))
}

pub fn query_descendant_order(
    root_id: &str,
    candidate_ids: &[&str],
) -> Result<Vec<String>, oxidase::error::HostError> {
    oxidase::dom::document().query_descendant_order(root_id, candidate_ids)
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

        let json = r#"{"kind":"click","pathIds":["outside-btn"]}"#;
        let payload: DocumentDismissEventPayload = serde_json::from_str(json).unwrap();
        assert_eq!(
            payload,
            DocumentDismissEventPayload::Click {
                path_ids: vec!["outside-btn".to_string()]
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

    #[test]
    fn test_document_pointer_stream_and_client_rect_noop() {
        use super::{
            DocumentPointerStreamGuard, get_element_client_rect, start_document_pointer_stream,
        };

        let mut guard = DocumentPointerStreamGuard::noop();
        guard.stop();

        let mut stream_guard = start_document_pointer_stream(|_pt| {});
        stream_guard.stop();

        assert_eq!(get_element_client_rect("nonexistent"), None);
    }
}
