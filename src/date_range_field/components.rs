use std::{borrow::Cow, cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use monoxus_core::foundation::date_time::field_policy::{
    DateFieldPolicy, FieldRangeEndpoint, FieldRangeRole, FieldSegmentItem, RangeFieldDefinition,
    RangeFieldPolicy, RangeFieldRelationships, RangeFieldState,
};

use crate::{
    date_field::{
        DateFieldInputRenderProps, DateFieldSegmentItem,
        components::{
            SegmentFieldInputSpec, SegmentFieldInputView, SegmentFieldRootSpec,
            render_segment_field_chrome, render_segment_field_input_with_runtime,
            use_segment_field_runtime,
        },
        default_date_field_placeholder,
        runtime::SegmentFieldUiPolicy,
    },
    foundation::{
        browser::local_today,
        compose::{merge_attributes_with_part, merge_global_attributes},
        date_time::{
            CalendarDate, CalendarLocale, ChronoRange, DateValue, DateValueRange,
            EditableSegmentKind, FieldInvalidEvent, Granularity, HourCycle,
        },
        shared::{Direction, ScopeHandle, use_direction_context, use_stable_id},
    },
};

use super::{
    attrs::{
        DateRangeFieldLabelAttrs, DateRangeFieldRootAttrs, RangeFieldLabelAttrs,
        RangeFieldRootAttrs,
    },
    runtime::{RangeFieldContext, RangeFieldRuntime, use_range_field_context},
    types::RangeFieldInputKind,
};

pub use crate::date_field::{
    DateFieldSegment as DateRangeFieldSegment,
    DateFieldSegmentProps as DateRangeFieldSegmentProps,
    DateFieldSegmentRenderProps as DateRangeFieldSegmentRenderProps,
};

/// Plain Rust specification for [`render_range_field_root`].
pub(crate) struct RangeFieldRootSpec<P: RangeFieldPolicy + SegmentFieldUiPolicy> {
    pub id: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub value: Option<Signal<ChronoRange<P::Value>>>,
    pub default_value: ChronoRange<P::Value>,
    pub on_value_change: Option<EventHandler<ChronoRange<P::Value>>>,
    pub on_start_value_change: Option<EventHandler<Option<P::Value>>>,
    pub on_end_value_change: Option<EventHandler<Option<P::Value>>>,
    pub placeholder: Option<Signal<P::Value>>,
    pub on_placeholder_change: Option<EventHandler<P::Value>>,
    pub granularity: P::GranularityProp,
    pub hour_cycle: Option<HourCycle>,
    pub locale: CalendarLocale,
    pub hide_time_zone: bool,
    pub min_value: Option<P::Value>,
    pub max_value: Option<P::Value>,
    pub disabled: bool,
    pub read_only: bool,
    pub required: bool,
    pub readonly_segments: Vec<EditableSegmentKind>,
    pub is_date_unavailable: Option<Callback<CalendarDate, bool>>,
    pub validate: Option<Callback<ChronoRange<P::Value>, Option<Vec<String>>>>,
    pub on_invalid: Option<EventHandler<FieldInvalidEvent>>,
    pub error_message_id: Option<String>,
    pub dir: Option<Direction>,
    pub is_invalid_override: Option<bool>,
    pub as_child: bool,
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
    pub debug_name: &'static str,
}

/// View passed to `render_as_child` inside [`render_range_field_root`].
pub(crate) struct RangeFieldRootView {
    pub attrs: RangeFieldRootAttrs,
    pub class: Option<String>,
    pub style: Option<String>,
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Initializes a [`RangeFieldRuntime<P>`], wires `B5`-compliant reactive prop/validate effects,
/// and provides `RangeFieldContext<P>` for descendants.
pub(crate) fn use_range_field_runtime<P>(
    spec: &RangeFieldRootSpec<P>,
    initial_placeholder: P::Value,
) -> RangeFieldRuntime<P>
where
    P: RangeFieldPolicy + SegmentFieldUiPolicy,
{
    let resolved_dir = use_direction_context(spec.dir);
    let stable_id = use_stable_id(spec.id.clone().map(Cow::Owned)).into_owned();
    let scope = use_hook(|| ScopeHandle::root(P::RANGE_SCOPE_PREFIX).child(stable_id.clone()));
    let rels = use_hook(|| RangeFieldRelationships::new(scope.clone()));

    let initial_value = spec.value.map(|s| *s.peek()).unwrap_or(spec.default_value);

    let def = RangeFieldDefinition::<P>::new()
        .with_locale(spec.locale.clone())
        .with_granularity(spec.granularity)
        .with_hour_cycle(spec.hour_cycle)
        .with_hide_time_zone(spec.hide_time_zone)
        .with_min_value(spec.min_value)
        .with_max_value(spec.max_value)
        .with_disabled(spec.disabled)
        .with_read_only(spec.read_only)
        .with_required(spec.required)
        .with_readonly_segments(spec.readonly_segments.clone())
        .with_error_message_id(spec.error_message_id.clone())
        .with_dir(resolved_dir);

    let state = use_signal(|| {
        let mut st = RangeFieldState::<P>::new(def.clone(), initial_value, initial_placeholder);
        if let Some(inv) = spec.is_invalid_override {
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

    let mut default_value = use_hook(|| CopyValue::new(spec.default_value));
    default_value.set(spec.default_value);

    let mut controlled_value = use_hook(|| CopyValue::new(spec.value));
    controlled_value.set(spec.value);

    let mut controlled_placeholder = use_hook(|| CopyValue::new(spec.placeholder));
    controlled_placeholder.set(spec.placeholder);

    let mut on_value_change = use_hook(|| CopyValue::new(spec.on_value_change));
    on_value_change.set(spec.on_value_change);

    let mut on_start_value_change = use_hook(|| CopyValue::new(spec.on_start_value_change));
    on_start_value_change.set(spec.on_start_value_change);

    let mut on_end_value_change = use_hook(|| CopyValue::new(spec.on_end_value_change));
    on_end_value_change.set(spec.on_end_value_change);

    let mut on_placeholder_change = use_hook(|| CopyValue::new(spec.on_placeholder_change));
    on_placeholder_change.set(spec.on_placeholder_change);

    let mut is_date_unavailable = use_hook(|| CopyValue::new(spec.is_date_unavailable));
    is_date_unavailable.set(spec.is_date_unavailable);

    let mut validate = use_hook(|| CopyValue::new(spec.validate));
    validate.set(spec.validate);

    let mut on_invalid = use_hook(|| CopyValue::new(spec.on_invalid));
    on_invalid.set(spec.on_invalid);

    let runtime = RangeFieldRuntime::<P> {
        state,
        rels,
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

    let mut predicate_rev = use_signal(|| 0_u64);
    let has_pred = spec.is_date_unavailable.is_some() || spec.validate.is_some();
    let mut pred_initialized = use_hook(|| CopyValue::new(false));
    let mut prev_had_pred = use_hook(|| CopyValue::new(has_pred));
    if pred_initialized.try_read().is_ok_and(|init| *init) {
        let had_pred = prev_had_pred.try_read().is_ok_and(|had| *had);
        prev_had_pred.set(has_pred);
        if has_pred || had_pred {
            let next_rev = predicate_rev.try_peek().ok().map(|cur| cur.wrapping_add(1));
            if let Some(next_rev) = next_rev {
                crate::foundation::state::sync_signal(&mut predicate_rev, next_rev);
            }
        }
    } else {
        pred_initialized.set(true);
    }

    // DEF-5C-09 & B5: Reconcile definition, predicates (`validate`, `is_date_unavailable`),
    // controlled signals, and validation inside guarded effects for BOTH DateRangeField and TimeRangeField.
    use_effect(use_reactive(
        (&def, &spec.is_date_unavailable, &spec.validate),
        {
            let mut rt = runtime.clone();
            move |(next_def, _next_unavailable, _next_validate)| {
                let _ = *predicate_rev.read();
                if *rt.state.peek().def() != next_def {
                    rt.state.write().sync_definition(next_def);
                }
                rt.sync_validation_and_notify();
            }
        },
    ));

    use_effect(use_reactive(
        (&spec.value, &spec.placeholder, &spec.is_invalid_override),
        {
            let mut rt = runtime.clone();
            move |(value_sig, placeholder_sig, inv_override)| {
                let ext_val = value_sig.map(|s| *s.read());
                let ext_ph = placeholder_sig.map(|s| *s.read());
                rt.reconcile_external_state(ext_val, ext_ph, inv_override);
            }
        },
    ));

    use_context_provider(|| RangeFieldContext::<P>(runtime.clone()));
    runtime
}

/// Renders the root group container for a pre-initialized [`RangeFieldRuntime<P>`].
pub(crate) fn render_range_field_root_with_runtime<P, F>(
    runtime: RangeFieldRuntime<P>,
    spec: RangeFieldRootSpec<P>,
    render_as_child: F,
) -> Element
where
    P: RangeFieldPolicy + SegmentFieldUiPolicy,
    F: FnOnce(RangeFieldRootView) -> Option<Element>,
{
    let attrs = runtime.root_attrs(spec.id.clone());

    if spec.as_child {
        let view = RangeFieldRootView {
            attrs: attrs.clone(),
            class: spec.class.clone(),
            style: spec.style.clone(),
            on_mounted: spec.on_mounted,
            attributes: spec.attributes.clone(),
            children: spec.children.clone(),
        };
        if let Some(el) = render_as_child(view) {
            return el;
        }
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::{}: as_child requires render_child; rendering the default div",
            spec.debug_name
        );
    }

    let attrs_with_part = merge_attributes_with_part(spec.attributes, P::RANGE_ROOT_DATA_ATTR);
    let merged = merge_global_attributes(
        spec.class.as_deref(),
        spec.style.as_deref(),
        attrs_with_part,
    );
    let on_mounted = spec.on_mounted;

    rsx! {
        div {
            id: "{attrs.id}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            role: attrs.role,
            "aria-labelledby": "{attrs.aria_labelledby}",
            dir: attrs.dir,
            "data-invalid": attrs.data_invalid,
            "data-disabled": attrs.data_disabled,
            onmounted: move |evt| {
                if let Some(handler) = on_mounted {
                    handler.call(evt);
                }
            },
            ..merged.rest,
            {spec.children}
        }
    }
}

/// Monomorphized plain-fn root renderer shared by `DateRangeFieldRoot` and `TimeRangeFieldRoot`
/// (fixes `B5` by including `&spec.validate` in `use_reactive` for both policies).
pub(crate) fn render_range_field_root<P, F>(
    spec: RangeFieldRootSpec<P>,
    initial_placeholder: P::Value,
    render_as_child: F,
) -> Element
where
    P: RangeFieldPolicy + SegmentFieldUiPolicy,
    F: FnOnce(RangeFieldRootView) -> Option<Element>,
{
    let runtime = use_range_field_runtime::<P>(&spec, initial_placeholder);
    render_range_field_root_with_runtime(runtime, spec, render_as_child)
}

/// Specification for [`render_range_field_input`].
pub(crate) struct RangeFieldInputSpec {
    pub kind: FieldRangeEndpoint,
    pub id: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub name: Option<String>,
    pub form: Option<String>,
    pub as_child: bool,
    pub render_children: Option<Callback<Vec<FieldSegmentItem>, Element>>,
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
    pub debug_name: &'static str,
}

/// Monomorphized plain-fn range endpoint input renderer shared by `DateRangeFieldInput` and
/// `TimeRangeFieldInput` (zero nested `#[component]` forwarding `Option<Callback>` per `#5894`,
/// zero `*RangePartBridge` wrapper).
pub(crate) fn render_range_field_input<P, F>(
    spec: RangeFieldInputSpec,
    render_as_child: F,
) -> Element
where
    P: RangeFieldPolicy + SegmentFieldUiPolicy,
    F: FnOnce(SegmentFieldInputView) -> Option<Element>,
{
    let range_rt = use_range_field_context::<P>();
    let part_invalid = range_rt.is_part_invalid(spec.kind);
    let def = range_rt.state.read().def().clone();
    let part_default_value = range_rt.default_part_value(spec.kind);
    let effective_label_id = range_rt.effective_label_id();
    let initial_placeholder = *range_rt.placeholder.peek();

    let (part_value_sig, range_role, part_scope_id) = match spec.kind {
        FieldRangeEndpoint::Start => (
            range_rt.start_value,
            FieldRangeRole::RangeStart,
            format!("{}-start", range_rt.rels.root_id()),
        ),
        FieldRangeEndpoint::End => (
            range_rt.end_value,
            FieldRangeRole::RangeEnd,
            format!("{}-end", range_rt.rels.root_id()),
        ),
    };

    let kind = spec.kind;
    let mut rt_for_change = range_rt.clone();
    let mut rt_for_ph = range_rt.clone();
    let rt_for_prev = range_rt.clone();
    let rt_for_next = range_rt.clone();

    let root_spec = SegmentFieldRootSpec::<P> {
        id: Some(part_scope_id),
        value: Some(part_value_sig),
        default_value: part_default_value,
        on_value_change: Some(EventHandler::new(move |next| {
            rt_for_change.update_part_value(kind, next);
        })),
        placeholder: Some(range_rt.placeholder),
        on_placeholder_change: Some(EventHandler::new(move |ph| {
            rt_for_ph.update_placeholder(ph);
        })),
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
        is_date_unavailable: None,
        validate: None,
        on_invalid: None,
        error_message_id: def.error_message_id,
        dir: Some(def.dir),
        is_invalid_override: Some(part_invalid),
        range_role,
        label_id_override: Some(effective_label_id),
        on_prev_boundary: Some(EventHandler::new(move |_| {
            rt_for_prev.handle_part_prev_boundary(kind);
        })),
        on_next_boundary: Some(EventHandler::new(move |_| {
            rt_for_next.handle_part_next_boundary(kind);
        })),
        children: rsx! {},
    };

    let field_rt = use_segment_field_runtime::<P>(&root_spec, initial_placeholder);
    let registered_kind = use_hook(|| Rc::new(RefCell::new(kind)));
    {
        let mut prev_kind = registered_kind.borrow_mut();
        if *prev_kind != kind {
            range_rt.unregister_part_runtime(*prev_kind, &field_rt);
            *prev_kind = kind;
        }
    }
    range_rt.register_part_runtime(kind, field_rt.clone());
    {
        let range_rt_drop = range_rt.clone();
        let field_rt_drop = field_rt.clone();
        let kind_drop = Rc::clone(&registered_kind);
        use_drop(move || {
            let current_kind = *kind_drop.borrow();
            range_rt_drop.unregister_part_runtime(current_kind, &field_rt_drop);
        });
    }

    let input_el = render_segment_field_input_with_runtime(
        field_rt.clone(),
        SegmentFieldInputSpec {
            id: spec.id,
            class: spec.class,
            style: spec.style,
            name: spec.name,
            form: spec.form,
            as_child: spec.as_child,
            render_children: spec.render_children,
            on_mounted: spec.on_mounted,
            attributes: spec.attributes,
            children: spec.children,
            debug_name: spec.debug_name,
        },
        render_as_child,
    );

    render_segment_field_chrome(&field_rt, input_el)
}

/// Specification for [`render_range_field_label`].
pub(crate) struct RangeFieldLabelSpec {
    pub id: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub as_child: bool,
    pub on_click: Option<EventHandler<MouseEvent>>,
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
    pub debug_name: &'static str,
}

/// View passed to `render_as_child` inside [`render_range_field_label`].
pub(crate) struct RangeFieldLabelView {
    pub attrs: RangeFieldLabelAttrs,
    pub class: Option<String>,
    pub style: Option<String>,
    pub on_click: EventHandler<MouseEvent>,
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Monomorphized plain-fn label renderer shared by `DateRangeFieldLabel` and `TimeRangeFieldLabel`,
/// with equality-guarded `custom_label_id` drop cleanup preventing keyed swap clobber.
pub(crate) fn render_range_field_label<P, F>(
    spec: RangeFieldLabelSpec,
    render_as_child: F,
) -> Element
where
    P: RangeFieldPolicy + SegmentFieldUiPolicy,
    F: FnOnce(RangeFieldLabelView) -> Option<Element>,
{
    let mut runtime = use_range_field_context::<P>();
    let normalized_id = spec.id.clone().filter(|s| !s.is_empty());
    let latest_label_id = use_hook(|| Rc::new(RefCell::new(normalized_id.clone())));
    {
        let mut prev = latest_label_id.borrow_mut();
        if *prev != normalized_id {
            if let Some(old) = prev.as_deref() {
                runtime.unregister_custom_label_id(old);
            }
            *prev = normalized_id;
        }
    }
    use_effect(use_reactive((&spec.id,), {
        let mut rt = runtime.clone();
        move |(custom_id,)| {
            rt.register_custom_label_id(custom_id);
        }
    }));
    {
        let mut rt_drop = runtime.clone();
        let latest_drop = Rc::clone(&latest_label_id);
        use_drop(move || {
            if let Some(id) = latest_drop.borrow().as_deref() {
                rt_drop.unregister_custom_label_id(id);
            }
        });
    }

    let attrs = runtime.label_attrs(spec.id.clone());
    let on_click = {
        let rt = runtime.clone();
        let user_cb = spec.on_click;
        EventHandler::new(move |evt: MouseEvent| {
            if rt.state.try_peek().is_ok_and(|st| !st.def().disabled) {
                rt.focus_first_segment();
            }
            if let Some(cb) = user_cb {
                cb.call(evt);
            }
        })
    };

    if spec.as_child {
        let view = RangeFieldLabelView {
            attrs: attrs.clone(),
            class: spec.class.clone(),
            style: spec.style.clone(),
            on_click,
            on_mounted: spec.on_mounted,
            attributes: spec.attributes.clone(),
            children: spec.children.clone(),
        };
        if let Some(el) = render_as_child(view) {
            return el;
        }
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::{}: as_child requires render_child; rendering the default span",
            spec.debug_name
        );
    }

    let attrs_with_part = merge_attributes_with_part(spec.attributes, P::RANGE_LABEL_DATA_ATTR);
    let merged = merge_global_attributes(
        spec.class.as_deref(),
        spec.style.as_deref(),
        attrs_with_part,
    );
    let on_mounted = spec.on_mounted;

    rsx! {
        span {
            id: "{attrs.id}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            "data-invalid": attrs.data_invalid,
            "data-disabled": attrs.data_disabled,
            onclick: move |evt| on_click.call(evt),
            onmounted: move |evt| {
                if let Some(handler) = on_mounted {
                    handler.call(evt);
                }
            },
            ..merged.rest,
            {spec.children}
        }
    }
}

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

    let render_child = props.render_child;
    render_range_field_root::<DateFieldPolicy, _>(
        RangeFieldRootSpec {
            id: props.id,
            class: props.class,
            style: props.style,
            value: props.value,
            default_value: props.default_value,
            on_value_change: props.on_value_change,
            on_start_value_change: props.on_start_value_change,
            on_end_value_change: props.on_end_value_change,
            placeholder: props.placeholder,
            on_placeholder_change: props.on_placeholder_change,
            granularity: props.granularity,
            hour_cycle: props.hour_cycle,
            locale: props.locale,
            hide_time_zone: props.hide_time_zone,
            min_value: props.min_value,
            max_value: props.max_value,
            disabled: props.disabled,
            read_only: props.read_only,
            required: props.required,
            readonly_segments: props.readonly_segments,
            is_date_unavailable: props.is_date_unavailable,
            validate: props.validate,
            on_invalid: props.on_invalid,
            error_message_id: props.error_message_id,
            dir: props.dir,
            is_invalid_override: props.is_invalid_override,
            as_child: props.as_child,
            on_mounted: props.on_mounted,
            attributes: props.attributes,
            children: props.children,
            debug_name: "DateRangeFieldRoot",
        },
        initial_placeholder,
        move |view| {
            render_child.map(|cb| {
                cb.call(DateRangeFieldRootRenderProps {
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

pub use DateRangeFieldRoot as DateRangeField;

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
#[component]
pub fn DateRangeFieldInput(props: DateRangeFieldInputProps) -> Element {
    let render_child = props.render_child;
    render_range_field_input::<DateFieldPolicy, _>(
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
            debug_name: "DateRangeFieldInput",
        },
        move |view| {
            render_child.map(|cb| {
                cb.call(DateFieldInputRenderProps {
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
    let render_child = props.render_child;
    render_range_field_label::<DateFieldPolicy, _>(
        RangeFieldLabelSpec {
            id: props.id,
            class: props.class,
            style: props.style,
            as_child: props.as_child,
            on_click: props.on_click,
            on_mounted: props.on_mounted,
            attributes: props.attributes,
            children: props.children,
            debug_name: "DateRangeFieldLabel",
        },
        move |view| {
            render_child.map(|cb| {
                cb.call(DateRangeFieldLabelRenderProps {
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
