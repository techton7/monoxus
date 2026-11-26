use dioxus::prelude::*;
use monoxus_core::foundation::date_time::field_policy::TimeFieldPolicy;

use crate::{
    date_field::components::{
        SegmentFieldInputSpec, SegmentFieldLabelSpec, SegmentFieldRootSpec,
        SegmentFieldSegmentSpec, render_segment_field_input, render_segment_field_label,
        render_segment_field_root, render_segment_field_segment,
    },
    foundation::{
        date_time::{
            CalendarLocale, EditableSegmentKind, FieldInvalidEvent, HourCycle, SegmentKind,
            TimeGranularity, TimeValue,
        },
        shared::Direction,
    },
};

use super::{
    attrs::{TimeFieldInputAttrs, TimeFieldLabelAttrs, TimeFieldSegmentAttrs},
    state::default_time_field_placeholder,
    types::{TimeFieldRangeRole, TimeFieldSegmentItem},
};

/// Props for [`TimeFieldRoot`].
#[derive(Clone, PartialEq, Props)]
pub struct TimeFieldRootProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub value: Option<Signal<Option<TimeValue>>>,
    #[props(default)]
    pub default_value: Option<TimeValue>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<Option<TimeValue>>>,
    #[props(default)]
    pub placeholder: Option<Signal<TimeValue>>,
    #[props(default)]
    pub default_placeholder: Option<TimeValue>,
    #[props(default)]
    pub on_placeholder_change: Option<EventHandler<TimeValue>>,
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
    pub validate: Option<Callback<TimeValue, Option<Vec<String>>>>,
    #[props(default)]
    pub on_invalid: Option<EventHandler<FieldInvalidEvent>>,
    #[props(default)]
    pub error_message_id: Option<String>,
    #[props(default)]
    pub dir: Option<Direction>,
    /// External invalid override (e.g., propagated by `TimeRangeField`).
    #[props(default)]
    pub is_invalid_override: Option<bool>,
    /// Standalone vs range endpoint role (used by `TimeRangeField`).
    #[props(default)]
    pub range_role: TimeFieldRangeRole,
    /// Optional external label ID (used by `TimeRangeField` to share its `TimeRangeFieldLabel`).
    #[props(default)]
    pub label_id_override: Option<String>,
    /// Cross-field boundary callback fired when navigating before the first interactive segment.
    #[props(default)]
    pub on_prev_boundary: Option<EventHandler<()>>,
    /// Cross-field boundary callback fired when navigating past the last interactive segment.
    #[props(default)]
    pub on_next_boundary: Option<EventHandler<()>>,
    #[props(default)]
    pub children: Element,
}

/// Root provider for segmented wall-clock time spinbutton fields.
#[component]
pub fn TimeFieldRoot(props: TimeFieldRootProps) -> Element {
    let initial_value = props
        .value
        .and_then(|s| *s.peek())
        .or(props.default_value);
    let initial_placeholder = props
        .placeholder
        .map(|s| *s.peek())
        .or(props.default_placeholder)
        .unwrap_or_else(|| {
            default_time_field_placeholder(initial_value, props.min_value, props.max_value)
        });

    render_segment_field_root::<TimeFieldPolicy>(
        SegmentFieldRootSpec {
            id: props.id,
            value: props.value,
            default_value: props.default_value,
            on_value_change: props.on_value_change,
            placeholder: props.placeholder,
            on_placeholder_change: props.on_placeholder_change,
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
            range_role: props.range_role,
            label_id_override: props.label_id_override,
            on_prev_boundary: props.on_prev_boundary,
            on_next_boundary: props.on_next_boundary,
            children: props.children,
        },
        initial_placeholder,
    )
}

pub use TimeFieldRoot as TimeField;

/// Render props passed to `as_child` for [`TimeFieldInput`].
#[derive(Clone, PartialEq, Props)]
pub struct TimeFieldInputRenderProps {
    pub attrs: TimeFieldInputAttrs,
    pub segments: Vec<TimeFieldSegmentItem>,
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

/// Props for [`TimeFieldInput`].
#[derive(Clone, PartialEq, Props)]
pub struct TimeFieldInputProps {
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

/// Container for `TimeField` segments and automatic hidden `<input>` form synchronization.
#[component]
pub fn TimeFieldInput(props: TimeFieldInputProps) -> Element {
    let render_child = props.render_child;
    render_segment_field_input::<TimeFieldPolicy, _>(
        SegmentFieldInputSpec {
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
            debug_name: "TimeFieldInput",
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

/// Render props passed to `as_child` for [`TimeFieldSegment`].
#[derive(Clone, PartialEq, Props)]
pub struct TimeFieldSegmentRenderProps {
    pub attrs: TimeFieldSegmentAttrs,
    pub segment: TimeFieldSegmentItem,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    pub on_key_down: EventHandler<KeyboardEvent>,
    pub on_focus: EventHandler<FocusEvent>,
    pub on_blur: EventHandler<FocusEvent>,
    pub on_paste: EventHandler<ClipboardEvent>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Props for [`TimeFieldSegment`].
#[derive(Clone, PartialEq, Props)]
pub struct TimeFieldSegmentProps {
    pub part: SegmentKind,
    #[props(default)]
    pub index: Option<usize>,
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<TimeFieldSegmentRenderProps, Element>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_focus: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub on_blur: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub on_paste: Option<EventHandler<ClipboardEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Option<Element>,
}

/// Individual spinbutton or literal segment inside a `TimeFieldInput`.
#[component]
pub fn TimeFieldSegment(props: TimeFieldSegmentProps) -> Element {
    let render_child = props.render_child;
    render_segment_field_segment::<TimeFieldPolicy, _>(
        SegmentFieldSegmentSpec {
            part: props.part,
            index: props.index,
            id: props.id,
            class: props.class,
            style: props.style,
            as_child: props.as_child,
            on_key_down: props.on_key_down,
            on_focus: props.on_focus,
            on_blur: props.on_blur,
            on_paste: props.on_paste,
            on_mounted: props.on_mounted,
            attributes: props.attributes,
            children: props.children,
            debug_name: "TimeFieldSegment",
        },
        move |view| {
            render_child.map(|cb| {
                cb.call(TimeFieldSegmentRenderProps {
                    attrs: view.attrs,
                    segment: view.segment,
                    class: view.class,
                    style: view.style,
                    on_key_down: view.on_key_down,
                    on_focus: view.on_focus,
                    on_blur: view.on_blur,
                    on_paste: view.on_paste,
                    on_mounted: view.on_mounted,
                    attributes: view.attributes,
                    children: view.children,
                })
            })
        },
    )
}

/// Render props passed to `as_child` for [`TimeFieldLabel`].
#[derive(Clone, PartialEq, Props)]
pub struct TimeFieldLabelRenderProps {
    pub attrs: TimeFieldLabelAttrs,
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

/// Props for [`TimeFieldLabel`].
#[derive(Clone, PartialEq, Props)]
pub struct TimeFieldLabelProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<TimeFieldLabelRenderProps, Element>>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Accessible label for `TimeField`; clicking focuses the first interactive segment.
#[component]
pub fn TimeFieldLabel(props: TimeFieldLabelProps) -> Element {
    let render_child = props.render_child;
    render_segment_field_label::<TimeFieldPolicy, _>(
        SegmentFieldLabelSpec {
            id: props.id,
            class: props.class,
            style: props.style,
            as_child: props.as_child,
            on_click: props.on_click,
            on_mounted: props.on_mounted,
            attributes: props.attributes,
            children: props.children,
            debug_name: "TimeFieldLabel",
        },
        move |view| {
            render_child.map(|cb| {
                cb.call(TimeFieldLabelRenderProps {
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
