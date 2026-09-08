use dioxus::{document, document::Eval};
use super::types::{ToastConfig, ToastId};

pub(crate) const TOAST_MONITOR_STOP: &str = "stop";
pub(crate) const TOAST_MONITOR_STOPPED: &str = "stopped";

/// Browser lifecycle and window event variants observed by the toast runtime.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToastBrowserEvent {
    /// Window/tab visibility state changed (`true` = visible, `false` = hidden).
    Visibility(bool),
    /// Hotkey pressed (e.g. "F8") navigating focus to the landmark viewport.
    Hotkey(String),
    /// Viewport hover state entered or left.
    Hover(bool),
    /// Viewport focus state entered or left.
    Focus(bool),
    /// Active pointer swipe gesture started or ended.
    SwipeActive(bool),
    /// Pointer swipe gesture crossed dismiss threshold for specific toast ID.
    SwipeDismiss(ToastId),
    /// Monitor gracefully shut down.
    Stopped,
    /// Unrecognized browser event payload.
    Unknown(String),
}

/// Parses the raw signal emitted from `src/toast/browser.js`.
pub fn parse_toast_browser_event(raw: &str) -> ToastBrowserEvent {
    if raw == "visibility:visible" {
        ToastBrowserEvent::Visibility(true)
    } else if raw == "visibility:hidden" {
        ToastBrowserEvent::Visibility(false)
    } else if raw == "hover:enter" {
        ToastBrowserEvent::Hover(true)
    } else if raw == "hover:leave" {
        ToastBrowserEvent::Hover(false)
    } else if raw == "focus:enter" {
        ToastBrowserEvent::Focus(true)
    } else if raw == "focus:leave" {
        ToastBrowserEvent::Focus(false)
    } else if raw == "swipe:start" {
        ToastBrowserEvent::SwipeActive(true)
    } else if raw == "swipe:end" {
        ToastBrowserEvent::SwipeActive(false)
    } else if let Some(id_str) = raw.strip_prefix("swipe:dismiss:") {
        if let Ok(id_val) = id_str.parse::<u64>() {
            ToastBrowserEvent::SwipeDismiss(ToastId(id_val))
        } else {
            ToastBrowserEvent::Unknown(raw.to_string())
        }
    } else if let Some(key) = raw.strip_prefix("hotkey:") {
        ToastBrowserEvent::Hotkey(key.to_string())
    } else if raw == TOAST_MONITOR_STOPPED {
        ToastBrowserEvent::Stopped
    } else {
        ToastBrowserEvent::Unknown(raw.to_string())
    }
}

/// Launches the colocated browser monitor for visibility change, gestures, and landmark hotkey focus.
///
/// Returns `None` if running outside an active DOM/browser environment (e.g. headless unit tests).
pub fn start_toast_browser_monitor(config: &ToastConfig, viewport_id: &str) -> Option<Eval> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let eval = document::eval(include_str!("browser.js"));
        let swipe_dirs_str = config
            .effective_swipe_directions()
            .iter()
            .map(|d| format!("\"{}\"", d.as_str()))
            .collect::<Vec<_>>()
            .join(",");
        let offset_json = format!(
            r#"{{"top":{},"right":{},"bottom":{},"left":{}}}"#,
            config.offset.top.as_ref().map(|v| format!("\"{}\"", v)).unwrap_or_else(|| "null".to_string()),
            config.offset.right.as_ref().map(|v| format!("\"{}\"", v)).unwrap_or_else(|| "null".to_string()),
            config.offset.bottom.as_ref().map(|v| format!("\"{}\"", v)).unwrap_or_else(|| "null".to_string()),
            config.offset.left.as_ref().map(|v| format!("\"{}\"", v)).unwrap_or_else(|| "null".to_string()),
        );
        let mobile_offset_json = format!(
            r#"{{"top":{},"right":{},"bottom":{},"left":{}}}"#,
            config.mobile_offset.top.as_ref().map(|v| format!("\"{}\"", v)).unwrap_or_else(|| "null".to_string()),
            config.mobile_offset.right.as_ref().map(|v| format!("\"{}\"", v)).unwrap_or_else(|| "null".to_string()),
            config.mobile_offset.bottom.as_ref().map(|v| format!("\"{}\"", v)).unwrap_or_else(|| "null".to_string()),
            config.mobile_offset.left.as_ref().map(|v| format!("\"{}\"", v)).unwrap_or_else(|| "null".to_string()),
        );
        let payload = format!(
            r#"{{"hotkey":"{}","viewport_id":"{}","gap":{},"swipe_threshold":{},"position":"{}","dir":"{}","swipe_directions":[{}],"offset":{},"mobile_offset":{}}}"#,
            config.hotkey,
            viewport_id,
            config.gap,
            config.swipe_threshold,
            config.position.as_str(),
            config.dir.as_str(),
            swipe_dirs_str,
            offset_json,
            mobile_offset_json,
        );
        let _ = eval.send(payload);
        eval
    }))
    .ok()
}

/// Gracefully signals the browser monitor to unbind all DOM listeners and terminate.
pub fn stop_toast_browser_monitor(monitor: &mut Eval) -> Result<(), String> {
    monitor
        .send(TOAST_MONITOR_STOP)
        .map_err(|e| format!("toast browser monitor stop failed: {e}"))
}

/// Awaits the next browser event emitted from `src/toast/browser.js`.
pub async fn recv_toast_browser_event(monitor: &mut Eval) -> Result<ToastBrowserEvent, String> {
    let raw: String = monitor
        .recv()
        .await
        .map_err(|e| format!("toast browser monitor recv failed: {e}"))?;
    Ok(parse_toast_browser_event(&raw))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_browser_events() {
        assert_eq!(
            parse_toast_browser_event("visibility:visible"),
            ToastBrowserEvent::Visibility(true)
        );
        assert_eq!(
            parse_toast_browser_event("visibility:hidden"),
            ToastBrowserEvent::Visibility(false)
        );
        assert_eq!(
            parse_toast_browser_event("hotkey:F8"),
            ToastBrowserEvent::Hotkey("F8".to_string())
        );
        assert_eq!(
            parse_toast_browser_event("hover:enter"),
            ToastBrowserEvent::Hover(true)
        );
        assert_eq!(
            parse_toast_browser_event("hover:leave"),
            ToastBrowserEvent::Hover(false)
        );
        assert_eq!(
            parse_toast_browser_event("focus:enter"),
            ToastBrowserEvent::Focus(true)
        );
        assert_eq!(
            parse_toast_browser_event("focus:leave"),
            ToastBrowserEvent::Focus(false)
        );
        assert_eq!(
            parse_toast_browser_event("swipe:start"),
            ToastBrowserEvent::SwipeActive(true)
        );
        assert_eq!(
            parse_toast_browser_event("swipe:end"),
            ToastBrowserEvent::SwipeActive(false)
        );
        assert_eq!(
            parse_toast_browser_event("swipe:dismiss:42"),
            ToastBrowserEvent::SwipeDismiss(ToastId(42))
        );
        assert_eq!(
            parse_toast_browser_event("stopped"),
            ToastBrowserEvent::Stopped
        );
        assert_eq!(
            parse_toast_browser_event("other:random"),
            ToastBrowserEvent::Unknown("other:random".to_string())
        );
    }
}
