use crate::foundation::{
    shared::data_disabled_attr,
    state::DataState,
};

use super::types::CheckedState;

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

/// Pure attribute publisher for `CheckboxRoot`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckboxRootAttributes {
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
    pub data_checkbox_root: &'static str,
    pub disabled: bool,
}

impl CheckboxRootAttributes {
    pub fn new(
        id: Option<String>,
        button_type: &'static str,
        checked: CheckedState,
        disabled: bool,
        required: bool,
        read_only: bool,
    ) -> Self {
        Self {
            id,
            r#type: button_type,
            role: "checkbox",
            aria_checked: checked.aria_checked(),
            aria_required: aria_required_attr(required),
            aria_readonly: aria_readonly_attr(read_only, disabled),
            data_state: checked.data_state(),
            data_disabled: data_disabled_attr(disabled),
            data_required: data_required_attr(required),
            data_readonly: data_readonly_attr(read_only, disabled),
            data_checkbox_root: "",
            disabled,
        }
    }

    pub fn data_state_str(&self) -> &'static str {
        match self.data_state {
            DataState::Checked => "checked",
            DataState::Unchecked => "unchecked",
            DataState::Indeterminate => "indeterminate",
            _ => "unchecked",
        }
    }
}

/// Pure attribute publisher for `CheckboxIndicator`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckboxIndicatorAttributes {
    pub id: Option<String>,
    pub data_state: DataState,
    pub data_disabled: Option<&'static str>,
    pub data_readonly: Option<&'static str>,
    pub data_checkbox_indicator: &'static str,
    pub style: String,
}

impl CheckboxIndicatorAttributes {
    pub fn new(
        id: Option<String>,
        checked: CheckedState,
        disabled: bool,
        read_only: bool,
        consumer_style: Option<&str>,
    ) -> Self {
        let style = match consumer_style {
            Some(s) if !s.trim().is_empty() => {
                let trimmed = s.trim();
                if trimmed.ends_with(';') {
                    format!("pointer-events: none; {}", trimmed)
                } else {
                    format!("pointer-events: none; {};", trimmed)
                }
            }
            _ => "pointer-events: none;".to_string(),
        };

        Self {
            id,
            data_state: checked.data_state(),
            data_disabled: data_disabled_attr(disabled),
            data_readonly: data_readonly_attr(read_only, disabled),
            data_checkbox_indicator: "",
            style,
        }
    }

    pub fn data_state_str(&self) -> &'static str {
        match self.data_state {
            DataState::Checked => "checked",
            DataState::Unchecked => "unchecked",
            DataState::Indeterminate => "indeterminate",
            _ => "unchecked",
        }
    }
}

/// Pure attribute publisher for `CheckboxGroup`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckboxGroupAttributes {
    pub id: Option<String>,
    pub role: &'static str,
    pub aria_labelledby: Option<String>,
    pub aria_readonly: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
    pub data_required: Option<&'static str>,
    pub data_readonly: Option<&'static str>,
    pub data_checkbox_group: &'static str,
}

impl CheckboxGroupAttributes {
    pub fn new(
        id: Option<String>,
        label_id: Option<String>,
        disabled: bool,
        required: bool,
        read_only: bool,
    ) -> Self {
        Self {
            id,
            role: "group",
            aria_labelledby: label_id,
            aria_readonly: aria_readonly_attr(read_only, disabled),
            data_disabled: data_disabled_attr(disabled),
            data_required: data_required_attr(required),
            data_readonly: data_readonly_attr(read_only, disabled),
            data_checkbox_group: "",
        }
    }
}

/// Pure attribute publisher for `CheckboxGroupLabel`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckboxGroupLabelAttributes {
    pub id: Option<String>,
    pub data_disabled: Option<&'static str>,
    pub data_readonly: Option<&'static str>,
    pub data_checkbox_group_label: &'static str,
}

impl CheckboxGroupLabelAttributes {
    pub fn new(id: Option<String>, group_disabled: bool, group_read_only: bool) -> Self {
        Self {
            id,
            data_disabled: data_disabled_attr(group_disabled),
            data_readonly: data_readonly_attr(group_read_only, group_disabled),
            data_checkbox_group_label: "",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkbox_root_attributes_truth_table() {
        // Default unchecked enabled
        let attrs = CheckboxRootAttributes::new(
            Some("cb-1".to_string()),
            "button",
            CheckedState::Unchecked,
            false,
            false,
            false,
        );
        assert_eq!(attrs.role, "checkbox");
        assert_eq!(attrs.aria_checked, "false");
        assert_eq!(attrs.aria_required, None);
        assert_eq!(attrs.aria_readonly, None);
        assert_eq!(attrs.data_state, DataState::Unchecked);
        assert_eq!(attrs.data_disabled, None);
        assert_eq!(attrs.data_required, None);
        assert_eq!(attrs.data_readonly, None);
        assert_eq!(attrs.data_checkbox_root, "");
        assert!(!attrs.disabled);

        // Checked enabled
        let checked_attrs = CheckboxRootAttributes::new(
            Some("cb-2".to_string()),
            "button",
            CheckedState::Checked,
            false,
            true,
            false,
        );
        assert_eq!(checked_attrs.aria_checked, "true");
        assert_eq!(checked_attrs.aria_required, Some("true"));
        assert_eq!(checked_attrs.data_required, Some(""));
        assert_eq!(checked_attrs.data_state, DataState::Checked);

        // Indeterminate
        let ind_attrs = CheckboxRootAttributes::new(
            Some("cb-3".to_string()),
            "button",
            CheckedState::Indeterminate,
            false,
            false,
            false,
        );
        assert_eq!(ind_attrs.aria_checked, "mixed");
        assert_eq!(ind_attrs.data_state, DataState::Indeterminate);

        // Disabled
        let dis_attrs = CheckboxRootAttributes::new(
            Some("cb-4".to_string()),
            "button",
            CheckedState::Checked,
            true,
            true,
            true,
        );
        assert!(dis_attrs.disabled);
        assert_eq!(dis_attrs.data_disabled, Some(""));
        assert_eq!(dis_attrs.aria_readonly, None); // omitted when disabled
        assert_eq!(dis_attrs.data_readonly, None);

        // Read-only
        let ro_attrs = CheckboxRootAttributes::new(
            Some("cb-5".to_string()),
            "button",
            CheckedState::Checked,
            false,
            false,
            true,
        );
        assert!(!ro_attrs.disabled);
        assert_eq!(ro_attrs.aria_readonly, Some("true"));
        assert_eq!(ro_attrs.data_readonly, Some(""));
        assert_eq!(ro_attrs.data_disabled, None);
    }

    #[test]
    fn test_checkbox_indicator_attributes_pointer_events_style() {
        let ind1 = CheckboxIndicatorAttributes::new(
            Some("ind-1".to_string()),
            CheckedState::Checked,
            false,
            false,
            None,
        );
        assert_eq!(ind1.style, "pointer-events: none;");
        assert_eq!(ind1.data_checkbox_indicator, "");

        let ind2 = CheckboxIndicatorAttributes::new(
            Some("ind-2".to_string()),
            CheckedState::Indeterminate,
            false,
            false,
            Some("color: red;"),
        );
        assert_eq!(ind2.style, "pointer-events: none; color: red;");
    }

    #[test]
    fn test_checkbox_group_conditional_aria_labelledby() {
        let group_without_label =
            CheckboxGroupAttributes::new(Some("grp-1".to_string()), None, false, false, false);
        assert_eq!(group_without_label.aria_labelledby, None);

        let group_with_label = CheckboxGroupAttributes::new(
            Some("grp-2".to_string()),
            Some("grp-2-label".to_string()),
            false,
            false,
            false,
        );
        assert_eq!(
            group_with_label.aria_labelledby.as_deref(),
            Some("grp-2-label")
        );
    }
}
