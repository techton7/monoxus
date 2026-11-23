use std::cell::RefCell;
use std::rc::Rc;

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
pub use oxidase::dom::{
    FocusOptions, ModalShieldGuard, ModalShieldMode, ResizeObserverBox, ScrollLockOptions,
};

#[allow(unused_imports)]
pub use monoxus_core::foundation::shared::{ElementScrollMetrics, ElementSize};

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

/// Starts an event-driven presence monitor; resolved `Missing`, `Hidden`, and
/// `NoAnimation` states complete immediately without a wall-clock fallback.
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
            root.query_selector(selector)
                .unwrap_or_else(|| root.clone())
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
                root.query_selector(&selector)
                    .unwrap_or_else(|| root.clone())
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

pub(crate) fn acquire_scroll_lock_with_options(lock_id: &str, prevent_body_pointer_events: bool) {
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
    let runtime = dioxus::core::Runtime::try_current();
    let scope_id = runtime.as_ref().and_then(|rt| rt.try_current_scope_id());
    let on_reset = Rc::new(RefCell::new(on_reset));
    oxidase::watcher::watch_form_reset(
        element_id,
        form_id,
        move |_sig: oxidase::watcher::FormResetEvent| {
            let reset = Rc::clone(&on_reset);
            if let Some(rt) = &runtime {
                let async_runtime = rt.clone();
                let scope = scope_id.unwrap_or(dioxus::core::ScopeId::ROOT);
                rt.in_scope(scope, || {
                    spawn_root_task(async move {
                        let _ = oxidase::frame::next_frame().await;
                        async_runtime.in_scope(scope, || {
                            if let Ok(mut reset) = reset.try_borrow_mut() {
                                reset();
                            }
                        });
                    });
                });
            } else {
                spawn_root_task(async move {
                    let _ = oxidase::frame::next_frame().await;
                    if let Ok(mut reset) = reset.try_borrow_mut() {
                        reset();
                    }
                });
            }
        },
    )
    .unwrap_or_else(|_| WatcherGuard::noop())
}

pub(crate) fn probe_element_form_ancestor(element_id: &str) -> Option<bool> {
    let doc = oxidase::dom::Document::current()?;
    if !doc.is_authoritative() {
        return None;
    }
    let el = doc.element_by_id(element_id)?;
    Some(el.closest_tag("form").is_some())
}

pub(crate) fn focus_element_sync_or_defer(target_id: &str, options: FocusOptions) {
    if let Some(doc) = oxidase::dom::Document::current()
        && doc.set_focus_with_options(target_id, options).is_ok()
    {
        return;
    }
    focus_element_by_id_with_options(target_id, options);
}

pub fn collect_form_data(
    form_element_id: &str,
) -> Result<Vec<(String, String)>, oxidase::error::HostError> {
    let Some(doc) = oxidase::dom::Document::current() else {
        return Err(oxidase::error::HostError::Unsupported(
            "No active document".into(),
        ));
    };
    doc.collect_form_data(form_element_id)
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

/// Measures the unscaled natural border-box size of an element without animation or transition constraints.
#[allow(dead_code)]
pub(crate) fn measure_element_without_animation(element_id: &str) -> Option<ElementSize> {
    let doc = oxidase::dom::Document::current()?;
    doc.measure_without_animation(element_id)
        .map(|(width, height)| ElementSize { width, height })
}

/// Returns `true` if the target element currently has a running or pending CSS animation or transition.
#[allow(dead_code)]
pub(crate) fn is_element_animating(element_id: &str) -> bool {
    oxidase::dom::Document::current().is_some_and(|doc| doc.is_element_animating(element_id))
}

/// Retrieves the current CSSOM scroll metrics (scroll offset, scroll size, client size) of an element synchronously.
#[allow(dead_code)]
pub(crate) fn get_element_scroll_metrics(element_id: &str) -> Option<ElementScrollMetrics> {
    let doc = oxidase::dom::Document::current()?;
    let el = doc.element_by_id(element_id)?;
    let scroll_left = el.scroll_x_sync().ok()?;
    let scroll_top = el.scroll_y_sync().ok()?;
    let scroll_width = el.scroll_width_sync().ok()?;
    let scroll_height = el.scroll_height_sync().ok()?;
    let client_width = el.client_width_sync().ok()?;
    let client_height = el.client_height_sync().ok()?;
    Some(ElementScrollMetrics {
        scroll_left,
        scroll_top,
        scroll_width,
        scroll_height,
        client_width,
        client_height,
    })
}

/// Sets the scroll position of an element synchronously.
#[allow(dead_code)]
pub(crate) fn set_element_scroll_position(element_id: &str, x: f64, y: f64) {
    if let Some(doc) = oxidase::dom::Document::current() {
        if let Some(el) = doc.element_by_id(element_id) {
            let _ = el.scroll_to_sync(x, y);
        }
    }
}

/// Sets an inline style property on an element by ID.
#[allow(dead_code)]
pub(crate) fn set_element_style_property(element_id: &str, property: &str, value: &str) {
    if let Some(doc) = oxidase::dom::Document::current() {
        if let Some(el) = doc.element_by_id(element_id) {
            let _ = el.set_style_property(property, value);
        }
    }
}

/// Removes an inline style property from an element by ID.
#[allow(dead_code)]
pub(crate) fn remove_element_style_property(element_id: &str, property: &str) {
    if let Some(doc) = oxidase::dom::Document::current() {
        if let Some(el) = doc.element_by_id(element_id) {
            let _ = el.remove_style_property(property);
        }
    }
}

/// Starts a scroll monitor on a specific element container, executing `on_scroll` within the Dioxus runtime context.
#[allow(dead_code)]
pub(crate) fn start_element_scroll_monitor(
    element_id: &str,
    mut on_scroll: impl FnMut() + 'static,
) -> WatcherGuard {
    let runtime = dioxus::core::Runtime::try_current();
    let scope_id = runtime.as_ref().and_then(|rt| rt.try_current_scope_id());
    oxidase::dom::observer::observe_element_scroll(element_id, move || {
        if let Some(rt) = &runtime {
            let scope = scope_id.unwrap_or(dioxus::core::ScopeId::ROOT);
            rt.in_scope(scope, || on_scroll());
        } else {
            on_scroll();
        }
    })
    .unwrap_or_else(|_| WatcherGuard::noop())
}

/// Starts an element resize monitor with the specified box model, executing `on_resize` within the Dioxus runtime context.
#[allow(dead_code)]
pub(crate) fn start_resize_monitor(
    element_id: &str,
    box_model: ResizeObserverBox,
    mut on_resize: impl FnMut(ElementSize) + 'static,
) -> WatcherGuard {
    let runtime = dioxus::core::Runtime::try_current();
    let scope_id = runtime.as_ref().and_then(|rt| rt.try_current_scope_id());
    oxidase::dom::observer::observe_element_resize_with_box(
        element_id,
        box_model,
        move |entry: oxidase::dom::observer::ResizeEntry| {
            let size = ElementSize {
                width: entry.content_rect.width,
                height: entry.content_rect.height,
            };
            if let Some(rt) = &runtime {
                let scope = scope_id.unwrap_or(dioxus::core::ScopeId::ROOT);
                rt.in_scope(scope, || on_resize(size));
            } else {
                on_resize(size);
            }
        },
    )
    .unwrap_or_else(|_| WatcherGuard::noop())
}

/// Reactive hook measuring an element's border-box dimensions and updating on resize with clean `use_drop` teardown.
#[allow(dead_code)]
pub(crate) fn use_element_size(element_id: &str) -> Signal<Option<ElementSize>> {
    use_element_size_with_box(element_id, ResizeObserverBox::BorderBox)
}

/// Reactive hook measuring an element's dimensions using the specified box model with clean `use_drop` teardown.
#[allow(dead_code)]
pub(crate) fn use_element_size_with_box(
    element_id: &str,
    box_model: ResizeObserverBox,
) -> Signal<Option<ElementSize>> {
    let mount_rev = use_signal(|| 0u64);
    use_element_size_with_mount(element_id, box_model, mount_rev)
}

/// Reactive hook measuring an element's dimensions using the specified box model and mount revision
/// with clean `use_drop` teardown.
#[allow(dead_code)]
pub(crate) fn use_element_size_with_mount(
    element_id: &str,
    box_model: ResizeObserverBox,
    mount_rev: Signal<u64>,
) -> Signal<Option<ElementSize>> {
    let element_id_str = element_id.to_string();
    let initial_size = measure_element_without_animation(&element_id_str)
        .or_else(|| {
            get_element_client_rect(&element_id_str).map(|r| ElementSize {
                width: r.width,
                height: r.height,
            })
        })
        .filter(|s| s.width > 0.0 || s.height > 0.0);
    let mut size_signal = use_signal(|| initial_size);
    let guard_cell = use_hook(|| Rc::new(RefCell::new(WatcherGuard::noop())));

    use_effect(use_reactive((&element_id_str,), {
        let guard_cell = Rc::clone(&guard_cell);
        move |(id,)| {
            let _ = mount_rev();
            if let Some(measured) = measure_element_without_animation(&id)
                .or_else(|| {
                    get_element_client_rect(&id).map(|r| ElementSize {
                        width: r.width,
                        height: r.height,
                    })
                })
                .filter(|s| s.width > 0.0 || s.height > 0.0)
                && size_signal.peek().as_ref() != Some(&measured)
            {
                size_signal.set(Some(measured));
            }
            let next_guard = start_resize_monitor(&id, box_model, move |new_size| {
                if new_size.width <= 0.0 && new_size.height <= 0.0 {
                    return;
                }
                if size_signal.peek().as_ref() != Some(&new_size) {
                    size_signal.set(Some(new_size));
                }
            });
            let prev_guard = std::mem::replace(&mut *guard_cell.borrow_mut(), next_guard);
            prev_guard.stop();
        }
    }));

    use_drop({
        let guard_cell = Rc::clone(&guard_cell);
        move || {
            let prev_guard = std::mem::replace(&mut *guard_cell.borrow_mut(), WatcherGuard::noop());
            prev_guard.stop();
        }
    });

    size_signal
}

/// Returns `true` when the active host document currently holds a non-empty text selection.
#[allow(dead_code)]
pub(crate) fn document_has_text_selection() -> bool {
    oxidase::dom::Document::current()
        .map(|doc| doc.has_text_selection())
        .unwrap_or(false)
}

/// Starts a passive document-wide `pointerup` observer returning an RAII [`WatcherGuard`].
#[allow(dead_code)]
pub(crate) fn start_document_pointer_up_observer(
    mut on_pointer_up: impl FnMut() + 'static,
) -> WatcherGuard {
    let runtime = dioxus::core::Runtime::try_current();
    oxidase::dom::observe_pointer_up(move || {
        let _guard = runtime
            .as_ref()
            .map(|rt| dioxus::core::RuntimeGuard::new(rt.clone()));
        on_pointer_up();
    })
    .unwrap_or_else(|_| WatcherGuard::noop())
}

/// Queries all matching descendant elements inside `element_id`.
#[allow(dead_code)]
pub(crate) fn query_selector_all_by_id(
    element_id: &str,
    selector: &str,
) -> Vec<oxidase::dom::Element> {
    let Some(el) = oxidase::dom::Document::current().and_then(|doc| doc.element_by_id(element_id)) else {
        return Vec::new();
    };
    el.query_selector_all(selector)
}

/// Dispatches a synchronous DOM click on `element_id`. Returns `true` when dispatched.
#[allow(dead_code)]
pub(crate) fn click_element_by_id(element_id: &str) -> bool {
    let Some(el) = oxidase::dom::Document::current().and_then(|doc| doc.element_by_id(element_id)) else {
        return false;
    };
    el.click_sync().is_ok()
}

/// Reads the current `(selection_start, selection_end)` UTF-16 range of the `<input>` identified by `element_id`.
#[allow(dead_code)]
pub(crate) fn get_input_selection_range(element_id: &str) -> Option<(usize, usize)> {
    let el = oxidase::dom::Document::current()?.element_by_id(element_id)?;
    el.selection_range()
}

/// Sets the `(start, end)` UTF-16 selection range on the `<input>` identified by `element_id`.
#[allow(dead_code)]
pub(crate) fn set_input_selection_range(element_id: &str, start: usize, end: usize) -> bool {
    let Some(el) = oxidase::dom::Document::current().and_then(|doc| doc.element_by_id(element_id)) else {
        return false;
    };
    el.set_selection_range(start, end).is_ok()
}

/// Selects the full text contents of the `<input>` identified by `element_id`.
#[allow(dead_code)]
pub(crate) fn select_input_by_id(element_id: &str) -> bool {
    let Some(el) = oxidase::dom::Document::current().and_then(|doc| doc.element_by_id(element_id)) else {
        return false;
    };
    el.select().is_ok()
}

/// Synchronously sets the value of the `<input>` identified by `element_id`.
#[allow(dead_code)]
pub(crate) fn set_input_value_by_id(element_id: &str, value: &str) -> bool {
    let Some(el) = oxidase::dom::Document::current().and_then(|doc| doc.element_by_id(element_id)) else {
        return false;
    };
    el.set_input_value(value).is_ok()
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

    #[test]
    fn test_element_metrics_and_monitors_noop() {
        use super::{
            ElementScrollMetrics, ElementSize, ResizeObserverBox, get_element_scroll_metrics,
            measure_element_without_animation, remove_element_style_property,
            set_element_scroll_position, set_element_style_property, start_element_scroll_monitor,
            start_resize_monitor,
        };

        assert_eq!(measure_element_without_animation("nonexistent"), None);
        assert_eq!(get_element_scroll_metrics("nonexistent"), None);
        set_element_scroll_position("nonexistent", 10.0, 20.0);
        set_element_style_property("nonexistent", "opacity", "1");
        remove_element_style_property("nonexistent", "opacity");

        let scroll_guard = start_element_scroll_monitor("nonexistent", || {});
        scroll_guard.stop();

        let resize_guard =
            start_resize_monitor("nonexistent", ResizeObserverBox::BorderBox, |_| {});
        resize_guard.stop();

        let size = ElementSize {
            width: 100.0,
            height: 50.0,
        };
        assert_eq!(size.width, 100.0);
        assert_eq!(size.height, 50.0);

        let metrics = ElementScrollMetrics::default();
        assert_eq!(metrics.scroll_left, 0.0);
        assert_eq!(metrics.scroll_top, 0.0);
    }
}
