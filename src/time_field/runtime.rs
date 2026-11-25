use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use dioxus::prelude::*;

use crate::{
    date_field::translate_segment_keydown,
    foundation::{
        browser::focus_element_by_id,
        date_time::{
            FieldInvalidEvent, FieldInvalidReason, FieldSegmentIntent, SegmentFocusMove,
            SegmentKind, TimeValue, parse_pasted_time_value, validate_time_value,
        },
    },
};

use super::{
    attrs::{TimeFieldInputAttrs, TimeFieldLabelAttrs, TimeFieldSegmentAttrs},
    relationships::TimeFieldRelationships,
    state::{
        TimeFieldState, first_interactive_time_segment, last_interactive_time_segment,
        resolve_adjacent_interactive_time_index,
    },
    types::{TimeFieldRangeRole, TimeFieldSegmentItem, TimeFieldSnapshot},
};

/// Reactive runtime for `TimeField`.
#[derive(Clone)]
pub struct TimeFieldRuntime {
    pub state: Signal<TimeFieldState>,
    pub rels: TimeFieldRelationships,
    pub range_role: Rc<RefCell<TimeFieldRangeRole>>,
    pub label_id_override: Signal<Option<String>>,
    pub custom_label_id: Signal<Option<String>>,
    pub segment_dom_ids: Rc<RefCell<BTreeMap<usize, String>>>,
    pub announcement: Signal<String>,
    pub last_invalid_reason: Signal<Option<FieldInvalidReason>>,
    pub controlled_value: Rc<RefCell<Option<Signal<Option<TimeValue>>>>>,
    pub controlled_placeholder: Rc<RefCell<Option<Signal<TimeValue>>>>,
    pub default_value: Rc<RefCell<Option<TimeValue>>>,
    pub on_value_change: Rc<RefCell<Option<EventHandler<Option<TimeValue>>>>>,
    pub on_placeholder_change: Rc<RefCell<Option<EventHandler<TimeValue>>>>,
    pub validate: Rc<RefCell<Option<Callback<TimeValue, Option<Vec<String>>>>>>,
    pub on_invalid: Rc<RefCell<Option<EventHandler<FieldInvalidEvent>>>>,
    pub on_prev_boundary: Rc<RefCell<Option<EventHandler<()>>>>,
    pub on_next_boundary: Rc<RefCell<Option<EventHandler<()>>>>,
}

impl PartialEq for TimeFieldRuntime {
    fn eq(&self, other: &Self) -> bool {
        self.rels == other.rels && self.state == other.state
    }
}

impl TimeFieldRuntime {
    pub fn snapshot(&self) -> TimeFieldSnapshot {
        self.state.read().snapshot(&self.rels)
    }

    pub fn segments(&self) -> Vec<TimeFieldSegmentItem> {
        self.state.read().rendered_segments(&self.rels)
    }

    pub fn effective_label_id(&self) -> String {
        self.custom_label_id
            .read()
            .clone()
            .filter(|s| !s.is_empty())
            .or_else(|| {
                self.label_id_override
                    .read()
                    .clone()
                    .filter(|s| !s.is_empty())
            })
            .unwrap_or_else(|| self.rels.label_id().to_string())
    }

    /// Registers a custom `id` from `TimeFieldLabel` so `aria-labelledby` stays synced (`DEF-5C-07`).
    pub fn register_custom_label_id(&mut self, id: Option<String>) {
        let normalized = id.filter(|s| !s.is_empty());
        if *self.custom_label_id.peek() != normalized {
            self.custom_label_id.set(normalized);
        }
    }

    pub fn register_segment_dom_id(&self, index: usize, dom_id: String) {
        self.segment_dom_ids.borrow_mut().insert(index, dom_id);
    }

    pub fn resolve_segment_dom_id(&self, index: usize, fallback: &str) -> String {
        self.segment_dom_ids
            .borrow()
            .get(&index)
            .cloned()
            .unwrap_or_else(|| fallback.to_string())
    }

    pub fn first_interactive_segment_id(&self) -> Option<String> {
        let segs = self.state.peek().rendered_segments(&self.rels);
        first_interactive_time_segment(&segs)
            .map(|s| self.resolve_segment_dom_id(s.index, &s.id))
    }

    pub fn last_interactive_segment_id(&self) -> Option<String> {
        let segs = self.state.peek().rendered_segments(&self.rels);
        last_interactive_time_segment(&segs)
            .map(|s| self.resolve_segment_dom_id(s.index, &s.id))
    }

    pub fn focus_first_segment(&self) {
        if let Some(id) = self.first_interactive_segment_id() {
            focus_element_by_id(&id);
        }
    }

    pub fn focus_last_segment(&self) {
        if let Some(id) = self.last_interactive_segment_id() {
            focus_element_by_id(&id);
        }
    }

    pub fn input_attrs(&self, id_override: Option<String>) -> TimeFieldInputAttrs {
        let st = self.state.read();
        let id = id_override
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.rels.input_id().to_string());
        let desc = if st.description_text().is_empty() {
            None
        } else {
            Some(self.rels.description_id().to_string())
        };
        TimeFieldInputAttrs::new(
            id,
            *self.range_role.borrow(),
            Some(self.effective_label_id()),
            desc,
            st.def().dir,
            st.is_invalid(),
            st.def().disabled,
        )
        .with_required(st.def().required)
    }

    pub fn label_attrs(&self, id_override: Option<String>) -> TimeFieldLabelAttrs {
        let st = self.state.read();
        let id = id_override
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.effective_label_id());
        TimeFieldLabelAttrs::new(id, st.is_invalid(), st.def().disabled)
    }

    pub fn segment_attrs(
        &self,
        item: &TimeFieldSegmentItem,
        id_override: Option<String>,
    ) -> TimeFieldSegmentAttrs {
        let st = self.state.read();
        let segs = st.rendered_segments(&self.rels);
        let first_idx = first_interactive_time_segment(&segs).map(|s| s.index);
        let is_first = first_idx == Some(item.index);
        let desc_id = if st.description_text().is_empty() {
            None
        } else {
            Some(self.rels.description_id())
        };
        let template = st.def().locale.field_template();
        let label_id = self.effective_label_id();
        let seg_readonly = st.def().is_segment_read_only(item.kind());
        TimeFieldSegmentAttrs::new(
            item,
            id_override,
            Some(&label_id),
            desc_id,
            st.def().error_message_id.as_deref(),
            is_first,
            &template,
            st.is_invalid(),
            st.def().disabled,
            seg_readonly,
        )
    }

    fn sync_validation_and_notify(&mut self) {
        let (current_val, min_val, max_val) = {
            let st = self.state.peek();
            (st.value(), st.def().min_value, st.def().max_value)
        };
        let custom_errors = current_val.and_then(|v| {
            self.validate
                .borrow()
                .as_ref()
                .and_then(|cb| cb.call(v))
        });

        let next_reason = validate_time_value(current_val, min_val, max_val, custom_errors).err();
        if self.state.peek().local_invalid_reason() != next_reason.as_ref() {
            self.state.write().set_local_invalid_reason(next_reason.clone());
        }
        let prev_reason = self.last_invalid_reason.peek().clone();
        if next_reason != prev_reason {
            self.last_invalid_reason.set(next_reason.clone());
            if let Some(reason) = next_reason {
                let messages = match &reason {
                    FieldInvalidReason::Custom(msgs) => msgs.clone(),
                    _ => Vec::new(),
                };
                if let Some(handler) = self.on_invalid.borrow().as_ref() {
                    handler.call(FieldInvalidEvent { reason, messages });
                }
            }
        }
    }

    fn emit_value_and_placeholder_changes(
        &mut self,
        prev_value: Option<TimeValue>,
        prev_placeholder: TimeValue,
    ) {
        let (next_value, next_placeholder) = {
            let st = self.state.peek();
            (st.value(), st.placeholder())
        };

        if next_value != prev_value {
            if let Some(mut sig) = *self.controlled_value.borrow()
                && *sig.peek() != next_value
            {
                sig.set(next_value);
            }
            if let Some(handler) = self.on_value_change.borrow().as_ref() {
                handler.call(next_value);
            }
        }

        if next_placeholder != prev_placeholder {
            if let Some(mut sig) = *self.controlled_placeholder.borrow()
                && *sig.peek() != next_placeholder
            {
                sig.set(next_placeholder);
            }
            if let Some(handler) = self.on_placeholder_change.borrow().as_ref() {
                handler.call(next_placeholder);
            }
        }
    }

    fn apply_focus_move(&self, current_index: usize, move_dir: SegmentFocusMove) {
        let segs = self.state.peek().rendered_segments(&self.rels);
        match resolve_adjacent_interactive_time_index(&segs, current_index, move_dir) {
            Ok(target_idx) => {
                if let Some(target) = segs.iter().find(|s| s.index == target_idx) {
                    let dom_id = self.resolve_segment_dom_id(target.index, &target.id);
                    focus_element_by_id(&dom_id);
                }
            }
            Err(SegmentFocusMove::Next) => {
                if let Some(handler) = self.on_next_boundary.borrow().as_ref() {
                    handler.call(());
                }
            }
            Err(SegmentFocusMove::Prev) => {
                if let Some(handler) = self.on_prev_boundary.borrow().as_ref() {
                    handler.call(());
                }
            }
        }
    }

    pub fn handle_segment_keydown(
        &mut self,
        index: usize,
        kind: SegmentKind,
        evt: &KeyboardEvent,
    ) {
        if evt.data().is_composing() {
            return;
        }
        // DEF-5C-12: Do not prevent_default() when the field is disabled.
        if self.state.peek().def().disabled {
            return;
        }
        let mods = evt.data().modifiers();
        let has_modifier = mods.ctrl() || mods.meta() || mods.alt();
        let dir = self.state.peek().def().dir;
        let (intent, should_prevent) = translate_segment_keydown(&evt.key(), dir, kind, has_modifier);
        if should_prevent {
            evt.prevent_default();
        }
        let Some(intent) = intent else {
            return;
        };

        let (prev_val, prev_ph) = {
            let st = self.state.peek();
            (st.value(), st.placeholder())
        };

        let result = self.state.write().apply_intent(kind, intent);
        if let Some(res) = result {
            if let Some(msg) = res.announcement {
                let desc = self.state.peek().description_text();
                let next_ann = if desc.is_empty() { msg } else { desc };
                if *self.announcement.peek() != next_ann {
                    self.announcement.set(next_ann);
                }
            } else if res.cleared_value {
                let empty = self.state.peek().def().locale.empty_announcement();
                if *self.announcement.peek() != empty {
                    self.announcement.set(empty);
                }
            }
            self.emit_value_and_placeholder_changes(prev_val, prev_ph);
            self.sync_validation_and_notify();

            if let Some(move_dir) = res.move_focus {
                self.apply_focus_move(index, move_dir);
            }
        }
    }

    pub fn handle_segment_blur(&mut self, kind: SegmentKind) {
        let (prev_val, prev_ph) = {
            let st = self.state.peek();
            (st.value(), st.placeholder())
        };
        let _ = self
            .state
            .write()
            .apply_intent(kind, FieldSegmentIntent::Blur);
        self.emit_value_and_placeholder_changes(prev_val, prev_ph);
        self.sync_validation_and_notify();
    }

    pub fn handle_segment_paste(&mut self, kind: SegmentKind, text: &str) -> bool {
        if parse_pasted_time_value(text).is_none() {
            return false;
        }
        let (prev_val, prev_ph) = {
            let st = self.state.peek();
            (st.value(), st.placeholder())
        };
        let result = self
            .state
            .write()
            .apply_intent(kind, FieldSegmentIntent::Paste(text.to_string()));
        if result.is_some() {
            let desc = self.state.peek().description_text();
            if !desc.is_empty() && *self.announcement.peek() != desc {
                self.announcement.set(desc);
            }
            self.emit_value_and_placeholder_changes(prev_val, prev_ph);
            self.sync_validation_and_notify();
            true
        } else {
            false
        }
    }

    pub fn reset_to_default(&mut self) {
        let (prev_val, prev_ph) = {
            let st = self.state.peek();
            (st.value(), st.placeholder())
        };
        let default_val = *self.default_value.borrow();
        self.state.write().reset_value(default_val);
        self.emit_value_and_placeholder_changes(prev_val, prev_ph);
        self.sync_validation_and_notify();
    }

    pub fn reconcile_external_state(
        &mut self,
        external_value: Option<Option<TimeValue>>,
        external_placeholder: Option<TimeValue>,
        is_invalid_override: Option<bool>,
    ) {
        if let Some(val) = external_value
            && self.state.peek().value() != val
        {
            self.state.write().set_value(val);
        }
        if let Some(ph) = external_placeholder
            && self.state.peek().placeholder() != ph
        {
            self.state.write().set_placeholder(ph);
        }
        if let Some(inv) = is_invalid_override
            && self.state.peek().external_invalid() != inv
        {
            self.state.write().set_external_invalid(inv);
        }
        self.sync_validation_and_notify();
    }
}

/// Dioxus context wrapping [`TimeFieldRuntime`].
#[derive(Clone, PartialEq)]
pub struct TimeFieldContext(pub TimeFieldRuntime);

/// Consumes the nearest ancestor [`TimeFieldRuntime`] context (panics if outside `TimeFieldRoot`).
pub fn use_time_field_context() -> TimeFieldRuntime {
    use_context::<TimeFieldContext>().0
}

/// Attempts to consume the nearest ancestor [`TimeFieldRuntime`] context.
pub fn try_use_time_field_context() -> Option<TimeFieldRuntime> {
    try_consume_context::<TimeFieldContext>().map(|ctx| ctx.0)
}
