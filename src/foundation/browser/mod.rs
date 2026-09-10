use dioxus::{document, document::Eval, prelude::*};

use crate::foundation::{compose::MountedHandle, overlay::PresenceCloseCycleId};

mod dom_bridge {
    dioxus_js_bindgen::bind_js!("src/foundation/browser/dom.ts"::*);
}

#[allow(dead_code)]
const PRESENCE_MONITOR_SIGNAL_STOP: &str = "stop";
#[allow(dead_code)]
const PRESENCE_MONITOR_SIGNAL_STOPPED: &str = "stopped";
#[allow(dead_code)]
const PRESENCE_MONITOR_SIGNAL_FALLBACK: &str = "fallback";
#[allow(dead_code)]
const PRESENCE_MONITOR_SIGNAL_ANIMATION_END: &str = "animationend";
#[allow(dead_code)]
const PRESENCE_MONITOR_SIGNAL_ANIMATION_CANCEL: &str = "animationcancel";
#[allow(dead_code)]
const PRESENCE_MONITOR_REASON_MISSING: &str = "missing";
#[allow(dead_code)]
const PRESENCE_MONITOR_REASON_HIDDEN: &str = "hidden";
#[allow(dead_code)]
const PRESENCE_MONITOR_REASON_NO_ANIMATION: &str = "no-animation";
const FLOATING_AUTO_UPDATE_SIGNAL_STOP: &str = "stop";
const FLOATING_AUTO_UPDATE_SIGNAL_STOPPED: &str = "stopped";
const FLOATING_AUTO_UPDATE_SIGNAL_SCROLL: &str = "scroll";
const FLOATING_AUTO_UPDATE_SIGNAL_UPDATE: &str = "update";
const DOCUMENT_DISMISS_SIGNAL_STOP: &str = "stop";
const DOCUMENT_DISMISS_SIGNAL_STOPPED: &str = "stopped";
const DOCUMENT_DISMISS_SIGNAL_POINTER_DOWN: &str = "pointerdown";
const DOCUMENT_DISMISS_SIGNAL_FOCUS_IN: &str = "focusin";
const DOCUMENT_DISMISS_SIGNAL_ESCAPE: &str = "escape";
const DOCUMENT_DISMISS_PATH_SEPARATOR: char = '\u{1f}';

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PresenceMonitorFallback {
    Missing,
    Hidden,
    NoAnimation,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
pub(crate) fn start_presence_monitor(root_id: &str, cycle_id: PresenceCloseCycleId) -> Eval {
    let eval = document::eval(include_str!("presence_monitor.js"));
    let _ = eval.send((root_id.to_string(), cycle_id.get()));
    eval
}

#[allow(dead_code)]
pub(crate) fn stop_presence_monitor(monitor: Eval) -> Result<(), String> {
    monitor
        .send(PRESENCE_MONITOR_SIGNAL_STOP)
        .map_err(|error| format!("presence monitor stop failed: {error}"))
}

#[allow(dead_code)]
pub(crate) async fn recv_presence_monitor_event(
    monitor: &mut Eval,
) -> Result<PresenceMonitorEvent, String> {
    let payload = monitor
        .recv::<String>()
        .await
        .map_err(|error| format!("presence monitor receive failed: {error}"))?;
    parse_presence_monitor_event(&payload)
}

#[allow(dead_code)]
fn parse_presence_monitor_event(payload: &str) -> Result<PresenceMonitorEvent, String> {
    let mut fields = payload.splitn(3, '\n');
    let signal = fields.next().unwrap_or_default();
    let cycle_id = fields
        .next()
        .ok_or_else(|| String::from("presence monitor payload missing cycle id"))
        .and_then(parse_presence_monitor_cycle_id)?;
    let detail = fields.next().unwrap_or_default();

    match signal {
        PRESENCE_MONITOR_SIGNAL_FALLBACK => Ok(PresenceMonitorEvent::Fallback {
            cycle_id,
            reason: parse_presence_monitor_fallback(detail)?,
        }),
        PRESENCE_MONITOR_SIGNAL_ANIMATION_END => {
            if detail.is_empty() {
                return Err(String::from(
                    "presence monitor animationend payload missing animation name",
                ));
            }

            Ok(PresenceMonitorEvent::AnimationEnd {
                cycle_id,
                animation_name: detail.to_string(),
            })
        }
        PRESENCE_MONITOR_SIGNAL_ANIMATION_CANCEL => {
            if detail.is_empty() {
                return Err(String::from(
                    "presence monitor animationcancel payload missing animation name",
                ));
            }

            Ok(PresenceMonitorEvent::AnimationCancel {
                cycle_id,
                animation_name: detail.to_string(),
            })
        }
        PRESENCE_MONITOR_SIGNAL_STOPPED => Ok(PresenceMonitorEvent::Stopped { cycle_id }),
        other => Err(format!("unknown presence monitor signal: {other}")),
    }
}

#[allow(dead_code)]
fn parse_presence_monitor_cycle_id(value: &str) -> Result<PresenceCloseCycleId, String> {
    let cycle_id = value
        .parse::<u64>()
        .map_err(|error| format!("invalid presence monitor cycle id: {error}"))?;
    Ok(PresenceCloseCycleId::from_raw(cycle_id))
}

#[allow(dead_code)]
fn parse_presence_monitor_fallback(value: &str) -> Result<PresenceMonitorFallback, String> {
    match value {
        PRESENCE_MONITOR_REASON_MISSING => Ok(PresenceMonitorFallback::Missing),
        PRESENCE_MONITOR_REASON_HIDDEN => Ok(PresenceMonitorFallback::Hidden),
        PRESENCE_MONITOR_REASON_NO_ANIMATION => Ok(PresenceMonitorFallback::NoAnimation),
        other => Err(format!("unknown presence monitor fallback reason: {other}")),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FloatingAutoUpdateEvent {
    Scroll,
    Update,
    Stopped,
}

pub(crate) fn start_floating_auto_update_monitor(anchor_ids: &[&str], content_id: &str) -> Eval {
    let eval = document::eval(include_str!("floating_auto_update.js"));
    let ids: Vec<String> = anchor_ids.iter().map(|id| id.to_string()).collect();
    let _ = eval.send((ids, content_id.to_string()));
    eval
}

pub(crate) fn stop_floating_auto_update_monitor(monitor: Eval) -> Result<(), String> {
    monitor
        .send(FLOATING_AUTO_UPDATE_SIGNAL_STOP)
        .map_err(|error| format!("floating auto-update stop failed: {error}"))
}

pub(crate) async fn recv_floating_auto_update_event(
    monitor: &mut Eval,
) -> Result<FloatingAutoUpdateEvent, String> {
    let signal: String = monitor
        .recv()
        .await
        .map_err(|error| format!("floating auto-update receive failed: {error}"))?;

    match signal.as_str() {
        FLOATING_AUTO_UPDATE_SIGNAL_SCROLL => Ok(FloatingAutoUpdateEvent::Scroll),
        FLOATING_AUTO_UPDATE_SIGNAL_UPDATE => Ok(FloatingAutoUpdateEvent::Update),
        FLOATING_AUTO_UPDATE_SIGNAL_STOPPED => Ok(FloatingAutoUpdateEvent::Stopped),
        other => Err(format!("unknown floating auto-update signal: {other}")),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DocumentDismissEvent {
    PointerDown { path_ids: Vec<String> },
    FocusIn { path_ids: Vec<String> },
    Escape,
    Stopped,
}

pub(crate) fn start_document_dismiss_monitor() -> Eval {
    document::eval(include_str!("document_dismiss.js"))
}

pub(crate) fn stop_document_dismiss_monitor(monitor: Eval) -> Result<(), String> {
    monitor
        .send(DOCUMENT_DISMISS_SIGNAL_STOP)
        .map_err(|error| format!("document dismiss stop failed: {error}"))
}

pub(crate) async fn recv_document_dismiss_event(
    monitor: &mut Eval,
) -> Result<DocumentDismissEvent, String> {
    let payload: String = monitor
        .recv()
        .await
        .map_err(|error| format!("document dismiss receive failed: {error}"))?;

    parse_document_dismiss_event(&payload)
}

fn parse_document_dismiss_event(payload: &str) -> Result<DocumentDismissEvent, String> {
    let mut fields = payload.splitn(2, '\n');
    let signal = fields.next().unwrap_or_default();
    let path_ids = parse_document_dismiss_path_ids(fields.next());

    match signal {
        DOCUMENT_DISMISS_SIGNAL_POINTER_DOWN => Ok(DocumentDismissEvent::PointerDown { path_ids }),
        DOCUMENT_DISMISS_SIGNAL_FOCUS_IN => Ok(DocumentDismissEvent::FocusIn { path_ids }),
        DOCUMENT_DISMISS_SIGNAL_ESCAPE => Ok(DocumentDismissEvent::Escape),
        DOCUMENT_DISMISS_SIGNAL_STOPPED => Ok(DocumentDismissEvent::Stopped),
        other => Err(format!("unknown document dismiss signal: {other}")),
    }
}

fn parse_document_dismiss_path_ids(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or_default()
        .split(DOCUMENT_DISMISS_PATH_SEPARATOR)
        .filter(|value| !value.is_empty())
        .map(String::from)
        .collect()
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

pub(crate) use dom_bridge::WatchFormResetWatcher;

pub(crate) fn start_form_reset_monitor(
    element_id: &str,
    form_id: Option<&str>,
    on_reset: impl FnMut() + 'static,
) -> WatchFormResetWatcher {
    let mut on_reset = on_reset;
    WatchFormResetWatcher::start(element_id, form_id, move |_sig| {
        on_reset();
    })
}

#[cfg(test)]
mod tests {
    use super::{
        DOCUMENT_DISMISS_SIGNAL_ESCAPE, DOCUMENT_DISMISS_SIGNAL_FOCUS_IN,
        DOCUMENT_DISMISS_SIGNAL_POINTER_DOWN, DocumentDismissEvent, PRESENCE_MONITOR_REASON_HIDDEN,
        PRESENCE_MONITOR_REASON_MISSING, PRESENCE_MONITOR_REASON_NO_ANIMATION,
        PRESENCE_MONITOR_SIGNAL_ANIMATION_CANCEL, PRESENCE_MONITOR_SIGNAL_ANIMATION_END,
        PRESENCE_MONITOR_SIGNAL_FALLBACK, PRESENCE_MONITOR_SIGNAL_STOPPED, PresenceMonitorEvent,
        PresenceMonitorFallback, parse_document_dismiss_event, parse_presence_monitor_event,
    };
    use crate::foundation::overlay::PresenceCloseCycleId;

    #[test]
    fn parses_document_dismiss_events() {
        assert_eq!(
            parse_document_dismiss_event(&format!(
                "{DOCUMENT_DISMISS_SIGNAL_POINTER_DOWN}\ncontent\u{1f}child"
            )),
            Ok(DocumentDismissEvent::PointerDown {
                path_ids: vec![String::from("content"), String::from("child")],
            }),
        );
        assert_eq!(
            parse_document_dismiss_event(&format!("{DOCUMENT_DISMISS_SIGNAL_FOCUS_IN}\ncontent")),
            Ok(DocumentDismissEvent::FocusIn {
                path_ids: vec![String::from("content")],
            }),
        );
        assert_eq!(
            parse_document_dismiss_event(&format!("{DOCUMENT_DISMISS_SIGNAL_ESCAPE}\n")),
            Ok(DocumentDismissEvent::Escape),
        );
    }

    #[test]
    fn parses_presence_monitor_events() {
        assert_eq!(
            parse_presence_monitor_event(&format!(
                "{PRESENCE_MONITOR_SIGNAL_FALLBACK}\n6\n{PRESENCE_MONITOR_REASON_MISSING}"
            )),
            Ok(PresenceMonitorEvent::Fallback {
                cycle_id: PresenceCloseCycleId::from_raw(6),
                reason: PresenceMonitorFallback::Missing,
            }),
        );
        assert_eq!(
            parse_presence_monitor_event(&format!(
                "{PRESENCE_MONITOR_SIGNAL_FALLBACK}\n7\n{PRESENCE_MONITOR_REASON_NO_ANIMATION}"
            )),
            Ok(PresenceMonitorEvent::Fallback {
                cycle_id: PresenceCloseCycleId::from_raw(7),
                reason: PresenceMonitorFallback::NoAnimation,
            }),
        );
        assert_eq!(
            parse_presence_monitor_event(&format!(
                "{PRESENCE_MONITOR_SIGNAL_FALLBACK}\n8\n{PRESENCE_MONITOR_REASON_HIDDEN}"
            )),
            Ok(PresenceMonitorEvent::Fallback {
                cycle_id: PresenceCloseCycleId::from_raw(8),
                reason: PresenceMonitorFallback::Hidden,
            }),
        );
        assert_eq!(
            parse_presence_monitor_event(&format!(
                "{PRESENCE_MONITOR_SIGNAL_ANIMATION_END}\n9\nfade-out"
            )),
            Ok(PresenceMonitorEvent::AnimationEnd {
                cycle_id: PresenceCloseCycleId::from_raw(9),
                animation_name: String::from("fade-out"),
            }),
        );
        assert_eq!(
            parse_presence_monitor_event(&format!(
                "{PRESENCE_MONITOR_SIGNAL_ANIMATION_CANCEL}\n10\nfade-out"
            )),
            Ok(PresenceMonitorEvent::AnimationCancel {
                cycle_id: PresenceCloseCycleId::from_raw(10),
                animation_name: String::from("fade-out"),
            }),
        );
        assert_eq!(
            parse_presence_monitor_event(&format!("{PRESENCE_MONITOR_SIGNAL_STOPPED}\n11\n")),
            Ok(PresenceMonitorEvent::Stopped {
                cycle_id: PresenceCloseCycleId::from_raw(11),
            }),
        );
    }

    #[test]
    fn rejects_invalid_presence_monitor_payloads() {
        assert!(parse_presence_monitor_event("fallback\nabc\nhidden").is_err());
        assert!(parse_presence_monitor_event("fallback\n7\nunknown").is_err());
        assert!(parse_presence_monitor_event("animationend\n7\n").is_err());
    }
}
