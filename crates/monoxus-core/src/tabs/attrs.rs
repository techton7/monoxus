use crate::foundation::{shared::data_disabled_attr, state::DataState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TabsRootAttributes {
    pub id: String,
    pub data_orientation: &'static str,
    pub disabled: bool,
}

impl TabsRootAttributes {
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
pub struct TabsListAttributes {
    pub id: String,
    pub role: &'static str,
    pub aria_orientation: &'static str,
    pub data_orientation: &'static str,
    pub disabled: bool,
}

impl TabsListAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn role(&self) -> &'static str {
        self.role
    }

    pub fn aria_orientation(&self) -> &'static str {
        self.aria_orientation
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
pub struct TabsTriggerAttributes {
    pub id: String,
    pub role: &'static str,
    pub is_selected: bool,
    pub aria_selected: &'static str,
    pub content_id: String,
    pub aria_controls: Option<String>,
    pub tabindex: i32,
    pub data_state: DataState,
    pub data_orientation: &'static str,
    pub data_value: String,
    pub disabled: bool,
}

impl TabsTriggerAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn role(&self) -> &'static str {
        self.role
    }

    pub fn is_selected(&self) -> bool {
        self.is_selected
    }

    pub fn aria_selected(&self) -> &'static str {
        self.aria_selected
    }

    pub fn target_content_id(&self) -> &str {
        &self.content_id
    }

    pub fn aria_controls(&self) -> Option<&str> {
        self.aria_controls.as_deref()
    }

    pub fn tabindex(&self) -> i32 {
        self.tabindex
    }

    pub fn data_state(&self) -> DataState {
        self.data_state.clone()
    }

    pub fn data_state_str(&self) -> &'static str {
        match self.data_state {
            DataState::Active => "active",
            _ => "inactive",
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
pub struct TabsContentAttributes {
    pub id: String,
    pub role: &'static str,
    pub aria_labelledby: String,
    pub tabindex: i32,
    pub hidden: bool,
    pub data_state: DataState,
    pub data_orientation: &'static str,
    pub data_value: String,
    pub disabled: bool,
}

impl TabsContentAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn role(&self) -> &'static str {
        self.role
    }

    pub fn aria_labelledby(&self) -> &str {
        &self.aria_labelledby
    }

    pub fn tabindex(&self) -> i32 {
        self.tabindex
    }

    pub fn is_hidden(&self) -> bool {
        self.hidden
    }

    pub fn data_state(&self) -> DataState {
        self.data_state.clone()
    }

    pub fn data_state_str(&self) -> &'static str {
        match self.data_state {
            DataState::Active => "active",
            _ => "inactive",
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
