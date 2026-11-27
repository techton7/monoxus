use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use dioxus::prelude::*;
use oxidase::WatcherGuard;
use oxidase::dom::FocusOptions;
use oxidase::dom::observer::CaptureEventMask;

use crate::foundation::compose::MountedHandle;

use super::spawn_root_task;

#[allow(dead_code)]
pub(crate) const DEFAULT_FOCUSABLE_SELECTOR: &str = "a[href],button:not([disabled]),input:not([disabled]),select:not([disabled]),textarea:not([disabled]),[tabindex]:not([tabindex='-1'])";

const TABBABLE_SELECTOR: &str = "a[href],area[href],button:not([disabled]),input:not([disabled]):not([type='hidden']),select:not([disabled]),textarea:not([disabled]),summary,[contenteditable]:not([contenteditable='false']),[tabindex]";
const SAVED_TABINDEX_ATTR: &str = "data-monoxus-tabindex";
const SAVED_TABINDEX_NONE: &str = "__none__";
static NEXT_TABBABLE_ID: AtomicU64 = AtomicU64::new(1);

pub(crate) fn focus_mounted_handle(handle: Option<MountedHandle>) -> bool {
    let Some(handle) = handle else {
        return false;
    };
    let el = oxidase::Element::from(handle);
    spawn_root_task(async move {
        let _ = el.focus_with_options(FocusOptions::new(true)).await;
    });
    true
}

#[allow(dead_code)]
pub(crate) async fn active_element_matches_id(target_id: &str) -> bool {
    oxidase::document()
        .active_element()
        .and_then(|el| el.id())
        .map(|id| id == target_id)
        .unwrap_or(false)
}

pub(crate) fn focus_element_by_id(target_id: &str) {
    focus_element_by_id_with_options(target_id, FocusOptions::new(true));
}

pub(crate) fn focus_element_by_id_with_options(target_id: &str, options: FocusOptions) {
    let tid = target_id.to_string();
    spawn_root_task(async move {
        for _ in 0..5 {
            let _ = oxidase::frame::next_frame().await;
            let doc = oxidase::dom::Document::current().unwrap_or_else(oxidase::document);
            if let Some(el) = doc.element_by_id(&tid)
                && el.focus_with_options(options).await.is_ok()
            {
                break;
            }
        }
    });
}

pub(crate) fn focus_element_sync_or_defer(target_id: &str, options: FocusOptions) {
    if let Some(doc) = oxidase::dom::Document::current()
        && doc.set_focus_with_options(target_id, options).is_ok()
    {
        return;
    }
    focus_element_by_id_with_options(target_id, options);
}

/// Attempts synchronous focus with `options`, verifying active focus ownership and falling back
/// to a bounded 5-frame deferred retry when the target has not yet mounted or accepted focus.
#[allow(dead_code)]
pub(crate) fn focus_element_reliable(target_id: &str, options: FocusOptions) {
    focus_element_sync_or_defer(target_id, options);
    if !is_element_focused_by_id(target_id) {
        focus_element_by_id_with_options(target_id, options);
    }
}

pub(crate) fn restore_focus_element_by_id(target_id: &str) {
    restore_focus_element_by_id_with_options(target_id, FocusOptions::new(true));
}

pub(crate) fn restore_focus_element_by_id_with_options(target_id: &str, options: FocusOptions) {
    focus_element_by_id_with_options(target_id, options);
}

pub(crate) fn is_element_focus_visible_by_id(target_id: &str) -> Option<bool> {
    let doc = oxidase::dom::Document::current()?;
    let el = doc.element_by_id(target_id)?;
    Some(el.matches_focus_visible())
}

pub(crate) fn is_element_focused_by_id(target_id: &str) -> bool {
    let Some(doc) = oxidase::dom::Document::current() else {
        return false;
    };
    let Some(el) = doc.element_by_id(target_id) else {
        return false;
    };
    el.has_focus()
}

/// Returns `true` when keyboard focus currently resides on or within `element_id`.
#[allow(dead_code)]
pub(crate) fn is_focus_within_element(element_id: &str) -> bool {
    oxidase::dom::Document::current().is_some_and(|doc| doc.is_focus_within(element_id))
}

/// Blurs the currently focused element in the active document, if any.
#[allow(dead_code)]
pub(crate) fn blur_active_element() -> bool {
    oxidase::dom::Document::current().is_some_and(|doc| doc.blur_active().is_ok())
}

fn is_tabbable_element(el: &oxidase::dom::Element) -> bool {
    if el.has_attribute("disabled")
        || el.has_attribute("hidden")
        || el.has_attribute("inert")
        || el.closest("[hidden],[inert],[aria-hidden='true']").is_some()
    {
        return false;
    }
    if el.get_attribute("aria-hidden").as_deref() == Some("true") {
        return false;
    }
    let tag = el.tag_name();
    if tag.as_deref() == Some("input")
        && el
            .get_attribute("type")
            .is_some_and(|t| t.eq_ignore_ascii_case("hidden"))
    {
        return false;
    }
    if let Some(raw_tabindex) = el.get_attribute("tabindex") {
        let trimmed = raw_tabindex.trim();
        match trimmed.parse::<i32>() {
            Ok(idx) if idx < 0 => return false,
            Ok(_) => return true,
            Err(_) => {
                let natively_focusable = matches!(
                    tag.as_deref(),
                    Some("a" | "area" | "button" | "input" | "select" | "textarea" | "summary")
                );
                if !natively_focusable {
                    return false;
                }
            }
        }
    }
    true
}

fn collect_tabbable_elements(container: &oxidase::dom::Element) -> Vec<oxidase::dom::Element> {
    container
        .query_selector_all(TABBABLE_SELECTOR)
        .into_iter()
        .filter(is_tabbable_element)
        .collect()
}

/// Returns `true` if the element identified by `element_id` exists and is genuinely keyboard-tabbable.
#[allow(dead_code)]
pub(crate) fn is_element_tabbable_by_id(element_id: &str) -> bool {
    let Some(doc) = oxidase::dom::Document::current() else {
        return false;
    };
    let Some(el) = doc.element_by_id(element_id) else {
        return false;
    };
    el.matches(TABBABLE_SELECTOR) && is_tabbable_element(&el)
}

fn ensure_element_id(el: &oxidase::dom::Element) -> String {
    if let Some(id) = el.id().filter(|id| !id.is_empty()) {
        return id;
    }
    let id = format!(
        "monoxus-tabbable-{}",
        NEXT_TABBABLE_ID.fetch_add(1, Ordering::Relaxed)
    );
    el.set_attribute("id", &id);
    id
}

/// Returns tabbable descendant element IDs under `container_id` in DOM order,
/// synthesizing stable IDs when a tabbable descendant has no `id` attribute.
#[allow(dead_code)]
pub fn get_tabbable_candidates(container_id: &str) -> Vec<String> {
    let Some(doc) = oxidase::dom::Document::current() else {
        return Vec::new();
    };
    let Some(container) = doc.element_by_id(container_id) else {
        return Vec::new();
    };
    collect_tabbable_elements(&container)
        .into_iter()
        .map(|el| ensure_element_id(&el))
        .collect()
}

/// Removes tabbable descendants under `container_id` from the keyboard tab order
/// by saving any prior `tabindex` in `data-monoxus-tabindex` and setting `tabindex="-1"`.
#[allow(dead_code)]
pub fn remove_from_tab_order(container_id: &str) {
    let Some(doc) = oxidase::dom::Document::current() else {
        return;
    };
    let Some(container) = doc.element_by_id(container_id) else {
        return;
    };
    for el in collect_tabbable_elements(&container) {
        if !el.has_attribute(SAVED_TABINDEX_ATTR) {
            match el.get_attribute("tabindex") {
                Some(prev) => el.set_attribute(SAVED_TABINDEX_ATTR, &prev),
                None => el.set_attribute(SAVED_TABINDEX_ATTR, SAVED_TABINDEX_NONE),
            }
        }
        el.set_attribute("tabindex", "-1");
    }
}

/// Restores tabbable descendants under `container_id` previously modified by [`remove_from_tab_order`].
#[allow(dead_code)]
pub fn restore_tab_order(container_id: &str) {
    let Some(doc) = oxidase::dom::Document::current() else {
        return;
    };
    let Some(container) = doc.element_by_id(container_id) else {
        return;
    };
    for el in container.query_selector_all(&format!("[{SAVED_TABINDEX_ATTR}]")) {
        let saved = el.get_attribute(SAVED_TABINDEX_ATTR);
        el.remove_attribute(SAVED_TABINDEX_ATTR);
        match saved.as_deref() {
            Some(SAVED_TABINDEX_NONE) => el.remove_attribute("tabindex"),
            Some(prev) => el.set_attribute("tabindex", prev),
            None => {}
        }
    }
}

/// Starts a document-wide keyboard-modality monitor: capture-phase `KeyDown` emits `true`,
/// and `PointerDown`/`PointerMove` emit `false` on transitions.
#[allow(dead_code)]
pub fn start_keyboard_modality_monitor(
    on_change: impl FnMut(bool) + 'static,
) -> WatcherGuard {
    let runtime = dioxus::core::Runtime::try_current();
    let scope_id = runtime.as_ref().and_then(|rt| rt.try_current_scope_id());
    let on_change_cell: Rc<RefCell<Box<dyn FnMut(bool)>>> =
        Rc::new(RefCell::new(Box::new(on_change)));
    let last_is_keyboard = Rc::new(Cell::new(None::<bool>));

    let emit = {
        let runtime = runtime.clone();
        let on_change_cell = Rc::clone(&on_change_cell);
        let last_is_keyboard = Rc::clone(&last_is_keyboard);
        move |is_keyboard: bool| {
            if last_is_keyboard.get() == Some(is_keyboard) {
                return;
            }
            last_is_keyboard.set(Some(is_keyboard));
            let invoke = || {
                if let Ok(mut cb) = on_change_cell.try_borrow_mut() {
                    cb(is_keyboard);
                }
            };
            if let Some(rt) = &runtime {
                let scope = scope_id.unwrap_or(dioxus::core::ScopeId::ROOT);
                rt.in_scope(scope, invoke);
            } else {
                invoke();
            }
        }
    };

    let emit_for_capture = emit.clone();
    let capture_guard = oxidase::dom::observer::observe_capture_events_with_mask(
        CaptureEventMask::KEY_DOWN | CaptureEventMask::POINTER_DOWN,
        move |event| match event.kind {
            oxidase::dom::observer::CaptureEventKind::KeyDown { .. } => emit_for_capture(true),
            oxidase::dom::observer::CaptureEventKind::PointerDown { .. } => {
                emit_for_capture(false)
            }
            oxidase::dom::observer::CaptureEventKind::Click { .. }
            | oxidase::dom::observer::CaptureEventKind::FocusIn => {}
        },
    )
    .unwrap_or_else(|_| WatcherGuard::noop());

    let pointer_move_guard = oxidase::dom::observer::observe_pointer_move(move |_x, _y| {
        emit(false);
    })
    .unwrap_or_else(|_| WatcherGuard::noop());

    WatcherGuard::with_cleanup("monoxus_keyboard_modality_monitor", 0, None, move || {
        capture_guard.stop();
        pointer_move_guard.stop();
    })
}

/// Reactive hook tracking whether the user is currently navigating via keyboard (`KeyDown` -> `true`,
/// `PointerDown`/`PointerMove` -> `false`) with automatic `use_drop` teardown.
#[allow(dead_code)]
pub fn use_is_using_keyboard() -> Signal<bool> {
    let mut is_using_keyboard = use_signal(|| false);
    let guard_cell = use_hook(|| {
        let guard = start_keyboard_modality_monitor(move |using_kb| {
            if *is_using_keyboard.peek() != using_kb {
                is_using_keyboard.set(using_kb);
            }
        });
        Rc::new(RefCell::new(guard))
    });

    use_drop({
        let guard_cell = Rc::clone(&guard_cell);
        move || {
            let prev = std::mem::replace(&mut *guard_cell.borrow_mut(), WatcherGuard::noop());
            prev.stop();
        }
    });

    is_using_keyboard
}

/// Composite focus-exit tracker for multi-part and portaled overlay controls whose DOM focus
/// moves across internal sub-elements (e.g. `RadioGroup`, `DateField`, `Select`, `Combobox`).
///
/// Because `dioxus-html` `FocusData` does not expose `relatedTarget`, `onfocusout` schedules a
/// single coalesced check racing `oxidase::frame::next_frame()` against a `10ms`
/// `futures_timer::Delay`, cancelled immediately if `onfocusin` fires on any participating region
/// or when the component unmounts (`alive` drop guard). If focus has truly left every element in
/// `boundary_ids`, the stored `on_focus_out` callback is invoked once.
///
/// Returns `(on_focus_in, on_focus_out)` handlers to attach to the composite root (and any
/// portaled content container). Stores `on_focus_out` inside a [`CopyValue`] to remain immune
/// to `Dioxus #5894` when callers toggle `Some(...) <-> None` across renders.
#[allow(dead_code)]
pub(crate) fn use_composite_focus_out(
    boundary_ids: Vec<String>,
    on_focus_out: Option<EventHandler<FocusEvent>>,
) -> (EventHandler<FocusEvent>, EventHandler<FocusEvent>) {
    let (on_in, on_out, _on_unmount_check) =
        use_composite_focus_out_tracked(boundary_ids, on_focus_out);
    (on_in, on_out)
}

/// Like [`use_composite_focus_out`], returning a third `EventHandler<()>` that triggers a
/// coalesced focus-exit verification when an overlay/popover sub-region closes or unmounts while
/// focus was inside the composite (preventing lost `onfocusout` events when dismiss unmounts
/// portaled/static content before queued DOM blur delivery).
#[allow(dead_code)]
pub(crate) fn use_composite_focus_out_tracked(
    boundary_ids: Vec<String>,
    on_focus_out: Option<EventHandler<FocusEvent>>,
) -> (
    EventHandler<FocusEvent>,
    EventHandler<FocusEvent>,
    EventHandler<()>,
) {
    let mut handler_slot = use_hook(|| CopyValue::new(on_focus_out));
    handler_slot.set(on_focus_out);

    let mut boundaries_slot = use_hook(|| CopyValue::new(boundary_ids.clone()));
    if boundaries_slot
        .try_read()
        .is_ok_and(|cur| *cur != boundary_ids)
    {
        boundaries_slot.set(boundary_ids);
    }

    let alive = use_hook(|| CopyValue::new(true));
    let pending_gen = use_hook(|| CopyValue::new(0_u64));
    let last_focus_evt: CopyValue<Option<FocusEvent>> = use_hook(|| CopyValue::new(None));

    use_drop(move || {
        let mut alive = alive;
        let mut pending_gen = pending_gen;
        if let Ok(mut flag) = alive.try_write() {
            *flag = false;
        }
        if let Ok(mut generation) = pending_gen.try_write() {
            *generation = generation.wrapping_add(1);
        }
    });

    let handle_focus_in = use_callback(move |evt: FocusEvent| {
        let mut pending_gen = pending_gen;
        if let Ok(mut generation) = pending_gen.try_write() {
            *generation = generation.wrapping_add(1);
        }
        let mut last = last_focus_evt;
        if let Ok(mut slot) = last.try_write() {
            *slot = Some(evt);
        }
    });

    let handle_focus_out = use_callback(move |evt: FocusEvent| {
        let has_handler = handler_slot.try_read().ok().and_then(|slot| *slot).is_some();
        if !has_handler {
            return;
        }

        let mut pending_gen = pending_gen;
        let expected_gen = match pending_gen.try_write() {
            Ok(mut generation) => {
                *generation = generation.wrapping_add(1);
                *generation
            }
            Err(_) => return,
        };

        spawn_root_task(async move {
            let _ = futures_util::future::select(
                Box::pin(oxidase::frame::next_frame()),
                Box::pin(futures_timer::Delay::new(Duration::from_millis(10))),
            )
            .await;

            if !alive.try_read().is_ok_and(|flag| *flag) {
                return;
            }
            if !pending_gen
                .try_read()
                .is_ok_and(|generation| *generation == expected_gen)
            {
                return;
            }

            let still_within = boundaries_slot.try_read().is_ok_and(|ids| {
                ids.iter()
                    .any(|id| !id.is_empty() && is_focus_within_element(id))
            });
            if still_within {
                return;
            }

            let mut last = last_focus_evt;
            if let Ok(mut slot) = last.try_write() {
                *slot = None;
            }

            let cb = handler_slot.try_read().ok().and_then(|slot| *slot);
            if let Some(handler) = cb {
                handler.call(evt);
            }
        });
    });

    let handle_region_dismiss = use_callback(move |(): ()| {
        let saved = last_focus_evt
            .try_read()
            .ok()
            .and_then(|slot| slot.clone());
        if let Some(evt) = saved {
            handle_focus_out.call(evt);
        }
    });

    (handle_focus_in, handle_focus_out, handle_region_dismiss)
}
