use crate::foundation::{
    shared::{ElementSize, data_disabled_attr},
    state::DataState,
};

pub use crate::collapsible::attrs::{format_disclosure_dimension, merge_disclosure_styles};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccordionRootAttributes {
    pub id: String,
    pub data_orientation: &'static str,
    pub disabled: bool,
}

impl AccordionRootAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn data_disabled(&self) -> Option<&'static str> {
        data_disabled_attr(self.disabled)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccordionItemAttributes {
    pub id: String,
    pub data_state: DataState,
    pub data_orientation: &'static str,
    pub data_value: String,
    pub disabled: bool,
}

impl AccordionItemAttributes {
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

    pub fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub fn data_value(&self) -> &str {
        &self.data_value
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn data_disabled(&self) -> Option<&'static str> {
        data_disabled_attr(self.disabled)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccordionHeaderAttributes {
    pub id: String,
    pub role: &'static str,
    pub aria_level: u32,
    pub data_heading_level: u32,
    pub data_state: DataState,
    pub data_orientation: &'static str,
    pub data_value: String,
    pub disabled: bool,
}

impl AccordionHeaderAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn role(&self) -> &'static str {
        self.role
    }

    pub fn aria_level(&self) -> u32 {
        self.aria_level
    }

    pub fn data_heading_level(&self) -> u32 {
        self.data_heading_level
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

    pub fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub fn data_value(&self) -> &str {
        &self.data_value
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn data_disabled(&self) -> Option<&'static str> {
        data_disabled_attr(self.disabled)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccordionTriggerAttributes {
    pub id: String,
    pub role: &'static str,
    pub is_open: bool,
    pub aria_expanded: &'static str,
    pub content_id: String,
    pub aria_controls: Option<String>,
    pub aria_disabled: Option<&'static str>,
    pub tabindex: i32,
    pub data_state: DataState,
    pub data_orientation: &'static str,
    pub data_value: String,
    pub disabled: bool,
}

impl AccordionTriggerAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn role(&self) -> &'static str {
        self.role
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn aria_expanded(&self) -> &'static str {
        self.aria_expanded
    }

    pub fn target_content_id(&self) -> &str {
        &self.content_id
    }

    pub fn aria_controls(&self) -> Option<&str> {
        self.aria_controls.as_deref()
    }

    pub fn aria_disabled(&self) -> Option<&'static str> {
        self.aria_disabled
    }

    pub fn tabindex(&self) -> i32 {
        self.tabindex
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

    pub fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub fn data_value(&self) -> &str {
        &self.data_value
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn data_disabled(&self) -> Option<&'static str> {
        data_disabled_attr(self.disabled)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccordionContentAttributes {
    pub id: String,
    pub role: &'static str,
    pub aria_labelledby: String,
    pub hidden: bool,
    pub data_state: DataState,
    pub data_orientation: &'static str,
    pub data_value: String,
    pub disabled: bool,
}

impl AccordionContentAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn role(&self) -> &'static str {
        self.role
    }

    pub fn aria_labelledby(&self) -> &str {
        &self.aria_labelledby
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

    pub fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub fn data_value(&self) -> &str {
        &self.data_value
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn data_disabled(&self) -> Option<&'static str> {
        data_disabled_attr(self.disabled)
    }
}

/// Formats the four standard accordion content dimension CSS custom properties.
pub fn format_accordion_css_vars(size: ElementSize) -> String {
    let h = format_disclosure_dimension(size.height);
    let w = format_disclosure_dimension(size.width);
    format!(
        "--radix-accordion-content-height: {h}; --radix-accordion-content-width: {w}; --monoxus-accordion-content-height: {h}; --monoxus-accordion-content-width: {w}"
    )
}
