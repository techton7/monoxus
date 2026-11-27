use dioxus::prelude::*;
use monoxus_core::foundation::date_time::field_policy::TimeFieldPolicy;

use crate::{
    date_range_field::components::{
        RangeFieldInputSpec, RangeFieldLabelSpec, RangeFieldRootSpec, render_range_field_input,
        render_range_field_label, render_range_field_root,
    },
    foundation::{
        date_time::{
            CalendarLocale, EditableSegmentKind, FieldInvalidEvent, HourCycle, TimeGranularity,
            TimeRange, TimeValue,
        },
        shared::Direction,
        state::ControlledBinding,
    },
    time_field::{TimeFieldInputRenderProps, TimeFieldSegmentItem, default_time_field_placeholder},
};

use super::{
    attrs::{TimeRangeFieldLabelAttrs, TimeRangeFieldRootAttrs},
    types::RangeFieldInputKind,
};

pub use crate::time_field::{
    TimeFieldSegment as TimeRangeFieldSegment,
    TimeFieldSegmentProps as TimeRangeFieldSegmentProps,
    TimeFieldSegmentRenderProps as TimeRangeFieldSegmentRenderProps,
};

/// Render props passed to `as_child` for [`TimeRangeFieldRoot`].
#[derive(Clone, PartialEq, Props)]
pub struct TimeRangeFieldRootRenderProps {
    pub attrs: TimeRangeFieldRootAttrs,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Props for [`TimeRangeFieldRoot`].
#[derive(Clone, PartialEq, Props)]
pub struct TimeRangeFieldRootProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub value: Option<Signal<TimeRange>>,
    #[props(default)]
    pub controlled_value: Option<ReadSignal<TimeRange>>,
    #[props(default)]
    pub default_value: TimeRange,
    #[props(default)]
    pub on_value_change: Option<EventHandler<TimeRange>>,
    #[props(default)]
    pub on_start_value_change: Option<EventHandler<Option<TimeValue>>>,
    #[props(default)]
    pub on_end_value_change: Option<EventHandler<Option<TimeValue>>>,
    #[props(default)]
    pub placeholder: Option<Signal<TimeValue>>,
    #[props(default)]
    pub controlled_placeholder: Option<ReadSignal<TimeValue>>,
    #[props(default)]
    pub default_placeholder: Option<TimeValue>,
    #[props(default)]
    pub on_placeholder_change: Option<EventHandler<TimeValue>>,
    #[props(default)]
    pub on_focus_out: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub granularity: TimeGranularity,
    #[props(default)]
    pub hour_cycle: Option<HourCycle>,
    #[props(default)]
    pub locale: CalendarLocale,
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
    pub readonly_segments: Vec<EditableSegmentKind>,
    #[props(default)]
    pub validate: Option<Callback<TimeRange, Option<Vec<String>>>>,
    #[props(default)]
    pub on_invalid: Option<EventHandler<FieldInvalidEvent>>,
    #[props(default)]
    pub error_message_id: Option<String>,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default)]
    pub is_invalid_override: Option<bool>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<TimeRangeFieldRootRenderProps, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Root group container (`<div role="group">`) for segmented time range fields.
#[component]
pub fn TimeRangeFieldRoot(props: TimeRangeFieldRootProps) -> Element {
    let value_binding = ControlledBinding::from_props(props.value, props.controlled_value);
    let placeholder_binding =
        ControlledBinding::from_props(props.placeholder, props.controlled_placeholder);
    let initial_value = value_binding.initial_or(props.default_value);
    let initial_placeholder = placeholder_binding
        .peek_cloned()
        .or(props.default_placeholder)
        .unwrap_or_else(|| {
            default_time_field_placeholder(
                initial_value.start.or(initial_value.end),
                props.min_value,
                props.max_value,
            )
        });

    let render_child = props.render_child;
    render_range_field_root::<TimeFieldPolicy, _>(
        RangeFieldRootSpec {
            id: props.id,
            class: props.class,
            style: props.style,
            value: props.value,
            controlled_value: props.controlled_value,
            default_value: props.default_value,
            on_value_change: props.on_value_change,
            on_start_value_change: props.on_start_value_change,
            on_end_value_change: props.on_end_value_change,
            placeholder: props.placeholder,
            controlled_placeholder: props.controlled_placeholder,
            on_placeholder_change: props.on_placeholder_change,
            on_focus_out: props.on_focus_out,
            granularity: props.granularity,
            hour_cycle: props.hour_cycle,
            locale: props.locale,
            hide_time_zone: false,
            min_value: props.min_value,
            max_value: props.max_value,
            disabled: props.disabled,
            read_only: props.read_only,
            required: props.required,
            readonly_segments: props.readonly_segments,
            is_date_unavailable: None,
            validate: props.validate,
            on_invalid: props.on_invalid,
            error_message_id: props.error_message_id,
            dir: props.dir,
            is_invalid_override: props.is_invalid_override,
            as_child: props.as_child,
            on_mounted: props.on_mounted,
            attributes: props.attributes,
            children: props.children,
            debug_name: "TimeRangeFieldRoot",
        },
        initial_placeholder,
        move |view| {
            render_child.map(|cb| {
                cb.call(TimeRangeFieldRootRenderProps {
                    attrs: view.attrs,
                    class: view.class,
                    style: view.style,
                    on_mounted: view.on_mounted,
                    attributes: view.attributes,
                    children: view.children,
                })
            })
        },
    )
}

pub use TimeRangeFieldRoot as TimeRangeField;

/// Props for [`TimeRangeFieldInput`].
#[derive(Clone, PartialEq, Props)]
pub struct TimeRangeFieldInputProps {
    pub kind: RangeFieldInputKind,
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub name: Option<String>,
    #[props(default)]
    pub form: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<TimeFieldInputRenderProps, Element>>,
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
    let render_child = props.render_child;
    render_range_field_input::<TimeFieldPolicy, _>(
        RangeFieldInputSpec {
            kind: props.kind,
            id: props.id,
            class: props.class,
            style: props.style,
            name: props.name,
            form: props.form,
            as_child: props.as_child,
            render_children: props.render_children,
            on_mounted: props.on_mounted,
            attributes: props.attributes,
            children: props.children,
            debug_name: "TimeRangeFieldInput",
        },
        move |view| {
            render_child.map(|cb| {
                cb.call(TimeFieldInputRenderProps {
                    attrs: view.attrs,
                    segments: view.segments,
                    class: view.class,
                    style: view.style,
                    on_mounted: view.on_mounted,
                    attributes: view.attributes,
                    children: view.children,
                })
            })
        },
    )
}

/// Render props passed to `as_child` for [`TimeRangeFieldLabel`].
#[derive(Clone, PartialEq, Props)]
pub struct TimeRangeFieldLabelRenderProps {
    pub attrs: TimeRangeFieldLabelAttrs,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    pub on_click: EventHandler<MouseEvent>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Props for [`TimeRangeFieldLabel`].
#[derive(Clone, PartialEq, Props)]
pub struct TimeRangeFieldLabelProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<TimeRangeFieldLabelRenderProps, Element>>,
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
    let render_child = props.render_child;
    render_range_field_label::<TimeFieldPolicy, _>(
        RangeFieldLabelSpec {
            id: props.id,
            class: props.class,
            style: props.style,
            as_child: props.as_child,
            on_click: props.on_click,
            on_mounted: props.on_mounted,
            attributes: props.attributes,
            children: props.children,
            debug_name: "TimeRangeFieldLabel",
        },
        move |view| {
            render_child.map(|cb| {
                cb.call(TimeRangeFieldLabelRenderProps {
                    attrs: view.attrs,
                    class: view.class,
                    style: view.style,
                    on_click: view.on_click,
                    on_mounted: view.on_mounted,
                    attributes: view.attributes,
                    children: view.children,
                })
            })
        },
    )
}
