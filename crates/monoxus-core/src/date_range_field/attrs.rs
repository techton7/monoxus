use crate::foundation::shared::{Direction, data_disabled_attr, data_invalid_attr};

/// Shared WAI-ARIA and `data-*` attributes for `DateRangeFieldRoot` and `TimeRangeFieldRoot` (`D-5`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RangeFieldRootAttrs {
    pub id: String,
    pub role: &'static str,
    pub aria_labelledby: String,
    pub aria_describedby: Option<String>,
    pub aria_invalid: Option<&'static str>,
    pub dir: &'static str,
    pub data_marker: &'static str,
    pub data_invalid: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
}

impl RangeFieldRootAttrs {
    pub fn new(
        id: String,
        label_id: String,
        dir: Direction,
        is_invalid: bool,
        disabled: bool,
    ) -> Self {
        Self {
            id,
            role: "group",
            aria_labelledby: label_id,
            aria_describedby: None,
            aria_invalid: if is_invalid { Some("true") } else { None },
            dir: dir.as_str(),
            data_marker: "",
            data_invalid: data_invalid_attr(is_invalid),
            data_disabled: data_disabled_attr(disabled),
        }
    }

    pub fn with_invalid(mut self, invalid: bool) -> Self {
        self.aria_invalid = if invalid { Some("true") } else { None };
        self.data_invalid = data_invalid_attr(invalid);
        self
    }

    pub fn with_aria_describedby(mut self, describedby: Option<String>) -> Self {
        self.aria_describedby = describedby.filter(|s| !s.trim().is_empty());
        self
    }
}

/// Shared WAI-ARIA and `data-*` attributes for `DateRangeFieldLabel` and `TimeRangeFieldLabel` (`D-5`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RangeFieldLabelAttrs {
    pub id: String,
    pub data_marker: &'static str,
    pub data_invalid: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
}

impl RangeFieldLabelAttrs {
    pub fn new(id: String, is_invalid: bool, disabled: bool) -> Self {
        Self {
            id,
            data_marker: "",
            data_invalid: data_disabled_attr(is_invalid),
            data_disabled: data_disabled_attr(disabled),
        }
    }
}

pub type DateRangeFieldRootAttrs = RangeFieldRootAttrs;
pub type DateRangeFieldLabelAttrs = RangeFieldLabelAttrs;
