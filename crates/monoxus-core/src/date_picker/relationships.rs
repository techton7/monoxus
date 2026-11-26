use std::marker::PhantomData;

use crate::foundation::shared::ScopeHandle;

/// Default CSS selector used by `DatePickerContent` for `pickerOpenFocus` (`OVL-3`).
pub const DATE_PICKER_OPEN_FOCUS_SELECTOR: &str = "[data-calendar-day][data-focused]";

use super::state::{DatePickerPolicy, PickerPolicy};

/// Deterministic scoped ID relationships shared by `DatePicker` and `DateRangePicker`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PickerRelationships<P: PickerPolicy> {
    scope: ScopeHandle,
    root_id: String,
    popover_scope: ScopeHandle,
    field_scope: ScopeHandle,
    calendar_scope: ScopeHandle,
    trigger_id: String,
    content_id: String,
    _policy: PhantomData<P>,
}

impl<P: PickerPolicy> PickerRelationships<P> {
    pub fn new(scope: ScopeHandle) -> Self {
        let root_id = scope.token().to_string();
        let popover_scope = scope.child("popover");
        let field_scope = scope.child("field");
        let calendar_scope = scope.child("calendar");
        let trigger_id = scope.qualify("trigger");
        let content_id = scope.qualify("content");
        Self {
            scope,
            root_id,
            popover_scope,
            field_scope,
            calendar_scope,
            trigger_id,
            content_id,
            _policy: PhantomData,
        }
    }

    #[inline]
    pub fn scope(&self) -> &ScopeHandle {
        &self.scope
    }

    #[inline]
    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    #[inline]
    pub fn popover_scope(&self) -> &ScopeHandle {
        &self.popover_scope
    }

    #[inline]
    pub fn field_scope(&self) -> &ScopeHandle {
        &self.field_scope
    }

    #[inline]
    pub fn calendar_scope(&self) -> &ScopeHandle {
        &self.calendar_scope
    }

    #[inline]
    pub fn trigger_id(&self) -> &str {
        &self.trigger_id
    }

    #[inline]
    pub fn content_id(&self) -> &str {
        &self.content_id
    }

    #[inline]
    pub const fn open_focus_selector(&self) -> &'static str {
        P::OPEN_FOCUS_SELECTOR
    }
}

/// Deterministic scoped ID relationships for `DatePicker` and its composed primitives.
pub type DatePickerRelationships = PickerRelationships<DatePickerPolicy>;
