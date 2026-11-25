use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use dioxus::prelude::keyboard_types::{Key, NamedKey};
use dioxus::prelude::*;

use crate::foundation::{
    browser::{focus_element_by_id, local_today_ymd},
    date_time::{
        CalendarDate, DateValue, FieldInvalidEvent, FieldInvalidReason, FieldSegmentIntent,
        SegmentFocusMove, SegmentKind, parse_day_period_key, parse_pasted_date_value,
    },
    shared::Direction,
};

use super::{
    attrs::{DateFieldInputAttrs, DateFieldLabelAttrs, DateFieldSegmentAttrs},
    relationships::DateFieldRelationships,
    state::{
        DateFieldState, first_interactive_segment, last_interactive_segment,
        resolve_adjacent_interactive_index,
    },
    types::{DateFieldRangeRole, DateFieldSegmentItem, DateFieldSnapshot},
};

/// Returns the host local calendar date via `oxidase::time::local_today_ymd()`.
pub fn local_today() -> CalendarDate {
    let (year, month, day) = local_today_ymd();
    CalendarDate::from_ymd_clamped(year, month, day)
}

/// Translates a Dioxus `keyboard_types::Key` into a host-neutral [`FieldSegmentIntent`].
///
/// Returns `(Option<FieldSegmentIntent>, should_prevent_default)`.
/// Never prevents `Tab`/`Shift+Tab` or modifier shortcuts (`Ctrl`/`Meta`/`Alt`).
pub fn translate_segment_keydown(
    key: &Key,
    dir: Direction,
    segment_kind: SegmentKind,
    has_modifier: bool,
) -> (Option<FieldSegmentIntent>, bool) {
    if has_modifier {
        return (None, false);
    }

    match key {
        Key::Named(NamedKey::Tab) => (None, false),
        Key::Named(NamedKey::ArrowUp) => (Some(FieldSegmentIntent::Increment), true),
        Key::Named(NamedKey::ArrowDown) => (Some(FieldSegmentIntent::Decrement), true),
        Key::Named(NamedKey::PageUp) => (Some(FieldSegmentIntent::PageUp), true),
        Key::Named(NamedKey::PageDown) => (Some(FieldSegmentIntent::PageDown), true),
        Key::Named(NamedKey::Home) => (Some(FieldSegmentIntent::Min), true),
        Key::Named(NamedKey::End) => (Some(FieldSegmentIntent::Max), true),
        Key::Named(NamedKey::Backspace) => (Some(FieldSegmentIntent::Backspace), true),
        Key::Named(NamedKey::Delete) => (Some(FieldSegmentIntent::Delete), true),
        Key::Named(NamedKey::ArrowRight) => {
            let intent = if dir == Direction::Rtl {
                FieldSegmentIntent::PrevSegment
            } else {
                FieldSegmentIntent::NextSegment
            };
            (Some(intent), true)
        }
        Key::Named(NamedKey::ArrowLeft) => {
            let intent = if dir == Direction::Rtl {
                FieldSegmentIntent::NextSegment
            } else {
                FieldSegmentIntent::PrevSegment
            };
            (Some(intent), true)
        }
        Key::Named(NamedKey::Enter) => (None, true),
        Key::Character(s) => {
            if s == " " {
                return (None, true);
            }
            if segment_kind == SegmentKind::DayPeriod
                && let Some(period) = parse_day_period_key(s)
            {
                return (Some(FieldSegmentIntent::DayPeriodKey(period)), true);
            }
            if s.len() == 1
                && let Some(b) = s.bytes().next()
                && b.is_ascii_digit()
            {
                return (Some(FieldSegmentIntent::Digit(b - b'0')), true);
            }
            (None, false)
        }
        _ => (None, false),
    }
}

/// Reactive runtime for `DateField`.
#[derive(Clone)]
pub struct DateFieldRuntime {
    pub state: Signal<DateFieldState>,
    pub rels: DateFieldRelationships,
    pub range_role: DateFieldRangeRole,
    pub label_id_override: Signal<Option<String>>,
    pub trailing_trigger_id: Signal<Option<String>>,
    pub segment_dom_ids: Rc<RefCell<BTreeMap<usize, String>>>,
    pub announcement: Signal<String>,
    pub last_invalid_reason: Signal<Option<FieldInvalidReason>>,
    pub controlled_value: Rc<RefCell<Option<Signal<Option<DateValue>>>>>,
    pub controlled_placeholder: Rc<RefCell<Option<Signal<DateValue>>>>,
    pub default_value: Option<DateValue>,
    pub on_value_change: Rc<RefCell<Option<EventHandler<Option<DateValue>>>>>,
    pub on_placeholder_change: Rc<RefCell<Option<EventHandler<DateValue>>>>,
    pub validate: Rc<RefCell<Option<Callback<DateValue, Option<Vec<String>>>>>>,
    pub on_invalid: Rc<RefCell<Option<EventHandler<FieldInvalidEvent>>>>,
    pub on_prev_boundary: Rc<RefCell<Option<EventHandler<()>>>>,
    pub on_next_boundary: Rc<RefCell<Option<EventHandler<()>>>>,
}

impl PartialEq for DateFieldRuntime {
    fn eq(&self, other: &Self) -> bool {
        self.rels == other.rels && self.state == other.state
    }
}

impl DateFieldRuntime {
    pub fn snapshot(&self) -> DateFieldSnapshot {
        self.state.read().snapshot(&self.rels)
    }

    pub fn segments(&self) -> Vec<DateFieldSegmentItem> {
        self.state.read().rendered_segments(&self.rels)
    }

    pub fn effective_label_id(&self) -> String {
        self.label_id_override
            .read()
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.rels.label_id().to_string())
    }

    /// Registers an optional trailing picker trigger button ID (`data-segment="trigger"`)
    /// so `ArrowRight` from the last editable segment focuses the trigger.
    pub fn register_trigger_id(&mut self, id: Option<String>) {
        if *self.trailing_trigger_id.peek() != id {
            self.trailing_trigger_id.set(id);
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
        first_interactive_segment(&segs)
            .map(|s| self.resolve_segment_dom_id(s.index, &s.id))
    }

    pub fn last_interactive_segment_id(&self) -> Option<String> {
        let segs = self.state.peek().rendered_segments(&self.rels);
        last_interactive_segment(&segs)
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

    pub fn input_attrs(&self, id_override: Option<String>) -> DateFieldInputAttrs {
        let st = self.state.read();
        let id = id_override
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.rels.input_id().to_string());
        let desc = if st.description_text().is_empty() {
            None
        } else {
            Some(self.rels.description_id().to_string())
        };
        DateFieldInputAttrs::new(
            id,
            self.range_role,
            Some(self.effective_label_id()),
            desc,
            st.def().dir,
            st.is_invalid(),
            st.def().disabled,
        )
    }

    pub fn label_attrs(&self, id_override: Option<String>) -> DateFieldLabelAttrs {
        let st = self.state.read();
        let id = id_override
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.effective_label_id());
        DateFieldLabelAttrs::new(id, st.is_invalid(), st.def().disabled)
    }

    pub fn segment_attrs(
        &self,
        item: &DateFieldSegmentItem,
        id_override: Option<String>,
    ) -> DateFieldSegmentAttrs {
        let st = self.state.read();
        let segs = st.rendered_segments(&self.rels);
        let first_idx = first_interactive_segment(&segs).map(|s| s.index);
        let is_first = first_idx == Some(item.index);
        let desc_id = if st.description_text().is_empty() {
            None
        } else {
            Some(self.rels.description_id())
        };
        let template = st.def().locale.field_template();
        let label_id = self.effective_label_id();
        let seg_readonly = st.def().is_segment_read_only(item.kind());
        DateFieldSegmentAttrs::new(
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
        let current_val = self.state.peek().value();
        let custom_errors = current_val.and_then(|v| {
            self.validate
                .borrow()
                .as_ref()
                .and_then(|cb| cb.call(v))
        });

        let next_reason = self.state.write().evaluate_validation(custom_errors);
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
        prev_value: Option<DateValue>,
        prev_placeholder: DateValue,
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
        match resolve_adjacent_interactive_index(&segs, current_index, move_dir) {
            Ok(target_idx) => {
                if let Some(target) = segs.iter().find(|s| s.index == target_idx) {
                    let dom_id = self.resolve_segment_dom_id(target.index, &target.id);
                    focus_element_by_id(&dom_id);
                }
            }
            Err(SegmentFocusMove::Next) => {
                if let Some(handler) = self.on_next_boundary.borrow().as_ref() {
                    handler.call(());
                } else if let Some(trigger_id) = self.trailing_trigger_id.peek().as_deref() {
                    focus_element_by_id(trigger_id);
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
        let granularity = self.state.peek().resolved_granularity();
        if parse_pasted_date_value(text, granularity).is_none() {
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
        self.state.write().set_value(self.default_value);
        self.emit_value_and_placeholder_changes(prev_val, prev_ph);
        self.sync_validation_and_notify();
    }

    pub fn reconcile_external_state(
        &mut self,
        external_value: Option<Option<DateValue>>,
        external_placeholder: Option<DateValue>,
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
        if let Some(inv) = is_invalid_override {
            self.state.write().set_external_invalid(inv);
        }
        self.sync_validation_and_notify();
    }
}

/// Dioxus context wrapping [`DateFieldRuntime`].
#[derive(Clone, PartialEq)]
pub struct DateFieldContext(pub DateFieldRuntime);

/// Consumes the nearest ancestor [`DateFieldRuntime`] context (panics if outside `DateFieldRoot`).
pub fn use_date_field_context() -> DateFieldRuntime {
    use_context::<DateFieldContext>().0
}

/// Attempts to consume the nearest ancestor [`DateFieldRuntime`] context.
pub fn try_use_date_field_context() -> Option<DateFieldRuntime> {
    try_consume_context::<DateFieldContext>().map(|ctx| ctx.0)
}
