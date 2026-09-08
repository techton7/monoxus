use super::types::*;

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
    pub config: ToastConfig,
    pub paused: bool,
    pub expanded: bool,
    pub interaction_state: ToastInteractionState,
    pub next_id: u64,
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
        Self {
            toasts: Vec::new(),
            config,
            paused: false,
            expanded: false,
            interaction_state: ToastInteractionState::default(),
            next_id: 0,
        }
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
            return existing.id;
        }

        if item.id.0 == 0 {
            self.next_id += 1;
            item.id = ToastId(self.next_id);
        }

        item.last_started_at = now;
        if self.paused {
            item.paused_at = Some(now);
        }

        let assigned_id = item.id;
        self.toasts.insert(0, item);
        assigned_id
    }

    /// Updates fields on an existing toast in place.
    pub fn update_toast<F: FnOnce(&mut ToastItem)>(&mut self, id: ToastId, updater: F) -> bool {
        if let Some(item) = self.toasts.iter_mut().find(|t| t.id == id) {
            updater(item);
            true
        } else {
            false
        }
    }

    /// Marks a toast as dismissing (data-state="closed").
    ///
    /// The item remains in the queue to permit exit animations until `remove_toast` is called.
    pub fn dismiss_toast(&mut self, id: ToastId) -> bool {
        if let Some(item) = self.toasts.iter_mut().find(|t| t.id == id && t.phase == ToastPhase::Active) {
            item.phase = ToastPhase::Dismissing;
            if let Some(cb) = &item.options.on_dismiss {
                cb.call(());
            }
            return true;
        }
        false
    }

    /// Evicts a toast completely from the active store.
    pub fn remove_toast(&mut self, id: ToastId) -> Option<ToastItem> {
        if let Some(idx) = self.toasts.iter().position(|t| t.id == id) {
            Some(self.toasts.remove(idx))
        } else {
            None
        }
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
    }

    /// Synchronizes paused and expanded state against active interaction flags.
    pub fn sync_interaction_state(&mut self) {
        let should_pause = self.interaction_state.should_pause();
        let should_expand = self.interaction_state.should_expand();
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
        self.interaction_state.hovered = hovered;
        self.sync_interaction_state();
    }

    /// Sets whether keyboard focus is currently inside the toaster viewport.
    pub fn set_focused(&mut self, focused: bool) {
        self.interaction_state.focused = focused;
        self.sync_interaction_state();
    }

    /// Sets whether a toast is actively undergoing a touch/pointer swipe gesture.
    pub fn set_swiping(&mut self, swiping: bool) {
        self.interaction_state.swiping = swiping;
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
