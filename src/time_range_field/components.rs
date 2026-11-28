use dioxus::prelude::*;
use ui_sdk::foundation::date_time::field_policy::TimeFieldPolicy;
use ui_sdk::kernel::ChangeEvent;

use crate::{
    agent::AgentHints,
    date_range_field::components::{
        RangeFieldInputSpec, RangeFieldLabelSpec, RangeFieldRootSpec, render_range_field_input,
        render_range_field_label, render_range_field_root,
    },
    foundation::{
        config::use_locale_context,
        date_time::{
            CalendarLocale, EditableSegmentKind, FieldInvalidEvent, HourCycle, TimeGranularity,
            TimeRange, TimeValue,
        },
        shared::Direction,
    },
    time_field::{TimeFieldSegmentItem, default_time_field_placeholder},
};

use super::types::RangeFieldInputKind;

pub use crate::time_field::{
    TimeFieldSegment as TimeRangeFieldSegment,
    TimeFieldSegmentProps as TimeRangeFieldSegmentProps,
};

/// Props for [`TimeRangeFieldRoot`].
#[derive(Clone, PartialEq, Props)]
pub struct TimeRangeFieldRootProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub value: Option<TimeRange>,
    #[props(default)]
    pub default_value: Option<TimeRange>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<TimeRange>>,
    #[props(default)]
    pub on_value_change_details: Option<EventHandler<ChangeEvent<TimeRange>>>,
    #[props(default)]
    pub start_value: Option<Option<TimeValue>>,
    #[props(default)]
    pub on_start_value_change: Option<EventHandler<Option<TimeValue>>>,
    #[props(default)]
    pub on_start_value_change_details: Option<EventHandler<ChangeEvent<Option<TimeValue>>>>,
    #[props(default)]
    pub end_value: Option<Option<TimeValue>>,
    #[props(default)]
    pub on_end_value_change: Option<EventHandler<Option<TimeValue>>>,
    #[props(default)]
    pub on_end_value_change_details: Option<EventHandler<ChangeEvent<Option<TimeValue>>>>,
    #[props(default)]
    pub placeholder: Option<TimeValue>,
    #[props(default)]
    pub default_placeholder: Option<TimeValue>,
    #[props(default)]
    pub on_placeholder_change: Option<EventHandler<TimeValue>>,
    #[props(default)]
    pub on_placeholder_change_details: Option<EventHandler<ChangeEvent<TimeValue>>>,
    #[props(default)]
    pub on_focus_out: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub granularity: TimeGranularity,
    #[props(default)]
    pub hour_cycle: Option<HourCycle>,
    #[props(default)]
    pub locale: Option<CalendarLocale>,
    #[props(default)]
    pub min_value: Option<TimeValue>,
    #[props(default)]
    pub max_value: Option<TimeValue>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub read_only: bool,
    #[props(default = false)]
    pub required: bool,
    #[props(default)]
    pub read_only_segments: Vec<EditableSegmentKind>,
    #[props(default)]
    pub validate: Option<Callback<TimeRange, Option<Vec<String>>>>,
    #[props(default)]
    pub on_invalid: Option<EventHandler<FieldInvalidEvent>>,
    #[props(default, into)]
    pub error_message_id: Option<String>,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(default)]
    pub agent: Option<AgentHints>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Root group container (`<div role="group">`) for segmented time range fields.
#[component]
pub fn TimeRangeFieldRoot(props: TimeRangeFieldRootProps) -> Element {
    let resolved_locale = use_locale_context(props.locale);
    let default_range = props.default_value.unwrap_or_default();
    let initial_start = props
        .start_value
        .unwrap_or_else(|| props.value.map_or(default_range.start, |v| v.start));
    let initial_end = props
        .end_value
        .unwrap_or_else(|| props.value.map_or(default_range.end, |v| v.end));
    let has_explicit_default_placeholder = props.default_placeholder.is_some();
    let initial_placeholder = props
        .placeholder
        .or(props.default_placeholder)
        .unwrap_or_else(|| {
            default_time_field_placeholder(
                initial_start.or(initial_end),
                props.min_value,
                props.max_value,
            )
        });

    render_range_field_root::<TimeFieldPolicy>(
        RangeFieldRootSpec {
            id: props.id,
            class: props.class,
            style: props.style,
            value: props.value,
            default_value: props.default_value,
            on_value_change: props.on_value_change,
            on_value_change_details: props.on_value_change_details,
            start_value: props.start_value,
            on_start_value_change: props.on_start_value_change,
            on_start_value_change_details: props.on_start_value_change_details,
            end_value: props.end_value,
            on_end_value_change: props.on_end_value_change,
            on_end_value_change_details: props.on_end_value_change_details,
            placeholder: props.placeholder,
            has_explicit_default_placeholder,
            on_placeholder_change: props.on_placeholder_change,
            on_placeholder_change_details: props.on_placeholder_change_details,
            on_focus_out: props.on_focus_out,
            granularity: props.granularity,
            hour_cycle: props.hour_cycle,
            locale: resolved_locale,
            hide_time_zone: false,
            min_value: props.min_value,
            max_value: props.max_value,
            disabled: props.disabled,
            read_only: props.read_only,
            required: props.required,
            read_only_segments: props.read_only_segments,
            is_date_unavailable: None,
            validate: props.validate,
            on_invalid: props.on_invalid,
            error_message_id: props.error_message_id,
            dir: props.dir,
            is_invalid_override: None,
            child: props.child,
            on_mounted: props.on_mounted,
            agent: props.agent,
            attributes: props.attributes,
            children: props.children,
        },
        initial_placeholder,
    )
}

pub use TimeRangeFieldRoot as TimeRangeField;

/// Props for [`TimeRangeFieldInput`].
#[derive(Clone, PartialEq, Props)]
pub struct TimeRangeFieldInputProps {
    pub kind: RangeFieldInputKind,
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default, into)]
    pub name: Option<String>,
    #[props(default, into)]
    pub form: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub render_children: Option<Callback<Vec<TimeFieldSegmentItem>, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Endpoint input (`Start` or `End`) inside a `TimeRangeFieldRoot`.
#[component]
pub fn TimeRangeFieldInput(props: TimeRangeFieldInputProps) -> Element {
    render_range_field_input::<TimeFieldPolicy>(RangeFieldInputSpec {
        kind: props.kind,
        id: props.id,
        class: props.class,
        style: props.style,
        name: props.name,
        form: props.form,
        child: props.child,
        render_children: props.render_children,
        on_mounted: props.on_mounted,
        attributes: props.attributes,
        children: props.children,
    })
}

/// Props for [`TimeRangeFieldLabel`].
#[derive(Clone, PartialEq, Props)]
pub struct TimeRangeFieldLabelProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Accessible label (`<span>`) for `TimeRangeField`; clicking focuses the first interactive segment.
#[component]
pub fn TimeRangeFieldLabel(props: TimeRangeFieldLabelProps) -> Element {
    render_range_field_label::<TimeFieldPolicy>(RangeFieldLabelSpec {
        id: props.id,
        class: props.class,
        style: props.style,
        child: props.child,
        on_click: props.on_click,
        on_mounted: props.on_mounted,
        attributes: props.attributes,
        children: props.children,
    })
}
