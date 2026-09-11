use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TooltipGraceEventPayload {
    GraceLeave,
}

#[allow(dead_code)]
pub type TooltipGraceEvent = TooltipGraceEventPayload;

mod dom_bridge {
    use super::TooltipGraceEventPayload;
    dioxus_js_interop::bind_js!("src/tooltip/browser.ts"::*);
}

pub(crate) use dioxus_js_interop::WatcherGuard;

/// Launches the browser-side Safe Polygon (Grace Area) monitor for hover transit protection.
///
/// Returns `None` if running outside an active DOM/browser environment (e.g. headless unit tests).
pub(crate) fn start_tooltip_grace_monitor(
    trigger_id: &str,
    content_id: &str,
    mut on_leave: impl FnMut() + 'static,
) -> Option<WatcherGuard> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        dom_bridge::watch_tooltip_grace(
            trigger_id,
            content_id,
            move |_event: TooltipGraceEventPayload| {
                on_leave();
            },
        )
    }))
    .ok()
}

