//! Pure host-agnostic toast queue and interaction state machine.

use std::collections::BTreeMap;
use std::time::Duration;

use super::types::{ToastConfig, ToastId, ToastPhase, ToastType};

/// Monotonic instant contract enabling pure host-neutral countdown calculations.
pub trait ToastClockInstant: Copy {
    /// Returns elapsed duration since `earlier`, saturating at zero if `self < earlier`.
    fn saturating_duration_since(&self, earlier: Self) -> Duration;
}

impl ToastClockInstant for u64 {
    #[inline]
    fn saturating_duration_since(&self, earlier: Self) -> Duration {
        Duration::from_millis(self.saturating_sub(earlier))
    }
}

/// Composite interaction tracking determining pause and expand state precedence.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ToastInteractionState {
    pub page_hidden: bool,
    pub hovered: bool,
    pub focused: bool,
    pub swiping: bool,
}

impl ToastInteractionState {
    #[inline]
    pub const fn should_pause(&self) -> bool {
        self.page_hidden || self.hovered || self.focused || self.swiping
    }

    #[inline]
    pub const fn should_expand(&self) -> bool {
        self.hovered || self.focused || self.swiping
    }
}

/// Pure queued toast record parameterized over host-specific metadata and monotonic clock instant.
#[derive(Clone, Debug, PartialEq)]
pub struct ToastQueueItem<TMeta, TInstant> {
    pub id: ToastId,
    pub generation: u64,
    pub content_generation: u64,
    pub toast_type: ToastType,
    pub title: String,
    pub description: Option<String>,
    pub phase: ToastPhase,
    pub duration: Duration,
    pub remaining_duration: Duration,
    pub created_at: TInstant,
    pub last_started_at: TInstant,
    pub paused_at: Option<TInstant>,
    pub meta: TMeta,
}

impl<TMeta, TInstant: Copy> ToastQueueItem<TMeta, TInstant> {
    pub fn new(
        id: ToastId,
        toast_type: ToastType,
        title: impl Into<String>,
        description: Option<String>,
        duration: Duration,
        now: TInstant,
        meta: TMeta,
    ) -> Self {
        Self {
            id,
            generation: 0,
            content_generation: 0,
            toast_type,
            title: title.into(),
            description,
            phase: ToastPhase::Active,
            duration,
            remaining_duration: duration,
            created_at: now,
            last_started_at: now,
            paused_at: None,
            meta,
        }
    }
}

/// Pure host-agnostic toast queue managing ordering, resurrection, height jitter damping,
/// viewport pointer press sequencing, pause/resume countdown accounting, and expand precedence.
///
/// Mutating transitions return `timers_dirty: bool` so host runtimes notify timer schedulers
/// only when countdown state actually changed.
#[derive(Clone, Debug)]
pub struct ToastQueue<TMeta, TInstant> {
    pub toasts: Vec<ToastQueueItem<TMeta, TInstant>>,
    pub toast_heights: BTreeMap<ToastId, f64>,
    pub config: ToastConfig,
    pub paused: bool,
    pub expanded: bool,
    pub interaction_state: ToastInteractionState,
    pub next_id: u64,
    pub last_focused_element_id: Option<String>,
    pub pointer_down_in_viewport: bool,
    pub pointer_down_seq: u64,
    deferred_hover_leave: bool,
}

impl<TMeta, TInstant> Default for ToastQueue<TMeta, TInstant> {
    fn default() -> Self {
        Self::new()
    }
}

impl<TMeta, TInstant> ToastQueue<TMeta, TInstant> {
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

    pub fn begin_viewport_pointer_press(&mut self) -> u64 {
        self.pointer_down_seq = self.pointer_down_seq.wrapping_add(1);
        self.pointer_down_in_viewport = true;
        self.pointer_down_seq
    }

    pub fn end_viewport_pointer_press(&mut self, seq: u64) {
        if self.pointer_down_seq == seq {
            self.pointer_down_in_viewport = false;
        }
    }

    pub fn clear_viewport_pointer_press(&mut self) {
        self.pointer_down_seq = self.pointer_down_seq.wrapping_add(1);
        self.pointer_down_in_viewport = false;
    }

    /// Returns whether at least one queued toast is currently in [`ToastPhase::Active`].
    pub fn has_active_toasts(&self) -> bool {
        self.toasts.iter().any(|t| t.phase == ToastPhase::Active)
    }

    /// Records the measured unscaled border-box height for a toast card.
    ///
    /// Ignores non-positive heights, unmounted IDs, and sub-half-pixel jitter (`<= 0.5px`).
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

    /// Updates fields on an existing toast in place. Returns `timers_dirty`.
    pub fn update_toast<F: FnOnce(&mut ToastQueueItem<TMeta, TInstant>)>(
        &mut self,
        id: ToastId,
        updater: F,
    ) -> bool {
        if let Some(item) = self.toasts.iter_mut().find(|t| t.id == id) {
            updater(item);
            item.content_generation = item.content_generation.wrapping_add(1);
            true
        } else {
            false
        }
    }

    /// Evaluates visibility for all queued toasts without dropping any items.
    pub fn visible_items(&self) -> Vec<(&ToastQueueItem<TMeta, TInstant>, bool)> {
        self.toasts
            .iter()
            .enumerate()
            .map(|(idx, item)| (item, idx < self.config.visible_toasts))
            .collect()
    }
}

impl<TMeta, TInstant: ToastClockInstant> ToastQueue<TMeta, TInstant> {
    /// Clears transient hover, focus, and swipe interaction state when no active toasts remain.
    /// Returns `timers_dirty`.
    pub fn clear_interaction_if_queue_empty(&mut self, now: TInstant) -> bool {
        if !self.has_active_toasts() {
            self.interaction_state.hovered = false;
            self.interaction_state.focused = false;
            self.interaction_state.swiping = false;
            self.deferred_hover_leave = false;
            self.clear_viewport_pointer_press();
            self.sync_interaction_state(now)
        } else {
            false
        }
    }

    /// Adds a new toast or updates/resurrects an existing toast matching `explicit_id.unwrap_or(item.id)`.
    ///
    /// Always prepends new items to index 0 (newest-first). Returns `(assigned_id, timers_dirty)`.
    pub fn add_toast(
        &mut self,
        mut item: ToastQueueItem<TMeta, TInstant>,
        explicit_id: Option<ToastId>,
        now: TInstant,
    ) -> (ToastId, bool) {
        let lookup_id = explicit_id.unwrap_or(item.id);

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
            existing.meta = item.meta;
            return (existing.id, true);
        }

        if let Some(explicit) = explicit_id {
            item.id = explicit;
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
        (assigned_id, true)
    }

    /// Marks a toast as dismissing (`data-state="closed"`), invoking `on_dismiss` with its metadata.
    /// Returns `true` when the toast transitioned from `Active` to `Dismissing` (`timers_dirty`).
    pub fn dismiss_toast_with<F: FnMut(&TMeta)>(
        &mut self,
        id: ToastId,
        now: TInstant,
        mut on_dismiss: F,
    ) -> bool {
        let mut dismissed = false;
        if let Some(item) = self
            .toasts
            .iter_mut()
            .find(|t| t.id == id && t.phase == ToastPhase::Active)
        {
            item.phase = ToastPhase::Dismissing;
            on_dismiss(&item.meta);
            dismissed = true;
        }
        if dismissed && !self.has_active_toasts() {
            self.clear_interaction_if_queue_empty(now);
        }
        dismissed
    }

    /// Marks a toast as dismissing without invoking an external callback. Returns `timers_dirty`.
    pub fn dismiss_toast(&mut self, id: ToastId, now: TInstant) -> bool {
        self.dismiss_toast_with(id, now, |_| {})
    }

    /// Evicts a toast completely from the active store. Returns `(removed, timers_dirty)`.
    pub fn remove_toast(
        &mut self,
        id: ToastId,
        now: TInstant,
    ) -> (Option<ToastQueueItem<TMeta, TInstant>>, bool) {
        self.toast_heights.remove(&id);
        let removed = self
            .toasts
            .iter()
            .position(|t| t.id == id)
            .map(|idx| self.toasts.remove(idx));
        let empty_dirty = self.clear_interaction_if_queue_empty(now);
        let timers_dirty = removed.is_some() || empty_dirty;
        (removed, timers_dirty)
    }

    /// Evicts a toast only if it is currently in `ToastPhase::Dismissing`.
    /// Returns `(removed, timers_dirty)`.
    pub fn remove_toast_if_dismissing(
        &mut self,
        id: ToastId,
        now: TInstant,
    ) -> (Option<ToastQueueItem<TMeta, TInstant>>, bool) {
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
        let empty_dirty = self.clear_interaction_if_queue_empty(now);
        let timers_dirty = removed.is_some() || empty_dirty;
        (removed, timers_dirty)
    }

    /// Evicts a toast only if it is in `ToastPhase::Dismissing` and matches `generation`.
    /// Returns `(removed, timers_dirty)`.
    pub fn remove_toast_if_dismissing_generation(
        &mut self,
        id: ToastId,
        generation: u64,
        now: TInstant,
    ) -> (Option<ToastQueueItem<TMeta, TInstant>>, bool) {
        let removed = if let Some(idx) = self.toasts.iter().position(|t| {
            t.id == id && t.phase == ToastPhase::Dismissing && t.generation == generation
        }) {
            self.toast_heights.remove(&id);
            Some(self.toasts.remove(idx))
        } else {
            None
        };
        let empty_dirty = self.clear_interaction_if_queue_empty(now);
        let timers_dirty = removed.is_some() || empty_dirty;
        (removed, timers_dirty)
    }

    /// Transitions all active toasts to dismissing, invoking `on_dismiss` for each.
    /// Returns `timers_dirty`.
    pub fn dismiss_all_with<F: FnMut(&TMeta)>(
        &mut self,
        now: TInstant,
        mut on_dismiss: F,
    ) -> bool {
        let mut any_dismissed = false;
        for item in &mut self.toasts {
            if item.phase == ToastPhase::Active {
                item.phase = ToastPhase::Dismissing;
                on_dismiss(&item.meta);
                any_dismissed = true;
            }
        }
        let empty_dirty = self.clear_interaction_if_queue_empty(now);
        any_dismissed || empty_dirty
    }

    /// Transitions all active toasts to dismissing. Returns `timers_dirty`.
    pub fn dismiss_all(&mut self, now: TInstant) -> bool {
        self.dismiss_all_with(now, |_| {})
    }

    /// Pauses all auto-dismiss countdown timers, recording elapsed time.
    /// Returns `true` when timers transitioned to paused (`timers_dirty`).
    pub fn pause_all(&mut self, now: TInstant) -> bool {
        if self.paused {
            return false;
        }
        self.paused = true;
        for item in &mut self.toasts {
            if item.phase == ToastPhase::Active && item.paused_at.is_none() {
                let elapsed = now.saturating_duration_since(item.last_started_at);
                item.remaining_duration = item.remaining_duration.saturating_sub(elapsed);
                item.paused_at = Some(now);
            }
        }
        true
    }

    /// Resumes all auto-dismiss countdown timers with their remaining durations.
    /// Returns `true` when timers transitioned to active (`timers_dirty`).
    pub fn resume_all(&mut self, now: TInstant) -> bool {
        if !self.paused {
            return false;
        }
        self.paused = false;
        for item in &mut self.toasts {
            if item.phase == ToastPhase::Active {
                item.last_started_at = now;
                item.paused_at = None;
            }
        }
        true
    }

    /// Synchronizes paused and expanded state against active interaction flags.
    /// Returns `timers_dirty`.
    pub fn sync_interaction_state(&mut self, now: TInstant) -> bool {
        let should_pause = self.interaction_state.should_pause();
        let should_expand = self.config.expand || self.interaction_state.should_expand();
        self.expanded = should_expand;
        if should_pause {
            self.pause_all(now)
        } else {
            self.resume_all(now)
        }
    }

    pub fn set_page_hidden(&mut self, hidden: bool, now: TInstant) -> bool {
        self.interaction_state.page_hidden = hidden;
        self.sync_interaction_state(now)
    }

    pub fn set_hovered(&mut self, hovered: bool, now: TInstant) -> bool {
        if hovered {
            self.deferred_hover_leave = false;
        }
        self.interaction_state.hovered = hovered;
        self.sync_interaction_state(now)
    }

    /// Handles pointer leaving the viewport while preserving hover lock during an active swipe.
    pub fn handle_viewport_mouse_leave(&mut self, now: TInstant) -> bool {
        if self.interaction_state.swiping {
            self.deferred_hover_leave = true;
            false
        } else {
            self.deferred_hover_leave = false;
            self.set_hovered(false, now)
        }
    }

    pub fn set_focused(&mut self, focused: bool, now: TInstant) -> bool {
        self.interaction_state.focused = focused;
        self.sync_interaction_state(now)
    }

    pub fn set_swiping(&mut self, swiping: bool, now: TInstant) -> bool {
        self.interaction_state.swiping = swiping;
        if !swiping && self.deferred_hover_leave {
            self.deferred_hover_leave = false;
            self.interaction_state.hovered = false;
        }
        self.sync_interaction_state(now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(id: u64, title: &str, now_ms: u64) -> ToastQueueItem<&'static str, u64> {
        ToastQueueItem::new(
            ToastId(id),
            ToastType::Default,
            title,
            None,
            Duration::from_millis(4000),
            now_ms,
            "meta",
        )
    }

    #[test]
    fn add_resurrect_dismiss_and_generation_guarded_removal() {
        let mut queue = ToastQueue::<&'static str, u64>::new();
        let (id1, dirty) = queue.add_toast(make_item(0, "First", 100), None, 100);
        assert!(dirty);
        assert_eq!(id1, ToastId(1));

        let (id2, _) = queue.add_toast(make_item(0, "Second", 200), None, 200);
        assert_eq!(id2, ToastId(2));
        // Newest-first ordering.
        assert_eq!(queue.toasts[0].id, id2);
        assert_eq!(queue.toasts[1].id, id1);

        // Dismiss id1 and verify stale generation does not evict after resurrection.
        let mut dismissed_meta = Vec::new();
        assert!(queue.dismiss_toast_with(id1, 300, |m| dismissed_meta.push(*m)));
        assert_eq!(dismissed_meta, vec!["meta"]);

        let gen_before = queue.toasts[1].generation;
        // Resurrect id1 in place.
        let (resurrected_id, _) = queue.add_toast(make_item(0, "First Updated", 400), Some(id1), 400);
        assert_eq!(resurrected_id, id1);
        assert_eq!(queue.toasts.len(), 2);
        assert_eq!(queue.toasts[1].phase, ToastPhase::Active);
        assert_eq!(queue.toasts[1].generation, gen_before + 1);

        // Stale eviction is ignored because toast is active and generation advanced.
        let (removed, dirty) = queue.remove_toast_if_dismissing_generation(id1, gen_before, 450);
        assert!(removed.is_none());
        assert!(!dirty);

        // Dismiss again and evict with matching generation.
        assert!(queue.dismiss_toast(id1, 500));
        let (removed, dirty) =
            queue.remove_toast_if_dismissing_generation(id1, gen_before + 1, 550);
        assert!(removed.is_some());
        assert!(dirty);
    }

    #[test]
    fn pause_resume_accounts_elapsed_time_and_deferred_hover_leave_during_swipe() {
        let mut queue = ToastQueue::<&'static str, u64>::new();
        let (id, _) = queue.add_toast(make_item(0, "Timer", 1000), None, 1000);

        // Hover at 2500ms -> pauses with 2500ms remaining (4000 - 1500).
        assert!(queue.set_hovered(true, 2500));
        assert!(queue.paused);
        assert!(queue.expanded);
        assert_eq!(queue.toasts[0].remaining_duration, Duration::from_millis(2500));

        // Swipe starts, then mouse leaves viewport: hover remains locked until swipe ends.
        assert!(!queue.set_swiping(true, 2600));
        assert!(!queue.handle_viewport_mouse_leave(2700));
        assert!(queue.interaction_state.hovered);
        assert!(queue.paused);

        // Ending swipe flushes deferred hover leave and resumes timers.
        assert!(queue.set_swiping(false, 3000));
        assert!(!queue.interaction_state.hovered);
        assert!(!queue.paused);
        assert!(!queue.expanded);
        assert_eq!(queue.toasts[0].last_started_at, 3000);

        // Height jitter <= 0.5px ignored; > 0.5px recorded.
        assert!(queue.set_toast_height(id, 64.0));
        assert!(!queue.set_toast_height(id, 64.4));
        assert!(queue.set_toast_height(id, 65.0));
        assert_eq!(queue.toast_height(id), Some(65.0));
    }

    #[test]
    fn viewport_pointer_press_sequence_guards_stale_release_and_clears_on_empty() {
        let mut queue = ToastQueue::<(), u64>::new();
        assert!(!queue.pointer_down_in_viewport);
        assert_eq!(queue.pointer_down_seq, 0);

        let seq1 = queue.begin_viewport_pointer_press();
        assert_eq!(seq1, 1);
        assert!(queue.pointer_down_in_viewport);

        let seq2 = queue.begin_viewport_pointer_press();
        assert_eq!(seq2, 2);
        queue.end_viewport_pointer_press(seq1);
        assert!(queue.pointer_down_in_viewport);

        queue.end_viewport_pointer_press(seq2);
        assert!(!queue.pointer_down_in_viewport);

        let seq3 = queue.begin_viewport_pointer_press();
        assert_eq!(seq3, 3);
        queue.clear_viewport_pointer_press();
        assert!(!queue.pointer_down_in_viewport);
        assert_eq!(queue.pointer_down_seq, 4);

        let seq5 = queue.begin_viewport_pointer_press();
        assert_eq!(seq5, 5);
        let _ = queue.clear_interaction_if_queue_empty(0);
        assert!(!queue.pointer_down_in_viewport);
        assert_eq!(queue.pointer_down_seq, 6);
    }
}
