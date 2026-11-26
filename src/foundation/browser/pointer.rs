use dioxus::prelude::{PointerData, PointerEvent};
use oxidase::WatcherGuard;

use crate::foundation::overlay::Point2D;

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

    #[allow(dead_code)]
    pub(crate) fn into_watcher_guard(mut self) -> WatcherGuard {
        self.guard.take().unwrap_or_else(WatcherGuard::noop)
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

/// Captures the active pointer on `mounted_el` or the event's resolved element target, returning
/// the captured `pointer_id` when capture succeeds.
#[allow(dead_code)]
pub(crate) fn try_set_pointer_capture(
    evt: &PointerData,
    mounted_el: Option<&oxidase::Element>,
) -> Option<i32> {
    let pointer_id = oxidase::dom::extract_pointer_id(evt);
    if let Some(el) = mounted_el
        && el.set_pointer_capture(pointer_id).is_ok()
    {
        return Some(pointer_id);
    }
    if let Some(target) = oxidase::dom::pointer_event_target(evt)
        && target.set_pointer_capture(pointer_id).is_ok()
    {
        return Some(pointer_id);
    }
    None
}

/// Releases pointer capture for `captured_pid` (or the pointer ID extracted from `evt_opt`)
/// across `mounted_el` and the event target, ignoring already-released errors.
#[allow(dead_code)]
pub(crate) fn try_release_pointer_capture(
    evt_opt: Option<&PointerEvent>,
    mounted_el: Option<&oxidase::Element>,
    captured_pid: Option<i32>,
) {
    let pointer_id =
        captured_pid.or_else(|| evt_opt.map(|evt| oxidase::dom::extract_pointer_id(&evt.data())));
    let Some(pointer_id) = pointer_id else {
        return;
    };
    if let Some(el) = mounted_el {
        let _ = el.release_pointer_capture(pointer_id);
    }
    if let Some(evt) = evt_opt
        && let Some(target) = oxidase::dom::pointer_event_target(&evt.data())
    {
        let _ = target.release_pointer_capture(pointer_id);
    }
}
