use crate::foundation::state::DataState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TabsRootAttributes {
    pub(crate) id: String,
    pub(crate) data_orientation: &'static str,
}

impl TabsRootAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TabsListAttributes {
    pub(crate) id: String,
    pub(crate) role: &'static str,
    pub(crate) aria_orientation: &'static str,
    pub(crate) data_orientation: &'static str,
    pub(crate) disabled: bool,
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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TabsTriggerAttributes {
    pub(crate) id: String,
    pub(crate) role: &'static str,
    pub(crate) is_selected: bool,
    pub(crate) aria_selected: &'static str,
    pub(crate) aria_controls: String,
    pub(crate) tabindex: i32,
    pub(crate) data_state: DataState,
    pub(crate) data_orientation: &'static str,
    pub(crate) data_value: String,
    pub(crate) disabled: bool,
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

    pub fn aria_controls(&self) -> &str {
        &self.aria_controls
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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TabsContentAttributes {
    pub(crate) id: String,
    pub(crate) role: &'static str,
    pub(crate) aria_labelledby: String,
    pub(crate) tabindex: i32,
    pub(crate) hidden: bool,
    pub(crate) data_state: DataState,
    pub(crate) data_orientation: &'static str,
    pub(crate) data_value: String,
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
}
