use crate::foundation::{
    compose::{Slottable, project_as_child},
    shared::{Direction, data_disabled_attr},
};

/// WAI-ARIA and `data-*` attributes for `TimeRangeFieldRoot`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeRangeFieldRootAttrs {
    pub id: String,
    pub role: &'static str,
    pub aria_labelledby: String,
    pub dir: &'static str,
    pub data_time_range_field_root: &'static str,
    pub data_invalid: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
}

impl TimeRangeFieldRootAttrs {
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
            data_time_range_field_root: "",
            data_invalid: data_disabled_attr(is_invalid),
            data_disabled: data_disabled_attr(disabled),
        }
    }
}

/// WAI-ARIA and `data-*` attributes for `TimeRangeFieldLabel` (`<span>`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeRangeFieldLabelAttrs {
    pub id: String,
    pub data_time_range_field_label: &'static str,
    pub data_invalid: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
}

impl TimeRangeFieldLabelAttrs {
    pub fn new(id: String, is_invalid: bool, disabled: bool) -> Self {
        Self {
            id,
            data_time_range_field_label: "",
            data_invalid: data_disabled_attr(is_invalid),
            data_disabled: data_disabled_attr(disabled),
        }
    }
}

pub fn project_time_range_field_root_as_child<T, C>(
    attrs: &TimeRangeFieldRootAttrs,
    target: T,
    content: Slottable<C>,
) -> (T, TimeRangeFieldRootAttrs, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

pub fn project_time_range_field_label_as_child<T, C>(
    attrs: &TimeRangeFieldLabelAttrs,
    target: T,
    content: Slottable<C>,
) -> (T, TimeRangeFieldLabelAttrs, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}
