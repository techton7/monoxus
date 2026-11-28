use std::ops::{Deref, DerefMut};

use dioxus::prelude::*;
use ui_sdk::date_picker::{PickerPolicy, PickerRelationships, PickerState};

use crate::foundation::{
    date_time::{CalendarConfig, CalendarDate, DateValue},
    shared::ScopeHandle,
    state::{ControlledBinding, sync_signal},
};

use super::state::DatePickerPolicy;

/// Shared calendar configuration forwarded from `DatePickerRoot` to `DatePickerCalendar`.
#[derive(Clone, Debug, PartialEq)]
pub struct DatePickerCalendarConfig {
    pub config: CalendarConfig,
    pub initial_focus: bool,
}

impl Deref for DatePickerCalendarConfig {
    type Target = CalendarConfig;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

/// Zero-overhead policy-parameterized runtime core shared by `DatePicker` and `DateRangePicker`.
pub struct PickerRuntimeCore<P: PickerPolicy> {
    pub state: Signal<PickerState<P>>,
    pub rels: PickerRelationships<P>,
    pub value: Signal<P::Value>,
    pub calendar_value: Signal<P::CalendarValue>,
    pub placeholder: Signal<DateValue>,
    pub calendar_placeholder: Signal<CalendarDate>,
    pub open: Signal<bool>,
    pub(crate) trigger_dom_id: Signal<Option<String>>,
    pub(crate) content_dom_id: Signal<Option<String>>,
    pub(crate) controlled_value: CopyValue<ControlledBinding<P::Value>>,
    pub(crate) controlled_placeholder: CopyValue<ControlledBinding<DateValue>>,
    pub(crate) controlled_open: CopyValue<ControlledBinding<bool>>,
    pub on_value_change: CopyValue<Option<EventHandler<P::Value>>>,
    pub on_placeholder_change: CopyValue<Option<EventHandler<DateValue>>>,
    pub on_open_change: CopyValue<Option<EventHandler<bool>>>,
    pub(crate) on_composite_focus_in: CopyValue<Option<EventHandler<FocusEvent>>>,
    pub(crate) on_composite_focus_out: CopyValue<Option<EventHandler<FocusEvent>>>,
    pub is_date_disabled: Signal<Option<Callback<CalendarDate, bool>>>,
    pub is_date_unavailable: Signal<Option<Callback<CalendarDate, bool>>>,
    pub predicate_rev: Signal<u64>,
}

impl<P: PickerPolicy> Clone for PickerRuntimeCore<P> {
    fn clone(&self) -> Self {
        Self {
            state: self.state,
            rels: self.rels.clone(),
            value: self.value,
            calendar_value: self.calendar_value,
            placeholder: self.placeholder,
            calendar_placeholder: self.calendar_placeholder,
            open: self.open,
            trigger_dom_id: self.trigger_dom_id,
            content_dom_id: self.content_dom_id,
            controlled_value: self.controlled_value,
            controlled_placeholder: self.controlled_placeholder,
            controlled_open: self.controlled_open,
            on_value_change: self.on_value_change,
            on_placeholder_change: self.on_placeholder_change,
            on_open_change: self.on_open_change,
            on_composite_focus_in: self.on_composite_focus_in,
            on_composite_focus_out: self.on_composite_focus_out,
            is_date_disabled: self.is_date_disabled,
            is_date_unavailable: self.is_date_unavailable,
            predicate_rev: self.predicate_rev,
        }
    }
}

impl<P: PickerPolicy> PartialEq for PickerRuntimeCore<P> {
    fn eq(&self, other: &Self) -> bool {
        self.rels == other.rels && self.state == other.state
    }
}

impl<P: PickerPolicy> PickerRuntimeCore<P> {
    pub fn snapshot(&self) -> P::Snapshot {
        self.state.read().snapshot()
    }

    pub(crate) fn register_trigger_dom_id(&self, id: String) {
        let mut sig = self.trigger_dom_id;
        sync_signal(&mut sig, Some(id));
    }

    pub(crate) fn current_trigger_dom_id(&self) -> String {
        self.trigger_dom_id
            .try_read()
            .ok()
            .and_then(|g| g.clone())
            .unwrap_or_else(|| {
                ScopeHandle::root("popover")
                    .child(self.rels.popover_scope().token())
                    .qualify("trigger")
            })
    }

    pub(crate) fn register_content_dom_id(&self, id: String) {
        let mut sig = self.content_dom_id;
        sync_signal(&mut sig, Some(id));
    }

    pub(crate) fn current_content_dom_id(&self) -> String {
        self.content_dom_id
            .try_read()
            .ok()
            .and_then(|g| g.clone())
            .unwrap_or_else(|| {
                ScopeHandle::root("popover")
                    .child(self.rels.popover_scope().token())
                    .qualify("content")
            })
    }

    pub(crate) fn set_composite_focus_handlers(
        &self,
        focus_in: Option<EventHandler<FocusEvent>>,
        focus_out: Option<EventHandler<FocusEvent>>,
    ) {
        let mut fi = self.on_composite_focus_in;
        fi.set(focus_in);
        let mut fo = self.on_composite_focus_out;
        fo.set(focus_out);
    }

    pub(crate) fn notify_composite_focus_in(&self, evt: FocusEvent) {
        if let Some(cb) = self.on_composite_focus_in.try_read().ok().and_then(|s| *s) {
            cb.call(evt);
        }
    }

    pub(crate) fn notify_composite_focus_out(&self, evt: FocusEvent) {
        if let Some(cb) = self.on_composite_focus_out.try_read().ok().and_then(|s| *s) {
            cb.call(evt);
        }
    }

    pub(crate) fn value_binding(&self) -> ControlledBinding<P::Value> {
        self.controlled_value
            .try_read()
            .map(|s| *s)
            .unwrap_or(ControlledBinding::Uncontrolled)
    }

    pub(crate) fn placeholder_binding(&self) -> ControlledBinding<DateValue> {
        self.controlled_placeholder
            .try_read()
            .map(|s| *s)
            .unwrap_or(ControlledBinding::Uncontrolled)
    }

    pub(crate) fn open_binding(&self) -> ControlledBinding<bool> {
        self.controlled_open
            .try_read()
            .map(|s| *s)
            .unwrap_or(ControlledBinding::Uncontrolled)
    }

    pub(crate) fn sync_signals_from_state(&mut self) {
        let (val, cal_val, ph, cal_ph, open) = {
            let st = self.state.peek();
            (
                st.value(),
                st.calendar_value(),
                st.placeholder(),
                st.calendar_placeholder(),
                st.is_open(),
            )
        };
        if *self.value.peek() != val {
            self.value.set(val);
        }
        if *self.calendar_value.peek() != cal_val {
            self.calendar_value.set(cal_val);
        }
        if *self.placeholder.peek() != ph {
            self.placeholder.set(ph);
        }
        if *self.calendar_placeholder.peek() != cal_ph {
            self.calendar_placeholder.set(cal_ph);
        }
        if *self.open.peek() != open {
            self.open.set(open);
        }
    }

    pub(crate) fn notify_value_change(&self, next: P::Value) {
        self.value_binding().try_write_two_way(next);
        if let Some(cb) = self.on_value_change.try_read().ok().and_then(|s| *s) {
            cb.call(next);
        }
    }

    pub(crate) fn notify_placeholder_change(&self, next: DateValue) {
        self.placeholder_binding().try_write_two_way(next);
        if let Some(cb) = self.on_placeholder_change.try_read().ok().and_then(|s| *s) {
            cb.call(next);
        }
    }

    pub(crate) fn notify_open_change(&self, next: bool) {
        self.open_binding().try_write_two_way(next);
        if let Some(cb) = self.on_open_change.try_read().ok().and_then(|s| *s) {
            cb.call(next);
        }
    }

    pub fn set_open(&mut self, next: bool) {
        let prev = self.state.peek().is_open();
        let open_binding = self.open_binding();
        if open_binding.is_read_only() {
            let mut candidate = self.state.peek().clone();
            let effective = candidate.set_open(next);
            if effective != prev {
                self.notify_open_change(effective);
            }
            if let Some(accepted) = open_binding.peek_cloned()
                && self.state.peek().is_open() != accepted
            {
                self.state.write().set_open(accepted);
            }
            self.sync_signals_from_state();
            return;
        }
        let effective = self.state.write().set_open(next);
        self.sync_signals_from_state();
        if effective != prev {
            self.notify_open_change(effective);
        }
    }

    pub fn handle_popover_open_change(&mut self, next: bool) {
        self.set_open(next);
    }

    pub fn handle_field_placeholder_change(&mut self, next: DateValue) {
        if self.state.peek().placeholder() == next {
            return;
        }
        let ph_binding = self.placeholder_binding();
        if ph_binding.is_read_only() {
            self.notify_placeholder_change(next);
            if let Some(accepted) = ph_binding.peek_cloned()
                && self.state.peek().placeholder() != accepted
            {
                self.state.write().set_placeholder(accepted);
            }
            self.sync_signals_from_state();
            return;
        }
        self.state.write().set_placeholder(next);
        self.sync_signals_from_state();
        self.notify_placeholder_change(next);
    }

    pub fn handle_calendar_placeholder_change(&mut self, next_date: CalendarDate) {
        let prev_ph = self.state.peek().placeholder();
        let ph_binding = self.placeholder_binding();
        if ph_binding.is_read_only() {
            let mut candidate = self.state.peek().clone();
            let next_ph = candidate.set_calendar_placeholder(next_date);
            if next_ph != prev_ph {
                self.notify_placeholder_change(next_ph);
            }
            if let Some(accepted) = ph_binding.peek_cloned()
                && self.state.peek().placeholder() != accepted
            {
                self.state.write().set_placeholder(accepted);
            }
            self.sync_signals_from_state();
            return;
        }
        let next_ph = self.state.write().set_calendar_placeholder(next_date);
        self.sync_signals_from_state();
        if next_ph != prev_ph {
            self.notify_placeholder_change(next_ph);
        }
    }
}

/// Reactive runtime coordinating `DateField`, `Popover`, and `Calendar` for `DatePicker`.
#[derive(Clone)]
pub struct DatePickerRuntime {
    pub core: PickerRuntimeCore<DatePickerPolicy>,
    pub calendar_config: Signal<DatePickerCalendarConfig>,
}

impl Deref for DatePickerRuntime {
    type Target = PickerRuntimeCore<DatePickerPolicy>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.core
    }
}

impl DerefMut for DatePickerRuntime {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.core
    }
}

impl PartialEq for DatePickerRuntime {
    fn eq(&self, other: &Self) -> bool {
        self.core == other.core
    }
}

impl DatePickerRuntime {
    pub fn handle_field_value_change(&mut self, next: Option<DateValue>) {
        let (prev_val, prev_ph) = {
            let st = self.state.peek();
            (st.value(), st.placeholder())
        };
        if prev_val == next {
            return;
        }
        let val_binding = self.value_binding();
        let ph_binding = self.placeholder_binding();

        if val_binding.is_read_only() {
            self.notify_value_change(next);
            if let Some(accepted) = val_binding.peek_cloned()
                && self.state.peek().value() != accepted
            {
                self.state.write().set_field_value(accepted);
                if ph_binding.is_read_only() {
                    self.state.write().set_placeholder(prev_ph);
                }
            }
            self.sync_signals_from_state();
            return;
        }

        self.state.write().set_field_value(next);
        let next_ph = self.state.peek().placeholder();
        if ph_binding.is_read_only() && next_ph != prev_ph {
            self.state.write().set_placeholder(prev_ph);
        }
        self.sync_signals_from_state();
        self.notify_value_change(next);
        if next_ph != prev_ph {
            self.notify_placeholder_change(next_ph);
            if ph_binding.is_read_only()
                && let Some(accepted_ph) = ph_binding.peek_cloned()
                && self.state.peek().placeholder() != accepted_ph
            {
                self.state.write().set_placeholder(accepted_ph);
                self.sync_signals_from_state();
            }
        }
    }

    pub fn handle_calendar_value_change(&mut self, next: Option<CalendarDate>) {
        let (prev_val, prev_ph) = {
            let st = self.state.peek();
            (st.value(), st.placeholder())
        };
        let val_binding = self.value_binding();
        let ph_binding = self.placeholder_binding();

        let mut candidate = self.state.peek().clone();
        let outcome = candidate.apply_calendar_value(next);

        if !val_binding.is_read_only() && outcome.next_value != prev_val {
            self.state.write().set_field_value(outcome.next_value);
        }
        if !ph_binding.is_read_only() && outcome.next_placeholder != prev_ph {
            self.state.write().set_placeholder(outcome.next_placeholder);
        } else if ph_binding.is_read_only() && self.state.peek().placeholder() != prev_ph {
            self.state.write().set_placeholder(prev_ph);
        }

        self.sync_signals_from_state();
        if outcome.next_value != prev_val {
            self.notify_value_change(outcome.next_value);
            if val_binding.is_read_only()
                && let Some(accepted) = val_binding.peek_cloned()
                && self.state.peek().value() != accepted
            {
                self.state.write().set_field_value(accepted);
                if ph_binding.is_read_only() {
                    self.state.write().set_placeholder(prev_ph);
                }
            }
        }
        if outcome.next_placeholder != prev_ph
            && (!val_binding.is_read_only() || self.state.peek().value() == outcome.next_value)
        {
            self.notify_placeholder_change(outcome.next_placeholder);
            if ph_binding.is_read_only()
                && let Some(accepted_ph) = ph_binding.peek_cloned()
                && self.state.peek().placeholder() != accepted_ph
            {
                self.state.write().set_placeholder(accepted_ph);
            }
        }
        self.sync_signals_from_state();
    }

    pub fn handle_calendar_date_select(&mut self, selected: CalendarDate) {
        let (prev_val, prev_ph, prev_open) = {
            let st = self.state.peek();
            (st.value(), st.placeholder(), st.is_open())
        };
        let val_binding = self.value_binding();
        let ph_binding = self.placeholder_binding();
        let open_binding = self.open_binding();

        let mut candidate = self.state.peek().clone();
        let outcome = candidate.commit_calendar_date_select(selected);
        let next_open = candidate.is_open();

        if !val_binding.is_read_only() && outcome.next_value != prev_val {
            self.state.write().set_field_value(outcome.next_value);
        }
        if !ph_binding.is_read_only() && outcome.next_placeholder != prev_ph {
            self.state.write().set_placeholder(outcome.next_placeholder);
        } else if ph_binding.is_read_only() && self.state.peek().placeholder() != prev_ph {
            self.state.write().set_placeholder(prev_ph);
        }
        if !open_binding.is_read_only() && next_open != prev_open {
            self.state.write().set_open(next_open);
        }

        self.sync_signals_from_state();
        if outcome.next_value != prev_val {
            self.notify_value_change(outcome.next_value);
            if val_binding.is_read_only()
                && let Some(accepted) = val_binding.peek_cloned()
                && self.state.peek().value() != accepted
            {
                self.state.write().set_field_value(accepted);
                if ph_binding.is_read_only() {
                    self.state.write().set_placeholder(prev_ph);
                }
            }
        }
        if outcome.next_placeholder != prev_ph
            && (!val_binding.is_read_only() || self.state.peek().value() == outcome.next_value)
        {
            self.notify_placeholder_change(outcome.next_placeholder);
            if ph_binding.is_read_only()
                && let Some(accepted_ph) = ph_binding.peek_cloned()
                && self.state.peek().placeholder() != accepted_ph
            {
                self.state.write().set_placeholder(accepted_ph);
            }
        }
        if next_open != prev_open {
            self.notify_open_change(next_open);
            if open_binding.is_read_only()
                && let Some(accepted_open) = open_binding.peek_cloned()
                && self.state.peek().is_open() != accepted_open
            {
                self.state.write().set_open(accepted_open);
            }
        }
        self.sync_signals_from_state();
    }

    pub fn reconcile_external_state(
        &mut self,
        external_value: Option<Option<DateValue>>,
        external_placeholder: Option<DateValue>,
        external_open: Option<bool>,
    ) {
        if let Some(val) = external_value
            && self.state.peek().value() != val
        {
            self.state.write().set_field_value(val);
        }
        if let Some(ph) = external_placeholder
            && self.state.peek().placeholder() != ph
        {
            self.state.write().set_placeholder(ph);
        }
        if let Some(open) = external_open
            && self.state.peek().is_open() != open
        {
            self.state.write().set_open(open);
        }
        self.sync_signals_from_state();
    }
}

/// Dioxus context wrapping [`DatePickerRuntime`].
#[derive(Clone, PartialEq)]
pub struct DatePickerContext(pub DatePickerRuntime);

/// Consumes the nearest ancestor [`DatePickerRuntime`] context (panics if outside `DatePickerRoot`).
pub fn use_date_picker_context() -> DatePickerRuntime {
    use_context::<DatePickerContext>().0
}

/// Attempts to consume the nearest ancestor [`DatePickerRuntime`] context.
pub fn try_use_date_picker_context() -> Option<DatePickerRuntime> {
    try_consume_context::<DatePickerContext>().map(|ctx| ctx.0)
}
