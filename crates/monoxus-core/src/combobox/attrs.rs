use crate::foundation::{
    overlay::{PlacementAlign, PlacementSide},
    shared::data_disabled_attr,
    state::DataState,
};

use super::types::ComboboxMode;

#[inline]
const fn bool_to_empty_attr(flag: bool) -> Option<&'static str> {
    if flag { Some("") } else { None }
}

#[inline]
const fn data_state_str(state: &DataState) -> &'static str {
    match state {
        DataState::Open => "open",
        _ => "closed",
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComboboxInputAttrs {
    pub id: String,
    pub role: &'static str,
    pub aria_autocomplete: &'static str,
    pub aria_expanded: &'static str,
    pub aria_controls: Option<String>,
    pub aria_activedescendant: Option<String>,
    pub disabled: bool,
    pub data_state: DataState,
    pub autocomplete: &'static str,
}

impl ComboboxInputAttrs {
    pub fn new(
        id: impl Into<String>,
        content_id: Option<String>,
        is_open: bool,
        highlighted_item_id: Option<String>,
        disabled: bool,
    ) -> Self {
        Self {
            id: id.into(),
            role: "combobox",
            aria_autocomplete: "list",
            aria_expanded: if is_open { "true" } else { "false" },
            aria_controls: content_id,
            aria_activedescendant: if is_open { highlighted_item_id } else { None },
            disabled,
            data_state: if is_open {
                DataState::Open
            } else {
                DataState::Closed
            },
            autocomplete: "off",
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn role(&self) -> &'static str {
        self.role
    }

    pub const fn aria_autocomplete(&self) -> &'static str {
        self.aria_autocomplete
    }

    pub const fn aria_expanded(&self) -> &'static str {
        self.aria_expanded
    }

    pub fn aria_controls(&self) -> Option<&str> {
        self.aria_controls.as_deref()
    }

    pub fn aria_activedescendant(&self) -> Option<&str> {
        self.aria_activedescendant.as_deref()
    }

    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub const fn data_state_str(&self) -> &'static str {
        data_state_str(&self.data_state)
    }

    pub const fn data_disabled(&self) -> Option<&'static str> {
        data_disabled_attr(self.disabled)
    }

    pub const fn autocomplete(&self) -> &'static str {
        self.autocomplete
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComboboxTriggerAttrs {
    pub id: String,
    pub aria_haspopup: &'static str,
    pub aria_expanded: &'static str,
    pub aria_controls: Option<String>,
    pub disabled: bool,
    pub data_state: DataState,
}

impl ComboboxTriggerAttrs {
    pub fn new(
        id: impl Into<String>,
        content_id: Option<String>,
        is_open: bool,
        disabled: bool,
    ) -> Self {
        Self {
            id: id.into(),
            aria_haspopup: "listbox",
            aria_expanded: if is_open { "true" } else { "false" },
            aria_controls: content_id,
            disabled,
            data_state: if is_open {
                DataState::Open
            } else {
                DataState::Closed
            },
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn aria_haspopup(&self) -> &'static str {
        self.aria_haspopup
    }

    pub const fn aria_expanded(&self) -> &'static str {
        self.aria_expanded
    }

    pub fn aria_controls(&self) -> Option<&str> {
        self.aria_controls.as_deref()
    }

    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub const fn data_state_str(&self) -> &'static str {
        data_state_str(&self.data_state)
    }

    pub const fn data_disabled(&self) -> Option<&'static str> {
        data_disabled_attr(self.disabled)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComboboxContentAttrs {
    pub id: String,
    pub role: &'static str,
    pub tabindex: i32,
    pub aria_multiselectable: Option<&'static str>,
    pub data_state: DataState,
    pub data_side: PlacementSide,
    pub data_align: PlacementAlign,
}

impl ComboboxContentAttrs {
    pub fn new(
        id: impl Into<String>,
        mode: ComboboxMode,
        is_open: bool,
        side: PlacementSide,
        align: PlacementAlign,
    ) -> Self {
        Self {
            id: id.into(),
            role: "listbox",
            tabindex: -1,
            aria_multiselectable: if mode.is_multiple() {
                Some("true")
            } else {
                None
            },
            data_state: if is_open {
                DataState::Open
            } else {
                DataState::Closed
            },
            data_side: side,
            data_align: align,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn role(&self) -> &'static str {
        self.role
    }

    pub const fn tabindex(&self) -> i32 {
        self.tabindex
    }

    pub const fn aria_multiselectable(&self) -> Option<&'static str> {
        self.aria_multiselectable
    }

    pub const fn data_state_str(&self) -> &'static str {
        data_state_str(&self.data_state)
    }

    pub const fn data_side(&self) -> PlacementSide {
        self.data_side
    }

    pub const fn data_side_str(&self) -> &'static str {
        self.data_side.as_str()
    }

    pub const fn data_align(&self) -> PlacementAlign {
        self.data_align
    }

    pub const fn data_align_str(&self) -> &'static str {
        match self.data_align {
            PlacementAlign::Start => "start",
            PlacementAlign::Center => "center",
            PlacementAlign::End => "end",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComboboxViewportAttrs {
    pub id: String,
    pub role: &'static str,
}

impl ComboboxViewportAttrs {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            role: "presentation",
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn role(&self) -> &'static str {
        self.role
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComboboxItemAttrs {
    pub id: String,
    pub role: &'static str,
    pub aria_selected: &'static str,
    pub aria_disabled: Option<&'static str>,
    pub value: String,
    pub label: String,
    pub selected: bool,
    pub highlighted: bool,
    pub disabled: bool,
}

impl ComboboxItemAttrs {
    pub fn new(
        id: impl Into<String>,
        value: impl Into<String>,
        label: impl Into<String>,
        selected: bool,
        highlighted: bool,
        disabled: bool,
    ) -> Self {
        let effective_highlighted = highlighted && !disabled;
        Self {
            id: id.into(),
            role: "option",
            aria_selected: if selected { "true" } else { "false" },
            aria_disabled: if disabled { Some("true") } else { None },
            value: value.into(),
            label: label.into(),
            selected,
            highlighted: effective_highlighted,
            disabled,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn role(&self) -> &'static str {
        self.role
    }

    pub const fn aria_selected(&self) -> &'static str {
        self.aria_selected
    }

    pub const fn aria_disabled(&self) -> Option<&'static str> {
        self.aria_disabled
    }

    pub fn data_value(&self) -> &str {
        &self.value
    }

    pub fn data_label(&self) -> &str {
        &self.label
    }

    pub const fn is_selected(&self) -> bool {
        self.selected
    }

    pub const fn is_highlighted(&self) -> bool {
        self.highlighted
    }

    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub const fn data_disabled(&self) -> Option<&'static str> {
        data_disabled_attr(self.disabled)
    }

    pub const fn data_highlighted(&self) -> Option<&'static str> {
        bool_to_empty_attr(self.highlighted)
    }

    pub const fn data_selected(&self) -> Option<&'static str> {
        bool_to_empty_attr(self.selected)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComboboxGroupAttrs {
    pub id: String,
    pub role: &'static str,
    pub aria_labelledby: Option<String>,
}

impl ComboboxGroupAttrs {
    pub fn new(id: impl Into<String>, heading_id: Option<String>) -> Self {
        Self {
            id: id.into(),
            role: "group",
            aria_labelledby: heading_id,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn role(&self) -> &'static str {
        self.role
    }

    pub fn aria_labelledby(&self) -> Option<&str> {
        self.aria_labelledby.as_deref()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComboboxGroupHeadingAttrs {
    pub id: String,
}

impl ComboboxGroupHeadingAttrs {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComboboxScrollButtonAttrs {
    pub aria_hidden: &'static str,
}

impl Default for ComboboxScrollButtonAttrs {
    fn default() -> Self {
        Self::new()
    }
}

impl ComboboxScrollButtonAttrs {
    pub const fn new() -> Self {
        Self {
            aria_hidden: "true",
        }
    }

    pub const fn aria_hidden(&self) -> &'static str {
        self.aria_hidden
    }
}
