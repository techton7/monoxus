use crate::foundation::state::DataState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToggleGroupRootAttributes {
    pub id: String,
    pub role: &'static str,
    pub dir: &'static str,
    pub data_orientation: &'static str,
    pub data_disabled: Option<&'static str>,
    pub data_toggle_group_root: &'static str,
}

impl ToggleGroupRootAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn role(&self) -> &'static str {
        self.role
    }

    pub fn dir(&self) -> &'static str {
        self.dir
    }

    pub fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub fn data_disabled(&self) -> Option<&'static str> {
        self.data_disabled
    }

    pub fn data_toggle_group_root(&self) -> &'static str {
        self.data_toggle_group_root
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToggleGroupItemAttributes {
    pub id: String,
    pub role: Option<&'static str>,
    pub aria_checked: Option<&'static str>,
    pub aria_pressed: Option<&'static str>,
    pub tabindex: i32,
    pub disabled: bool,
    pub data_disabled: Option<&'static str>,
    pub data_state: DataState,
    pub data_value: String,
    pub data_orientation: &'static str,
    pub data_toggle_group_item: &'static str,
}

impl ToggleGroupItemAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn role(&self) -> Option<&'static str> {
        self.role
    }

    pub fn aria_checked(&self) -> Option<&'static str> {
        self.aria_checked
    }

    pub fn aria_pressed(&self) -> Option<&'static str> {
        self.aria_pressed
    }

    pub fn tabindex(&self) -> i32 {
        self.tabindex
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn data_disabled(&self) -> Option<&'static str> {
        self.data_disabled
    }

    pub fn data_state(&self) -> DataState {
        self.data_state.clone()
    }

    pub fn data_state_str(&self) -> &str {
        self.data_state.as_str()
    }

    pub fn data_value(&self) -> &str {
        &self.data_value
    }

    pub fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub fn data_toggle_group_item(&self) -> &'static str {
        self.data_toggle_group_item
    }
}
