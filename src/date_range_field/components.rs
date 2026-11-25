use std::{borrow::Cow, cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::{
    date_field::{
        DateFieldInput, DateFieldInputRenderProps, DateFieldRangeRole, DateFieldRoot,
        DateFieldSegmentItem, default_date_field_placeholder, local_today, use_date_field_context,
    },
    foundation::{
        date_time::{
            CalendarDate, CalendarLocale, DateValue, DateValueRange, EditableSegmentKind,
            FieldInvalidEvent, Granularity, HourCycle,
        },
        shared::{Direction, ScopeHandle, use_direction_context, use_stable_id},
    },
};

use super::{
    attrs::{DateRangeFieldLabelAttrs, DateRangeFieldRootAttrs},
    relationships::DateRangeFieldRelationships,
    runtime::{DateRangeFieldContext, DateRangeFieldRuntime, use_date_range_field_context},
    state::{DateRangeFieldDefinition, DateRangeFieldState},
    types::RangeFieldInputKind,
};

pub use crate::date_field::{
    DateFieldSegment as DateRangeFieldSegment,
    DateFieldSegmentProps as DateRangeFieldSegmentProps,
    DateFieldSegmentRenderProps as DateRangeFieldSegmentRenderProps,
};

/// Render props passed to `as_child` for [`DateRangeFieldRoot`].
#[derive(Clone, PartialEq, Props)]
pub struct DateRangeFieldRootRenderProps {
    pub attrs: DateRangeFieldRootAttrs,
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

/// Props for [`DateRangeFieldRoot`].
#[derive(Clone, PartialEq, Props)]
pub struct DateRangeFieldRootProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub value: Option<Signal<DateValueRange>>,
    #[props(default)]
    pub default_value: DateValueRange,
    #[props(default)]
    pub on_value_change: Option<EventHandler<DateValueRange>>,
    #[props(default)]
    pub on_start_value_change: Option<EventHandler<Option<DateValue>>>,
    #[props(default)]
    pub on_end_value_change: Option<EventHandler<Option<DateValue>>>,
    #[props(default)]
    pub placeholder: Option<Signal<DateValue>>,
    #[props(default)]
    pub default_placeholder: Option<DateValue>,
    #[props(default)]
    pub on_placeholder_change: Option<EventHandler<DateValue>>,
    #[props(default)]
    pub granularity: Option<Granularity>,
    #[props(default)]
    pub hour_cycle: Option<HourCycle>,
    #[props(default)]
    pub locale: CalendarLocale,
    #[props(default = false)]
    pub hide_time_zone: bool,
    #[props(default)]
    pub min_value: Option<DateValue>,
    #[props(default)]
    pub max_value: Option<DateValue>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub read_only: bool,
    #[props(default = false)]
    pub required: bool,
    #[props(default)]
    pub readonly_segments: Vec<EditableSegmentKind>,
    #[props(default)]
    pub is_date_unavailable: Option<Callback<CalendarDate, bool>>,
    #[props(default)]
    pub validate: Option<Callback<DateValueRange, Option<Vec<String>>>>,
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
    pub render_child: Option<Callback<DateRangeFieldRootRenderProps, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Root group container (`<div role="group">`) for segmented date range fields.
#[component]
pub fn DateRangeFieldRoot(props: DateRangeFieldRootProps) -> Element {
    let resolved_dir = use_direction_context(props.dir);
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned)).into_owned();
    let scope = use_hook(|| ScopeHandle::root("date-range-field").child(stable_id.clone()));
    let rels = use_hook(|| DateRangeFieldRelationships::new(scope.clone()));
    let today = use_hook(local_today);

    let initial_value = props
        .value
        .map(|s| *s.peek())
        .unwrap_or(props.default_value);
    let initial_placeholder = props
        .placeholder
        .map(|s| *s.peek())
        .or(props.default_placeholder)
        .unwrap_or_else(|| {
            default_date_field_placeholder(
                initial_value.start.or(initial_value.end),
                props.granularity,
                props.min_value,
                props.max_value,
                today,
            )
        });

    let def = DateRangeFieldDefinition::new()
        .with_locale(props.locale.clone())
        .with_granularity(props.granularity)
        .with_hour_cycle(props.hour_cycle)
        .with_hide_time_zone(props.hide_time_zone)
        .with_min_value(props.min_value)
        .with_max_value(props.max_value)
        .with_disabled(props.disabled)
        .with_read_only(props.read_only)
        .with_required(props.required)
        .with_readonly_segments(props.readonly_segments.clone())
        .with_error_message_id(props.error_message_id.clone())
        .with_dir(resolved_dir);

    let state = use_signal(|| {
        let mut st = DateRangeFieldState::new(def.clone(), initial_value, initial_placeholder);
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
    let trailing_trigger_id = use_signal(|| None::<String>);

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

    let is_date_unavailable = use_hook(|| Rc::new(RefCell::new(props.is_date_unavailable)));
    *is_date_unavailable.borrow_mut() = props.is_date_unavailable;

    let validate = use_hook(|| Rc::new(RefCell::new(props.validate)));
    *validate.borrow_mut() = props.validate;

    let on_invalid = use_hook(|| Rc::new(RefCell::new(props.on_invalid)));
    *on_invalid.borrow_mut() = props.on_invalid;

    let runtime = DateRangeFieldRuntime {
        state,
        rels: rels.clone(),
        start_value,
        end_value,
        placeholder,
        custom_label_id,
        last_invalid_reason,
        trailing_trigger_id,
        start_runtime,
        end_runtime,
        default_value,
        controlled_value,
        controlled_placeholder,
        on_value_change,
        on_start_value_change,
        on_end_value_change,
        on_placeholder_change,
        is_date_unavailable,
        validate,
        on_invalid,
    };

    // DEF-5C-09: Reconcile definition, predicates, controlled signals, and validation inside
    // guarded effects.
    use_effect(use_reactive(
        (&def, &props.is_date_unavailable, &props.validate),
        {
            let mut rt = runtime.clone();
            move |(next_def, _next_unavailable, _next_validate)| {
                if *rt.state.peek().def() != next_def {
                    rt.state.write().sync_definition(next_def);
                }
                rt.sync_validation_and_notify();
            }
        },
    ));

    use_effect(use_reactive(
        (&props.value, &props.placeholder, &props.is_invalid_override),
        {
            let mut rt = runtime.clone();
            move |(value_sig, placeholder_sig, inv_override)| {
                let ext_val = value_sig.map(|s| *s.read());
                let ext_ph = placeholder_sig.map(|s| *s.read());
                rt.reconcile_external_state(ext_val, ext_ph, inv_override);
            }
        },
    ));

    use_context_provider(|| DateRangeFieldContext(runtime.clone()));

    let attrs = runtime.root_attrs(props.id.clone());

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(DateRangeFieldRootRenderProps {
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
            "monoxus::DateRangeFieldRoot: as_child requires render_child; rendering the default div"
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
            "data-date-range-field-root": attrs.data_date_range_field_root,
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

pub use DateRangeFieldRoot as DateRangeField;

#[component]
fn DateRangePartBridge(kind: RangeFieldInputKind, children: Element) -> Element {
    let range_rt = use_date_range_field_context();
    let field_rt = use_date_field_context();
    range_rt.register_part_runtime(kind, field_rt);
    children
}

/// Props for [`DateRangeFieldInput`].
#[derive(Clone, PartialEq, Props)]
pub struct DateRangeFieldInputProps {
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
    pub render_child: Option<Callback<DateFieldInputRenderProps, Element>>,
    #[props(default)]
    pub render_children: Option<Callback<Vec<DateFieldSegmentItem>, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Endpoint input (`Start` or `End`) inside a `DateRangeFieldRoot`.
///
/// Renders `role="presentation"` with an independent hidden `<input>` when `name` is set,
/// and delegates segments to an isolated child `DateFieldRoot` wired for cross-field traversal
/// and range-level invalidity propagation.
#[component]
pub fn DateRangeFieldInput(props: DateRangeFieldInputProps) -> Element {
    let range_rt = use_date_range_field_context();
    let part_invalid = range_rt.is_part_invalid(props.kind);
    let def = range_rt.state.read().def().clone();
    let part_default_value = range_rt.default_part_value(props.kind);
    let effective_label_id = range_rt.effective_label_id();

    let (part_value_sig, range_role, part_scope_id) = match props.kind {
        RangeFieldInputKind::Start => (
            range_rt.start_value,
            DateFieldRangeRole::RangeStart,
            format!("{}-start", range_rt.rels.root_id()),
        ),
        RangeFieldInputKind::End => (
            range_rt.end_value,
            DateFieldRangeRole::RangeEnd,
            format!("{}-end", range_rt.rels.root_id()),
        ),
    };

    let kind = props.kind;
    let mut rt_for_change = range_rt.clone();
    let mut rt_for_ph = range_rt.clone();
    let rt_for_prev = range_rt.clone();
    let rt_for_next = range_rt.clone();

    rsx! {
        DateFieldRoot {
            id: Some(part_scope_id),
            value: Some(part_value_sig),
            default_value: part_default_value,
            on_value_change: move |next| rt_for_change.update_part_value(kind, next),
            placeholder: Some(range_rt.placeholder),
            on_placeholder_change: move |ph| rt_for_ph.update_placeholder(ph),
            granularity: def.granularity,
            hour_cycle: def.hour_cycle,
            locale: def.locale,
            hide_time_zone: def.hide_time_zone,
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
            DateRangePartBridge {
                kind,
                DateFieldInput {
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

/// Render props passed to `as_child` for [`DateRangeFieldLabel`].
#[derive(Clone, PartialEq, Props)]
pub struct DateRangeFieldLabelRenderProps {
    pub attrs: DateRangeFieldLabelAttrs,
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

/// Props for [`DateRangeFieldLabel`].
#[derive(Clone, PartialEq, Props)]
pub struct DateRangeFieldLabelProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<DateRangeFieldLabelRenderProps, Element>>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Accessible label (`<span>`) for `DateRangeField`; clicking focuses the first interactive segment.
#[component]
pub fn DateRangeFieldLabel(props: DateRangeFieldLabelProps) -> Element {
    let runtime = use_date_range_field_context();
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
            return render_child.call(DateRangeFieldLabelRenderProps {
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
            "monoxus::DateRangeFieldLabel: as_child requires render_child; rendering the default span"
        );
    }

    rsx! {
        span {
            id: "{attrs.id}",
            class: props.class,
            style: props.style,
            "data-date-range-field-label": attrs.data_date_range_field_label,
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
