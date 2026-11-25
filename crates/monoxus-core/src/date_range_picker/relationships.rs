use crate::foundation::shared::ScopeHandle;

/// Default CSS selector used by `DateRangePickerContent` for `pickerOpenFocus` (`OVL-3`).
pub const DATE_RANGE_PICKER_OPEN_FOCUS_SELECTOR: &str = "[data-range-calendar-day][data-focused]";

/// Deterministic scoped ID relationships for `DateRangePicker` and its composed primitives.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DateRangePickerRelationships {
    scope: ScopeHandle,
    root_id: String,
    popover_scope: ScopeHandle,
    field_scope: ScopeHandle,
    calendar_scope: ScopeHandle,
    trigger_id: String,
    content_id: String,
}

impl DateRangePickerRelationships {
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
        DATE_RANGE_PICKER_OPEN_FOCUS_SELECTOR
    }
}
