use dioxus::{document, document::Eval, prelude::*};
use dioxus_use_js::use_js;

use crate::foundation::compose::MountedHandle;

use_js!("src/foundation/browser/viewport.js"::*);
use_js!("src/foundation/browser/focus.js"::*);
use_js!("src/foundation/browser/scroll_lock.js"::*);
use_js!("src/foundation/browser/portal.js"::*);
use_js!("src/foundation/browser/select.js"::*);

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
    let res: Result<[f64; 2], _> = getViewportSize().await;
    res.map_err(|error| format!("viewport query failed: {error}"))
}

pub(crate) async fn active_element_matches_id(target_id: &str) -> bool {
    let target_id = target_id.to_string();
    let res: Result<bool, _> = isElementActive(target_id).await;
    res.unwrap_or(false)
}

pub(crate) async fn is_reference_hidden(anchor_ids: &[&str]) -> Result<bool, String> {
    let ids: Vec<String> = anchor_ids.iter().map(|id| id.to_string()).collect();
    let res: Result<Option<bool>, _> = isReferenceHidden(ids).await;
    let hidden = res.map_err(|error| format!("reference hidden query failed: {error}"))?;
    Ok(hidden.unwrap_or(false))
}

pub(crate) fn focus_element_by_id(target_id: &str) {
    focus_element_by_id_with_options(target_id, false);
}

pub(crate) fn restore_focus_element_by_id(target_id: &str) {
    focus_element_by_id_with_options(target_id, true);
}

fn focus_element_by_id_with_options(target_id: &str, prevent_scroll: bool) {
    let target_id = target_id.to_string();
    spawn(async move {
        let _: Result<(), _> = focusElementByIdWithOptions(target_id, prevent_scroll).await;
    });
}

pub(crate) fn focus_first_focusable(content_id: &str, focusable_selector: Option<&str>) {
    let content_id = content_id.to_string();
    let selector = focusable_selector
        .unwrap_or(DEFAULT_FOCUSABLE_SELECTOR)
        .to_string();
    spawn(async move {
        let _: Result<(), _> = focusFirstFocusable(content_id, selector).await;
    });
}

pub(crate) fn acquire_scroll_lock(lock_id: &str) {
    let lock_id = lock_id.to_string();
    spawn(async move {
        let _: Result<(), _> = acquireScrollLock(lock_id).await;
    });
}

pub(crate) fn release_scroll_lock(lock_id: &str, restore_delay: Option<u64>) {
    let lock_id = lock_id.to_string();
    let delay_ms = restore_delay.unwrap_or_default();
    spawn(async move {
        let _: Result<(), _> = releaseScrollLock(lock_id, delay_ms).await;
    });
}

pub(crate) fn scroll_element_into_view_nearest(element_id: &str) {
    let element_id = element_id.to_string();
    spawn(async move {
        let _: Result<(), _> = scrollElementIntoViewNearest(element_id).await;
    });
}

pub(crate) fn set_body_user_select_none() {
    spawn(async move {
        let _: Result<(), _> = setBodyUserSelect(true).await;
    });
}

pub(crate) fn restore_body_user_select() {
    spawn(async move {
        let _: Result<(), _> = setBodyUserSelect(false).await;
    });
}

pub(crate) fn teleport_element_to_host(element_id: &str, host_id: Option<&str>) {
    let element_id = element_id.to_string();
    let host_id = host_id.map(|s| s.to_string());
    spawn(async move {
        let _: Result<(), _> = teleportElementToHost(element_id, host_id).await;
    });
}

pub(crate) fn remove_element_by_id(element_id: &str) {
    let element_id = element_id.to_string();
    spawn(async move {
        let _: Result<(), _> = removeElementById(element_id).await;
    });
}

pub(crate) async fn get_document_option_order(content_id: &str) -> Vec<String> {
    let content_id = content_id.to_string();
    let res: Result<Vec<String>, _> = getDocumentOptionOrder(content_id).await;
    res.unwrap_or_default()
}

pub(crate) async fn measure_select_floating_placement(
    trigger_id: &str,
    content_id: &str,
    custom_anchor_id: Option<&str>,
    boundary_id: Option<&str>,
) -> Option<[f64; 10]> {
    let trigger_id = trigger_id.to_string();
    let content_id = content_id.to_string();
    let custom_anchor_id = custom_anchor_id.map(|s| s.to_string());
    let boundary_id = boundary_id.map(|s| s.to_string());
    let res: Result<Option<[f64; 10]>, _> = measureSelectFloatingPlacement(
        trigger_id,
        content_id,
        custom_anchor_id,
        boundary_id,
    )
    .await;
    res.ok().flatten()
}

pub(crate) const FORM_RESET_SIGNAL_STOP: &str = "stop";
pub(crate) const FORM_RESET_SIGNAL_STOPPED: &str = "stopped";
pub(crate) const FORM_RESET_SIGNAL_RESET: &str = "reset";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FormResetEvent {
    Reset,
    Stopped,
}

pub(crate) fn start_form_reset_monitor(element_id: &str, form_id: Option<&str>) -> Eval {
    let eval = document::eval(include_str!("form_reset.js"));
    let _ = eval.send((element_id.to_string(), form_id.map(|s| s.to_string())));
    eval
}

pub(crate) fn stop_form_reset_monitor(monitor: Eval) -> Result<(), String> {
    monitor
        .send(FORM_RESET_SIGNAL_STOP)
        .map_err(|error| format!("form reset stop failed: {error}"))
}

pub(crate) async fn recv_form_reset_event(monitor: &mut Eval) -> Result<FormResetEvent, String> {
    let raw = monitor
        .recv::<String>()
        .await
        .map_err(|error| format!("form reset recv failed: {error}"))?;
    match raw.as_str() {
        FORM_RESET_SIGNAL_RESET => Ok(FormResetEvent::Reset),
        FORM_RESET_SIGNAL_STOPPED => Ok(FormResetEvent::Stopped),
        other => Err(format!("unexpected form reset event: {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DOCUMENT_DISMISS_SIGNAL_ESCAPE, DOCUMENT_DISMISS_SIGNAL_FOCUS_IN,
        DOCUMENT_DISMISS_SIGNAL_POINTER_DOWN, DocumentDismissEvent, parse_document_dismiss_event,
    };

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
}
