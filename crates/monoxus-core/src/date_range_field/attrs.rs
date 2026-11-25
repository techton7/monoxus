use crate::foundation::{
    compose::{Slottable, project_as_child},
    shared::{Direction, data_disabled_attr},
};

/// WAI-ARIA and `data-*` attributes for `DateRangeFieldRoot`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DateRangeFieldRootAttrs {
    pub id: String,
    pub role: &'static str,
    pub aria_labelledby: String,
    pub dir: &'static str,
    pub data_date_range_field_root: &'static str,
    pub data_invalid: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
}

impl DateRangeFieldRootAttrs {
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
            dir: dir.as_str(),
            data_date_range_field_root: "",
            data_invalid: data_disabled_attr(is_invalid),
            data_disabled: data_disabled_attr(disabled),
        }
    }
}

/// WAI-ARIA and `data-*` attributes for `DateRangeFieldLabel` (`<span>`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DateRangeFieldLabelAttrs {
    pub id: String,
    pub data_date_range_field_label: &'static str,
    pub data_invalid: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
}

impl DateRangeFieldLabelAttrs {
    pub fn new(id: String, is_invalid: bool, disabled: bool) -> Self {
        Self {
            id,
            data_date_range_field_label: "",
            data_invalid: data_disabled_attr(is_invalid),
            data_disabled: data_disabled_attr(disabled),
        }
    }
}

pub fn project_date_range_field_root_as_child<T, C>(
    attrs: &DateRangeFieldRootAttrs,
    target: T,
    content: Slottable<C>,
) -> (T, DateRangeFieldRootAttrs, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

pub fn project_date_range_field_label_as_child<T, C>(
    attrs: &DateRangeFieldLabelAttrs,
    target: T,
    content: Slottable<C>,
) -> (T, DateRangeFieldLabelAttrs, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}
