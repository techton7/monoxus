use crate::foundation::state::DataState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RadioGroupRootAttributes {
    pub id: String,
    pub role: &'static str,
    pub dir: &'static str,
    pub aria_orientation: Option<&'static str>,
    pub aria_required: Option<&'static str>,
    pub aria_disabled: Option<&'static str>,
    pub aria_readonly: Option<&'static str>,
    pub data_orientation: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
    pub data_required: Option<&'static str>,
    pub data_readonly: Option<&'static str>,
    pub data_radio_group_root: &'static str,
}

impl RadioGroupRootAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn role(&self) -> &'static str {
        self.role
    }

    pub fn dir(&self) -> &'static str {
        self.dir
    }

    pub fn aria_orientation(&self) -> Option<&'static str> {
        self.aria_orientation
    }

    pub fn aria_required(&self) -> Option<&'static str> {
        self.aria_required
    }

    pub fn aria_disabled(&self) -> Option<&'static str> {
        self.aria_disabled
    }

    pub fn aria_readonly(&self) -> Option<&'static str> {
        self.aria_readonly
    }

    pub fn data_orientation(&self) -> Option<&'static str> {
        self.data_orientation
    }

    pub fn data_disabled(&self) -> Option<&'static str> {
        self.data_disabled
    }

    pub fn data_required(&self) -> Option<&'static str> {
        self.data_required
    }

    pub fn data_readonly(&self) -> Option<&'static str> {
        self.data_readonly
    }

    pub fn data_radio_group_root(&self) -> &'static str {
        self.data_radio_group_root
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RadioGroupItemAttributes {
    pub id: String,
    pub role: &'static str,
    pub aria_checked: &'static str,
    pub tabindex: i32,
    pub disabled: bool,
    pub data_state: DataState,
    pub data_disabled: Option<&'static str>,
    pub data_readonly: Option<&'static str>,
    pub data_orientation: Option<&'static str>,
    pub data_value: String,
    pub data_radio_group_item: &'static str,
}

impl RadioGroupItemAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn role(&self) -> &'static str {
        self.role
    }

    pub fn aria_checked(&self) -> &'static str {
        self.aria_checked
    }

    pub fn tabindex(&self) -> i32 {
        self.tabindex
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn data_state(&self) -> DataState {
        self.data_state.clone()
    }

    pub fn data_state_str(&self) -> &str {
        self.data_state.as_str()
    }

    pub fn data_disabled(&self) -> Option<&'static str> {
        self.data_disabled
    }

    pub fn data_readonly(&self) -> Option<&'static str> {
        self.data_readonly
    }

    pub fn data_orientation(&self) -> Option<&'static str> {
        self.data_orientation
    }

    pub fn data_value(&self) -> &str {
        &self.data_value
    }

    pub fn data_radio_group_item(&self) -> &'static str {
        self.data_radio_group_item
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RadioGroupIndicatorAttributes {
    pub id: String,
    pub data_state: DataState,
    pub data_disabled: Option<&'static str>,
    pub data_readonly: Option<&'static str>,
    pub data_orientation: Option<&'static str>,
    pub data_radio_group_indicator: &'static str,
    pub style: &'static str,
}

impl RadioGroupIndicatorAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_state(&self) -> DataState {
        self.data_state.clone()
    }

    pub fn data_state_str(&self) -> &str {
        self.data_state.as_str()
    }

    pub fn data_disabled(&self) -> Option<&'static str> {
        self.data_disabled
    }

    pub fn data_readonly(&self) -> Option<&'static str> {
        self.data_readonly
    }

    pub fn data_orientation(&self) -> Option<&'static str> {
        self.data_orientation
    }

    pub fn data_radio_group_indicator(&self) -> &'static str {
        self.data_radio_group_indicator
    }

    pub fn style(&self) -> &'static str {
        self.style
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foundation::state::DataState;

    #[test]
    fn test_radio_group_root_attributes_defaults() {
        let attrs = RadioGroupRootAttributes {
            id: "rg-root".to_string(),
            role: "radiogroup",
            dir: "ltr",
            aria_orientation: None,
            aria_required: None,
            aria_disabled: None,
            aria_readonly: None,
            data_orientation: None,
            data_disabled: None,
            data_required: None,
            data_readonly: None,
            data_radio_group_root: "",
        };

        assert_eq!(attrs.id(), "rg-root");
        assert_eq!(attrs.role(), "radiogroup");
        assert_eq!(attrs.dir(), "ltr");
        assert_eq!(attrs.aria_orientation(), None);
        assert_eq!(attrs.data_orientation(), None);
        assert_eq!(attrs.aria_disabled(), None);
        assert_eq!(attrs.data_disabled(), None);
        assert_eq!(attrs.aria_readonly(), None);
        assert_eq!(attrs.data_readonly(), None);
        assert_eq!(attrs.data_radio_group_root(), "");
    }

    #[test]
    fn test_radio_group_item_attributes_checked_and_readonly() {
        let attrs = RadioGroupItemAttributes {
            id: "rg-item-1".to_string(),
            role: "radio",
            aria_checked: "true",
            tabindex: 0,
            disabled: false,
            data_state: DataState::Checked,
            data_disabled: None,
            data_readonly: Some(""),
            data_orientation: Some("vertical"),
            data_value: "opt1".to_string(),
            data_radio_group_item: "",
        };

        assert_eq!(attrs.role(), "radio");
        assert_eq!(attrs.aria_checked(), "true");
        assert_eq!(attrs.tabindex(), 0);
        assert!(!attrs.is_disabled());
        assert_eq!(attrs.data_state_str(), "checked");
        assert_eq!(attrs.data_readonly(), Some(""));
        assert_eq!(attrs.data_orientation(), Some("vertical"));
        assert_eq!(attrs.data_value(), "opt1");
    }

    #[test]
    fn test_radio_group_indicator_attributes() {
        let attrs = RadioGroupIndicatorAttributes {
            id: "rg-ind-1".to_string(),
            data_state: DataState::Checked,
            data_disabled: None,
            data_readonly: None,
            data_orientation: None,
            data_radio_group_indicator: "",
            style: "pointer-events: none;",
        };

        assert_eq!(attrs.data_state_str(), "checked");
        assert_eq!(attrs.data_radio_group_indicator(), "");
        assert_eq!(attrs.style(), "pointer-events: none;");
    }
}
