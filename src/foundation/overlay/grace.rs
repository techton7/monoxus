use std::{cell::RefCell, rc::Rc, time::Duration};

use dioxus::prelude::*;
use futures_timer::Delay;

pub use ui_sdk::foundation::overlay::grace::*;

use crate::foundation::browser::{spawn_root_task, start_document_pointer_stream};

use super::watcher::{WatcherSlot, use_watcher_slot};

/// Zero-overhead `Copy` cancellable timer handle backed by [`CopyValue`] generation tokens
/// and automatic `use_drop` cancellation.
#[derive(Clone, Copy, PartialEq)]
pub struct CancellableDelay {
    generation: CopyValue<u64>,
    task: CopyValue<Option<dioxus::core::Task>>,
    alive: CopyValue<bool>,
}

impl CancellableDelay {
    pub fn is_pending(&self) -> bool {
        self.task.try_read().is_ok_and(|slot| slot.is_some())
    }

    pub fn generation(&self) -> u64 {
        self.generation.try_read().map(|g| *g).unwrap_or(0)
    }

    /// Bumps the generation token and cancels any pending task immediately.
    pub fn cancel(&self) -> u64 {
        let mut gen_slot = self.generation;
        let next = gen_slot
            .try_write()
            .map(|mut g| {
                *g = g.wrapping_add(1);
                *g
            })
            .unwrap_or(0);
        let mut task_slot = self.task;
        let old = task_slot.try_write().ok().and_then(|mut t| t.take());
        if let Some(task) = old {
            task.cancel();
        }
        next
    }

    /// Cancels any previous timer and schedules `on_fire` after `delay_ms` milliseconds.
    pub fn schedule(&self, delay_ms: u64, on_fire: impl FnOnce() + 'static) -> u64 {
        let expected_gen = self.cancel();
        let handle = *self;
        let task = spawn_root_task(async move {
            Delay::new(Duration::from_millis(delay_ms)).await;
            if !handle.alive.try_read().is_ok_and(|a| *a)
                || handle.generation() != expected_gen
            {
                return;
            }
            let mut task_slot = handle.task;
            let _ = task_slot.try_write().map(|mut t| {
                *t = None;
            });
            on_fire();
        });
        let mut task_slot = self.task;
        if let Ok(mut slot) = task_slot.try_write() {
            *slot = Some(task);
        } else {
            task.cancel();
        }
        expected_gen
    }

    /// Cancels any previous timer and schedules `on_fire` on the next animation frame.
    pub fn schedule_next_frame(&self, on_fire: impl FnOnce() + 'static) -> u64 {
        let expected_gen = self.cancel();
        let handle = *self;
        let task = spawn_root_task(async move {
            let _ = oxidase::frame::next_frame().await;
            if !handle.alive.try_read().is_ok_and(|a| *a)
                || handle.generation() != expected_gen
            {
                return;
            }
            let mut task_slot = handle.task;
            let _ = task_slot.try_write().map(|mut t| {
                *t = None;
            });
            on_fire();
        });
        let mut task_slot = self.task;
        if let Ok(mut slot) = task_slot.try_write() {
            *slot = Some(task);
        } else {
            task.cancel();
        }
        expected_gen
    }
}

/// Allocates a [`CancellableDelay`] that cancels automatically on component drop.
pub fn use_cancellable_delay() -> CancellableDelay {
    let generation = use_hook(|| CopyValue::new(0_u64));
    let task = use_hook(|| CopyValue::new(Option::<dioxus::core::Task>::None));
    let mut alive = use_hook(|| CopyValue::new(true));

    let handle = CancellableDelay {
        generation,
        task,
        alive,
    };

    use_drop(move || {
        if let Ok(mut flag) = alive.try_write() {
            *flag = false;
        }
        handle.cancel();
    });

    handle
}

/// Outcome emitted when an active [`GraceCorridorHandle`] completes, progresses, or terminates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GraceCorridorOutcome {
    InTransit(Point2D),
    ReachedTarget(Point2D),
    ExitedCorridor(Point2D),
    Stalled,
}

/// Computes Euclidean distance from `point` to the nearest edge of `rect` (`0.0` when inside).
pub fn point_to_rect_distance(point: Point2D, rect: &Rect2D) -> f64 {
    let dx = if point.x < rect.left() {
        rect.left() - point.x
    } else if point.x > rect.right() {
        point.x - rect.right()
    } else {
        0.0
    };
    let dy = if point.y < rect.top() {
        rect.top() - point.y
    } else if point.y > rect.bottom() {
        point.y - rect.bottom()
    } else {
        0.0
    };
    (dx * dx + dy * dy).sqrt()
}

/// Zero-overhead `Copy` grace corridor handle backed by [`WatcherSlot`], [`CancellableDelay`],
/// and [`GraceTransit`].
///
/// Implements:
/// - monotonic progress timer re-arming (`SUBMENU_INTENT_MIN_PROGRESS_PX = 1.0`, `bits#2192`),
/// - `last_pointer_x`/`last_pointer_y` tracking for coalesced `pointerleave` direction
///   recomputation (`radix menu.tsx:488`),
/// - re-entrant self-stopping via `WatcherSlot` without `Delay(ZERO)` workarounds (`oxidase v0.4.7`).
#[derive(Clone, Copy, PartialEq)]
pub struct GraceCorridorHandle {
    watcher: WatcherSlot,
    timeout_delay: CancellableDelay,
    transit: Signal<Option<GraceTransit>>,
    last_pointer_x: CopyValue<Option<f64>>,
    last_pointer_y: CopyValue<Option<f64>>,
    best_progress_dist: CopyValue<Option<f64>>,
    alive: CopyValue<bool>,
}

impl GraceCorridorHandle {
    pub fn is_active(&self) -> bool {
        self.transit.try_peek().is_ok_and(|t| t.is_some())
    }

    /// Records the latest observed pointer coordinates for coalesced `pointerleave` direction
    /// recomputation (`radix menu.tsx:488`).
    pub fn record_pointer(&self, x: f64, y: f64) {
        let mut lx = self.last_pointer_x;
        if let Ok(mut w) = lx.try_write() {
            *w = Some(x);
        }
        let mut ly = self.last_pointer_y;
        if let Ok(mut w) = ly.try_write() {
            *w = Some(y);
        }
    }

    pub fn last_pointer(&self) -> Option<Point2D> {
        let x = self.last_pointer_x.try_read().ok().and_then(|g| *g)?;
        let y = self.last_pointer_y.try_read().ok().and_then(|g| *g)?;
        Some(Point2D::new(x, y))
    }

    /// Returns `client_x - last_pointer_x` so coalesced `pointerleave` events can recompute
    /// horizontal exit direction accurately (`radix menu.tsx:488`).
    pub fn horizontal_leave_delta(&self, client_x: f64) -> Option<f64> {
        self.last_pointer_x
            .try_read()
            .ok()
            .and_then(|g| *g)
            .map(|prev_x| client_x - prev_x)
    }

    /// Stops the pointer stream, cancels any stall timer, and clears active corridor geometry.
    pub fn clear(&self) {
        self.timeout_delay.cancel();
        self.watcher.clear();
        let mut best = self.best_progress_dist;
        if let Ok(mut w) = best.try_write() {
            *w = None;
        }
        let mut transit_sig = self.transit;
        if transit_sig.try_peek().is_ok_and(|t| t.is_some())
            && let Ok(mut w) = transit_sig.try_write()
        {
            *w = None;
        }
    }

    /// Starts monitoring pointer movement through `transit`.
    ///
    /// When `stall_timeout_ms` is `Some(ms)`, arms a stall timer that re-arms whenever pointer
    /// progress toward `transit.to_rect` advances by at least [`SUBMENU_INTENT_MIN_PROGRESS_PX`]
    /// (`1.0px`, `bits#2192`), and emits [`GraceCorridorOutcome::Stalled`] if the pointer stalls.
    pub fn start(
        &self,
        transit: GraceTransit,
        stall_timeout_ms: Option<u64>,
        on_outcome: impl FnMut(GraceCorridorOutcome) + 'static,
    ) {
        self.clear();

        let initial_dist = point_to_rect_distance(transit.exit_point, &transit.to_rect);
        let mut best = self.best_progress_dist;
        if let Ok(mut w) = best.try_write() {
            *w = Some(initial_dist);
        }
        let mut transit_sig = self.transit;
        if let Ok(mut w) = transit_sig.try_write() {
            *w = Some(transit.clone());
        }

        let outcome_cb: Rc<RefCell<Box<dyn FnMut(GraceCorridorOutcome)>>> =
            Rc::new(RefCell::new(Box::new(on_outcome)));

        let handle = *self;
        if let Some(ms) = stall_timeout_ms {
            arm_stall_timer(handle, ms, Rc::clone(&outcome_cb));
        }

        let stream_cb = Rc::clone(&outcome_cb);
        self.watcher.arm_with(move |expected_gen| {
            start_document_pointer_stream(move |point| {
                if !handle.alive.try_read().is_ok_and(|a| *a)
                    || handle.watcher.generation() != expected_gen
                {
                    return;
                }
                handle.record_pointer(point.x, point.y);

                let Some(active_transit) = handle.transit.try_peek().ok().and_then(|t| t.clone())
                else {
                    handle.clear();
                    return;
                };

                match active_transit.inspect_point(point) {
                    GraceTransitStatus::ReachedTarget => {
                        handle.clear();
                        if let Ok(mut cb) = stream_cb.try_borrow_mut() {
                            cb(GraceCorridorOutcome::ReachedTarget(point));
                        }
                    }
                    GraceTransitStatus::ExitedCorridor => {
                        handle.clear();
                        if let Ok(mut cb) = stream_cb.try_borrow_mut() {
                            cb(GraceCorridorOutcome::ExitedCorridor(point));
                        }
                    }
                    GraceTransitStatus::InTransit => {
                        if let Ok(mut cb) = stream_cb.try_borrow_mut() {
                            cb(GraceCorridorOutcome::InTransit(point));
                        }
                        if let Some(ms) = stall_timeout_ms {
                            let dist = point_to_rect_distance(point, &active_transit.to_rect);
                            let prev_best = handle
                                .best_progress_dist
                                .try_read()
                                .ok()
                                .and_then(|g| *g)
                                .unwrap_or(f64::INFINITY);
                            if prev_best - dist >= SUBMENU_INTENT_MIN_PROGRESS_PX {
                                let mut best_slot = handle.best_progress_dist;
                                if let Ok(mut w) = best_slot.try_write() {
                                    *w = Some(dist);
                                }
                                arm_stall_timer(handle, ms, Rc::clone(&stream_cb));
                            }
                        }
                    }
                }
            })
            .into_watcher_guard()
        });
    }
}

fn arm_stall_timer(
    handle: GraceCorridorHandle,
    ms: u64,
    cb_cell: Rc<RefCell<Box<dyn FnMut(GraceCorridorOutcome)>>>,
) {
    handle.timeout_delay.schedule(ms, move || {
        if !handle.alive.try_read().is_ok_and(|a| *a) || !handle.is_active() {
            return;
        }
        handle.clear();
        if let Ok(mut cb) = cb_cell.try_borrow_mut() {
            cb(GraceCorridorOutcome::Stalled);
        }
    });
}

/// Allocates a [`GraceCorridorHandle`] with automatic `use_drop` cleanup.
pub fn use_grace_corridor() -> GraceCorridorHandle {
    let watcher = use_watcher_slot();
    let timeout_delay = use_cancellable_delay();
    let transit = use_signal(|| None);
    let last_pointer_x = use_hook(|| CopyValue::new(Option::<f64>::None));
    let last_pointer_y = use_hook(|| CopyValue::new(Option::<f64>::None));
    let best_progress_dist = use_hook(|| CopyValue::new(Option::<f64>::None));
    let mut alive = use_hook(|| CopyValue::new(true));

    let handle = GraceCorridorHandle {
        watcher,
        timeout_delay,
        transit,
        last_pointer_x,
        last_pointer_y,
        best_progress_dist,
        alive,
    };

    use_drop(move || {
        if let Ok(mut flag) = alive.try_write() {
            *flag = false;
        }
        handle.clear();
    });

    handle
}

#[cfg(test)]
mod tests {
    use super::{Point2D, Rect2D, SUBMENU_INTENT_MIN_PROGRESS_PX, point_to_rect_distance};

    #[test]
    fn point_to_rect_distance_measures_progress_toward_submenu_target() {
        let target = Rect2D::new(200.0, 100.0, 120.0, 80.0);
        assert_eq!(point_to_rect_distance(Point2D::new(250.0, 140.0), &target), 0.0);

        let d0 = point_to_rect_distance(Point2D::new(150.0, 140.0), &target);
        let d_stall = point_to_rect_distance(Point2D::new(150.5, 140.0), &target);
        let d_progress = point_to_rect_distance(Point2D::new(152.0, 140.0), &target);

        assert!((d0 - 50.0).abs() < 1e-9);
        assert!(d0 - d_stall < SUBMENU_INTENT_MIN_PROGRESS_PX);
        assert!(d0 - d_progress >= SUBMENU_INTENT_MIN_PROGRESS_PX);
    }
}
