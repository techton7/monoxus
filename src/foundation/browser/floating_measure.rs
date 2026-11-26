use monoxus_core::foundation::shared::ElementSize;
use oxidase::WatcherGuard;
use oxidase::watcher::{DismissOptions, FloatingAutoUpdateEvent, PresenceMonitorFallback};

use crate::foundation::overlay::{PresenceCloseCycleId, Rect, Rect2D};

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
    on_event: impl FnMut(DocumentDismissEvent) + 'static,
) -> WatcherGuard {
    start_document_dismiss_monitor_with_options(DismissOptions::new(boundaries), on_event)
}

pub(crate) fn start_document_dismiss_monitor_with_options(
    options: DismissOptions,
    mut on_event: impl FnMut(DocumentDismissEvent) + 'static,
) -> WatcherGuard {
    let runtime = dioxus::core::Runtime::try_current();
    oxidase::watcher::watch_document_dismiss_with_options(
        options,
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

pub(crate) async fn is_reference_hidden(anchor_ids: &[&str]) -> Result<bool, String> {
    oxidase::runtime::is_reference_hidden(anchor_ids)
        .await
        .map_err(|e| e.to_string())
}

pub fn get_element_client_rect(id: &str) -> Option<Rect2D> {
    let doc = oxidase::dom::document();
    doc.measure_rect(id)
        .map(|rect| Rect2D::new(rect.x, rect.y, rect.width, rect.height))
}

/// Returns the CSSOM computed `z-index` string (`"auto"` or integer string) for `element_id`.
pub fn get_element_computed_z_index(element_id: &str) -> Option<String> {
    oxidase::dom::document().computed_z_index_by_id(element_id)
}

/// Returns the CSSOM computed `z-index` string (`"auto"` or integer string) for `element`.
#[allow(dead_code)]
pub(crate) fn computed_z_index_for_element(element: &oxidase::dom::Element) -> Option<String> {
    element.computed_z_index()
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

/// Measures the CSSOM `offsetLeft`/`offsetTop`/`offsetWidth`/`offsetHeight` rect of `id`
/// relative to its `offsetParent` without CSS transform adjustments.
#[allow(dead_code)]
pub fn get_element_offset_rect(id: &str) -> Option<Rect> {
    let doc = oxidase::dom::Document::current()?;
    let el = doc.element_by_id(id)?;
    let rect = el.offset_rect()?;
    Some(Rect::new(
        rect.x as f32,
        rect.y as f32,
        rect.width as f32,
        rect.height as f32,
    ))
}
