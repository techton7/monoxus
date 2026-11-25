use crate::foundation::shared::data_disabled_attr;

/// WAI-ARIA and `data-*` attributes for `DatePickerTrigger`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DatePickerTriggerAttrs {
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

impl DatePickerTriggerAttrs {
    pub fn new(
        id: String,
        aria_controls: String,
        open: bool,
        disabled: bool,
    ) -> Self {
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

/// WAI-ARIA and `data-*` attributes for `DatePickerContent` and `DatePickerContentStatic`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DatePickerContentAttrs {
    pub id: String,
    pub role: &'static str,
    pub tabindex: &'static str,
    pub data_state: &'static str,
    pub data_side: Option<&'static str>,
    pub data_align: Option<&'static str>,
    pub data_date_picker_content: &'static str,
}

impl DatePickerContentAttrs {
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
            data_date_picker_content: "",
        }
    }
}
