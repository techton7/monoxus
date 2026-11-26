use dioxus::prelude::*;

use crate::foundation::browser::WatcherGuard;

/// Zero-overhead `Copy` slot managing a single [`WatcherGuard`] with monotonic generation
/// invalidation and Dioxus `BorrowMutError`-safe drop strictly outside the signal write lock.
///
/// Relies on `oxidase v0.4.7`'s re-entrant `Closure<dyn Fn>` + `try_borrow_mut()` and
/// `WatcherGuard::drop()` eval-skip guarantees (no `Delay(ZERO)` deferral needed).
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct WatcherSlot {
    guard: Signal<Option<WatcherGuard>>,
    generation: Signal<u64>,
}

impl WatcherSlot {
    /// Returns the current monotonic generation counter without subscribing.
    pub fn generation(&self) -> u64 {
        self.generation.try_peek().map(|g| *g).unwrap_or(0)
    }

    /// Bumps the generation counter and drops any active guard strictly outside the signal borrow.
    pub fn clear(&self) -> u64 {
        let mut gen_sig = self.generation;
        let next = gen_sig
            .try_write()
            .map(|mut g| {
                *g = g.wrapping_add(1);
                *g
            })
            .unwrap_or(0);
        let mut guard_sig = self.guard;
        let old = guard_sig.try_write().ok().and_then(|mut w| w.take());
        drop(old);
        next
    }

    /// Replaces the active guard with `next_guard`, dropping any previous guard outside the signal
    /// borrow, and returns the new monotonic generation token.
    #[allow(dead_code)]
    pub fn replace(&self, next_guard: WatcherGuard) -> u64 {
        let generation = self.clear();
        let mut guard_sig = self.guard;
        if let Ok(mut w) = guard_sig.try_write() {
            *w = Some(next_guard);
        } else {
            next_guard.stop();
        }
        generation
    }

    /// Arms a new guard produced by `make_guard(generation)` after clearing the prior guard,
    /// allowing callbacks to check `slot.generation() == generation` before mutating state.
    pub fn arm_with(&self, make_guard: impl FnOnce(u64) -> WatcherGuard) -> u64 {
        let generation = self.clear();
        let next_guard = make_guard(generation);
        let mut guard_sig = self.guard;
        if let Ok(mut w) = guard_sig.try_write() {
            *w = Some(next_guard);
        } else {
            next_guard.stop();
        }
        generation
    }

    /// Clears the slot only if its generation still matches `expected_gen`.
    #[allow(dead_code)]
    pub fn clear_if_generation(&self, expected_gen: u64) -> bool {
        if self.generation() == expected_gen {
            self.clear();
            true
        } else {
            false
        }
    }
}

/// Allocates a [`WatcherSlot`] that automatically clears and stops its active guard on unmount.
pub(crate) fn use_watcher_slot() -> WatcherSlot {
    let slot = WatcherSlot {
        guard: use_signal(|| None),
        generation: use_signal(|| 0_u64),
    };
    use_drop(move || {
        slot.clear();
    });
    slot
}
