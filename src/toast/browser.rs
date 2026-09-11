use super::types::ToastConfig;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToastBrowserOptions {
    pub hotkey: String,
    pub viewport_id: String,
    pub gap: f64,
    pub swipe_threshold: f64,
    pub position: String,
    pub dir: String,
    pub swipe_directions: Vec<String>,
    pub offset_top: String,
    pub offset_right: String,
    pub offset_bottom: String,
    pub offset_left: String,
    pub mobile_offset_top: String,
    pub mobile_offset_right: String,
    pub mobile_offset_bottom: String,
    pub mobile_offset_left: String,
}

impl ToastBrowserOptions {
    pub fn from_config(config: &ToastConfig, viewport_id: &str) -> Self {
        let default_offset = "32px";
        let default_mobile = "16px";
        Self {
            hotkey: config.hotkey.clone(),
            viewport_id: viewport_id.to_string(),
            gap: config.gap,
            swipe_threshold: config.swipe_threshold,
            position: config.position.as_str().to_string(),
            dir: config.dir.as_str().to_string(),
            swipe_directions: config
                .effective_swipe_directions()
                .iter()
                .map(|d| d.as_str().to_string())
                .collect(),
            offset_top: config
                .offset
                .top
                .clone()
                .unwrap_or_else(|| default_offset.to_string()),
            offset_right: config
                .offset
                .right
                .clone()
                .unwrap_or_else(|| default_offset.to_string()),
            offset_bottom: config
                .offset
                .bottom
                .clone()
                .unwrap_or_else(|| default_offset.to_string()),
            offset_left: config
                .offset
                .left
                .clone()
                .unwrap_or_else(|| default_offset.to_string()),
            mobile_offset_top: config
                .mobile_offset
                .top
                .clone()
                .unwrap_or_else(|| default_mobile.to_string()),
            mobile_offset_right: config
                .mobile_offset
                .right
                .clone()
                .unwrap_or_else(|| default_mobile.to_string()),
            mobile_offset_bottom: config
                .mobile_offset
                .bottom
                .clone()
                .unwrap_or_else(|| default_mobile.to_string()),
            mobile_offset_left: config
                .mobile_offset
                .left
                .clone()
                .unwrap_or_else(|| default_mobile.to_string()),
        }
    }
}

/// Browser lifecycle and window event variants observed by the toast runtime.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ToastEventPayload {
    /// Window/tab visibility state changed (`true` = visible, `false` = hidden).
    Visibility { visible: bool },
    /// Hotkey pressed (e.g. "F8") navigating focus to the landmark viewport.
    Hotkey { key: String },
    /// Viewport hover state entered or left.
    Hover { hovered: bool },
    /// Viewport focus state entered or left.
    Focus { focused: bool },
    /// Active pointer swipe gesture started or ended.
    SwipeActive { swiping: bool },
    /// Pointer swipe gesture crossed dismiss threshold for specific toast ID.
    SwipeDismiss { id: u64 },
    /// Monitor gracefully shut down.
    Stopped,
}

pub type ToastBrowserEvent = ToastEventPayload;

mod dom_bridge {
    use super::{ToastBrowserOptions, ToastEventPayload};
    dioxus_js_interop::bind_js!("src/toast/browser.ts"::*);
}

pub use dioxus_js_interop::WatcherGuard;

/// Parses the typed JSON payload emitted from `src/toast/browser.ts`.
pub fn parse_toast_browser_event(
    json: &str,
) -> Result<ToastBrowserEvent, dioxus_js_interop::serde_json::Error> {
    dioxus_js_interop::serde_json::from_str(json)
}

/// Launches the colocated browser monitor for visibility change, gestures, and landmark hotkey focus.
///
/// Returns `None` if running outside an active DOM/browser environment (e.g. headless unit tests).
pub fn start_toast_browser_monitor(
    config: &ToastConfig,
    viewport_id: &str,
    mut on_event: impl FnMut(ToastBrowserEvent) + 'static,
) -> Option<WatcherGuard> {
    let options = ToastBrowserOptions::from_config(config, viewport_id);
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        dom_bridge::watch_toast(&options, move |event: ToastEventPayload| {
            on_event(event);
        })
    }))
    .ok()
}

/// Gracefully signals the browser monitor to unbind all DOM listeners and terminate.
pub fn stop_toast_browser_monitor(watcher: WatcherGuard) {
    watcher.stop();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_browser_events() {
        assert_eq!(
            parse_toast_browser_event(r#"{"kind":"visibility","visible":true}"#).unwrap(),
            ToastBrowserEvent::Visibility { visible: true }
        );
        assert_eq!(
            parse_toast_browser_event(r#"{"kind":"visibility","visible":false}"#).unwrap(),
            ToastBrowserEvent::Visibility { visible: false }
        );
        assert_eq!(
            parse_toast_browser_event(r#"{"kind":"hotkey","key":"F8"}"#).unwrap(),
            ToastBrowserEvent::Hotkey {
                key: "F8".to_string()
            }
        );
        assert_eq!(
            parse_toast_browser_event(r#"{"kind":"hover","hovered":true}"#).unwrap(),
            ToastBrowserEvent::Hover { hovered: true }
        );
        assert_eq!(
            parse_toast_browser_event(r#"{"kind":"hover","hovered":false}"#).unwrap(),
            ToastBrowserEvent::Hover { hovered: false }
        );
        assert_eq!(
            parse_toast_browser_event(r#"{"kind":"focus","focused":true}"#).unwrap(),
            ToastBrowserEvent::Focus { focused: true }
        );
        assert_eq!(
            parse_toast_browser_event(r#"{"kind":"focus","focused":false}"#).unwrap(),
            ToastBrowserEvent::Focus { focused: false }
        );
        assert_eq!(
            parse_toast_browser_event(r#"{"kind":"swipe_active","swiping":true}"#).unwrap(),
            ToastBrowserEvent::SwipeActive { swiping: true }
        );
        assert_eq!(
            parse_toast_browser_event(r#"{"kind":"swipe_active","swiping":false}"#).unwrap(),
            ToastBrowserEvent::SwipeActive { swiping: false }
        );
        assert_eq!(
            parse_toast_browser_event(r#"{"kind":"swipe_dismiss","id":42}"#).unwrap(),
            ToastBrowserEvent::SwipeDismiss { id: 42 }
        );
        assert_eq!(
            parse_toast_browser_event(r#"{"kind":"stopped"}"#).unwrap(),
            ToastBrowserEvent::Stopped
        );
    }
}
