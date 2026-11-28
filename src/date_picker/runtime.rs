use std::ops::{Deref, DerefMut};

use dioxus::prelude::*;
use ui_sdk::date_picker::{PickerPolicy, PickerRelationships, PickerState};
use ui_sdk::kernel::{ChangeContext, ChangeReason, ChangeSource};

use crate::{
    agent::{cancel_current_agent_action, current_change_context},
    foundation::{
        date_time::{CalendarConfig, CalendarDate, DateValue},
        shared::ScopeHandle,
        state::{ControllableSignal, sync_signal},
    },
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
    pub(crate) value_ctl: ControllableSignal<P::Value>,
    pub(crate) placeholder_ctl: ControllableSignal<DateValue>,
    pub(crate) open_ctl: ControllableSignal<bool>,
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
            value_ctl: self.value_ctl,
            placeholder_ctl: self.placeholder_ctl,
            open_ctl: self.open_ctl,
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

    pub fn set_open(&mut self, next: bool) {
        let ctx = current_change_context()
            .unwrap_or_else(|| ChangeContext::new(ChangeReason::ValueChange, ChangeSource::Mouse));
        let _ = self.set_open_with_context(next, ctx);
    }

    pub(crate) fn set_open_with_context(&mut self, next: bool, ctx: ChangeContext) -> bool {
        let prev = self.state.peek().is_open();
        let mut candidate = self.state.peek().clone();
        let effective = candidate.set_open(next);
        if effective == prev {
            return false;
        }
        let outcome = self.open_ctl.set_with_context(effective, ctx);
        if outcome.is_canceled() {
            cancel_current_agent_action();
            return true;
        }
        if outcome.is_committed() {
            self.state.write().set_open(effective);
            self.sync_signals_from_state();
        }
        false
    }

    pub fn handle_popover_open_change(&mut self, next: bool) {
        self.set_open(next);
    }

    pub fn handle_field_placeholder_change(&mut self, next: DateValue) {
        if self.state.peek().placeholder() == next {
            return;
        }
        let ctx = current_change_context()
            .unwrap_or_else(|| ChangeContext::new(ChangeReason::ValueChange, ChangeSource::Mouse));
        let outcome = self.placeholder_ctl.set_with_context(next, ctx);
        if outcome.is_canceled() {
            cancel_current_agent_action();
            return;
        }
        if outcome.is_committed() {
            self.state.write().set_placeholder(next);
            self.sync_signals_from_state();
        }
    }

    pub fn handle_calendar_placeholder_change(&mut self, next_date: CalendarDate) {
        let prev_ph = self.state.peek().placeholder();
        let mut candidate = self.state.peek().clone();
        let next_ph = candidate.set_calendar_placeholder(next_date);
        if next_ph == prev_ph {
            return;
        }
        let ctx = current_change_context()
            .unwrap_or_else(|| ChangeContext::new(ChangeReason::ValueChange, ChangeSource::Mouse));
        let outcome = self.placeholder_ctl.set_with_context(next_ph, ctx);
        if outcome.is_canceled() {
            cancel_current_agent_action();
            return;
        }
        if outcome.is_committed() {
            self.state.write().set_placeholder(next_ph);
            self.sync_signals_from_state();
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
        let ctx = current_change_context()
            .unwrap_or_else(|| ChangeContext::new(ChangeReason::ValueChange, ChangeSource::Mouse));
        let _ = self.handle_field_value_change_with_context(next, ctx);
    }

    pub(crate) fn handle_field_value_change_with_context(
        &mut self,
        next: Option<DateValue>,
        ctx: ChangeContext,
    ) -> bool {
        let (prev_val, prev_ph) = {
            let st = self.state.peek();
            (st.value(), st.placeholder())
        };
        if prev_val == next {
            return false;
        }

        let outcome = self.value_ctl.set_with_context(next, ctx);
        if outcome.is_canceled() {
            cancel_current_agent_action();
            return true;
        }
        if outcome.is_committed() {
            self.state.write().set_field_value(next);
            let next_ph = self.state.peek().placeholder();
            if next_ph != prev_ph {
                let ph_outcome = self.placeholder_ctl.set_with_context(next_ph, ctx);
                if !ph_outcome.is_committed() {
                    self.state.write().set_placeholder(prev_ph);
                }
            }
            self.sync_signals_from_state();
        }
        false
    }

    pub fn handle_calendar_value_change(&mut self, next: Option<CalendarDate>) {
        let (prev_val, prev_ph) = {
            let st = self.state.peek();
            (st.value(), st.placeholder())
        };

        let mut candidate = self.state.peek().clone();
        let outcome = candidate.apply_calendar_value(next);
        let ctx = current_change_context()
            .unwrap_or_else(|| ChangeContext::new(ChangeReason::ValueChange, ChangeSource::Mouse));

        if outcome.next_value != prev_val {
            let val_outcome = self.value_ctl.set_with_context(outcome.next_value, ctx);
            if val_outcome.is_canceled() {
                cancel_current_agent_action();
                return;
            }
            if val_outcome.is_committed() {
                self.state.write().set_field_value(outcome.next_value);
            }
        }
        if outcome.next_placeholder != prev_ph && !self.value_ctl.is_controlled() {
            let ph_outcome = self
                .placeholder_ctl
                .set_with_context(outcome.next_placeholder, ctx);
            if ph_outcome.is_committed() {
                self.state.write().set_placeholder(outcome.next_placeholder);
            } else {
                self.state.write().set_placeholder(prev_ph);
            }
        } else if self.placeholder_ctl.is_controlled() && self.state.peek().placeholder() != prev_ph
        {
            self.state.write().set_placeholder(prev_ph);
        }

        self.sync_signals_from_state();
    }

    pub fn handle_calendar_date_select(&mut self, selected: CalendarDate) {
        let (prev_val, prev_ph, prev_open) = {
            let st = self.state.peek();
            (st.value(), st.placeholder(), st.is_open())
        };

        let mut candidate = self.state.peek().clone();
        let outcome = candidate.commit_calendar_date_select(selected);
        let next_open = candidate.is_open();
        let ctx = current_change_context()
            .unwrap_or_else(|| ChangeContext::new(ChangeReason::ValueChange, ChangeSource::Mouse));

        if outcome.next_value != prev_val {
            let val_outcome = self.value_ctl.set_with_context(outcome.next_value, ctx);
            if val_outcome.is_canceled() {
                cancel_current_agent_action();
                return;
            }
            if val_outcome.is_committed() {
                self.state.write().set_field_value(outcome.next_value);
            }
        }
        if outcome.next_placeholder != prev_ph && !self.value_ctl.is_controlled() {
            let ph_outcome = self
                .placeholder_ctl
                .set_with_context(outcome.next_placeholder, ctx);
            if ph_outcome.is_committed() {
                self.state.write().set_placeholder(outcome.next_placeholder);
            } else {
                self.state.write().set_placeholder(prev_ph);
            }
        } else if self.placeholder_ctl.is_controlled() && self.state.peek().placeholder() != prev_ph
        {
            self.state.write().set_placeholder(prev_ph);
        }
        if next_open != prev_open {
            let open_outcome = self.open_ctl.set_with_context(next_open, ctx);
            if open_outcome.is_committed() {
                self.state.write().set_open(next_open);
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
