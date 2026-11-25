use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::{
    foundation::date_time::{
        FieldInvalidEvent, FieldInvalidReason, TimeRange, TimeValue, validate_time_range,
    },
    time_field::TimeFieldRuntime,
};

use super::{
    attrs::{TimeRangeFieldLabelAttrs, TimeRangeFieldRootAttrs},
    relationships::TimeRangeFieldRelationships,
    state::TimeRangeFieldState,
    types::{RangeFieldInputKind, TimeRangeFieldSnapshot},
};

/// Reactive runtime for `TimeRangeField`.
#[derive(Clone)]
pub struct TimeRangeFieldRuntime {
    pub state: Signal<TimeRangeFieldState>,
    pub rels: TimeRangeFieldRelationships,
    pub start_value: Signal<Option<TimeValue>>,
    pub end_value: Signal<Option<TimeValue>>,
    pub placeholder: Signal<TimeValue>,
    pub custom_label_id: Signal<Option<String>>,
    pub last_invalid_reason: Signal<Option<FieldInvalidReason>>,
    pub start_runtime: Rc<RefCell<Option<TimeFieldRuntime>>>,
    pub end_runtime: Rc<RefCell<Option<TimeFieldRuntime>>>,
    pub default_value: Rc<RefCell<TimeRange>>,
    pub controlled_value: Rc<RefCell<Option<Signal<TimeRange>>>>,
    pub controlled_placeholder: Rc<RefCell<Option<Signal<TimeValue>>>>,
    pub on_value_change: Rc<RefCell<Option<EventHandler<TimeRange>>>>,
    pub on_start_value_change: Rc<RefCell<Option<EventHandler<Option<TimeValue>>>>>,
    pub on_end_value_change: Rc<RefCell<Option<EventHandler<Option<TimeValue>>>>>,
    pub on_placeholder_change: Rc<RefCell<Option<EventHandler<TimeValue>>>>,
    pub validate: Rc<RefCell<Option<Callback<TimeRange, Option<Vec<String>>>>>>,
    pub on_invalid: Rc<RefCell<Option<EventHandler<FieldInvalidEvent>>>>,
}

impl PartialEq for TimeRangeFieldRuntime {
    fn eq(&self, other: &Self) -> bool {
        self.rels == other.rels && self.state == other.state
    }
}

impl TimeRangeFieldRuntime {
    pub fn snapshot(&self) -> TimeRangeFieldSnapshot {
        self.state.read().snapshot()
    }

    pub fn effective_label_id(&self) -> String {
        self.custom_label_id
            .read()
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.rels.label_id().to_string())
    }

    /// Registers a custom `id` from `TimeRangeFieldLabel` so `aria-labelledby` stays synced (`DEF-5C-07`).
    pub fn register_custom_label_id(&mut self, id: Option<String>) {
        let normalized = id.filter(|s| !s.is_empty());
        if *self.custom_label_id.peek() != normalized {
            self.custom_label_id.set(normalized);
        }
    }

    pub fn default_part_value(&self, kind: RangeFieldInputKind) -> Option<TimeValue> {
        let dv = *self.default_value.borrow();
        match kind {
            RangeFieldInputKind::Start => dv.start,
            RangeFieldInputKind::End => dv.end,
        }
    }

    pub fn is_part_invalid(&self, kind: RangeFieldInputKind) -> bool {
        self.state.read().is_part_invalid(kind)
    }

    pub fn root_attrs(&self, id_override: Option<String>) -> TimeRangeFieldRootAttrs {
        let st = self.state.read();
        let id = id_override
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.rels.root_id().to_string());
        TimeRangeFieldRootAttrs::new(
            id,
            self.effective_label_id(),
            st.def().dir,
            st.is_invalid(),
            st.def().disabled,
        )
    }

    pub fn label_attrs(&self, id_override: Option<String>) -> TimeRangeFieldLabelAttrs {
        let st = self.state.read();
        let id = id_override
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.effective_label_id());
        TimeRangeFieldLabelAttrs::new(id, st.is_invalid(), st.def().disabled)
    }

    pub fn register_part_runtime(&self, kind: RangeFieldInputKind, rt: TimeFieldRuntime) {
        match kind {
            RangeFieldInputKind::Start => *self.start_runtime.borrow_mut() = Some(rt),
            RangeFieldInputKind::End => *self.end_runtime.borrow_mut() = Some(rt),
        }
    }

    pub fn focus_first_segment(&self) {
        if let Some(rt) = self.start_runtime.borrow().as_ref() {
            rt.focus_first_segment();
        } else if let Some(rt) = self.end_runtime.borrow().as_ref() {
            rt.focus_first_segment();
        }
    }

    pub fn focus_last_segment(&self) {
        if let Some(rt) = self.end_runtime.borrow().as_ref() {
            rt.focus_last_segment();
        } else if let Some(rt) = self.start_runtime.borrow().as_ref() {
            rt.focus_last_segment();
        }
    }

    pub fn handle_part_next_boundary(&self, kind: RangeFieldInputKind) {
        if kind == RangeFieldInputKind::Start
            && let Some(rt) = self.end_runtime.borrow().as_ref()
        {
            rt.focus_first_segment();
        }
    }

    pub fn handle_part_prev_boundary(&self, kind: RangeFieldInputKind) {
        if kind == RangeFieldInputKind::End
            && let Some(rt) = self.start_runtime.borrow().as_ref()
        {
            rt.focus_last_segment();
        }
    }

    pub fn sync_validation_and_notify(&mut self) {
        let (range, min_val, max_val) = {
            let st = self.state.peek();
            (st.value(), st.def().min_value, st.def().max_value)
        };
        let custom_errors = if range.is_complete() {
            self.validate
                .borrow()
                .as_ref()
                .and_then(|cb| cb.call(range))
        } else {
            None
        };

        let next_reason = validate_time_range(range, min_val, max_val, custom_errors).err();
        if self.state.peek().invalid_reason() != next_reason.as_ref() {
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
                if let Some(handler) = self.on_invalid.borrow().as_ref() {
                    handler.call(FieldInvalidEvent { reason, messages });
                }
            }
        }
    }

    pub fn update_part_value(&mut self, kind: RangeFieldInputKind, next: Option<TimeValue>) {
        let prev_range = self.state.peek().value();
        let prev_part = match kind {
            RangeFieldInputKind::Start => prev_range.start,
            RangeFieldInputKind::End => prev_range.end,
        };
        if prev_part == next {
            return;
        }

        self.state.write().set_part_value(kind, next);
        let next_range = self.state.peek().value();

        match kind {
            RangeFieldInputKind::Start => {
                if *self.start_value.peek() != next {
                    self.start_value.set(next);
                }
                if let Some(cb) = self.on_start_value_change.borrow().as_ref() {
                    cb.call(next);
                }
            }
            RangeFieldInputKind::End => {
                if *self.end_value.peek() != next {
                    self.end_value.set(next);
                }
                if let Some(cb) = self.on_end_value_change.borrow().as_ref() {
                    cb.call(next);
                }
            }
        }

        if let Some(mut sig) = *self.controlled_value.borrow()
            && *sig.peek() != next_range
        {
            sig.set(next_range);
        }
        if let Some(cb) = self.on_value_change.borrow().as_ref() {
            cb.call(next_range);
        }

        self.sync_validation_and_notify();
    }

    pub fn update_placeholder(&mut self, next: TimeValue) {
        if *self.placeholder.peek() == next {
            return;
        }
        self.placeholder.set(next);
        self.state.write().set_placeholder(next);
        if let Some(mut sig) = *self.controlled_placeholder.borrow()
            && *sig.peek() != next
        {
            sig.set(next);
        }
        if let Some(cb) = self.on_placeholder_change.borrow().as_ref() {
            cb.call(next);
        }
    }

    pub fn reconcile_external_state(
        &mut self,
        external_value: Option<TimeRange>,
        external_placeholder: Option<TimeValue>,
        is_invalid_override: Option<bool>,
    ) {
        if let Some(val) = external_value
            && self.state.peek().value() != val
        {
            self.state.write().set_value(val);
            if *self.start_value.peek() != val.start {
                self.start_value.set(val.start);
            }
            if *self.end_value.peek() != val.end {
                self.end_value.set(val.end);
            }
        }
        if let Some(ph) = external_placeholder
            && *self.placeholder.peek() != ph
        {
            self.placeholder.set(ph);
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

/// Dioxus context wrapping [`TimeRangeFieldRuntime`].
#[derive(Clone, PartialEq)]
pub struct TimeRangeFieldContext(pub TimeRangeFieldRuntime);

/// Consumes the nearest ancestor [`TimeRangeFieldRuntime`] context (panics if outside `TimeRangeFieldRoot`).
pub fn use_time_range_field_context() -> TimeRangeFieldRuntime {
    use_context::<TimeRangeFieldContext>().0
}

/// Attempts to consume the nearest ancestor [`TimeRangeFieldRuntime`] context.
pub fn try_use_time_range_field_context() -> Option<TimeRangeFieldRuntime> {
    try_consume_context::<TimeRangeFieldContext>().map(|ctx| ctx.0)
}
