use crate::foundation::{date_time::CalendarDate, shared::ScopeHandle};

/// Deterministic DOM ID builder for all parts of a `RangeCalendar` instance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RangeCalendarRelationships {
    scope: ScopeHandle,
    root_id: String,
    header_id: String,
    heading_id: String,
    accessible_heading_id: String,
    live_region_id: String,
    prev_button_id: String,
    next_button_id: String,
    month_select_id: String,
    year_select_id: String,
}

impl RangeCalendarRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            root_id: scope.token(),
            header_id: scope.qualify("header"),
            heading_id: scope.qualify("heading"),
            accessible_heading_id: scope.qualify("accessible-heading"),
            live_region_id: scope.qualify("live-region"),
            prev_button_id: scope.qualify("prev-button"),
            next_button_id: scope.qualify("next-button"),
            month_select_id: scope.qualify("month-select"),
            year_select_id: scope.qualify("year-select"),
            scope,
        }
    }

    pub fn scope(&self) -> &ScopeHandle {
        &self.scope
    }

    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    pub fn header_id(&self) -> &str {
        &self.header_id
    }

    pub fn heading_id(&self) -> &str {
        &self.heading_id
    }

    pub fn accessible_heading_id(&self) -> &str {
        &self.accessible_heading_id
    }

    pub fn live_region_id(&self) -> &str {
        &self.live_region_id
    }

    pub fn prev_button_id(&self) -> &str {
        &self.prev_button_id
    }

    pub fn next_button_id(&self) -> &str {
        &self.next_button_id
    }

    pub fn month_select_id(&self) -> &str {
        &self.month_select_id
    }

    pub fn year_select_id(&self) -> &str {
        &self.year_select_id
    }

    pub fn grid_id(&self, month_idx: usize) -> String {
        self.scope.qualify(format!("grid-{month_idx}"))
    }

    pub fn grid_head_id(&self, month_idx: usize) -> String {
        self.scope.qualify(format!("grid-head-{month_idx}"))
    }

    pub fn grid_body_id(&self, month_idx: usize) -> String {
        self.scope.qualify(format!("grid-body-{month_idx}"))
    }

    pub fn grid_row_id(&self, month_idx: usize, row_idx: usize) -> String {
        self.scope.qualify(format!("grid-row-{month_idx}-{row_idx}"))
    }

    pub fn head_cell_id(&self, month_idx: usize, col_idx: usize) -> String {
        self.scope.qualify(format!("head-cell-{month_idx}-{col_idx}"))
    }

    pub fn cell_id(&self, month_idx: usize, date: CalendarDate) -> String {
        self.scope.qualify(format!("cell-{month_idx}-{date}"))
    }

    pub fn day_id(&self, month_idx: usize, date: CalendarDate) -> String {
        self.scope.qualify(format!("day-{month_idx}-{date}"))
    }
}
