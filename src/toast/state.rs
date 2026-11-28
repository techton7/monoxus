use std::ops::{Deref, DerefMut};

use crate::compat::toast::state::{ToastQueue, ToastQueueItem};

pub use crate::compat::toast::state::ToastInteractionState;

use super::types::*;

/// Reactive toast queue store delegating pure queue, height damping, pause/resume countdown,
/// expand precedence, and pointer-press sequencing to [`crate::compat::toast::state::ToastQueue`].
#[derive(Clone)]
pub struct ToastStore {
    queue: ToastQueue<ToastOptions, ToastInstant>,
}

impl Default for ToastStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for ToastStore {
    type Target = ToastQueue<ToastOptions, ToastInstant>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.queue
    }
}

impl DerefMut for ToastStore {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.queue
    }
}

impl ToastStore {
    pub fn new() -> Self {
        Self::with_config(ToastConfig::default())
    }

    pub fn with_config(config: ToastConfig) -> Self {
        Self {
            queue: ToastQueue::with_config(config),
        }
    }

    pub(crate) fn begin_viewport_pointer_press(&mut self) -> u64 {
        self.queue.begin_viewport_pointer_press()
    }

    pub(crate) fn end_viewport_pointer_press(&mut self, seq: u64) {
        self.queue.end_viewport_pointer_press(seq);
    }

    pub(crate) fn clear_viewport_pointer_press(&mut self) {
        self.queue.clear_viewport_pointer_press();
    }

    /// Returns whether at least one queued toast is currently in [`ToastPhase::Active`].
    pub fn has_active_toasts(&self) -> bool {
        self.queue.has_active_toasts()
    }

    /// Clears transient hover, focus, and swipe interaction state when no active toasts remain.
    pub fn clear_interaction_if_queue_empty(&mut self) {
        if self
            .queue
            .clear_interaction_if_queue_empty(ToastInstant::now())
        {
            super::runtime::notify_toast_timers();
        }
    }

    /// Records the measured unscaled border-box height for a toast card.
    pub fn set_toast_height(&mut self, id: ToastId, height: f64) -> bool {
        self.queue.set_toast_height(id, height)
    }

    /// Returns the measured unscaled border-box height for a toast card if recorded.
    pub fn toast_height(&self, id: ToastId) -> Option<f64> {
        self.queue.toast_height(id)
    }

    /// Adds a new toast or updates/resurrects an existing toast with matching ID.
    pub fn add_toast(&mut self, item: ToastItem) -> ToastId {
        let explicit_id = item.options.id;
        let (assigned_id, dirty) =
            self.queue
                .add_toast(ToastQueueItem::from(item), explicit_id, ToastInstant::now());
        if dirty {
            super::runtime::notify_toast_timers();
        }
        assigned_id
    }

    /// Updates fields on an existing toast in place.
    pub fn update_toast<F: FnOnce(&mut ToastQueueItem<ToastOptions, ToastInstant>)>(
        &mut self,
        id: ToastId,
        updater: F,
    ) -> bool {
        if self.queue.update_toast(id, updater) {
            super::runtime::notify_toast_timers();
            true
        } else {
            false
        }
    }

    /// Marks a toast as dismissing (`data-state="closed"`).
    pub fn dismiss_toast(&mut self, id: ToastId) -> bool {
        let dismissed = self
            .queue
            .dismiss_toast_with(id, ToastInstant::now(), |opts| {
                if let Some(cb) = &opts.on_dismiss {
                    cb.call(());
                }
            });
        if dismissed {
            super::runtime::notify_toast_timers();
        }
        dismissed
    }

    /// Evicts a toast completely from the active store.
    pub fn remove_toast(&mut self, id: ToastId) -> Option<ToastItem> {
        let (removed, dirty) = self.queue.remove_toast(id, ToastInstant::now());
        if dirty {
            super::runtime::notify_toast_timers();
        }
        removed.map(ToastItem::from)
    }

    /// Evicts a toast from the store only if it is currently in `ToastPhase::Dismissing`.
    pub fn remove_toast_if_dismissing(&mut self, id: ToastId) -> Option<ToastItem> {
        let (removed, dirty) = self
            .queue
            .remove_toast_if_dismissing(id, ToastInstant::now());
        if dirty {
            super::runtime::notify_toast_timers();
        }
        removed.map(ToastItem::from)
    }

    /// Evicts a toast from the store only if it is in `ToastPhase::Dismissing` and matches `generation`.
    pub fn remove_toast_if_dismissing_generation(
        &mut self,
        id: ToastId,
        generation: u64,
    ) -> Option<ToastItem> {
        let (removed, dirty) = self.queue.remove_toast_if_dismissing_generation(
            id,
            generation,
            ToastInstant::now(),
        );
        if dirty {
            super::runtime::notify_toast_timers();
        }
        removed.map(ToastItem::from)
    }

    /// Transitions all active toasts to dismissing.
    pub fn dismiss_all(&mut self) {
        let _ = self
            .queue
            .dismiss_all_with(ToastInstant::now(), |opts| {
                if let Some(cb) = &opts.on_dismiss {
                    cb.call(());
                }
            });
        super::runtime::notify_toast_timers();
    }

    /// Pauses all auto-dismiss countdown timers, recording elapsed time.
    pub fn pause_all(&mut self) {
        if self.queue.pause_all(ToastInstant::now()) {
            super::runtime::notify_toast_timers();
        }
    }

    /// Resumes all auto-dismiss countdown timers with their remaining durations.
    pub fn resume_all(&mut self) {
        if self.queue.resume_all(ToastInstant::now()) {
            super::runtime::notify_toast_timers();
        }
    }

    /// Synchronizes paused and expanded state against active interaction flags.
    pub fn sync_interaction_state(&mut self) {
        if self.queue.sync_interaction_state(ToastInstant::now()) {
            super::runtime::notify_toast_timers();
        }
    }

    /// Sets whether the document/tab is currently hidden.
    pub fn set_page_hidden(&mut self, hidden: bool) {
        if self.queue.set_page_hidden(hidden, ToastInstant::now()) {
            super::runtime::notify_toast_timers();
        }
    }

    /// Sets whether the toaster viewport is currently hovered by pointer.
    pub fn set_hovered(&mut self, hovered: bool) {
        if self.queue.set_hovered(hovered, ToastInstant::now()) {
            super::runtime::notify_toast_timers();
        }
    }

    /// Handles pointer leaving the viewport while preserving hover lock during an active swipe.
    pub(crate) fn handle_viewport_mouse_leave(&mut self) {
        if self.queue.handle_viewport_mouse_leave(ToastInstant::now()) {
            super::runtime::notify_toast_timers();
        }
    }

    /// Sets whether keyboard focus is currently inside the toaster viewport.
    pub fn set_focused(&mut self, focused: bool) {
        if self.queue.set_focused(focused, ToastInstant::now()) {
            super::runtime::notify_toast_timers();
        }
    }

    /// Sets whether a toast is actively undergoing a touch/pointer swipe gesture.
    pub fn set_swiping(&mut self, swiping: bool) {
        if self.queue.set_swiping(swiping, ToastInstant::now()) {
            super::runtime::notify_toast_timers();
        }
    }

    /// Evaluates visibility for all queued toasts without dropping any items.
    pub fn visible_items(&self) -> Vec<(&ToastQueueItem<ToastOptions, ToastInstant>, bool)> {
        self.queue.visible_items()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_pointer_press_sequence_guards_stale_release_and_clears_on_empty() {
        let mut store = ToastStore::new();
        assert!(!store.pointer_down_in_viewport);
        assert_eq!(store.pointer_down_seq, 0);

        let seq1 = store.begin_viewport_pointer_press();
        assert_eq!(seq1, 1);
        assert!(store.pointer_down_in_viewport);

        let seq2 = store.begin_viewport_pointer_press();
        assert_eq!(seq2, 2);
        assert!(store.pointer_down_in_viewport);

        store.end_viewport_pointer_press(seq1);
        assert!(store.pointer_down_in_viewport);

        store.end_viewport_pointer_press(seq2);
        assert!(!store.pointer_down_in_viewport);

        let seq3 = store.begin_viewport_pointer_press();
        assert_eq!(seq3, 3);
        store.clear_viewport_pointer_press();
        assert!(!store.pointer_down_in_viewport);
        assert_eq!(store.pointer_down_seq, 4);

        let seq5 = store.begin_viewport_pointer_press();
        assert_eq!(seq5, 5);
        store.clear_interaction_if_queue_empty();
        assert!(!store.pointer_down_in_viewport);
        assert_eq!(store.pointer_down_seq, 6);
    }
}
