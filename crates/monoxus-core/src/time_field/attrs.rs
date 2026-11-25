use crate::foundation::{
    compose::{Slottable, project_as_child},
    date_time::{FieldLocaleTemplate, SegmentKind},
    shared::{Direction, data_disabled_attr},
};

use super::types::{TimeFieldRangeRole, TimeFieldSegmentItem};

/// WAI-ARIA and `data-*` attributes for `TimeFieldInput`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeFieldInputAttrs {
    pub id: String,
    pub role: &'static str,
    pub aria_labelledby: Option<String>,
    pub aria_describedby: Option<String>,
    pub aria_disabled: &'static str,
    pub aria_required: Option<&'static str>,
    pub dir: &'static str,
    pub data_time_field_input: &'static str,
    pub data_range_field_part: Option<&'static str>,
    pub data_invalid: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
    pub data_required: Option<&'static str>,
}

impl TimeFieldInputAttrs {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        range_role: TimeFieldRangeRole,
        label_id: Option<String>,
        description_id: Option<String>,
        dir: Direction,
        is_invalid: bool,
        disabled: bool,
    ) -> Self {
        Self {
            id,
            role: range_role.input_role(),
            aria_labelledby: label_id.filter(|s| !s.is_empty()),
            aria_describedby: description_id.filter(|s| !s.is_empty()),
            aria_disabled: if disabled { "true" } else { "false" },
            aria_required: None,
            dir: dir.as_str(),
            data_time_field_input: "",
            data_range_field_part: range_role.range_part_attr(),
            data_invalid: data_disabled_attr(is_invalid),
            data_disabled: data_disabled_attr(disabled),
            data_required: None,
        }
    }

    pub fn with_required(mut self, required: bool) -> Self {
        self.aria_required = if required { Some("true") } else { None };
        self.data_required = data_disabled_attr(required);
        self
    }
}

/// WAI-ARIA and `data-*` attributes for `TimeFieldLabel`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeFieldLabelAttrs {
    pub id: String,
    pub data_time_field_label: &'static str,
    pub data_invalid: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
}

impl TimeFieldLabelAttrs {
    pub fn new(id: String, is_invalid: bool, disabled: bool) -> Self {
        Self {
            id,
            data_time_field_label: "",
            data_invalid: data_disabled_attr(is_invalid),
            data_disabled: data_disabled_attr(disabled),
        }
    }
}

/// WAI-ARIA and `data-*` attributes for `TimeFieldSegment` (emitting `data-time-field-segment=""` per `5C D-4`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeFieldSegmentAttrs {
    pub id: String,
    pub role: Option<&'static str>,
    pub tabindex: Option<&'static str>,
    pub inputmode: Option<&'static str>,
    pub aria_label: Option<String>,
    pub aria_labelledby: Option<String>,
    pub aria_describedby: Option<String>,
    pub aria_valuemin: Option<String>,
    pub aria_valuemax: Option<String>,
    pub aria_valuenow: Option<String>,
    pub aria_valuetext: Option<String>,
    pub aria_invalid: Option<&'static str>,
    pub aria_disabled: Option<&'static str>,
    pub aria_readonly: Option<&'static str>,
    pub aria_hidden: Option<&'static str>,
    pub data_segment: &'static str,
    pub data_time_field_segment: &'static str,
    pub data_placeholder: Option<&'static str>,
    pub data_invalid: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
    pub data_readonly: Option<&'static str>,
}

impl TimeFieldSegmentAttrs {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        item: &TimeFieldSegmentItem,
        id_override: Option<String>,
        label_id: Option<&str>,
        description_id: Option<&str>,
        error_message_id: Option<&str>,
        is_first_interactive: bool,
        template: &FieldLocaleTemplate,
        is_invalid: bool,
        disabled: bool,
        read_only: bool,
    ) -> Self {
        let id = id_override
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| item.id.clone());
        let kind = item.segment.kind;
        let data_segment = kind.as_data_segment();

        match kind {
            SegmentKind::Literal => Self {
                id,
                role: None,
                tabindex: None,
                inputmode: None,
                aria_label: None,
                aria_labelledby: None,
                aria_describedby: None,
                aria_valuemin: None,
                aria_valuemax: None,
                aria_valuenow: None,
                aria_valuetext: None,
                aria_invalid: None,
                aria_disabled: None,
                aria_readonly: None,
                aria_hidden: Some("true"),
                data_segment,
                data_time_field_segment: "",
                data_placeholder: None,
                data_invalid: data_disabled_attr(is_invalid),
                data_disabled: data_disabled_attr(disabled),
                data_readonly: data_disabled_attr(read_only),
            },
            _ => {
                let aria_labelledby = label_id
                    .filter(|s| !s.is_empty())
                    .map(|lbl| format!("{id} {lbl}"));
                let aria_describedby = if is_first_interactive {
                    let mut ids = Vec::new();
                    if let Some(desc) = description_id.filter(|s| !s.is_empty()) {
                        ids.push(desc);
                    }
                    if is_invalid && let Some(err) = error_message_id.filter(|s| !s.is_empty()) {
                        ids.push(err);
                    }
                    (!ids.is_empty()).then(|| ids.join(" "))
                } else {
                    None
                };

                let inputmode = if kind == SegmentKind::DayPeriod {
                    Some("text")
                } else {
                    Some("numeric")
                };

                Self {
                    id,
                    role: Some("spinbutton"),
                    tabindex: if disabled { None } else { Some("0") },
                    inputmode,
                    aria_label: Some(template.aria_labels.for_kind(kind).to_string()),
                    aria_labelledby,
                    aria_describedby,
                    aria_valuemin: item.segment.min.map(|v| v.to_string()),
                    aria_valuemax: item.segment.max.map(|v| v.to_string()),
                    aria_valuenow: item.segment.now.map(|v| v.to_string()),
                    aria_valuetext: item.segment.value_text.clone(),
                    aria_invalid: if is_invalid { Some("true") } else { None },
                    aria_disabled: Some(if disabled { "true" } else { "false" }),
                    aria_readonly: Some(if read_only { "true" } else { "false" }),
                    aria_hidden: None,
                    data_segment,
                    data_time_field_segment: "",
                    data_placeholder: data_disabled_attr(item.segment.is_placeholder),
                    data_invalid: data_disabled_attr(is_invalid),
                    data_disabled: data_disabled_attr(disabled),
                    data_readonly: data_disabled_attr(read_only),
                }
            }
        }
    }
}

pub fn project_time_field_input_as_child<T, C>(
    attrs: &TimeFieldInputAttrs,
    target: T,
    content: Slottable<C>,
) -> (T, TimeFieldInputAttrs, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

pub fn project_time_field_label_as_child<T, C>(
    attrs: &TimeFieldLabelAttrs,
    target: T,
    content: Slottable<C>,
) -> (T, TimeFieldLabelAttrs, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

pub fn project_time_field_segment_as_child<T, C>(
    attrs: &TimeFieldSegmentAttrs,
    target: T,
    content: Slottable<C>,
) -> (T, TimeFieldSegmentAttrs, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}
