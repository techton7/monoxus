use crate::foundation::{browser::ElementSize, shared::data_disabled_attr, state::DataState};

use super::types::CollapsibleStateRequest;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollapsibleRootAttributes {
    pub(crate) id: String,
    pub(crate) data_state: DataState,
    pub(crate) disabled: bool,
}

impl CollapsibleRootAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_state(&self) -> DataState {
        self.data_state.clone()
    }

    pub fn data_state_str(&self) -> &'static str {
        match self.data_state {
            DataState::Open => "open",
            _ => "closed",
        }
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn data_disabled(&self) -> Option<&'static str> {
        data_disabled_attr(self.disabled)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollapsibleTriggerAttributes {
    pub(crate) id: String,
    pub(crate) content_id: String,
    pub(crate) aria_controls: Option<String>,
    pub(crate) aria_expanded: &'static str,
    pub(crate) data_state: DataState,
    pub(crate) disabled: bool,
    pub(crate) state_request: CollapsibleStateRequest,
}

impl CollapsibleTriggerAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn target_content_id(&self) -> &str {
        &self.content_id
    }

    pub fn aria_controls(&self) -> Option<&str> {
        self.aria_controls.as_deref()
    }

    pub fn aria_expanded(&self) -> &'static str {
        self.aria_expanded
    }

    pub fn data_state(&self) -> DataState {
        self.data_state.clone()
    }

    pub fn data_state_str(&self) -> &'static str {
        match self.data_state {
            DataState::Open => "open",
            _ => "closed",
        }
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn data_disabled(&self) -> Option<&'static str> {
        data_disabled_attr(self.disabled)
    }

    pub const fn state_request(&self) -> CollapsibleStateRequest {
        self.state_request
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollapsibleContentAttributes {
    pub(crate) id: String,
    pub(crate) hidden: bool,
    pub(crate) data_state: DataState,
    pub(crate) disabled: bool,
}

impl CollapsibleContentAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn is_hidden(&self) -> bool {
        self.hidden
    }

    pub fn data_state(&self) -> DataState {
        self.data_state.clone()
    }

    pub fn data_state_str(&self) -> &'static str {
        match self.data_state {
            DataState::Open => "open",
            _ => "closed",
        }
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn data_disabled(&self) -> Option<&'static str> {
        data_disabled_attr(self.disabled)
    }
}

/// Formats a disclosure dimension value cleanly in pixels (e.g. "120px" or "120.5px").
pub fn format_disclosure_dimension(val: f64) -> String {
    if val.is_nan() || val.is_infinite() || val <= 0.0 {
        return "0px".to_string();
    }
    if val.fract().abs() < 1e-4 {
        format!("{:.0}px", val)
    } else {
        let rounded = (val * 100.0).round() / 100.0;
        let s = format!("{:.2}", rounded);
        let trimmed = s.trim_end_matches('0').trim_end_matches('.');
        format!("{trimmed}px")
    }
}

/// Formats the four standard collapsible content dimension CSS custom properties.
pub fn format_collapsible_css_vars(size: ElementSize) -> String {
    let h = format_disclosure_dimension(size.height);
    let w = format_disclosure_dimension(size.width);
    format!(
        "--radix-collapsible-content-height: {h}; --radix-collapsible-content-width: {w}; --monoxus-collapsible-content-height: {h}; --monoxus-collapsible-content-width: {w}"
    )
}

/// Safely merges user style, mount animation guard style, and dimension CSS custom properties.
pub fn merge_disclosure_styles(
    user_style: Option<&str>,
    guard_style: Option<&str>,
    css_vars: Option<&str>,
) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(s) = user_style {
        let s = s.trim();
        if !s.is_empty() {
            parts.push(s.trim_end_matches(';'));
        }
    }
    if let Some(s) = guard_style {
        let s = s.trim();
        if !s.is_empty() {
            parts.push(s.trim_end_matches(';'));
        }
    }
    if let Some(s) = css_vars {
        let s = s.trim();
        if !s.is_empty() {
            parts.push(s.trim_end_matches(';'));
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("; "))
    }
}
