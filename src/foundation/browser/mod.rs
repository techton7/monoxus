use dioxus::prelude::*;

use crate::foundation::{compose::MountedHandle, overlay::PresenceCloseCycleId};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DocumentDismissEventPayload {
    PointerDown {
        #[serde(rename = "pathIds")]
        path_ids: Vec<String>,
    },
    FocusIn {
        #[serde(rename = "pathIds")]
        path_ids: Vec<String>,
    },
    Escape,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresenceMonitorFallback {
    Missing,
    Hidden,
    NoAnimation,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PresenceEventPayload {
    Fallback {
        #[serde(rename = "cycleId")]
        cycle_id: u64,
        reason: PresenceMonitorFallback,
    },
    AnimationEnd {
        #[serde(rename = "cycleId")]
        cycle_id: u64,
        #[serde(rename = "animationName")]
        animation_name: String,
    },
    AnimationCancel {
        #[serde(rename = "cycleId")]
        cycle_id: u64,
        #[serde(rename = "animationName")]
        animation_name: String,
    },
    Stopped {
        #[serde(rename = "cycleId")]
        cycle_id: u64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FormResetEventPayload {
    Reset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FloatingAutoUpdatePayload {
    Scroll,
    Update,
}

pub type FloatingAutoUpdateEvent = FloatingAutoUpdatePayload;

mod dom_bridge {
    use super::{
        DocumentDismissEventPayload, FloatingAutoUpdatePayload, FormResetEventPayload,
        PresenceEventPayload,
    };
    dioxus_js_bindgen::bind_js!("src/foundation/browser/dom.ts"::*);
}

pub use dioxus_js_bindgen::WatcherGuard;

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
    dom_bridge::watch_presence(
        root_id,
        cycle_id.get() as f64,
        move |payload: PresenceEventPayload| {
            let event = match payload {
                PresenceEventPayload::Fallback { cycle_id, reason } => {
                    PresenceMonitorEvent::Fallback {
                        cycle_id: PresenceCloseCycleId::from_raw(cycle_id),
                        reason,
                    }
                }
                PresenceEventPayload::AnimationEnd {
                    cycle_id,
                    animation_name,
                } => PresenceMonitorEvent::AnimationEnd {
                    cycle_id: PresenceCloseCycleId::from_raw(cycle_id),
                    animation_name,
                },
                PresenceEventPayload::AnimationCancel {
                    cycle_id,
                    animation_name,
                } => PresenceMonitorEvent::AnimationCancel {
                    cycle_id: PresenceCloseCycleId::from_raw(cycle_id),
                    animation_name,
                },
                PresenceEventPayload::Stopped { cycle_id } => PresenceMonitorEvent::Stopped {
                    cycle_id: PresenceCloseCycleId::from_raw(cycle_id),
                },
            };
            on_event(event);
        },
    )
}

pub(crate) fn start_floating_auto_update_monitor(
    anchor_ids: &[&str],
    content_id: &str,
    mut on_event: impl FnMut(FloatingAutoUpdateEvent) + 'static,
) -> WatcherGuard {
    dom_bridge::watch_floating_auto_update(anchor_ids, content_id, move |payload: FloatingAutoUpdatePayload| {
        on_event(payload);
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DocumentDismissEvent {
    PointerDown { path_ids: Vec<String> },
    FocusIn { path_ids: Vec<String> },
    Escape,
}

pub(crate) fn start_document_dismiss_monitor(
    mut on_event: impl FnMut(DocumentDismissEvent) + 'static,
) -> WatcherGuard {
    dom_bridge::watch_document_dismiss(move |payload: DocumentDismissEventPayload| {
        let event = match payload {
            DocumentDismissEventPayload::PointerDown { path_ids } => {
                DocumentDismissEvent::PointerDown { path_ids }
            }
            DocumentDismissEventPayload::FocusIn { path_ids } => {
                DocumentDismissEvent::FocusIn { path_ids }
            }
            DocumentDismissEventPayload::Escape => DocumentDismissEvent::Escape,
        };
        on_event(event);
    })
}

pub(crate) const DEFAULT_FOCUSABLE_SELECTOR: &str = "a[href],button:not([disabled]),input:not([disabled]),select:not([disabled]),textarea:not([disabled]),[tabindex]:not([tabindex='-1'])";

pub(crate) fn focus_mounted_handle(handle: Option<MountedHandle>) -> bool {
    let Some(handle) = handle else {
        return false;
    };

    spawn(async move {
        let _ = handle.set_focus(true).await;
    });
    true
}

pub(crate) async fn get_viewport_size() -> Result<[f64; 2], String> {
    dom_bridge::get_viewport_size().await.map_err(|e| e.to_string())
}

pub(crate) async fn active_element_matches_id(target_id: &str) -> bool {
    dom_bridge::is_element_active(target_id).await.unwrap_or(false)
}

pub(crate) async fn is_reference_hidden(anchor_ids: &[&str]) -> Result<bool, String> {
    let res = dom_bridge::is_reference_hidden(anchor_ids).await.map_err(|e| e.to_string())?;
    Ok(res.unwrap_or(false))
}

pub(crate) fn focus_element_by_id(target_id: &str) {
    focus_element_by_id_with_options(target_id, false);
}

pub(crate) fn restore_focus_element_by_id(target_id: &str) {
    focus_element_by_id_with_options(target_id, true);
}

fn focus_element_by_id_with_options(target_id: &str, prevent_scroll: bool) {
    dom_bridge::focus_element_by_id_with_options(target_id, prevent_scroll);
}

pub(crate) fn focus_first_focusable(content_id: &str, focusable_selector: Option<&str>) {
    let selector = focusable_selector.unwrap_or(DEFAULT_FOCUSABLE_SELECTOR);
    dom_bridge::focus_first_focusable(content_id, selector);
}

pub(crate) fn acquire_scroll_lock(lock_id: &str) {
    dom_bridge::acquire_scroll_lock(lock_id);
}

pub(crate) fn release_scroll_lock(lock_id: &str, restore_delay: Option<u64>) {
    let delay_ms = restore_delay.unwrap_or_default() as f64;
    dom_bridge::release_scroll_lock(lock_id, delay_ms);
}

pub(crate) fn scroll_element_into_view_nearest(element_id: &str) {
    dom_bridge::scroll_element_into_view_nearest(element_id);
}

pub(crate) fn set_body_user_select_none() {
    dom_bridge::set_body_user_select(true);
}

pub(crate) fn restore_body_user_select() {
    dom_bridge::set_body_user_select(false);
}

pub(crate) fn teleport_element_to_host(element_id: &str, host_id: Option<&str>) {
    dom_bridge::teleport_element_to_host(element_id, host_id);
}

pub(crate) fn remove_element_by_id(element_id: &str) {
    dom_bridge::remove_element_by_id(element_id);
}

pub(crate) async fn measure_floating_placement(
    anchor_id: &str,
    content_id: &str,
    custom_anchor_id: Option<&str>,
    boundary_id: Option<&str>,
) -> Option<[f64; 10]> {
    dom_bridge::measure_floating_placement(anchor_id, content_id, custom_anchor_id, boundary_id)
        .await
        .ok()
        .flatten()
}

pub(crate) fn start_form_reset_monitor(
    element_id: &str,
    form_id: Option<&str>,
    on_reset: impl FnMut() + 'static,
) -> WatcherGuard {
    let mut on_reset = on_reset;
    dom_bridge::watch_form_reset(element_id, form_id, move |_sig: FormResetEventPayload| {
        on_reset();
    })
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
