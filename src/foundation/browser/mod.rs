use std::cell::RefCell;
use std::rc::Rc;

use monoxus_core::foundation::date_time::CalendarDate;

pub(crate) mod floating_measure;
pub(crate) mod focus;
pub(crate) mod pointer;
pub(crate) mod portal;
pub(crate) mod scroll_lock;
pub(crate) mod viewport;

#[cfg(test)]
mod tests;

#[allow(unused_imports)]
pub use oxidase::watcher::{
    DismissEvent, DocumentDismissEvent as DocumentDismissEventPayload, FloatingAutoUpdateEvent,
    FloatingAutoUpdatePayload, FormResetEvent, FormResetEventPayload, PresenceEvent,
    PresenceEventPayload, PresenceFallbackReason, PresenceMonitorFallback,
};

pub use oxidase::WatcherGuard;
#[allow(unused_imports)]
pub use oxidase::dom::{
    FocusOptions, ModalShieldGuard, ModalShieldMode, ResizeObserverBox, ScrollLockOptions,
};

#[allow(unused_imports)]
pub use monoxus_core::foundation::shared::{ElementScrollMetrics, ElementSize};

#[allow(unused_imports)]
pub use self::floating_measure::*;
#[allow(unused_imports)]
pub use self::focus::*;
#[allow(unused_imports)]
pub use self::pointer::*;
#[allow(unused_imports)]
pub use self::portal::*;
#[allow(unused_imports)]
pub use self::scroll_lock::*;
#[allow(unused_imports)]
pub use self::viewport::*;

pub fn spawn_root_task<F: std::future::Future<Output = ()> + 'static>(
    fut: F,
) -> dioxus::core::Task {
    dioxus::core::spawn_forever(fut)
}

/// Returns the host local calendar date as `(year, month, day)` (`month` in `1..=12`, `day` in `1..=31`).
#[allow(dead_code)]
pub fn local_today_ymd() -> (i32, u8, u8) {
    oxidase::time::local_today_ymd()
}

/// Returns the canonical host local [`CalendarDate`] via [`local_today_ymd`].
#[allow(dead_code)]
pub fn local_today() -> CalendarDate {
    let (year, month, day) = local_today_ymd();
    CalendarDate::from_ymd_clamped(year, month, day)
}

/// Starts a host window `blur` monitor returning an RAII [`WatcherGuard`].
#[allow(dead_code)]
pub fn start_window_blur_monitor(mut on_blur: impl FnMut() + 'static) -> WatcherGuard {
    let runtime = dioxus::core::Runtime::try_current();
    let scope_id = runtime.as_ref().and_then(|rt| rt.try_current_scope_id());
    oxidase::dom::observer::observe_window_blur(move || {
        if let Some(rt) = &runtime {
            let scope = scope_id.unwrap_or(dioxus::core::ScopeId::ROOT);
            rt.in_scope(scope, || on_blur());
        } else {
            on_blur();
        }
    })
    .unwrap_or_else(|_| WatcherGuard::noop())
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

/// Queries all matching descendant elements inside `element_id`.
#[allow(dead_code)]
pub(crate) fn query_selector_all_by_id(
    element_id: &str,
    selector: &str,
) -> Vec<oxidase::dom::Element> {
    let Some(el) = oxidase::dom::Document::current().and_then(|doc| doc.element_by_id(element_id))
    else {
        return Vec::new();
    };
    el.query_selector_all(selector)
}

/// Dispatches a synchronous DOM click on `element_id`. Returns `true` when dispatched.
#[allow(dead_code)]
pub(crate) fn click_element_by_id(element_id: &str) -> bool {
    let Some(el) = oxidase::dom::Document::current().and_then(|doc| doc.element_by_id(element_id))
    else {
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
    let Some(el) = oxidase::dom::Document::current().and_then(|doc| doc.element_by_id(element_id))
    else {
        return false;
    };
    el.set_selection_range(start, end).is_ok()
}

/// Synchronously sets the value of the `<input>` identified by `element_id`.
#[allow(dead_code)]
pub(crate) fn set_input_value_by_id(element_id: &str, value: &str) -> bool {
    let Some(el) = oxidase::dom::Document::current().and_then(|doc| doc.element_by_id(element_id))
    else {
        return false;
    };
    el.set_input_value(value).is_ok()
}
