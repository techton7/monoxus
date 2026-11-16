use super::types::*;
use std::collections::BTreeMap;

/// Composite interaction tracking determining pause and expand state precedence.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ToastInteractionState {
    pub page_hidden: bool,
    pub hovered: bool,
    pub focused: bool,
    pub swiping: bool,
}

impl ToastInteractionState {
    #[inline]
    pub fn should_pause(&self) -> bool {
        self.page_hidden || self.hovered || self.focused || self.swiping
    }

    #[inline]
    pub fn should_expand(&self) -> bool {
        self.hovered || self.focused || self.swiping
    }
}

/// Pure reactive state store managing the uncapped toast queue and timers.
#[derive(Clone)]
pub struct ToastStore {
    pub toasts: Vec<ToastItem>,
    pub toast_heights: BTreeMap<ToastId, f64>,
    pub config: ToastConfig,
    pub paused: bool,
    pub expanded: bool,
    pub interaction_state: ToastInteractionState,
    pub next_id: u64,
    pub last_focused_element_id: Option<String>,
    pub(crate) pointer_down_in_viewport: bool,
    pub(crate) pointer_down_seq: u64,
    deferred_hover_leave: bool,
}

impl Default for ToastStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ToastStore {
    pub fn new() -> Self {
        Self::with_config(ToastConfig::default())
    }

    pub fn with_config(config: ToastConfig) -> Self {
        let expanded = config.expand;
        Self {
            toasts: Vec::new(),
            toast_heights: BTreeMap::new(),
            config,
            paused: false,
            expanded,
            interaction_state: ToastInteractionState::default(),
            next_id: 0,
            last_focused_element_id: None,
            pointer_down_in_viewport: false,
            pointer_down_seq: 0,
            deferred_hover_leave: false,
        }
    }

    pub(crate) fn begin_viewport_pointer_press(&mut self) -> u64 {
        self.pointer_down_seq = self.pointer_down_seq.wrapping_add(1);
        self.pointer_down_in_viewport = true;
        self.pointer_down_seq
    }

    pub(crate) fn end_viewport_pointer_press(&mut self, seq: u64) {
        if self.pointer_down_seq == seq {
            self.pointer_down_in_viewport = false;
        }
    }

    pub(crate) fn clear_viewport_pointer_press(&mut self) {
        self.pointer_down_seq = self.pointer_down_seq.wrapping_add(1);
        self.pointer_down_in_viewport = false;
    }

    /// Returns whether at least one queued toast is currently in [`ToastPhase::Active`].
    pub fn has_active_toasts(&self) -> bool {
        self.toasts.iter().any(|t| t.phase == ToastPhase::Active)
    }

    /// Clears transient hover, focus, and swipe interaction state when no active toasts remain.
    pub fn clear_interaction_if_queue_empty(&mut self) {
        if !self.has_active_toasts() {
            self.interaction_state.hovered = false;
            self.interaction_state.focused = false;
            self.interaction_state.swiping = false;
            self.deferred_hover_leave = false;
            self.clear_viewport_pointer_press();
            self.sync_interaction_state();
        }
    }

    /// Records the measured unscaled border-box height for a toast card.
    ///
    /// Ignores non-positive heights, unmounted IDs, and sub-half-pixel jitter (`<= 0.5px`)
    /// so reactive layout passes cannot trigger infinite resize loops.
    pub fn set_toast_height(&mut self, id: ToastId, height: f64) -> bool {
        if !height.is_finite() || height <= 0.0 || !self.toasts.iter().any(|t| t.id == id) {
            return false;
        }
        if let Some(&existing_h) = self.toast_heights.get(&id)
            && (existing_h - height).abs() <= 0.5
        {
            return false;
        }
        self.toast_heights.insert(id, height);
        true
    }

    /// Returns the measured unscaled border-box height for a toast card if recorded.
    pub fn toast_height(&self, id: ToastId) -> Option<f64> {
        self.toast_heights.get(&id).copied()
    }

    /// Adds a new toast or updates/resurrects an existing toast with matching ID.
    ///
    /// Always prepends new items to index 0 (deterministic newest-first order).
    /// If an item with matching ID already exists, it is updated in place and
    /// resurrected to `ToastPhase::Active` without creating a duplicate record.
    pub fn add_toast(&mut self, mut item: ToastItem) -> ToastId {
        let lookup_id = item.options.id.unwrap_or(item.id);
        let now = ToastInstant::now();

        if let Some(existing) = self.toasts.iter_mut().find(|t| t.id == lookup_id) {
            existing.generation = existing.generation.wrapping_add(1);
            existing.content_generation = existing.content_generation.wrapping_add(1);
            existing.title = item.title;
            existing.description = item.description;
            existing.toast_type = item.toast_type;
            existing.phase = ToastPhase::Active;
            existing.duration = item.duration;
            existing.remaining_duration = item.duration;
            existing.created_at = now;
            existing.last_started_at = now;
            existing.paused_at = if self.paused { Some(now) } else { None };
            existing.options = item.options;
            let id = existing.id;
            super::runtime::notify_toast_timers();
            return id;
        }

        if item.id.0 == 0 {
            self.next_id += 1;
            item.id = ToastId(self.next_id);
        } else if item.id.0 > self.next_id {
            self.next_id = item.id.0;
        }

        item.last_started_at = now;
        if self.paused {
            item.paused_at = Some(now);
        }

        let assigned_id = item.id;
        self.toast_heights.remove(&assigned_id);
        self.toasts.insert(0, item);
        super::runtime::notify_toast_timers();
        assigned_id
    }

    /// Updates fields on an existing toast in place.
    pub fn update_toast<F: FnOnce(&mut ToastItem)>(&mut self, id: ToastId, updater: F) -> bool {
        if let Some(item) = self.toasts.iter_mut().find(|t| t.id == id) {
            updater(item);
            item.content_generation = item.content_generation.wrapping_add(1);
            super::runtime::notify_toast_timers();
            true
        } else {
            false
        }
    }

    /// Marks a toast as dismissing (data-state="closed").
    ///
    /// The item remains in the queue to permit exit animations until `remove_toast` is called.
    pub fn dismiss_toast(&mut self, id: ToastId) -> bool {
        let mut dismissed = false;
        if let Some(item) = self
            .toasts
            .iter_mut()
            .find(|t| t.id == id && t.phase == ToastPhase::Active)
        {
            item.phase = ToastPhase::Dismissing;
            if let Some(cb) = &item.options.on_dismiss {
                cb.call(());
            }
            dismissed = true;
        }
        if dismissed {
            if !self.has_active_toasts() {
                self.clear_interaction_if_queue_empty();
            }
            super::runtime::notify_toast_timers();
        }
        dismissed
    }

    /// Evicts a toast completely from the active store.
    pub fn remove_toast(&mut self, id: ToastId) -> Option<ToastItem> {
        self.toast_heights.remove(&id);
        let removed = self
            .toasts
            .iter()
            .position(|t| t.id == id)
            .map(|idx| self.toasts.remove(idx));
        self.clear_interaction_if_queue_empty();
        if removed.is_some() {
            super::runtime::notify_toast_timers();
        }
        removed
    }

    /// Evicts a toast from the store only if it is currently in `ToastPhase::Dismissing`.
    ///
    /// Prevents entrance `animationend`/`transitionend` events or stale timers from evicting
    /// active or resurrected toasts.
    pub fn remove_toast_if_dismissing(&mut self, id: ToastId) -> Option<ToastItem> {
        let removed = if let Some(idx) = self
            .toasts
            .iter()
            .position(|t| t.id == id && t.phase == ToastPhase::Dismissing)
        {
            self.toast_heights.remove(&id);
            Some(self.toasts.remove(idx))
        } else {
            None
        };
        self.clear_interaction_if_queue_empty();
        if removed.is_some() {
            super::runtime::notify_toast_timers();
        }
        removed
    }

    /// Evicts a toast from the store only if it is in `ToastPhase::Dismissing` and matches
    /// the lifecycle `generation` recorded when dismissal began.
    pub fn remove_toast_if_dismissing_generation(
        &mut self,
        id: ToastId,
        generation: u64,
    ) -> Option<ToastItem> {
        let removed = if let Some(idx) = self.toasts.iter().position(|t| {
            t.id == id && t.phase == ToastPhase::Dismissing && t.generation == generation
        }) {
            self.toast_heights.remove(&id);
            Some(self.toasts.remove(idx))
        } else {
            None
        };
        self.clear_interaction_if_queue_empty();
        if removed.is_some() {
            super::runtime::notify_toast_timers();
        }
        removed
    }

    /// Transitions all active toasts to dismissing.
    pub fn dismiss_all(&mut self) {
        for item in &mut self.toasts {
            if item.phase == ToastPhase::Active {
                item.phase = ToastPhase::Dismissing;
                if let Some(cb) = &item.options.on_dismiss {
                    cb.call(());
                }
            }
        }
        self.clear_interaction_if_queue_empty();
        super::runtime::notify_toast_timers();
    }

    /// Pauses all auto-dismiss countdown timers, recording elapsed time.
    pub fn pause_all(&mut self) {
        if self.paused {
            return;
        }
        self.paused = true;
        let now = ToastInstant::now();
        for item in &mut self.toasts {
            if item.phase == ToastPhase::Active && item.paused_at.is_none() {
                let elapsed = now.saturating_duration_since(item.last_started_at);
                item.remaining_duration = item.remaining_duration.saturating_sub(elapsed);
                item.paused_at = Some(now);
            }
        }
        super::runtime::notify_toast_timers();
    }

    /// Resumes all auto-dismiss countdown timers with their remaining durations.
    pub fn resume_all(&mut self) {
        if !self.paused {
            return;
        }
        self.paused = false;
        let now = ToastInstant::now();
        for item in &mut self.toasts {
            if item.phase == ToastPhase::Active {
                item.last_started_at = now;
                item.paused_at = None;
            }
        }
        super::runtime::notify_toast_timers();
    }

    /// Synchronizes paused and expanded state against active interaction flags.
    pub fn sync_interaction_state(&mut self) {
        let should_pause = self.interaction_state.should_pause();
        let should_expand = self.config.expand || self.interaction_state.should_expand();
        self.expanded = should_expand;
        if should_pause {
            self.pause_all();
        } else {
            self.resume_all();
        }
    }

    /// Sets whether the document/tab is currently hidden.
    pub fn set_page_hidden(&mut self, hidden: bool) {
        self.interaction_state.page_hidden = hidden;
        self.sync_interaction_state();
    }

    /// Sets whether the toaster viewport is currently hovered by pointer.
    pub fn set_hovered(&mut self, hovered: bool) {
        if hovered {
            self.deferred_hover_leave = false;
        }
        self.interaction_state.hovered = hovered;
        self.sync_interaction_state();
    }

    /// Handles pointer leaving the viewport while preserving hover lock during an active swipe.
    pub(crate) fn handle_viewport_mouse_leave(&mut self) {
        if self.interaction_state.swiping {
            self.deferred_hover_leave = true;
        } else {
            self.deferred_hover_leave = false;
            self.set_hovered(false);
        }
    }

    /// Sets whether keyboard focus is currently inside the toaster viewport.
    pub fn set_focused(&mut self, focused: bool) {
        self.interaction_state.focused = focused;
        self.sync_interaction_state();
    }

    /// Sets whether a toast is actively undergoing a touch/pointer swipe gesture.
    pub fn set_swiping(&mut self, swiping: bool) {
        self.interaction_state.swiping = swiping;
        if !swiping && self.deferred_hover_leave {
            self.deferred_hover_leave = false;
            self.interaction_state.hovered = false;
        }
        self.sync_interaction_state();
    }

    /// Evaluates visibility for all queued toasts without dropping any items.
    ///
    /// Items at index `< config.visible_toasts` receive `visible = true`.
    /// Items at index `>= config.visible_toasts` receive `visible = false`.
    pub fn visible_items(&self) -> Vec<(&ToastItem, bool)> {
        self.toasts
            .iter()
            .enumerate()
            .map(|(idx, item)| (item, idx < self.config.visible_toasts))
            .collect()
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

        // Starting a second pointer press before seq1's deferred frame callback runs
        let seq2 = store.begin_viewport_pointer_press();
        assert_eq!(seq2, 2);
        assert!(store.pointer_down_in_viewport);

        // Stale seq1 release must NOT clear the active seq2 press
        store.end_viewport_pointer_press(seq1);
        assert!(store.pointer_down_in_viewport);

        // Matching seq2 release clears pointer_down_in_viewport
        store.end_viewport_pointer_press(seq2);
        assert!(!store.pointer_down_in_viewport);

        // clear_viewport_pointer_press bumps seq and clears flag immediately
        let seq3 = store.begin_viewport_pointer_press();
        assert_eq!(seq3, 3);
        store.clear_viewport_pointer_press();
        assert!(!store.pointer_down_in_viewport);
        assert_eq!(store.pointer_down_seq, 4);

        // clear_interaction_if_queue_empty also invalidates pending seq
        let seq5 = store.begin_viewport_pointer_press();
        assert_eq!(seq5, 5);
        store.clear_interaction_if_queue_empty();
        assert!(!store.pointer_down_in_viewport);
        assert_eq!(store.pointer_down_seq, 6);
    }
}
