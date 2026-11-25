use std::{borrow::Cow, cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::{
    foundation::{
        date_time::{
            CalendarLocale, EditableSegmentKind, FieldInvalidEvent, HourCycle, TimeGranularity,
            TimeRange, TimeValue,
        },
        shared::{Direction, ScopeHandle, use_direction_context, use_stable_id},
    },
    time_field::{
        TimeFieldInput, TimeFieldInputRenderProps, TimeFieldRangeRole, TimeFieldRoot,
        TimeFieldSegmentItem, default_time_field_placeholder, use_time_field_context,
    },
};

use super::{
    attrs::{TimeRangeFieldLabelAttrs, TimeRangeFieldRootAttrs},
    relationships::TimeRangeFieldRelationships,
    runtime::{TimeRangeFieldContext, TimeRangeFieldRuntime, use_time_range_field_context},
    state::{TimeRangeFieldDefinition, TimeRangeFieldState},
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
    let resolved_dir = use_direction_context(props.dir);
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned)).into_owned();
    let scope = use_hook(|| ScopeHandle::root("time-range-field").child(stable_id.clone()));
    let rels = use_hook(|| TimeRangeFieldRelationships::new(scope.clone()));

    let initial_value = props
        .value
        .map(|s| *s.peek())
        .unwrap_or(props.default_value);
    let initial_placeholder = props
        .placeholder
        .map(|s| *s.peek())
        .or(props.default_placeholder)
        .unwrap_or_else(|| {
            default_time_field_placeholder(
                initial_value.start.or(initial_value.end),
                props.min_value,
                props.max_value,
            )
        });

    let def = TimeRangeFieldDefinition::new()
        .with_locale(props.locale.clone())
        .with_granularity(props.granularity)
        .with_hour_cycle(props.hour_cycle)
        .with_min_value(props.min_value)
        .with_max_value(props.max_value)
        .with_disabled(props.disabled)
        .with_read_only(props.read_only)
        .with_required(props.required)
        .with_readonly_segments(props.readonly_segments.clone())
        .with_error_message_id(props.error_message_id.clone())
        .with_dir(resolved_dir);

    let state = use_signal(|| {
        let mut st = TimeRangeFieldState::new(def.clone(), initial_value, initial_placeholder);
        if let Some(inv) = props.is_invalid_override {
            st.set_external_invalid(inv);
        }
        st
    });

    let start_value = use_signal(|| initial_value.start);
    let end_value = use_signal(|| initial_value.end);
    let placeholder = use_signal(|| initial_placeholder);
    let custom_label_id = use_signal(|| None::<String>);
    let last_invalid_reason = use_signal(|| None);

    let start_runtime = use_hook(|| Rc::new(RefCell::new(None)));
    let end_runtime = use_hook(|| Rc::new(RefCell::new(None)));

    let default_value = use_hook(|| Rc::new(RefCell::new(props.default_value)));
    *default_value.borrow_mut() = props.default_value;

    let controlled_value = use_hook(|| Rc::new(RefCell::new(props.value)));
    *controlled_value.borrow_mut() = props.value;

    let controlled_placeholder = use_hook(|| Rc::new(RefCell::new(props.placeholder)));
    *controlled_placeholder.borrow_mut() = props.placeholder;

    let on_value_change = use_hook(|| Rc::new(RefCell::new(props.on_value_change)));
    *on_value_change.borrow_mut() = props.on_value_change;

    let on_start_value_change = use_hook(|| Rc::new(RefCell::new(props.on_start_value_change)));
    *on_start_value_change.borrow_mut() = props.on_start_value_change;

    let on_end_value_change = use_hook(|| Rc::new(RefCell::new(props.on_end_value_change)));
    *on_end_value_change.borrow_mut() = props.on_end_value_change;

    let on_placeholder_change = use_hook(|| Rc::new(RefCell::new(props.on_placeholder_change)));
    *on_placeholder_change.borrow_mut() = props.on_placeholder_change;

    let validate = use_hook(|| Rc::new(RefCell::new(props.validate)));
    *validate.borrow_mut() = props.validate;

    let on_invalid = use_hook(|| Rc::new(RefCell::new(props.on_invalid)));
    *on_invalid.borrow_mut() = props.on_invalid;

    let runtime = TimeRangeFieldRuntime {
        state,
        rels: rels.clone(),
        start_value,
        end_value,
        placeholder,
        custom_label_id,
        last_invalid_reason,
        start_runtime,
        end_runtime,
        default_value,
        controlled_value,
        controlled_placeholder,
        on_value_change,
        on_start_value_change,
        on_end_value_change,
        on_placeholder_change,
        validate,
        on_invalid,
    };

    // DEF-5C-09: Reconcile definition, controlled signals, and validation inside a guarded effect.
    use_effect(use_reactive(
        (
            &def,
            &props.value,
            &props.placeholder,
            &props.is_invalid_override,
        ),
        {
            let mut rt = runtime.clone();
            move |(next_def, value_sig, placeholder_sig, inv_override)| {
                if *rt.state.peek().def() != next_def {
                    rt.state.write().sync_definition(next_def);
                }
                let ext_val = value_sig.map(|s| *s.read());
                let ext_ph = placeholder_sig.map(|s| *s.read());
                rt.reconcile_external_state(ext_val, ext_ph, inv_override);
            }
        },
    ));

    use_context_provider(|| TimeRangeFieldContext(runtime.clone()));

    let attrs = runtime.root_attrs(props.id.clone());

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(TimeRangeFieldRootRenderProps {
                attrs,
                class: props.class,
                style: props.style,
                on_mounted: props.on_mounted,
                attributes: props.attributes,
                children: props.children,
            });
        }
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::TimeRangeFieldRoot: as_child requires render_child; rendering the default div"
        );
    }

    rsx! {
        div {
            id: "{attrs.id}",
            class: props.class,
            style: props.style,
            role: attrs.role,
            "aria-labelledby": "{attrs.aria_labelledby}",
            dir: attrs.dir,
            "data-time-range-field-root": attrs.data_time_range_field_root,
            "data-invalid": attrs.data_invalid,
            "data-disabled": attrs.data_disabled,
            onmounted: move |evt| {
                if let Some(ref handler) = props.on_mounted {
                    handler.call(evt);
                }
            },
            ..props.attributes,
            {props.children}
        }
    }
}

pub use TimeRangeFieldRoot as TimeRangeField;

#[component]
fn TimeRangePartBridge(kind: RangeFieldInputKind, children: Element) -> Element {
    let range_rt = use_time_range_field_context();
    let field_rt = use_time_field_context();
    range_rt.register_part_runtime(kind, field_rt);
    children
}

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
    let range_rt = use_time_range_field_context();
    let part_invalid = range_rt.is_part_invalid(props.kind);
    let def = range_rt.state.read().def().clone();
    let part_default_value = range_rt.default_part_value(props.kind);
    let effective_label_id = range_rt.effective_label_id();

    let (part_value_sig, range_role, part_scope_id) = match props.kind {
        RangeFieldInputKind::Start => (
            range_rt.start_value,
            TimeFieldRangeRole::RangeStart,
            format!("{}-start", range_rt.rels.root_id()),
        ),
        RangeFieldInputKind::End => (
            range_rt.end_value,
            TimeFieldRangeRole::RangeEnd,
            format!("{}-end", range_rt.rels.root_id()),
        ),
    };

    let kind = props.kind;
    let mut rt_for_change = range_rt.clone();
    let mut rt_for_ph = range_rt.clone();
    let rt_for_prev = range_rt.clone();
    let rt_for_next = range_rt.clone();

    rsx! {
        TimeFieldRoot {
            id: Some(part_scope_id),
            value: Some(part_value_sig),
            default_value: part_default_value,
            on_value_change: move |next| rt_for_change.update_part_value(kind, next),
            placeholder: Some(range_rt.placeholder),
            on_placeholder_change: move |ph| rt_for_ph.update_placeholder(ph),
            granularity: def.granularity,
            hour_cycle: def.hour_cycle,
            locale: def.locale,
            min_value: def.min_value,
            max_value: def.max_value,
            disabled: def.disabled,
            read_only: def.read_only,
            required: def.required,
            readonly_segments: def.readonly_segments,
            error_message_id: def.error_message_id,
            dir: Some(def.dir),
            is_invalid_override: Some(part_invalid),
            range_role,
            label_id_override: Some(effective_label_id),
            on_prev_boundary: move |_| rt_for_prev.handle_part_prev_boundary(kind),
            on_next_boundary: move |_| rt_for_next.handle_part_next_boundary(kind),
            TimeRangePartBridge {
                kind,
                TimeFieldInput {
                    id: props.id,
                    class: props.class,
                    style: props.style,
                    name: props.name,
                    form: props.form,
                    as_child: props.as_child,
                    render_child: props.render_child,
                    render_children: props.render_children,
                    on_mounted: props.on_mounted,
                    attributes: props.attributes,
                    {props.children}
                }
            }
        }
    }
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
    let runtime = use_time_range_field_context();
    use_effect(use_reactive((&props.id,), {
        let mut rt = runtime.clone();
        move |(custom_id,)| {
            rt.register_custom_label_id(custom_id);
        }
    }));
    {
        let mut rt_drop = runtime.clone();
        use_drop(move || {
            rt_drop.register_custom_label_id(None);
        });
    }
    let attrs = runtime.label_attrs(props.id.clone());

    let on_click = {
        let rt = runtime.clone();
        let user_cb = props.on_click;
        EventHandler::new(move |evt: MouseEvent| {
            if !rt.state.peek().def().disabled {
                rt.focus_first_segment();
            }
            if let Some(cb) = user_cb {
                cb.call(evt);
            }
        })
    };

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(TimeRangeFieldLabelRenderProps {
                attrs,
                class: props.class,
                style: props.style,
                on_click,
                on_mounted: props.on_mounted,
                attributes: props.attributes,
                children: props.children,
            });
        }
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::TimeRangeFieldLabel: as_child requires render_child; rendering the default span"
        );
    }

    rsx! {
        span {
            id: "{attrs.id}",
            class: props.class,
            style: props.style,
            "data-time-range-field-label": attrs.data_time_range_field_label,
            "data-invalid": attrs.data_invalid,
            "data-disabled": attrs.data_disabled,
            onclick: move |evt| on_click.call(evt),
            onmounted: move |evt| {
                if let Some(ref handler) = props.on_mounted {
                    handler.call(evt);
                }
            },
            ..props.attributes,
            {props.children}
        }
    }
}
