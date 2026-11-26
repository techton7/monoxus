use std::cell::Cell;
use std::rc::Rc;

use dioxus::prelude::*;

use crate::foundation::{
    browser::{
        ElementScrollMetrics, ElementSize, WatcherGuard, get_element_scroll_metrics,
        pointer::{
            try_release_pointer_capture as foundation_release_pointer_capture,
            try_set_pointer_capture as foundation_set_pointer_capture,
        },
        set_element_scroll_position, set_element_style_property, start_element_scroll_monitor,
    },
    shared::{Direction, Orientation},
    state::{sync_f64_signal, sync_signal},
};

use super::{
    state::{get_thumb_offset_from_scroll, is_scrolling_within_scrollbar_bounds},
    types::{ScrollAreaSizes, ScrollAreaVisibilityState},
};

#[derive(Clone, Copy, PartialEq)]
pub struct ScrollAreaRuntime {
    pub hover_visible: Signal<bool>,
    pub scroll_state: Signal<ScrollAreaVisibilityState>,
    pub scrollbar_x_enabled: Signal<bool>,
    pub scrollbar_y_enabled: Signal<bool>,
    pub scrollbar_x_visible: Signal<bool>,
    pub scrollbar_y_visible: Signal<bool>,
    pub corner_width: Signal<f64>,
    pub corner_height: Signal<f64>,
    pub viewport_metrics: Signal<Option<ElementScrollMetrics>>,
}

impl ScrollAreaRuntime {
    pub fn new() -> Self {
        Self {
            hover_visible: Signal::new(false),
            scroll_state: Signal::new(ScrollAreaVisibilityState::Hidden),
            scrollbar_x_enabled: Signal::new(false),
            scrollbar_y_enabled: Signal::new(false),
            scrollbar_x_visible: Signal::new(false),
            scrollbar_y_visible: Signal::new(false),
            corner_width: Signal::new(0.0),
            corner_height: Signal::new(0.0),
            viewport_metrics: Signal::new(None),
        }
    }

    pub fn hover_visible(&self) -> bool {
        (self.hover_visible)()
    }

    pub fn scroll_state(&self) -> ScrollAreaVisibilityState {
        (self.scroll_state)()
    }

    pub fn scrollbar_x_enabled(&self) -> bool {
        (self.scrollbar_x_enabled)()
    }

    pub fn scrollbar_y_enabled(&self) -> bool {
        (self.scrollbar_y_enabled)()
    }

    pub fn scrollbar_x_visible(&self) -> bool {
        (self.scrollbar_x_visible)()
    }

    pub fn scrollbar_y_visible(&self) -> bool {
        (self.scrollbar_y_visible)()
    }

    pub fn corner_width(&self) -> f64 {
        (self.corner_width)()
    }

    pub fn corner_height(&self) -> f64 {
        (self.corner_height)()
    }

    pub fn viewport_metrics(&self) -> Option<ElementScrollMetrics> {
        (self.viewport_metrics)()
    }

    /// Reads scroll metrics for `viewport_id` and synchronizes `viewport_metrics` and axis overflow
    /// visibility signals without subscribing.
    pub(crate) fn sync_viewport_metrics(
        &mut self,
        viewport_id: &str,
        size_override: Option<ElementSize>,
    ) {
        if let Some(metrics) = get_element_scroll_metrics(viewport_id) {
            sync_signal(&mut self.viewport_metrics, Some(metrics));
            let (vw, vh) = size_override
                .map(|s| (s.width, s.height))
                .unwrap_or((metrics.client_width, metrics.client_height));
            sync_signal(&mut self.scrollbar_x_visible, vw < metrics.scroll_width);
            sync_signal(&mut self.scrollbar_y_visible, vh < metrics.scroll_height);
        }
    }

    /// Synchronizes corner dimensions for `orientation` using bitwise `f64` equality and drop-safe
    /// `try_write`, allowing collapse to `0.0` (`radix#2383/#4009`).
    pub(crate) fn sync_corner_size(&mut self, orientation: Orientation, width: f64, height: f64) {
        match orientation {
            Orientation::Horizontal => {
                sync_f64_signal(&mut self.corner_height, height);
            }
            Orientation::Vertical => {
                sync_f64_signal(&mut self.corner_width, width);
            }
            Orientation::Both => {
                sync_f64_signal(&mut self.corner_height, height);
                sync_f64_signal(&mut self.corner_width, width);
            }
        }
    }
}

impl Default for ScrollAreaRuntime {
    fn default() -> Self {
        Self::new()
    }
}

pub fn use_scroll_area_runtime() -> ScrollAreaRuntime {
    use_hook(ScrollAreaRuntime::new)
}

pub struct HideDelayTimer {
    generation: Rc<Cell<u64>>,
}

impl HideDelayTimer {
    pub fn new() -> Self {
        Self {
            generation: Rc::new(Cell::new(0)),
        }
    }

    pub fn cancel(&self) {
        self.generation.set(self.generation.get().wrapping_add(1));
    }

    pub fn schedule<F>(&self, delay_ms: u64, on_elapsed: F)
    where
        F: FnOnce() + 'static,
    {
        let next_gen = self.generation.get().wrapping_add(1);
        self.generation.set(next_gen);
        let generation_cell = Rc::clone(&self.generation);

        spawn(async move {
            futures_timer::Delay::new(std::time::Duration::from_millis(delay_ms)).await;
            if generation_cell.get() == next_gen {
                on_elapsed();
            }
        });
    }
}

impl Default for HideDelayTimer {
    fn default() -> Self {
        Self::new()
    }
}

pub fn try_set_pointer_capture(
    evt: &PointerData,
    mounted_el: Option<&oxidase::Element>,
) -> Option<i32> {
    foundation_set_pointer_capture(evt, mounted_el)
}

pub fn try_release_pointer_capture(
    evt_opt: Option<&PointerEvent>,
    mounted_el: Option<&oxidase::Element>,
    captured_pid: Option<i32>,
) {
    foundation_release_pointer_capture(evt_opt, mounted_el, captured_pid)
}

pub fn forward_wheel_to_viewport(
    viewport_id: &str,
    delta_x: f64,
    delta_y: f64,
    orientation: Orientation,
) -> bool {
    let Some(metrics) = get_element_scroll_metrics(viewport_id) else {
        return false;
    };

    match orientation {
        Orientation::Horizontal => {
            if delta_x == 0.0 {
                return false;
            }
            let max_scroll = (metrics.scroll_width - metrics.client_width).max(0.0);
            if max_scroll <= 0.0 {
                return false;
            }
            let next_scroll = metrics.scroll_left + delta_x;
            set_element_scroll_position(viewport_id, next_scroll, metrics.scroll_top);
            let did_scroll = get_element_scroll_metrics(viewport_id)
                .map(|after| (after.scroll_left - metrics.scroll_left).abs() > 0.1)
                .unwrap_or(false);
            let within_bounds = is_scrolling_within_scrollbar_bounds(next_scroll.abs(), max_scroll)
                || (delta_x > 0.0 && metrics.scroll_left < max_scroll - 0.5)
                || (delta_x < 0.0 && metrics.scroll_left > 0.5)
                || (delta_x < 0.0 && metrics.scroll_left > -max_scroll + 0.5);
            did_scroll || within_bounds
        }
        Orientation::Vertical | Orientation::Both => {
            if delta_y == 0.0 {
                return false;
            }
            let max_scroll = (metrics.scroll_height - metrics.client_height).max(0.0);
            if max_scroll <= 0.0 {
                return false;
            }
            let next_scroll = metrics.scroll_top + delta_y;
            set_element_scroll_position(viewport_id, metrics.scroll_left, next_scroll);
            let did_scroll = get_element_scroll_metrics(viewport_id)
                .map(|after| (after.scroll_top - metrics.scroll_top).abs() > 0.1)
                .unwrap_or(false);
            let within_bounds = is_scrolling_within_scrollbar_bounds(next_scroll, max_scroll)
                || (delta_y > 0.0 && metrics.scroll_top < max_scroll - 0.5)
                || (delta_y < 0.0 && metrics.scroll_top > 0.5);
            did_scroll || within_bounds
        }
    }
}

fn format_thumb_transform(
    current_scroll: f64,
    current_sizes: ScrollAreaSizes,
    orientation: Orientation,
    dir: Direction,
) -> String {
    let offset = get_thumb_offset_from_scroll(current_scroll, current_sizes, dir);
    match orientation {
        Orientation::Horizontal => format!("translate3d({offset}px, 0, 0)"),
        Orientation::Vertical | Orientation::Both => format!("translate3d(0, {offset}px, 0)"),
    }
}

pub fn start_unlinked_scroll_sync(
    relationships: Signal<crate::scroll_area::relationships::ScrollAreaRelationships>,
    thumb_id_sig: Signal<String>,
    orientation: Orientation,
    dir: Direction,
    sizes: Signal<ScrollAreaSizes>,
) -> WatcherGuard {
    let vid = relationships.peek().viewport_id().to_string();
    let initial_scroll = if let Some(metrics) = get_element_scroll_metrics(&vid) {
        let current_scroll = match orientation {
            Orientation::Horizontal => metrics.scroll_left,
            Orientation::Vertical | Orientation::Both => metrics.scroll_top,
        };
        let tid = thumb_id_sig.peek().clone();
        let transform = format_thumb_transform(current_scroll, *sizes.peek(), orientation, dir);
        set_element_style_property(&tid, "transform", &transform);
        current_scroll
    } else {
        f64::NAN
    };

    let prev_scroll = Rc::new(Cell::new(initial_scroll));
    start_element_scroll_monitor(&vid, move || {
        let vid_cur = relationships.peek().viewport_id().to_string();
        let tid = thumb_id_sig.peek().clone();
        if let Some(metrics) = get_element_scroll_metrics(&vid_cur) {
            let current_scroll = match orientation {
                Orientation::Horizontal => metrics.scroll_left,
                Orientation::Vertical | Orientation::Both => metrics.scroll_top,
            };
            if current_scroll != prev_scroll.get() {
                prev_scroll.set(current_scroll);
                let transform =
                    format_thumb_transform(current_scroll, *sizes.peek(), orientation, dir);
                set_element_style_property(&tid, "transform", &transform);
            }
        }
    })
}

pub fn spawn_unlinked_scroll_sync(
    relationships: Signal<crate::scroll_area::relationships::ScrollAreaRelationships>,
    thumb_id_sig: Signal<String>,
    orientation: Orientation,
    dir: Direction,
    sizes: Signal<ScrollAreaSizes>,
    is_active: Rc<Cell<bool>>,
) {
    spawn(async move {
        let mut prev_scroll = f64::NAN;
        while is_active.get() {
            oxidase::frame::next_frame().await;
            if !is_active.get() {
                break;
            }
            let vid = relationships.peek().viewport_id().to_string();
            let tid = thumb_id_sig.peek().clone();
            if let Some(metrics) = get_element_scroll_metrics(&vid) {
                let current_scroll = match orientation {
                    Orientation::Horizontal => metrics.scroll_left,
                    Orientation::Vertical | Orientation::Both => metrics.scroll_top,
                };
                if current_scroll != prev_scroll {
                    prev_scroll = current_scroll;
                    let transform =
                        format_thumb_transform(current_scroll, *sizes.peek(), orientation, dir);
                    set_element_style_property(&tid, "transform", &transform);
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_corner_size_resets_to_zero_on_collapse() {
        let mut dom = VirtualDom::new(|| rsx! {});
        dom.rebuild_in_place();
        dom.in_scope(ScopeId::APP, || {
            let mut rt = ScrollAreaRuntime::new();
            rt.sync_corner_size(Orientation::Vertical, 14.0, 200.0);
            rt.sync_corner_size(Orientation::Horizontal, 300.0, 12.0);
            assert_eq!(rt.corner_width(), 14.0);
            assert_eq!(rt.corner_height(), 12.0);

            rt.sync_corner_size(Orientation::Vertical, 0.0, 0.0);
            rt.sync_corner_size(Orientation::Horizontal, 0.0, 0.0);
            assert_eq!(rt.corner_width(), 0.0);
            assert_eq!(rt.corner_height(), 0.0);
        });
    }
}
