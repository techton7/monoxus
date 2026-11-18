use crate::foundation::{shared::data_disabled_attr, state::DataState};

#[inline]
pub const fn data_required_attr(required: bool) -> Option<&'static str> {
    if required {
        Some("")
    } else {
        None
    }
}

#[inline]
pub const fn aria_required_attr(required: bool) -> Option<&'static str> {
    if required {
        Some("true")
    } else {
        None
    }
}

#[inline]
pub const fn data_readonly_attr(read_only: bool, disabled: bool) -> Option<&'static str> {
    if read_only && !disabled {
        Some("")
    } else {
        None
    }
}

#[inline]
pub const fn aria_readonly_attr(read_only: bool, disabled: bool) -> Option<&'static str> {
    if read_only && !disabled {
        Some("true")
    } else {
        None
    }
}

/// Pure attribute publisher for [`SwitchRoot`](super::components::SwitchRoot).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SwitchRootAttributes {
    pub id: Option<String>,
    pub r#type: &'static str,
    pub role: &'static str,
    pub aria_checked: &'static str,
    pub aria_required: Option<&'static str>,
    pub aria_readonly: Option<&'static str>,
    pub data_state: DataState,
    pub data_disabled: Option<&'static str>,
    pub data_required: Option<&'static str>,
    pub data_readonly: Option<&'static str>,
    pub data_switch_root: &'static str,
    pub disabled: bool,
}

impl SwitchRootAttributes {
    pub fn new(
        id: Option<String>,
        button_type: &'static str,
        checked: bool,
        disabled: bool,
        required: bool,
        read_only: bool,
    ) -> Self {
        Self {
            id,
            r#type: button_type,
            role: "switch",
            aria_checked: if checked { "true" } else { "false" },
            aria_required: aria_required_attr(required),
            aria_readonly: aria_readonly_attr(read_only, disabled),
            data_state: if checked {
                DataState::Checked
            } else {
                DataState::Unchecked
            },
            data_disabled: data_disabled_attr(disabled),
            data_required: data_required_attr(required),
            data_readonly: data_readonly_attr(read_only, disabled),
            data_switch_root: "",
            disabled,
        }
    }

    pub fn data_state_str(&self) -> &'static str {
        match self.data_state {
            DataState::Checked => "checked",
            _ => "unchecked",
        }
    }
}

/// Pure attribute publisher for [`SwitchThumb`](super::components::SwitchThumb).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SwitchThumbAttributes {
    pub id: Option<String>,
    pub data_state: DataState,
    pub data_disabled: Option<&'static str>,
    pub data_required: Option<&'static str>,
    pub data_readonly: Option<&'static str>,
    pub data_switch_thumb: &'static str,
}

impl SwitchThumbAttributes {
    pub fn new(
        id: Option<String>,
        checked: bool,
        disabled: bool,
        required: bool,
        read_only: bool,
    ) -> Self {
        Self {
            id,
            data_state: if checked {
                DataState::Checked
            } else {
                DataState::Unchecked
            },
            data_disabled: data_disabled_attr(disabled),
            data_required: data_required_attr(required),
            data_readonly: data_readonly_attr(read_only, disabled),
            data_switch_thumb: "",
        }
    }

    pub fn data_state_str(&self) -> &'static str {
        match self.data_state {
            DataState::Checked => "checked",
            _ => "unchecked",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_switch_root_and_thumb_attributes_truth_table() {
        let root_attrs = SwitchRootAttributes::new(
            Some("sw-1".to_string()),
            "button",
            false,
            false,
            false,
            false,
        );
        assert_eq!(root_attrs.role, "switch");
        assert_eq!(root_attrs.aria_checked, "false");
        assert_eq!(root_attrs.data_state, DataState::Unchecked);
        assert_eq!(root_attrs.data_state_str(), "unchecked");
        assert_eq!(root_attrs.aria_required, None);
        assert_eq!(root_attrs.aria_readonly, None);
        assert_eq!(root_attrs.data_disabled, None);
        assert_eq!(root_attrs.data_required, None);
        assert_eq!(root_attrs.data_readonly, None);
        assert_eq!(root_attrs.data_switch_root, "");
        assert!(!root_attrs.disabled);

        let thumb_attrs = SwitchThumbAttributes::new(
            Some("thumb-1".to_string()),
            false,
            false,
            false,
            false,
        );
        assert_eq!(thumb_attrs.data_state, DataState::Unchecked);
        assert_eq!(thumb_attrs.data_disabled, None);
        assert_eq!(thumb_attrs.data_required, None);
        assert_eq!(thumb_attrs.data_readonly, None);
        assert_eq!(thumb_attrs.data_switch_thumb, "");

        // Checked, required, read-only
        let checked_root = SwitchRootAttributes::new(
            Some("sw-2".to_string()),
            "button",
            true,
            false,
            true,
            true,
        );
        assert_eq!(checked_root.aria_checked, "true");
        assert_eq!(checked_root.aria_required, Some("true"));
        assert_eq!(checked_root.aria_readonly, Some("true"));
        assert_eq!(checked_root.data_state, DataState::Checked);
        assert_eq!(checked_root.data_required, Some(""));
        assert_eq!(checked_root.data_readonly, Some(""));
        assert_eq!(checked_root.data_disabled, None);

        let checked_thumb = SwitchThumbAttributes::new(
            Some("thumb-2".to_string()),
            true,
            false,
            true,
            true,
        );
        assert_eq!(checked_thumb.data_state, DataState::Checked);
        assert_eq!(checked_thumb.data_required, Some(""));
        assert_eq!(checked_thumb.data_readonly, Some(""));
        assert_eq!(checked_thumb.data_disabled, None);
    }
}
