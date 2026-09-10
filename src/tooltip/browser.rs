use dioxus::{document, document::Eval};

pub(crate) const TOOLTIP_MONITOR_STOP: &str = "stop";
pub(crate) const TOOLTIP_MONITOR_STOPPED: &str = "stopped";
pub(crate) const TOOLTIP_GRACE_LEAVE: &str = "grace:leave";

/// Events emitted by the tooltip browser-side grace area monitor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TooltipBrowserEvent {
    /// Pointer departed the safe polygon without entering the destination element.
    GraceLeave,
    /// Monitor gracefully shut down.
    Stopped,
    /// Unrecognized event string.
    Unknown(String),
}

/// Parses raw signal emitted from `src/tooltip/browser.js`.
pub(crate) fn parse_tooltip_browser_event(raw: &str) -> TooltipBrowserEvent {
    if raw == TOOLTIP_GRACE_LEAVE {
        TooltipBrowserEvent::GraceLeave
    } else if raw == TOOLTIP_MONITOR_STOPPED {
        TooltipBrowserEvent::Stopped
    } else {
        TooltipBrowserEvent::Unknown(raw.to_string())
    }
}

/// Launches the browser-side Safe Polygon (Grace Area) monitor for hover transit protection.
///
/// Returns `None` if running outside an active DOM/browser environment (e.g. headless unit tests).
pub(crate) fn start_tooltip_grace_monitor(trigger_id: &str, content_id: &str) -> Option<Eval> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let eval = document::eval(include_str!("browser.js"));
        let _ = eval.send((trigger_id.to_string(), content_id.to_string()));
        eval
    }))
    .ok()
}

/// Signals the active grace area monitor to disconnect its DOM listeners and terminate.
pub(crate) fn stop_tooltip_grace_monitor(eval: Eval) -> Result<(), String> {
    eval.send(TOOLTIP_MONITOR_STOP)
        .map_err(|error| format!("failed to send stop to tooltip grace monitor: {error}"))
}

#[cfg(test)]
mod tests {
    use super::{
        TOOLTIP_GRACE_LEAVE, TOOLTIP_MONITOR_STOPPED, TooltipBrowserEvent,
        parse_tooltip_browser_event,
    };

    #[test]
    fn parse_tooltip_browser_event_recognizes_known_signals() {
        assert_eq!(
            parse_tooltip_browser_event(TOOLTIP_GRACE_LEAVE),
            TooltipBrowserEvent::GraceLeave
        );
        assert_eq!(
            parse_tooltip_browser_event(TOOLTIP_MONITOR_STOPPED),
            TooltipBrowserEvent::Stopped
        );
        assert_eq!(
            parse_tooltip_browser_event("something_else"),
            TooltipBrowserEvent::Unknown("something_else".to_string())
        );
    }
}
