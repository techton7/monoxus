use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use dioxus::prelude::keyboard_types::{Key, NamedKey};
use dioxus::prelude::*;
use monoxus_core::foundation::date_time::field_policy::{
    DateFieldPolicy, FieldRangeRole, FieldRelationships, FieldSegmentItem, SegmentFieldPolicy,
    SegmentFieldSnapshot, SegmentFieldState, TimeFieldPolicy, first_interactive_field_segment,
    last_interactive_field_segment, resolve_adjacent_interactive_field_index,
};

use crate::foundation::{
    browser::focus_element_by_id,
    date_time::{
        CalendarDate, DateValue, FieldInvalidEvent, FieldInvalidReason, FieldSegmentIntent,
        SegmentFocusMove, SegmentKind, parse_day_period_key, parse_pasted_date_value,
        parse_pasted_time_value,
    },
    keyboard::is_ime_composing_key,
    shared::Direction,
};

use super::attrs::{SegmentFieldInputAttrs, SegmentFieldLabelAttrs, SegmentFieldSegmentAttrs};

/// Zero-overhead UI policy extension specializing [`SegmentFieldRuntime`] and
/// [`crate::date_range_field::RangeFieldRuntime`] for `DateField`/`DateRangeField` vs
/// `TimeField`/`TimeRangeField`.
pub trait SegmentFieldUiPolicy: SegmentFieldPolicy {
    const SCOPE_PREFIX: &'static str;
    const RANGE_SCOPE_PREFIX: &'static str;

    fn extract_calendar_date(value: Self::Value) -> Option<CalendarDate>;

    fn can_parse_paste(text: &str, granularity: Self::ResolvedGranularity) -> bool;
}

impl SegmentFieldUiPolicy for DateFieldPolicy {
    const SCOPE_PREFIX: &'static str = "date-field";
    const RANGE_SCOPE_PREFIX: &'static str = "date-range-field";

    #[inline]
    fn extract_calendar_date(value: DateValue) -> Option<CalendarDate> {
        Some(value.date())
    }

    #[inline]
    fn can_parse_paste(text: &str, granularity: Self::ResolvedGranularity) -> bool {
        parse_pasted_date_value(text, granularity).is_some()
    }
}

impl SegmentFieldUiPolicy for TimeFieldPolicy {
    const SCOPE_PREFIX: &'static str = "time-field";
    const RANGE_SCOPE_PREFIX: &'static str = "time-range-field";

    #[inline]
    fn extract_calendar_date(_value: Self::Value) -> Option<CalendarDate> {
        None
    }

    #[inline]
    fn can_parse_paste(text: &str, _granularity: Self::ResolvedGranularity) -> bool {
        parse_pasted_time_value(text).is_some()
    }
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
        Key::Named(NamedKey::Enter) => (None, false),
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

/// Zero-overhead policy-parameterized reactive runtime shared by `DateField` and `TimeField`.
pub struct SegmentFieldRuntime<P: SegmentFieldUiPolicy> {
    pub state: Signal<SegmentFieldState<P>>,
    pub rels: FieldRelationships,
    pub range_role: CopyValue<FieldRangeRole>,
    pub label_id_override: Signal<Option<String>>,
    pub custom_label_id: Signal<Option<String>>,
    pub trailing_trigger_id: Signal<Option<String>>,
    pub segment_dom_ids: Rc<RefCell<BTreeMap<usize, String>>>,
    pub announcement: Signal<String>,
    pub last_invalid_reason: Signal<Option<FieldInvalidReason>>,
    pub controlled_value: CopyValue<Option<Signal<Option<P::Value>>>>,
    pub controlled_placeholder: CopyValue<Option<Signal<P::Value>>>,
    pub default_value: CopyValue<Option<P::Value>>,
    pub on_value_change: CopyValue<Option<EventHandler<Option<P::Value>>>>,
    pub on_placeholder_change: CopyValue<Option<EventHandler<P::Value>>>,
    pub is_date_unavailable: CopyValue<Option<Callback<CalendarDate, bool>>>,
    pub validate: CopyValue<Option<Callback<P::Value, Option<Vec<String>>>>>,
    pub on_invalid: CopyValue<Option<EventHandler<FieldInvalidEvent>>>,
    pub on_prev_boundary: CopyValue<Option<EventHandler<()>>>,
    pub on_next_boundary: CopyValue<Option<EventHandler<()>>>,
}

impl<P: SegmentFieldUiPolicy> Clone for SegmentFieldRuntime<P> {
    fn clone(&self) -> Self {
        Self {
            state: self.state,
            rels: self.rels.clone(),
            range_role: self.range_role,
            label_id_override: self.label_id_override,
            custom_label_id: self.custom_label_id,
            trailing_trigger_id: self.trailing_trigger_id,
            segment_dom_ids: Rc::clone(&self.segment_dom_ids),
            announcement: self.announcement,
            last_invalid_reason: self.last_invalid_reason,
            controlled_value: self.controlled_value,
            controlled_placeholder: self.controlled_placeholder,
            default_value: self.default_value,
            on_value_change: self.on_value_change,
            on_placeholder_change: self.on_placeholder_change,
            is_date_unavailable: self.is_date_unavailable,
            validate: self.validate,
            on_invalid: self.on_invalid,
            on_prev_boundary: self.on_prev_boundary,
            on_next_boundary: self.on_next_boundary,
        }
    }
}

impl<P: SegmentFieldUiPolicy> PartialEq for SegmentFieldRuntime<P> {
    fn eq(&self, other: &Self) -> bool {
        self.rels == other.rels && self.state == other.state
    }
}

impl<P: SegmentFieldUiPolicy> SegmentFieldRuntime<P> {
    pub fn snapshot(&self) -> SegmentFieldSnapshot<P> {
        self.state.read().snapshot(&self.rels)
    }

    pub fn segments(&self) -> Vec<FieldSegmentItem> {
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

    /// Registers a custom `id` from `*FieldLabel` so `aria-labelledby` stays synced (`DEF-5C-07`).
    pub fn register_custom_label_id(&mut self, id: Option<String>) {
        let normalized = id.filter(|s| !s.is_empty());
        if self.custom_label_id.try_peek().is_ok_and(|cur| *cur != normalized)
            && let Ok(mut guard) = self.custom_label_id.try_write()
        {
            *guard = normalized;
        }
    }

    /// Clears `custom_label_id` only if it still matches `expected`, preventing keyed label
    /// swaps from clobbering the replacement label's ID.
    pub fn unregister_custom_label_id(&mut self, expected: &str) {
        if self
            .custom_label_id
            .try_peek()
            .ok()
            .and_then(|cur| cur.as_deref().map(|s| s == expected))
            .unwrap_or(false)
            && let Ok(mut guard) = self.custom_label_id.try_write()
        {
            *guard = None;
        }
    }

    /// Registers an optional trailing picker trigger button ID (`data-segment="trigger"`)
    /// so navigating past the last editable segment focuses the trigger.
    pub fn register_trigger_id(&mut self, id: Option<String>) {
        if self
            .trailing_trigger_id
            .try_peek()
            .is_ok_and(|cur| *cur != id)
            && let Ok(mut guard) = self.trailing_trigger_id.try_write()
        {
            *guard = id;
        }
    }

    /// Clears `trailing_trigger_id` only when it still matches `expected`, preventing keyed
    /// trigger swaps from clobbering the replacement trigger's ID.
    pub fn unregister_trigger_id(&mut self, expected: &str) {
        if self
            .trailing_trigger_id
            .try_peek()
            .ok()
            .and_then(|cur| cur.as_deref().map(|s| s == expected))
            .unwrap_or(false)
            && let Ok(mut guard) = self.trailing_trigger_id.try_write()
        {
            *guard = None;
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
        first_interactive_field_segment(&segs)
            .map(|s| self.resolve_segment_dom_id(s.index, &s.id))
    }

    pub fn last_interactive_segment_id(&self) -> Option<String> {
        let segs = self.state.peek().rendered_segments(&self.rels);
        last_interactive_field_segment(&segs)
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

    pub fn input_attrs(&self, id_override: Option<String>) -> SegmentFieldInputAttrs {
        let st = self.state.read();
        let id = id_override
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.rels.input_id().to_string());
        let desc = if st.description_text().is_empty() {
            None
        } else {
            Some(self.rels.description_id().to_string())
        };
        let role = self.range_role.try_read().map(|r| *r).unwrap_or_default();
        SegmentFieldInputAttrs::new(
            id,
            role,
            Some(self.effective_label_id()),
            desc,
            st.def().dir,
            st.is_invalid(),
            st.def().disabled,
        )
        .with_required(st.def().required)
    }

    pub fn label_attrs(&self, id_override: Option<String>) -> SegmentFieldLabelAttrs {
        let st = self.state.read();
        let id = id_override
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.effective_label_id());
        SegmentFieldLabelAttrs::new(id, st.is_invalid(), st.def().disabled)
    }

    pub fn segment_attrs(
        &self,
        item: &FieldSegmentItem,
        id_override: Option<String>,
    ) -> SegmentFieldSegmentAttrs {
        let st = self.state.read();
        let segs = st.rendered_segments(&self.rels);
        let first_idx = first_interactive_field_segment(&segs).map(|s| s.index);
        let is_first = first_idx == Some(item.index);
        let desc_id = if st.description_text().is_empty() {
            None
        } else {
            Some(self.rels.description_id())
        };
        let template = st.def().locale.field_template();
        let label_id = self.effective_label_id();
        let seg_readonly = st.def().is_segment_read_only(item.kind());
        SegmentFieldSegmentAttrs::new(
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

    pub fn sync_validation_and_notify(&mut self) {
        let (current_val, min_val, max_val) = {
            let st = self.state.peek();
            (st.value(), st.def().min_value, st.def().max_value)
        };
        let is_unavailable = current_val
            .and_then(P::extract_calendar_date)
            .is_some_and(|date| {
                self.is_date_unavailable
                    .try_read()
                    .ok()
                    .and_then(|slot| *slot)
                    .is_some_and(|cb| cb.call(date))
            });
        let custom_errors = if is_unavailable {
            None
        } else {
            current_val.and_then(|v| {
                self.validate
                    .try_read()
                    .ok()
                    .and_then(|slot| *slot)
                    .and_then(|cb| cb.call(v))
            })
        };

        let next_reason =
            P::validate_value(current_val, min_val, max_val, is_unavailable, custom_errors).err();
        if self.state.peek().local_invalid_reason() != next_reason.as_ref() {
            self.state
                .write()
                .set_local_invalid_reason(next_reason.clone());
        }
        let prev_reason = self.last_invalid_reason.peek().clone();
        if next_reason != prev_reason {
            self.last_invalid_reason.set(next_reason.clone());
            if let Some(reason) = next_reason {
                let messages = match &reason {
                    FieldInvalidReason::Custom(msgs) => msgs.clone(),
                    _ => Vec::new(),
                };
                if let Some(handler) = self.on_invalid.try_read().ok().and_then(|slot| *slot) {
                    handler.call(FieldInvalidEvent { reason, messages });
                }
            }
        }
    }

    fn emit_value_and_placeholder_changes(
        &mut self,
        prev_value: Option<P::Value>,
        prev_placeholder: P::Value,
    ) {
        let (next_value, next_placeholder) = {
            let st = self.state.peek();
            (st.value(), st.placeholder())
        };

        if next_value != prev_value {
            if let Some(mut sig) = self.controlled_value.try_read().ok().and_then(|slot| *slot)
                && *sig.peek() != next_value
            {
                sig.set(next_value);
            }
            if let Some(handler) = self.on_value_change.try_read().ok().and_then(|slot| *slot) {
                handler.call(next_value);
            }
        }

        if next_placeholder != prev_placeholder {
            if let Some(mut sig) = self
                .controlled_placeholder
                .try_read()
                .ok()
                .and_then(|slot| *slot)
                && *sig.peek() != next_placeholder
            {
                sig.set(next_placeholder);
            }
            if let Some(handler) = self
                .on_placeholder_change
                .try_read()
                .ok()
                .and_then(|slot| *slot)
            {
                handler.call(next_placeholder);
            }
        }
    }

    fn apply_focus_move(&self, current_index: usize, move_dir: SegmentFocusMove) {
        let segs = self.state.peek().rendered_segments(&self.rels);
        match resolve_adjacent_interactive_field_index(&segs, current_index, move_dir) {
            Ok(target_idx) => {
                if let Some(target) = segs.iter().find(|s| s.index == target_idx) {
                    let dom_id = self.resolve_segment_dom_id(target.index, &target.id);
                    focus_element_by_id(&dom_id);
                }
            }
            Err(SegmentFocusMove::Next) => {
                if let Some(handler) = self.on_next_boundary.try_read().ok().and_then(|slot| *slot)
                {
                    handler.call(());
                } else if let Some(trigger_id) = self.trailing_trigger_id.peek().as_deref() {
                    focus_element_by_id(trigger_id);
                }
            }
            Err(SegmentFocusMove::Prev) => {
                if let Some(handler) = self.on_prev_boundary.try_read().ok().and_then(|slot| *slot)
                {
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
        if is_ime_composing_key(evt) {
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
        let granularity = self.state.peek().resolved_granularity();
        if !P::can_parse_paste(text, granularity) {
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
        let default_val = self.default_value.try_read().ok().and_then(|v| *v);
        self.state.write().reset_value(default_val);
        self.emit_value_and_placeholder_changes(prev_val, prev_ph);
        self.sync_validation_and_notify();
    }

    pub fn reconcile_external_state(
        &mut self,
        external_value: Option<Option<P::Value>>,
        external_placeholder: Option<P::Value>,
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

/// Dioxus context wrapping [`SegmentFieldRuntime`].
pub struct SegmentFieldContext<P: SegmentFieldUiPolicy>(pub SegmentFieldRuntime<P>);

impl<P: SegmentFieldUiPolicy> Clone for SegmentFieldContext<P> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<P: SegmentFieldUiPolicy> PartialEq for SegmentFieldContext<P> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

/// Consumes the nearest ancestor [`SegmentFieldRuntime<P>`] context.
pub fn use_segment_field_context<P: SegmentFieldUiPolicy>() -> SegmentFieldRuntime<P> {
    use_context::<SegmentFieldContext<P>>().0
}

/// Attempts to consume the nearest ancestor [`SegmentFieldRuntime<P>`] context.
pub fn try_use_segment_field_context<P: SegmentFieldUiPolicy>() -> Option<SegmentFieldRuntime<P>> {
    try_consume_context::<SegmentFieldContext<P>>().map(|ctx| ctx.0)
}

/// Reactive runtime for `DateField`.
pub type DateFieldRuntime = SegmentFieldRuntime<DateFieldPolicy>;

/// Dioxus context wrapping [`DateFieldRuntime`].
pub type DateFieldContext = SegmentFieldContext<DateFieldPolicy>;

/// Consumes the nearest ancestor [`DateFieldRuntime`] context (panics if outside `DateFieldRoot`).
pub fn use_date_field_context() -> DateFieldRuntime {
    use_segment_field_context::<DateFieldPolicy>()
}

/// Attempts to consume the nearest ancestor [`DateFieldRuntime`] context.
pub fn try_use_date_field_context() -> Option<DateFieldRuntime> {
    try_use_segment_field_context::<DateFieldPolicy>()
}
