use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;
use monoxus_core::foundation::shared::{ElementScrollMetrics, ElementSize};
use oxidase::WatcherGuard;
use oxidase::dom::ResizeObserverBox;

use super::floating_measure::{get_element_client_rect, measure_element_without_animation};

pub(crate) async fn get_viewport_size() -> Result<[f64; 2], String> {
    let (width, height) = oxidase::window()
        .inner_size()
        .await
        .map_err(|e| e.to_string())?;
    if width > 0.0 && height > 0.0 {
        Ok([width, height])
    } else {
        Err("Viewport dimensions are unavailable or non-positive".to_string())
    }
}

#[allow(dead_code)]
pub(crate) fn scroll_element_into_view_nearest(element_id: &str) {
    let doc = oxidase::dom::document();
    if let Some(el) = doc.element_by_id(element_id) {
        let _ = el.scroll_into_view_sync_with_options(oxidase::dom::ScrollIntoViewOptions {
            behavior: oxidase::dom::ScrollBehavior::Auto,
            block: oxidase::dom::ScrollLogicalPosition::Nearest,
            inline: oxidase::dom::ScrollLogicalPosition::Nearest,
        });
    }
}

/// Retrieves the current CSSOM scroll metrics (scroll offset, scroll size, client size) of an element synchronously.
#[allow(dead_code)]
pub(crate) fn get_element_scroll_metrics(element_id: &str) -> Option<ElementScrollMetrics> {
    let doc = oxidase::dom::Document::current()?;
    let el = doc.element_by_id(element_id)?;
    let scroll_left = el.scroll_x_sync().ok()?;
    let scroll_top = el.scroll_y_sync().ok()?;
    let scroll_width = el.scroll_width_sync().ok()?;
    let scroll_height = el.scroll_height_sync().ok()?;
    let client_width = el.client_width_sync().ok()?;
    let client_height = el.client_height_sync().ok()?;
    Some(ElementScrollMetrics {
        scroll_left,
        scroll_top,
        scroll_width,
        scroll_height,
        client_width,
        client_height,
    })
}

/// Sets the scroll position of an element synchronously.
#[allow(dead_code)]
pub(crate) fn set_element_scroll_position(element_id: &str, x: f64, y: f64) {
    if let Some(doc) = oxidase::dom::Document::current()
        && let Some(el) = doc.element_by_id(element_id)
    {
        let _ = el.scroll_to_sync(x, y);
    }
}

/// Sets an inline style property on an element by ID.
#[allow(dead_code)]
pub(crate) fn set_element_style_property(element_id: &str, property: &str, value: &str) {
    if let Some(doc) = oxidase::dom::Document::current()
        && let Some(el) = doc.element_by_id(element_id)
    {
        let _ = el.set_style_property(property, value);
    }
}

/// Starts a scroll monitor on a specific element container, executing `on_scroll` within the Dioxus runtime context.
#[allow(dead_code)]
pub(crate) fn start_element_scroll_monitor(
    element_id: &str,
    mut on_scroll: impl FnMut() + 'static,
) -> WatcherGuard {
    let runtime = dioxus::core::Runtime::try_current();
    let scope_id = runtime.as_ref().and_then(|rt| rt.try_current_scope_id());
    oxidase::dom::observer::observe_element_scroll(element_id, move || {
        if let Some(rt) = &runtime {
            let scope = scope_id.unwrap_or(dioxus::core::ScopeId::ROOT);
            rt.in_scope(scope, || on_scroll());
        } else {
            on_scroll();
        }
    })
    .unwrap_or_else(|_| WatcherGuard::noop())
}

/// Starts an element resize monitor with the specified box model, delegating directly to
/// `oxidase v0.4.7`'s pooled Web `ResizeObserver` and executing `on_resize` within the Dioxus runtime.
#[allow(dead_code)]
pub(crate) fn start_resize_monitor(
    element_id: &str,
    box_model: ResizeObserverBox,
    mut on_resize: impl FnMut(ElementSize) + 'static,
) -> WatcherGuard {
    let runtime = dioxus::core::Runtime::try_current();
    let scope_id = runtime.as_ref().and_then(|rt| rt.try_current_scope_id());
    oxidase::dom::observer::observe_element_resize_with_box(
        element_id,
        box_model,
        move |entry: oxidase::dom::observer::ResizeEntry| {
            let size = ElementSize {
                width: entry.content_rect.width,
                height: entry.content_rect.height,
            };
            if let Some(rt) = &runtime {
                let scope = scope_id.unwrap_or(dioxus::core::ScopeId::ROOT);
                rt.in_scope(scope, || on_resize(size));
            } else {
                on_resize(size);
            }
        },
    )
    .unwrap_or_else(|_| WatcherGuard::noop())
}

/// Reactive hook measuring an element's dimensions using the specified box model and mount revision
/// with clean `use_drop` teardown (relying directly on `oxidase v0.4.7`'s pooled `ResizeObserver`).
#[allow(dead_code)]
pub(crate) fn use_element_size_with_mount(
    element_id: &str,
    box_model: ResizeObserverBox,
    mount_rev: Signal<u64>,
) -> Signal<Option<ElementSize>> {
    let element_id_str = element_id.to_string();
    let initial_size = measure_element_without_animation(&element_id_str)
        .or_else(|| {
            get_element_client_rect(&element_id_str).map(|r| ElementSize {
                width: r.width,
                height: r.height,
            })
        })
        .filter(|s| s.width > 0.0 || s.height > 0.0);
    let mut size_signal = use_signal(|| initial_size);
    let guard_cell = use_hook(|| Rc::new(RefCell::new(WatcherGuard::noop())));

    use_effect(use_reactive((&element_id_str,), {
        let guard_cell = Rc::clone(&guard_cell);
        move |(id,)| {
            let _ = mount_rev();
            if let Some(measured) = measure_element_without_animation(&id)
                .or_else(|| {
                    get_element_client_rect(&id).map(|r| ElementSize {
                        width: r.width,
                        height: r.height,
                    })
                })
                .filter(|s| s.width > 0.0 || s.height > 0.0)
                && size_signal.peek().as_ref() != Some(&measured)
            {
                size_signal.set(Some(measured));
            }
            let next_guard = start_resize_monitor(&id, box_model, move |new_size| {
                if new_size.width <= 0.0 && new_size.height <= 0.0 {
                    return;
                }
                if size_signal.peek().as_ref() != Some(&new_size) {
                    size_signal.set(Some(new_size));
                }
            });
            let prev_guard = std::mem::replace(&mut *guard_cell.borrow_mut(), next_guard);
            prev_guard.stop();
        }
    }));

    use_drop({
        let guard_cell = Rc::clone(&guard_cell);
        move || {
            let prev_guard = std::mem::replace(&mut *guard_cell.borrow_mut(), WatcherGuard::noop());
            prev_guard.stop();
        }
    });

    size_signal
}
