use crate::foundation::shared::data_disabled_attr;

/// Shared WAI-ARIA and `data-*` attributes for `DatePickerTrigger` and `DateRangePickerTrigger`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PickerTriggerAttrs {
    pub id: String,
    pub aria_haspopup: &'static str,
    pub aria_expanded: &'static str,
    pub aria_controls: String,
    pub tabindex: Option<&'static str>,
    pub disabled: bool,
    pub data_segment: &'static str,
    pub data_state: &'static str,
    pub data_disabled: Option<&'static str>,
}

impl PickerTriggerAttrs {
    pub fn new(id: String, aria_controls: String, open: bool, disabled: bool) -> Self {
        Self {
            id,
            aria_haspopup: "dialog",
            aria_expanded: if open { "true" } else { "false" },
            aria_controls,
            tabindex: (!disabled).then_some("0"),
            disabled,
            data_segment: "trigger",
            data_state: if open { "open" } else { "closed" },
            data_disabled: data_disabled_attr(disabled),
        }
    }
}

/// Shared WAI-ARIA and `data-*` attributes for `DatePickerContent*` and `DateRangePickerContent*` (`D-5`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PickerContentAttrs {
    pub id: String,
    pub role: &'static str,
    pub tabindex: &'static str,
    pub data_state: &'static str,
    pub data_side: Option<&'static str>,
    pub data_align: Option<&'static str>,
    pub data_marker: &'static str,
}

impl PickerContentAttrs {
    pub fn new(
        id: String,
        open: bool,
        data_side: Option<&'static str>,
        data_align: Option<&'static str>,
    ) -> Self {
        Self {
            id,
            role: "dialog",
            tabindex: "-1",
            data_state: if open { "open" } else { "closed" },
            data_side,
            data_align,
            data_marker: "",
        }
    }
}

pub type DatePickerTriggerAttrs = PickerTriggerAttrs;
pub type DatePickerContentAttrs = PickerContentAttrs;
