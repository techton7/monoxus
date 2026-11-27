use crate::foundation::{
    overlay::{PlacementAlign, PlacementSide, PortalHost},
    state::DataState,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectRootAttributes {
    pub id: String,
    pub data_state: DataState,
    pub disabled: bool,
    pub invalid: bool,
}

impl SelectRootAttributes {
    pub fn with_invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

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
        crate::foundation::shared::data_disabled_attr(self.disabled)
    }

    pub fn is_invalid(&self) -> bool {
        self.invalid
    }

    pub fn data_invalid(&self) -> Option<&'static str> {
        crate::foundation::shared::data_invalid_attr(self.invalid)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectTriggerAttributes {
    pub id: String,
    pub role: &'static str,
    pub aria_haspopup: &'static str,
    pub aria_expanded: &'static str,
    pub aria_controls: Option<String>,
    pub content_id: String,
    pub aria_disabled: Option<&'static str>,
    pub aria_required: Option<&'static str>,
    pub aria_invalid: Option<&'static str>,
    pub aria_describedby: Option<String>,
    pub aria_labelledby: Option<String>,
    pub data_state: DataState,
    pub data_placeholder: bool,
    pub disabled: bool,
    pub invalid: bool,
    pub data_invalid: Option<&'static str>,
    pub tabindex: i32,
}

impl SelectTriggerAttributes {
    pub fn with_invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self.aria_invalid = invalid.then_some("true");
        self.data_invalid = crate::foundation::shared::data_invalid_attr(invalid);
        self
    }

    pub fn with_aria_describedby(mut self, aria_describedby: Option<String>) -> Self {
        self.aria_describedby = aria_describedby;
        self
    }

    pub fn with_aria_labelledby(mut self, aria_labelledby: Option<String>) -> Self {
        self.aria_labelledby = aria_labelledby;
        self
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn role(&self) -> &'static str {
        self.role
    }

    pub fn aria_haspopup(&self) -> &'static str {
        self.aria_haspopup
    }

    pub fn aria_expanded(&self) -> &'static str {
        self.aria_expanded
    }

    pub fn aria_controls(&self) -> Option<&str> {
        self.aria_controls.as_deref()
    }

    pub fn target_content_id(&self) -> &str {
        &self.content_id
    }

    pub fn aria_disabled(&self) -> Option<&'static str> {
        self.aria_disabled
    }

    pub fn aria_required(&self) -> Option<&'static str> {
        self.aria_required
    }

    pub fn aria_invalid(&self) -> Option<&'static str> {
        self.aria_invalid
    }

    pub fn aria_describedby(&self) -> Option<&str> {
        self.aria_describedby.as_deref()
    }

    pub fn aria_labelledby(&self) -> Option<&str> {
        self.aria_labelledby.as_deref()
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
        crate::foundation::shared::data_disabled_attr(self.disabled)
    }

    pub fn is_invalid(&self) -> bool {
        self.invalid
    }

    pub fn data_invalid(&self) -> Option<&'static str> {
        self.data_invalid
    }

    pub fn is_placeholder(&self) -> bool {
        self.data_placeholder
    }

    pub fn data_placeholder_str(&self) -> &'static str {
        if self.data_placeholder {
            "true"
        } else {
            "false"
        }
    }

    pub fn tabindex(&self) -> i32 {
        self.tabindex
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectPortalAttributes {
    pub host: PortalHost,
}

impl SelectPortalAttributes {
    pub fn host(&self) -> &PortalHost {
        &self.host
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectContentAttributes {
    pub id: String,
    pub role: &'static str,
    pub tabindex: i32,
    pub aria_activedescendant: Option<String>,
    pub data_state: DataState,
    pub data_side: PlacementSide,
    pub data_align: PlacementAlign,
    pub reference_hidden: bool,
}

impl SelectContentAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn role(&self) -> &'static str {
        self.role
    }

    pub fn tabindex(&self) -> i32 {
        self.tabindex
    }

    pub fn aria_activedescendant(&self) -> Option<&str> {
        self.aria_activedescendant.as_deref()
    }

    pub fn data_state_str(&self) -> &'static str {
        match self.data_state {
            DataState::Open => "open",
            _ => "closed",
        }
    }

    pub fn data_side(&self) -> PlacementSide {
        self.data_side
    }

    pub fn data_side_str(&self) -> &'static str {
        self.data_side.as_str()
    }

    pub fn data_align(&self) -> PlacementAlign {
        self.data_align
    }

    pub fn data_align_str(&self) -> &'static str {
        match self.data_align {
            PlacementAlign::Start => "start",
            PlacementAlign::Center => "center",
            PlacementAlign::End => "end",
        }
    }

    pub fn is_reference_hidden(&self) -> bool {
        self.reference_hidden
    }

    pub fn data_reference_hidden_str(&self) -> Option<&'static str> {
        if self.reference_hidden {
            Some("true")
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectItemAttributes {
    pub id: String,
    pub role: &'static str,
    pub aria_selected: &'static str,
    pub aria_disabled: Option<&'static str>,
    pub data_state: &'static str,
    pub is_highlighted: bool,
    pub disabled: bool,
    pub value: String,
    pub label: String,
}

impl SelectItemAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn role(&self) -> &'static str {
        self.role
    }

    pub fn aria_selected(&self) -> &'static str {
        self.aria_selected
    }

    pub fn aria_disabled(&self) -> Option<&'static str> {
        self.aria_disabled
    }

    pub fn data_state(&self) -> &'static str {
        self.data_state
    }

    pub fn is_highlighted(&self) -> bool {
        self.is_highlighted
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn data_disabled(&self) -> Option<&'static str> {
        crate::foundation::shared::data_disabled_attr(self.disabled)
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn data_value(&self) -> &str {
        &self.value
    }

    pub fn data_label(&self) -> &str {
        &self.label
    }

    pub fn data_selected(&self) -> &'static str {
        self.aria_selected
    }
}
