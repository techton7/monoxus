use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::{
    date_field::DateFieldRuntime,
    foundation::{
        browser::focus_element_by_id,
        date_time::{DateValue, DateValueRange, FieldInvalidEvent, FieldInvalidReason},
    },
};

use super::{
    attrs::{DateRangeFieldLabelAttrs, DateRangeFieldRootAttrs},
    relationships::DateRangeFieldRelationships,
    state::DateRangeFieldState,
    types::{DateRangeFieldSnapshot, RangeFieldInputKind},
};

/// Reactive runtime for `DateRangeField`.
#[derive(Clone)]
pub struct DateRangeFieldRuntime {
    pub state: Signal<DateRangeFieldState>,
    pub rels: DateRangeFieldRelationships,
    pub start_value: Signal<Option<DateValue>>,
    pub end_value: Signal<Option<DateValue>>,
    pub placeholder: Signal<DateValue>,
    pub last_invalid_reason: Signal<Option<FieldInvalidReason>>,
    pub trailing_trigger_id: Signal<Option<String>>,
    pub start_runtime: Rc<RefCell<Option<DateFieldRuntime>>>,
    pub end_runtime: Rc<RefCell<Option<DateFieldRuntime>>>,
    pub controlled_value: Rc<RefCell<Option<Signal<DateValueRange>>>>,
    pub controlled_placeholder: Rc<RefCell<Option<Signal<DateValue>>>>,
    pub on_value_change: Rc<RefCell<Option<EventHandler<DateValueRange>>>>,
    pub on_start_value_change: Rc<RefCell<Option<EventHandler<Option<DateValue>>>>>,
    pub on_end_value_change: Rc<RefCell<Option<EventHandler<Option<DateValue>>>>>,
    pub on_placeholder_change: Rc<RefCell<Option<EventHandler<DateValue>>>>,
    pub validate: Rc<RefCell<Option<Callback<DateValueRange, Option<Vec<String>>>>>>,
    pub on_invalid: Rc<RefCell<Option<EventHandler<FieldInvalidEvent>>>>,
}

impl PartialEq for DateRangeFieldRuntime {
    fn eq(&self, other: &Self) -> bool {
        self.rels == other.rels && self.state == other.state
    }
}

impl DateRangeFieldRuntime {
    pub fn snapshot(&self) -> DateRangeFieldSnapshot {
        self.state.read().snapshot()
    }

    pub fn root_attrs(&self, id_override: Option<String>) -> DateRangeFieldRootAttrs {
        let st = self.state.read();
        let id = id_override
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.rels.root_id().to_string());
        DateRangeFieldRootAttrs::new(
            id,
            self.rels.label_id().to_string(),
            st.def().dir,
            st.is_invalid(),
            st.def().disabled,
        )
    }

    pub fn label_attrs(&self, id_override: Option<String>) -> DateRangeFieldLabelAttrs {
        let st = self.state.read();
        let id = id_override
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.rels.label_id().to_string());
        DateRangeFieldLabelAttrs::new(id, st.is_invalid(), st.def().disabled)
    }

    pub fn register_part_runtime(&self, kind: RangeFieldInputKind, rt: DateFieldRuntime) {
        match kind {
            RangeFieldInputKind::Start => *self.start_runtime.borrow_mut() = Some(rt),
            RangeFieldInputKind::End => *self.end_runtime.borrow_mut() = Some(rt),
        }
    }

    /// Registers an optional trailing picker trigger ID (`DateRangePickerTrigger`)
    /// so `ArrowRight` from the last `End` segment focuses the trigger.
    pub fn register_trigger_id(&mut self, id: Option<String>) {
        if *self.trailing_trigger_id.peek() != id {
            self.trailing_trigger_id.set(id);
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
        match kind {
            RangeFieldInputKind::Start => {
                if let Some(rt) = self.end_runtime.borrow().as_ref() {
                    rt.focus_first_segment();
                }
            }
            RangeFieldInputKind::End => {
                if let Some(trigger_id) = self.trailing_trigger_id.peek().as_deref() {
                    focus_element_by_id(trigger_id);
                }
            }
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
        let range = self.state.peek().value();
        let custom_errors = if range.is_complete() {
            self.validate
                .borrow()
                .as_ref()
                .and_then(|cb| cb.call(range))
        } else {
            None
        };

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

    pub fn update_part_value(&mut self, kind: RangeFieldInputKind, next: Option<DateValue>) {
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

    pub fn update_placeholder(&mut self, next: DateValue) {
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
        external_value: Option<DateValueRange>,
        external_placeholder: Option<DateValue>,
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
        if let Some(inv) = is_invalid_override {
            self.state.write().set_external_invalid(inv);
        }
        self.sync_validation_and_notify();
    }
}

/// Dioxus context wrapping [`DateRangeFieldRuntime`].
#[derive(Clone, PartialEq)]
pub struct DateRangeFieldContext(pub DateRangeFieldRuntime);

/// Consumes the nearest ancestor [`DateRangeFieldRuntime`] context (panics if outside `DateRangeFieldRoot`).
pub fn use_date_range_field_context() -> DateRangeFieldRuntime {
    use_context::<DateRangeFieldContext>().0
}

/// Attempts to consume the nearest ancestor [`DateRangeFieldRuntime`] context.
pub fn try_use_date_range_field_context() -> Option<DateRangeFieldRuntime> {
    try_consume_context::<DateRangeFieldContext>().map(|ctx| ctx.0)
}
