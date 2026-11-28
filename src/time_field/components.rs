use dioxus::prelude::*;
use ui_sdk::foundation::date_time::field_policy::{FieldRangeRole, TimeFieldPolicy};
use ui_sdk::kernel::ChangeEvent;

use crate::{
    agent::AgentHints,
    date_field::components::{
        SegmentFieldInputSpec, SegmentFieldLabelSpec, SegmentFieldRootSpec,
        SegmentFieldSegmentSpec, render_segment_field_input, render_segment_field_label,
        render_segment_field_root, render_segment_field_segment,
    },
    foundation::{
        config::use_locale_context,
        date_time::{
            CalendarLocale, EditableSegmentKind, FieldInvalidEvent, HourCycle, SegmentKind,
            TimeGranularity, TimeValue,
        },
        shared::Direction,
    },
};

use super::{state::default_time_field_placeholder, types::TimeFieldSegmentItem};

/// Props for [`TimeFieldRoot`].
#[derive(Clone, PartialEq, Props)]
pub struct TimeFieldRootProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default)]
    pub value: Option<Option<TimeValue>>,
    #[props(default)]
    pub default_value: Option<TimeValue>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<Option<TimeValue>>>,
    #[props(default)]
    pub on_value_change_details: Option<EventHandler<ChangeEvent<Option<TimeValue>>>>,
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
    pub validate: Option<Callback<TimeValue, Option<Vec<String>>>>,
    #[props(default)]
    pub on_invalid: Option<EventHandler<FieldInvalidEvent>>,
    #[props(default, into)]
    pub error_message_id: Option<String>,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default)]
    pub agent: Option<AgentHints>,
    #[props(default)]
    pub children: Element,
}

/// Root provider for segmented wall-clock time spinbutton fields.
#[component]
pub fn TimeFieldRoot(props: TimeFieldRootProps) -> Element {
    let resolved_locale = use_locale_context(props.locale);
    let initial_value = props.value.unwrap_or(props.default_value);
    let has_explicit_default_placeholder = props.default_placeholder.is_some();
    let initial_placeholder = props
        .placeholder
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
            on_value_change_details: props.on_value_change_details,
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
            range_role: FieldRangeRole::Standalone,
            label_id_override: None,
            on_prev_boundary: None,
            on_next_boundary: None,
            agent: props.agent,
            children: props.children,
        },
        initial_placeholder,
    )
}

pub use TimeFieldRoot as TimeField;

/// Props for [`TimeFieldInput`].
#[derive(Clone, PartialEq, Props)]
pub struct TimeFieldInputProps {
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
    pub on_focus_out: Option<EventHandler<FocusEvent>>,
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
    render_segment_field_input::<TimeFieldPolicy>(SegmentFieldInputSpec {
        id: props.id,
        class: props.class,
        style: props.style,
        name: props.name,
        form: props.form,
        child: props.child,
        render_children: props.render_children,
        on_focus_out: props.on_focus_out,
        on_mounted: props.on_mounted,
        attributes: props.attributes,
        children: props.children,
    })
}

/// Props for [`TimeFieldSegment`].
#[derive(Clone, PartialEq, Props)]
pub struct TimeFieldSegmentProps {
    pub part: SegmentKind,
    #[props(default)]
    pub index: Option<usize>,
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
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
    render_segment_field_segment::<TimeFieldPolicy>(SegmentFieldSegmentSpec {
        part: props.part,
        index: props.index,
        id: props.id,
        class: props.class,
        style: props.style,
        child: props.child,
        on_key_down: props.on_key_down,
        on_focus: props.on_focus,
        on_blur: props.on_blur,
        on_paste: props.on_paste,
        on_mounted: props.on_mounted,
        attributes: props.attributes,
        children: props.children,
    })
}

/// Props for [`TimeFieldLabel`].
#[derive(Clone, PartialEq, Props)]
pub struct TimeFieldLabelProps {
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

/// Accessible label for `TimeField`; clicking focuses the first interactive segment.
#[component]
pub fn TimeFieldLabel(props: TimeFieldLabelProps) -> Element {
    render_segment_field_label::<TimeFieldPolicy>(SegmentFieldLabelSpec {
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
