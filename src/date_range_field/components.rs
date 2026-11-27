use std::{borrow::Cow, cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use ui_sdk::foundation::date_time::field_policy::{
    DateFieldPolicy, FieldRangeEndpoint, FieldRangeRole, FieldSegmentItem, RangeFieldDefinition,
    RangeFieldPolicy, RangeFieldRelationships, RangeFieldState,
};

use crate::{
    date_field::{
        DateFieldSegmentItem,
        components::{
            SegmentFieldInputSpec, SegmentFieldRootSpec, render_segment_field_chrome,
            render_segment_field_input_with_runtime, use_segment_field_runtime,
        },
        default_date_field_placeholder,
        runtime::SegmentFieldUiPolicy,
    },
    field::try_use_field_context,
    foundation::{
        browser::{focus::use_composite_focus_out, local_today},
        compose::{
            collect_user_attrs, extract_aria_invalid_and_describedby, merge_props, push_attr_opt,
            push_event_handler, resolve_invalid_and_describedby,
        },
        date_time::{
            CalendarDate, CalendarLocale, ChronoRange, DateValue, DateValueRange,
            EditableSegmentKind, FieldInvalidEvent, Granularity, HourCycle,
        },
        shared::{Direction, ScopeHandle, use_direction_context, use_stable_id},
        state::{sync_signal, use_controllable_signal},
    },
};

use super::{
    runtime::{RangeFieldContext, RangeFieldRuntime, use_range_field_context},
    types::RangeFieldInputKind,
};

pub use crate::date_field::{
    DateFieldSegment as DateRangeFieldSegment,
    DateFieldSegmentProps as DateRangeFieldSegmentProps,
};

/// Plain Rust specification for [`render_range_field_root`].
pub(crate) struct RangeFieldRootSpec<P: RangeFieldPolicy + SegmentFieldUiPolicy> {
    pub id: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub value: Option<ChronoRange<P::Value>>,
    pub default_value: Option<ChronoRange<P::Value>>,
    pub on_value_change: Option<EventHandler<ChronoRange<P::Value>>>,
    pub start_value: Option<Option<P::Value>>,
    pub on_start_value_change: Option<EventHandler<Option<P::Value>>>,
    pub end_value: Option<Option<P::Value>>,
    pub on_end_value_change: Option<EventHandler<Option<P::Value>>>,
    pub placeholder: Option<P::Value>,
    pub on_placeholder_change: Option<EventHandler<P::Value>>,
    pub on_focus_out: Option<EventHandler<FocusEvent>>,
    pub granularity: P::GranularityProp,
    pub hour_cycle: Option<HourCycle>,
    pub locale: CalendarLocale,
    pub hide_time_zone: bool,
    pub min_value: Option<P::Value>,
    pub max_value: Option<P::Value>,
    pub disabled: bool,
    pub read_only: bool,
    pub required: bool,
    pub read_only_segments: Vec<EditableSegmentKind>,
    pub is_date_unavailable: Option<Callback<CalendarDate, bool>>,
    pub validate: Option<Callback<ChronoRange<P::Value>, Option<Vec<String>>>>,
    pub on_invalid: Option<EventHandler<FieldInvalidEvent>>,
    pub error_message_id: Option<String>,
    pub dir: Option<Direction>,
    pub is_invalid_override: Option<bool>,
    pub child: Option<Callback<Vec<Attribute>, Element>>,
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
    let field_ctx = try_use_field_context();
    let effective_disabled = spec.disabled || field_ctx.is_some_and(|f| f.is_disabled());
    let effective_read_only = spec.read_only || field_ctx.is_some_and(|f| f.is_read_only());
    let effective_required = spec.required || field_ctx.is_some_and(|f| f.is_required());
    let effective_invalid_override = spec
        .is_invalid_override
        .or_else(|| field_ctx.map(|f| f.is_invalid()));

    let resolved_dir = use_direction_context(spec.dir);
    let stable_id = use_stable_id(spec.id.clone().map(Cow::Owned)).into_owned();
    let scope = use_hook(|| ScopeHandle::root(P::RANGE_SCOPE_PREFIX).child(stable_id.clone()));
    let rels = use_hook(|| RangeFieldRelationships::new(scope.clone()));

    let default_range = spec.default_value.unwrap_or_default();
    let initial_start = spec
        .start_value
        .unwrap_or_else(|| spec.value.map_or(default_range.start, |v| v.start));
    let initial_end = spec
        .end_value
        .unwrap_or_else(|| spec.value.map_or(default_range.end, |v| v.end));
    let initial_value = ChronoRange {
        start: initial_start,
        end: initial_end,
    };

    let value_ctl = use_controllable_signal(spec.value, initial_value, spec.on_value_change);
    let start_ctl =
        use_controllable_signal(spec.start_value, initial_start, spec.on_start_value_change);
    let end_ctl = use_controllable_signal(spec.end_value, initial_end, spec.on_end_value_change);
    let placeholder_ctl =
        use_controllable_signal(spec.placeholder, initial_placeholder, spec.on_placeholder_change);

    let mut start_value = start_ctl.signal();
    let mut end_value = end_ctl.signal();
    let placeholder = placeholder_ctl.signal();

    let def = RangeFieldDefinition::<P>::new()
        .with_locale(spec.locale.clone())
        .with_granularity(spec.granularity)
        .with_hour_cycle(spec.hour_cycle)
        .with_hide_time_zone(spec.hide_time_zone)
        .with_min_value(spec.min_value)
        .with_max_value(spec.max_value)
        .with_disabled(effective_disabled)
        .with_read_only(effective_read_only)
        .with_required(effective_required)
        .with_readonly_segments(spec.read_only_segments.clone())
        .with_error_message_id(spec.error_message_id.clone())
        .with_dir(resolved_dir);

    let state = use_signal(|| {
        let mut st = RangeFieldState::<P>::new(def.clone(), initial_value, initial_placeholder);
        if let Some(inv) = effective_invalid_override {
            st.set_external_invalid(inv);
        }
        st
    });

    let root_dom_id = use_signal(|| rels.root_id().to_string());
    let custom_label_id = use_signal(|| None::<String>);
    let last_invalid_reason = use_signal(|| None);
    let trailing_trigger_id = use_signal(|| None::<String>);

    let start_runtime = use_hook(|| Rc::new(RefCell::new(None)));
    let end_runtime = use_hook(|| Rc::new(RefCell::new(None)));

    // Freeze mount-time reset baseline (`use_hook`) from controlled-or-default.
    let default_value = use_hook(|| CopyValue::new(initial_value));

    let mut on_value_change = use_hook(|| CopyValue::new(spec.on_value_change));
    on_value_change.set(spec.on_value_change);

    let mut on_start_value_change = use_hook(|| CopyValue::new(spec.on_start_value_change));
    on_start_value_change.set(spec.on_start_value_change);

    let mut on_end_value_change = use_hook(|| CopyValue::new(spec.on_end_value_change));
    on_end_value_change.set(spec.on_end_value_change);

    let mut is_date_unavailable = use_hook(|| CopyValue::new(spec.is_date_unavailable));
    is_date_unavailable.set(spec.is_date_unavailable);

    let mut validate = use_hook(|| CopyValue::new(spec.validate));
    validate.set(spec.validate);

    let mut on_invalid = use_hook(|| CopyValue::new(spec.on_invalid));
    on_invalid.set(spec.on_invalid);

    let parent_focus_in = use_hook(|| CopyValue::new(None::<EventHandler<FocusEvent>>));
    let parent_focus_out = use_hook(|| CopyValue::new(None::<EventHandler<FocusEvent>>));

    let mut runtime = RangeFieldRuntime::<P> {
        state,
        rels,
        root_dom_id,
        start_value,
        end_value,
        placeholder,
        custom_label_id,
        last_invalid_reason,
        trailing_trigger_id,
        start_runtime,
        end_runtime,
        default_value,
        value_ctl,
        start_ctl,
        end_ctl,
        placeholder_ctl,
        on_value_change,
        on_start_value_change,
        on_end_value_change,
        is_date_unavailable,
        validate,
        on_invalid,
        parent_focus_in,
        parent_focus_out,
    };

    // Synchronous render-time reconciliation for external controlled props & invalid override.
    {
        let mut target_val = spec.value.unwrap_or_else(|| runtime.state.peek().value());
        if let Some(s) = spec.start_value {
            target_val.start = s;
        }
        if let Some(e) = spec.end_value {
            target_val.end = e;
        }
        if runtime.state.peek().value() != target_val {
            runtime.state.write().set_value(target_val);
        }
        if !start_ctl.is_controlled() && *start_value.peek() != target_val.start {
            sync_signal(&mut start_value, target_val.start);
        }
        if !end_ctl.is_controlled() && *end_value.peek() != target_val.end {
            sync_signal(&mut end_value, target_val.end);
        }
        if let Some(ph) = spec.placeholder
            && runtime.state.peek().placeholder() != ph
        {
            runtime.state.write().set_placeholder(ph);
        }
        let ext_inv_now = effective_invalid_override.unwrap_or(false);
        if runtime.state.peek().external_invalid() != ext_inv_now {
            runtime.state.write().set_external_invalid(ext_inv_now);
        }
    }

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
                sync_signal(&mut predicate_rev, next_rev);
            }
        }
    } else {
        pred_initialized.set(true);
    }

    // DEF-5C-09 & B5: Reconcile definition, predicates (`validate`, `is_date_unavailable`),
    // controlled props, and validation inside guarded effects for BOTH DateRangeField and TimeRangeField.
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

    let eff_inv_bool = effective_invalid_override.unwrap_or(false);
    use_effect(use_reactive(
        (
            &spec.value,
            &spec.start_value,
            &spec.end_value,
            &spec.placeholder,
            &eff_inv_bool,
        ),
        {
            let mut rt = runtime.clone();
            move |(ext_val, ext_start, ext_end, ext_ph, inv_override)| {
                rt.reconcile_external_state(ext_val, ext_start, ext_end, ext_ph, Some(inv_override));
            }
        },
    ));

    use_context_provider(|| RangeFieldContext::<P>(runtime.clone()));
    runtime
}

/// Renders the root group container for a pre-initialized [`RangeFieldRuntime<P>`].
pub(crate) fn render_range_field_root_with_runtime<P>(
    runtime: RangeFieldRuntime<P>,
    spec: RangeFieldRootSpec<P>,
) -> Element
where
    P: RangeFieldPolicy + SegmentFieldUiPolicy,
{
    let field_ctx = try_use_field_context();
    let extracted = extract_aria_invalid_and_describedby(spec.attributes);
    let raw_attrs = runtime.root_attrs(spec.id);
    runtime.set_root_dom_id(raw_attrs.id.clone());

    let field_describedby = field_ctx.and_then(|f| f.aria_describedby(None));
    let (effective_invalid, effective_describedby) = resolve_invalid_and_describedby(
        None,
        extracted.aria_invalid,
        Some(raw_attrs.data_invalid.is_some()),
        false,
        None,
        extracted.aria_describedby.as_deref(),
        field_describedby.as_deref(),
    );
    let attrs = raw_attrs
        .with_invalid(effective_invalid)
        .with_aria_describedby(effective_describedby);

    let (handle_focus_in, handle_focus_out) =
        use_composite_focus_out(vec![attrs.id.clone()], spec.on_focus_out);

    let parent_fi_rt = runtime.clone();
    let parent_fo_rt = runtime;

    let mut base = Vec::with_capacity(12);
    base.push(Attribute::new("id", attrs.id, None, false));
    base.push(Attribute::new("role", attrs.role, None, false));
    base.push(Attribute::new(
        "aria-labelledby",
        attrs.aria_labelledby,
        None,
        false,
    ));
    push_attr_opt(&mut base, "aria-describedby", attrs.aria_describedby);
    push_attr_opt(&mut base, "aria-invalid", attrs.aria_invalid);
    base.push(Attribute::new("dir", attrs.dir, None, false));
    push_attr_opt(&mut base, "data-invalid", attrs.data_invalid);
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);
    base.push(Attribute::new(P::RANGE_ROOT_DATA_ATTR, "", None, false));
    push_event_handler(
        &mut base,
        "onfocusin",
        Some(EventHandler::new(move |evt: FocusEvent| {
            handle_focus_in.call(evt.clone());
            parent_fi_rt.notify_parent_focus_in(evt);
        })),
    );
    push_event_handler(
        &mut base,
        "onfocusout",
        Some(EventHandler::new(move |evt: FocusEvent| {
            handle_focus_out.call(evt.clone());
            parent_fo_rt.notify_parent_focus_out(evt);
        })),
    );

    let mut user = collect_user_attrs(spec.class, spec.style, extracted.rest);
    push_event_handler(&mut user, "onmounted", spec.on_mounted);

    let merged = merge_props(base, user);

    if let Some(child) = spec.child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            {spec.children}
        }
    }
}

/// Monomorphized plain-fn root renderer shared by `DateRangeFieldRoot` and `TimeRangeFieldRoot`
/// (fixes `B5` by including `&spec.validate` in `use_reactive` for both policies).
pub(crate) fn render_range_field_root<P>(
    spec: RangeFieldRootSpec<P>,
    initial_placeholder: P::Value,
) -> Element
where
    P: RangeFieldPolicy + SegmentFieldUiPolicy,
{
    let runtime = use_range_field_runtime::<P>(&spec, initial_placeholder);
    render_range_field_root_with_runtime(runtime, spec)
}

/// Specification for [`render_range_field_input`].
pub(crate) struct RangeFieldInputSpec {
    pub kind: FieldRangeEndpoint,
    pub id: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub name: Option<String>,
    pub form: Option<String>,
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    pub render_children: Option<Callback<Vec<FieldSegmentItem>, Element>>,
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Monomorphized plain-fn range endpoint input renderer shared by `DateRangeFieldInput` and
/// `TimeRangeFieldInput` (zero nested `#[component]` forwarding `Option<Callback>` per `#5894`,
/// zero `*RangePartBridge` wrapper).
pub(crate) fn render_range_field_input<P>(spec: RangeFieldInputSpec) -> Element
where
    P: RangeFieldPolicy + SegmentFieldUiPolicy,
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

    let part_controlled = range_rt.value_ctl.is_controlled()
        || match spec.kind {
            FieldRangeEndpoint::Start => range_rt.start_ctl.is_controlled(),
            FieldRangeEndpoint::End => range_rt.end_ctl.is_controlled(),
        };
    let ph_controlled = range_rt.placeholder_ctl.is_controlled();
    let part_val_now = (part_value_sig)();
    let ph_now = (range_rt.placeholder)();

    let root_spec = SegmentFieldRootSpec::<P> {
        id: Some(part_scope_id),
        value: part_controlled.then_some(part_val_now),
        default_value: part_default_value,
        on_value_change: Some(EventHandler::new(move |next| {
            rt_for_change.update_part_value(kind, next);
        })),
        placeholder: ph_controlled.then_some(ph_now),
        on_placeholder_change: Some(EventHandler::new(move |ph| {
            rt_for_ph.update_placeholder(ph);
        })),
        on_focus_out: None,
        granularity: def.granularity,
        hour_cycle: def.hour_cycle,
        locale: def.locale,
        hide_time_zone: def.hide_time_zone,
        min_value: def.min_value,
        max_value: def.max_value,
        disabled: def.disabled,
        read_only: def.read_only,
        required: def.required,
        read_only_segments: def.readonly_segments,
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

    let mut field_rt = use_segment_field_runtime::<P>(&root_spec, initial_placeholder);
    // Sync parent-initiated changes (calendar selection, form reset) into uncontrolled endpoint
    // without self-echoing user segment typing (which would clear multi-digit typing buffers).
    let mut prev_part_val = use_hook(|| CopyValue::new(part_val_now));
    let mut prev_ph = use_hook(|| CopyValue::new(ph_now));
    if prev_part_val
        .try_read()
        .is_ok_and(|prev| *prev != part_val_now)
    {
        prev_part_val.set(part_val_now);
        if field_rt.state.peek().value() != part_val_now {
            field_rt.reconcile_external_state(Some(part_val_now), None, None);
        }
    }
    if prev_ph.try_read().is_ok_and(|prev| *prev != ph_now) {
        prev_ph.set(ph_now);
        if field_rt.state.peek().placeholder() != ph_now {
            field_rt.reconcile_external_state(None, Some(ph_now), None);
        }
    }
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
            child: spec.child,
            render_children: spec.render_children,
            on_focus_out: None,
            on_mounted: spec.on_mounted,
            attributes: spec.attributes,
            children: spec.children,
        },
    );

    render_segment_field_chrome(&field_rt, input_el)
}

/// Specification for [`render_range_field_label`].
pub(crate) struct RangeFieldLabelSpec {
    pub id: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    pub on_click: Option<EventHandler<MouseEvent>>,
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Monomorphized plain-fn label renderer shared by `DateRangeFieldLabel` and `TimeRangeFieldLabel`,
/// with equality-guarded `custom_label_id` drop cleanup preventing keyed swap clobber.
pub(crate) fn render_range_field_label<P>(spec: RangeFieldLabelSpec) -> Element
where
    P: RangeFieldPolicy + SegmentFieldUiPolicy,
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

    let attrs = runtime.label_attrs(spec.id);
    let base_on_click = {
        let rt = runtime;
        EventHandler::new(move |_: MouseEvent| {
            if rt.state.try_peek().is_ok_and(|st| !st.def().disabled) {
                rt.focus_first_segment();
            }
        })
    };

    let mut base = Vec::with_capacity(5);
    base.push(Attribute::new("id", attrs.id, None, false));
    push_attr_opt(&mut base, "data-invalid", attrs.data_invalid);
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);
    base.push(Attribute::new(P::RANGE_LABEL_DATA_ATTR, "", None, false));
    push_event_handler(&mut base, "onclick", Some(base_on_click));

    let mut user = collect_user_attrs(spec.class, spec.style, spec.attributes);
    push_event_handler(&mut user, "onclick", spec.on_click);
    push_event_handler(&mut user, "onmounted", spec.on_mounted);

    let merged = merge_props(base, user);

    if let Some(child) = spec.child {
        return child.call(merged);
    }

    rsx! {
        span {
            ..merged,
            {spec.children}
        }
    }
}

/// Props for [`DateRangeFieldRoot`].
#[derive(Clone, PartialEq, Props)]
pub struct DateRangeFieldRootProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub value: Option<DateValueRange>,
    #[props(default)]
    pub default_value: Option<DateValueRange>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<DateValueRange>>,
    #[props(default)]
    pub start_value: Option<Option<DateValue>>,
    #[props(default)]
    pub on_start_value_change: Option<EventHandler<Option<DateValue>>>,
    #[props(default)]
    pub end_value: Option<Option<DateValue>>,
    #[props(default)]
    pub on_end_value_change: Option<EventHandler<Option<DateValue>>>,
    #[props(default)]
    pub placeholder: Option<DateValue>,
    #[props(default)]
    pub default_placeholder: Option<DateValue>,
    #[props(default)]
    pub on_placeholder_change: Option<EventHandler<DateValue>>,
    #[props(default)]
    pub on_focus_out: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub granularity: Option<Granularity>,
    #[props(default)]
    pub hour_cycle: Option<HourCycle>,
    #[props(default, into)]
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
    pub read_only_segments: Vec<EditableSegmentKind>,
    #[props(default)]
    pub is_date_unavailable: Option<Callback<CalendarDate, bool>>,
    #[props(default)]
    pub validate: Option<Callback<DateValueRange, Option<Vec<String>>>>,
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
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Root group container (`<div role="group">`) for segmented date range fields.
#[component]
pub fn DateRangeFieldRoot(props: DateRangeFieldRootProps) -> Element {
    let today = use_hook(local_today);
    let default_range = props.default_value.unwrap_or_default();
    let initial_start = props
        .start_value
        .unwrap_or_else(|| props.value.map_or(default_range.start, |v| v.start));
    let initial_end = props
        .end_value
        .unwrap_or_else(|| props.value.map_or(default_range.end, |v| v.end));
    let initial_placeholder = props
        .placeholder
        .or(props.default_placeholder)
        .unwrap_or_else(|| {
            default_date_field_placeholder(
                initial_start.or(initial_end),
                props.granularity,
                props.min_value,
                props.max_value,
                today,
            )
        });

    render_range_field_root::<DateFieldPolicy>(
        RangeFieldRootSpec {
            id: props.id,
            class: props.class,
            style: props.style,
            value: props.value,
            default_value: props.default_value,
            on_value_change: props.on_value_change,
            start_value: props.start_value,
            on_start_value_change: props.on_start_value_change,
            end_value: props.end_value,
            on_end_value_change: props.on_end_value_change,
            placeholder: props.placeholder,
            on_placeholder_change: props.on_placeholder_change,
            on_focus_out: props.on_focus_out,
            granularity: props.granularity,
            hour_cycle: props.hour_cycle,
            locale: props.locale,
            hide_time_zone: props.hide_time_zone,
            min_value: props.min_value,
            max_value: props.max_value,
            disabled: props.disabled,
            read_only: props.read_only,
            required: props.required,
            read_only_segments: props.read_only_segments,
            is_date_unavailable: props.is_date_unavailable,
            validate: props.validate,
            on_invalid: props.on_invalid,
            error_message_id: props.error_message_id,
            dir: props.dir,
            is_invalid_override: None,
            child: props.child,
            on_mounted: props.on_mounted,
            attributes: props.attributes,
            children: props.children,
        },
        initial_placeholder,
    )
}

pub use DateRangeFieldRoot as DateRangeField;

/// Props for [`DateRangeFieldInput`].
#[derive(Clone, PartialEq, Props)]
pub struct DateRangeFieldInputProps {
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
    render_range_field_input::<DateFieldPolicy>(RangeFieldInputSpec {
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

/// Props for [`DateRangeFieldLabel`].
#[derive(Clone, PartialEq, Props)]
pub struct DateRangeFieldLabelProps {
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

/// Accessible label (`<span>`) for `DateRangeField`; clicking focuses the first interactive segment.
#[component]
pub fn DateRangeFieldLabel(props: DateRangeFieldLabelProps) -> Element {
    render_range_field_label::<DateFieldPolicy>(RangeFieldLabelSpec {
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
