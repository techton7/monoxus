use std::{borrow::Cow, cell::RefCell, collections::BTreeMap, rc::Rc};

use dioxus::prelude::*;
use ui_sdk::foundation::date_time::field_policy::{
    DateFieldPolicy, FieldRangeRole, FieldRelationships, FieldSegmentItem, SegmentFieldDefinition,
    SegmentFieldState,
};
use ui_sdk::kernel::ChangeEvent;

use crate::{
    agent::{
        ActionDescriptor, AgentHints, AgentSurface, ComponentSnapshot, DispatchExecResult,
        SnapshotValue, SurfaceMetaDefaults, use_agent_surface,
    },
    field::try_use_field_context,
    foundation::{
        browser::{focus::use_composite_focus_out, local_today},
        compose::{
            collect_user_attrs, extract_aria_invalid_and_describedby, merge_aria_describedby,
            merge_props, push_attr_opt, push_event_handler, resolve_invalid_and_describedby,
        },
        config::use_locale_context,
        date_time::{
            CalendarDate, CalendarLocale, DateValue, EditableSegmentKind, FieldInvalidEvent,
            FieldSegment, Granularity, HourCycle, SegmentKind,
        },
        form::{HiddenFormInput, HiddenInputKind, use_form_control_sync_with_epoch},
        shared::{Direction, ScopeHandle, use_direction_context, use_stable_id},
        state::{ControllableConfig, sync_signal, use_controllable_signal_config},
    },
    visually_hidden::VisuallyHidden,
};

use super::{
    runtime::{
        SegmentFieldContext, SegmentFieldRuntime, SegmentFieldUiPolicy, use_segment_field_context,
    },
    state::default_date_field_placeholder,
    types::DateFieldSegmentItem,
};

/// Plain Rust specification for [`use_segment_field_runtime`] and [`render_segment_field_root`]
/// (zero internal `#[component]` taking `Option<Callback|EventHandler>` per `Dioxus #5894`).
pub(crate) struct SegmentFieldRootSpec<P: SegmentFieldUiPolicy> {
    pub id: Option<String>,
    pub value: Option<Option<P::Value>>,
    pub default_value: Option<P::Value>,
    pub on_value_change: Option<EventHandler<Option<P::Value>>>,
    pub on_value_change_details: Option<EventHandler<ChangeEvent<Option<P::Value>>>>,
    pub placeholder: Option<P::Value>,
    pub has_explicit_default_placeholder: bool,
    pub on_placeholder_change: Option<EventHandler<P::Value>>,
    pub on_placeholder_change_details: Option<EventHandler<ChangeEvent<P::Value>>>,
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
    pub validate: Option<Callback<P::Value, Option<Vec<String>>>>,
    pub on_invalid: Option<EventHandler<FieldInvalidEvent>>,
    pub error_message_id: Option<String>,
    pub dir: Option<Direction>,
    pub is_invalid_override: Option<bool>,
    pub range_role: FieldRangeRole,
    pub label_id_override: Option<String>,
    pub on_prev_boundary: Option<EventHandler<()>>,
    pub on_next_boundary: Option<EventHandler<()>>,
    pub agent: Option<AgentHints>,
    pub children: Element,
}

/// Initializes a [`SegmentFieldRuntime<P>`], wires `B5`-compliant reactive prop/validate effects,
/// and provides `SegmentFieldContext<P>` for descendants.
pub(crate) fn use_segment_field_runtime<P: SegmentFieldUiPolicy>(
    spec: &SegmentFieldRootSpec<P>,
    initial_placeholder: P::Value,
) -> SegmentFieldRuntime<P> {
    let field_ctx = try_use_field_context();
    let effective_disabled = spec.disabled || field_ctx.is_some_and(|f| f.is_disabled());
    let effective_read_only = spec.read_only || field_ctx.is_some_and(|f| f.is_read_only());
    let effective_required = spec.required || field_ctx.is_some_and(|f| f.is_required());
    let effective_invalid_override = spec
        .is_invalid_override
        .or_else(|| field_ctx.map(|f| f.is_invalid()));

    let resolved_dir = use_direction_context(spec.dir);
    let stable_id = use_stable_id(spec.id.clone().map(Cow::Owned)).into_owned();
    let scope = use_hook(|| ScopeHandle::root(P::SCOPE_PREFIX).child(stable_id.clone()));
    let rels = use_hook(|| FieldRelationships::new(scope.clone()));

    let value_ctl = use_controllable_signal_config(
        ControllableConfig::new(spec.value, spec.default_value)
            .with_explicit_default(spec.default_value.is_some())
            .with_on_change(spec.on_value_change)
            .with_on_change_details(spec.on_value_change_details)
            .with_component(P::COMPONENT_NAME)
            .with_prop("value"),
    );
    let placeholder_ctl = use_controllable_signal_config(
        ControllableConfig::new(spec.placeholder, initial_placeholder)
            .with_explicit_default(spec.has_explicit_default_placeholder)
            .with_on_change(spec.on_placeholder_change)
            .with_on_change_details(spec.on_placeholder_change_details)
            .with_component(P::COMPONENT_NAME)
            .with_prop("placeholder"),
    );

    let initial_value = value_ctl.peek();

    let def = SegmentFieldDefinition::<P>::new()
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
        let mut st = SegmentFieldState::<P>::new(def.clone(), initial_value, initial_placeholder);
        if let Some(inv) = effective_invalid_override {
            st.set_external_invalid(inv);
        }
        st
    });

    let input_dom_id = use_signal(|| rels.input_id().to_string());
    let label_id_override = use_signal(|| spec.label_id_override.clone());
    let custom_label_id = use_signal(|| None::<String>);
    let trailing_trigger_id = use_signal(|| None::<String>);
    let announcement = use_signal(String::new);
    let last_invalid_reason = use_signal(|| None);

    let mut range_role = use_hook(|| CopyValue::new(spec.range_role));
    range_role.set(spec.range_role);

    // Freeze mount-time reset baseline (`use_hook`) from controlled-or-default.
    let default_value = use_hook(|| CopyValue::new(initial_value));

    let mut is_date_unavailable = use_hook(|| CopyValue::new(spec.is_date_unavailable));
    is_date_unavailable.set(spec.is_date_unavailable);

    let mut validate = use_hook(|| CopyValue::new(spec.validate));
    validate.set(spec.validate);

    let mut on_invalid = use_hook(|| CopyValue::new(spec.on_invalid));
    on_invalid.set(spec.on_invalid);

    let mut on_prev_boundary = use_hook(|| CopyValue::new(spec.on_prev_boundary));
    on_prev_boundary.set(spec.on_prev_boundary);

    let mut on_next_boundary = use_hook(|| CopyValue::new(spec.on_next_boundary));
    on_next_boundary.set(spec.on_next_boundary);

    let mut on_root_focus_out = use_hook(|| CopyValue::new(spec.on_focus_out));
    on_root_focus_out.set(spec.on_focus_out);

    let parent_focus_in = use_hook(|| CopyValue::new(None::<EventHandler<FocusEvent>>));
    let parent_focus_out = use_hook(|| CopyValue::new(None::<EventHandler<FocusEvent>>));

    let segment_dom_ids = use_hook(|| Rc::new(RefCell::new(BTreeMap::new())));

    let mut runtime = SegmentFieldRuntime::<P> {
        state,
        rels,
        range_role,
        input_dom_id,
        label_id_override,
        custom_label_id,
        trailing_trigger_id,
        segment_dom_ids,
        announcement,
        last_invalid_reason,
        value_ctl,
        placeholder_ctl,
        default_value,
        is_date_unavailable,
        validate,
        on_invalid,
        on_prev_boundary,
        on_next_boundary,
        on_root_focus_out,
        parent_focus_in,
        parent_focus_out,
    };

    // Synchronous render-time reconciliation for external controlled props & invalid override.
    // Preserves partial segment drafts because `set_value` only fires when `state.value() != val`.
    {
        if let Some(val) = spec.value
            && runtime.state.peek().value() != val
        {
            runtime.state.write().set_value(val);
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
    // controlled props, and validation inside guarded effects for BOTH DateField and TimeField.
    use_effect(use_reactive(
        (
            &def,
            &spec.label_id_override,
            &spec.is_date_unavailable,
            &spec.validate,
        ),
        {
            let mut rt = runtime.clone();
            move |(next_def, label_override, _next_unavailable, _next_validate)| {
                let _ = *predicate_rev.read();
                if *rt.label_id_override.peek() != label_override {
                    rt.label_id_override.set(label_override);
                }
                if *rt.state.peek().def() != next_def {
                    rt.state.write().sync_definition(next_def);
                }
                rt.sync_validation_and_notify();
            }
        },
    ));

    let eff_inv_bool = effective_invalid_override.unwrap_or(false);
    use_effect(use_reactive(
        (&spec.value, &spec.placeholder, &eff_inv_bool),
        {
            let mut rt = runtime.clone();
            move |(ext_val, ext_ph, inv_override)| {
                rt.reconcile_external_state(ext_val, ext_ph, Some(inv_override));
            }
        },
    ));

    use_context_provider(|| SegmentFieldContext::<P>(runtime.clone()));
    runtime
}

/// Renders the accessible description and live-region chrome around `body` for a [`SegmentFieldRuntime<P>`].
pub(crate) fn render_segment_field_chrome<P: SegmentFieldUiPolicy>(
    runtime: &SegmentFieldRuntime<P>,
    body: Element,
) -> Element {
    let description_text = runtime.state.read().description_text();
    let announcement_text = (runtime.announcement)();
    let desc_id = runtime.rels.description_id().to_string();
    let live_id = runtime.rels.live_region_id().to_string();

    rsx! {
        if !description_text.is_empty() {
            VisuallyHidden {
                id: Some(desc_id),
                "{description_text}"
            }
        }
        VisuallyHidden {
            id: Some(live_id),
            role: "status",
            "aria-live": "polite",
            "aria-atomic": "true",
            "{announcement_text}"
        }
        {body}
    }
}

/// Monomorphized plain-fn root renderer shared by `DateFieldRoot` and `TimeFieldRoot`.
pub(crate) fn render_segment_field_root<P: SegmentFieldUiPolicy>(
    spec: SegmentFieldRootSpec<P>,
    initial_placeholder: P::Value,
) -> Element {
    let runtime = use_segment_field_runtime::<P>(&spec, initial_placeholder);

    {
        fn build_snap<P: SegmentFieldUiPolicy>(rt: &SegmentFieldRuntime<P>) -> ComponentSnapshot {
            let (val, ph, ro) = rt
                .state
                .try_peek()
                .map(|st| (st.value(), Some(st.placeholder()), st.def().read_only))
                .unwrap_or((None, None, false));
            ComponentSnapshot::new()
                .with_field(
                    "value",
                    val.map(P::format_iso)
                        .map(SnapshotValue::Text)
                        .unwrap_or(SnapshotValue::Null),
                )
                .with_field(
                    "placeholder",
                    ph.map(P::format_iso)
                        .map(SnapshotValue::Text)
                        .unwrap_or(SnapshotValue::Null),
                )
                .with_field("read_only", SnapshotValue::Bool(ro))
        }

        let rt_for_surface = runtime.clone();
        let rt_for_dispatch = runtime.clone();
        let rt_for_reader = runtime.clone();
        let (eff_disabled, eff_read_only) = runtime
            .state
            .try_peek()
            .map(|st| (st.def().disabled, st.def().read_only))
            .unwrap_or((spec.disabled, spec.read_only));

        use_agent_surface(
            spec.agent.as_ref(),
            SurfaceMetaDefaults::default(),
            || {
                let mut actions = Vec::new();
                if !eff_read_only {
                    actions.push(ActionDescriptor::new(
                        "set",
                        format!("Set ISO {} value", P::AGENT_VALUE_LABEL),
                        true,
                    ));
                    actions.push(ActionDescriptor::new(
                        "clear",
                        format!("Clear {} value", P::AGENT_VALUE_LABEL),
                        false,
                    ));
                }
                AgentSurface::new(P::AGENT_KIND, P::AGENT_DEFAULT_LABEL)
                    .with_disabled(eff_disabled)
                    .with_snapshot(build_snap(&rt_for_surface))
                    .with_actions(actions)
            },
            || {
                let rt_handle = rt_for_dispatch;
                Rc::new(move |dispatch| {
                    let mut rt = rt_handle.clone();
                    match dispatch.action.as_str() {
                    "set" => match dispatch.args.get("value") {
                        Some(SnapshotValue::Null) => {
                            let canceled = rt.set_value_from_agent(None, dispatch.ctx);
                            DispatchExecResult::ok().with_canceled(canceled)
                        }
                        Some(SnapshotValue::Text(raw)) => {
                            let Some(parsed) = P::parse_iso(raw.trim()) else {
                                return DispatchExecResult::ok().with_host_error(format!(
                                    "invalid ISO {}: {raw}",
                                    P::AGENT_VALUE_LABEL
                                ));
                            };
                            let canceled = rt.set_value_from_agent(Some(parsed), dispatch.ctx);
                            DispatchExecResult::ok().with_canceled(canceled)
                        }
                        _ => DispatchExecResult::ok().with_host_error(format!(
                            "missing ISO {} argument `value`",
                            P::AGENT_VALUE_LABEL
                        )),
                    },
                    "clear" => {
                        let canceled = rt.set_value_from_agent(None, dispatch.ctx);
                        DispatchExecResult::ok().with_canceled(canceled)
                    }
                    other => DispatchExecResult::ok().with_host_error(format!(
                        "unsupported {} action: {other}",
                        P::AGENT_KIND
                    )),
                    }
                })
            },
            || {
                let rt = rt_for_reader;
                Rc::new(move || build_snap(&rt))
            },
        );
    }

    render_segment_field_chrome(&runtime, spec.children)
}

/// Specification for [`render_segment_field_input`] and [`render_segment_field_input_with_runtime`].
pub(crate) struct SegmentFieldInputSpec {
    pub id: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub name: Option<String>,
    pub form: Option<String>,
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    pub render_children: Option<Callback<Vec<FieldSegmentItem>, Element>>,
    pub on_focus_out: Option<EventHandler<FocusEvent>>,
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Monomorphized plain-fn input renderer taking an explicit [`SegmentFieldRuntime<P>`]
/// (used by both standalone `*FieldInput` and range endpoint `*RangeFieldInput`).
pub(crate) fn render_segment_field_input_with_runtime<P: SegmentFieldUiPolicy>(
    runtime: SegmentFieldRuntime<P>,
    spec: SegmentFieldInputSpec,
) -> Element {
    let field_ctx = try_use_field_context();
    let snap = runtime.snapshot();
    let extracted = extract_aria_invalid_and_describedby(spec.attributes);
    let raw_attrs = runtime.input_attrs(spec.id.clone());
    runtime.set_input_dom_id(raw_attrs.id.clone());

    let field_describedby = field_ctx.and_then(|f| f.aria_describedby(None));
    let merged_internal_and_field_desc = merge_aria_describedby([
        raw_attrs.aria_describedby.as_deref(),
        field_describedby.as_deref(),
    ]);
    let (effective_invalid, effective_describedby) = resolve_invalid_and_describedby(
        None,
        extracted.aria_invalid,
        Some(raw_attrs.data_invalid.is_some()),
        false,
        None,
        extracted.aria_describedby.as_deref(),
        merged_internal_and_field_desc.as_deref(),
    );
    let attrs = raw_attrs
        .with_invalid(effective_invalid)
        .with_aria_describedby(effective_describedby);

    let mut input_focus_out_slot = use_hook(|| CopyValue::new(spec.on_focus_out));
    input_focus_out_slot.set(spec.on_focus_out);
    let root_focus_out_slot = runtime.on_root_focus_out;
    let combined_focus_out = use_callback(move |evt: FocusEvent| {
        if let Some(cb) = input_focus_out_slot.try_peek().ok().and_then(|s| *s) {
            cb.call(evt.clone());
        }
        if let Some(cb) = root_focus_out_slot.try_peek().ok().and_then(|s| *s) {
            cb.call(evt);
        }
    });
    let (handle_focus_in, handle_focus_out) =
        use_composite_focus_out(vec![attrs.id.clone()], Some(combined_focus_out));

    let mut reset_rt = runtime.clone();
    let (show_hidden, reset_epoch) = use_form_control_sync_with_epoch(
        &attrs.id,
        spec.name.as_deref(),
        spec.form.as_deref(),
        move || {
            reset_rt.reset_to_default();
        },
    );

    let has_name = spec.name.as_deref().is_some_and(|n| !n.is_empty());
    let render_hidden =
        has_name || (show_hidden && spec.form.as_deref().is_some_and(|f| !f.is_empty()));

    let body = if let Some(ref render_children) = spec.render_children {
        render_children.call(snap.segments.clone())
    } else {
        spec.children
    };

    let inner = rsx! {
        {body}
        if render_hidden {
            HiddenFormInput {
                id: Some(runtime.rels.hidden_input_id().to_string()),
                kind: HiddenInputKind::Hidden,
                name: spec.name.clone(),
                value: snap.iso_value.clone(),
                disabled: snap.disabled,
                required: snap.required,
                read_only: snap.read_only,
                form: spec.form.clone(),
                focus_target_id: runtime.first_interactive_segment_id(),
                reset_epoch,
            }
        }
    };

    let parent_fi_rt = runtime.clone();
    let parent_fo_rt = runtime;

    let mut base = Vec::with_capacity(16);
    base.push(Attribute::new("id", attrs.id, None, false));
    base.push(Attribute::new("role", attrs.role, None, false));
    push_attr_opt(&mut base, "aria-labelledby", attrs.aria_labelledby);
    push_attr_opt(&mut base, "aria-describedby", attrs.aria_describedby);
    push_attr_opt(&mut base, "aria-invalid", attrs.aria_invalid);
    base.push(Attribute::new("aria-disabled", attrs.aria_disabled, None, false));
    push_attr_opt(&mut base, "aria-required", attrs.aria_required);
    base.push(Attribute::new("dir", attrs.dir, None, false));
    push_attr_opt(&mut base, "data-range-field-part", attrs.data_range_field_part);
    push_attr_opt(&mut base, "data-invalid", attrs.data_invalid);
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);
    push_attr_opt(&mut base, "data-required", attrs.data_required);
    base.push(Attribute::new(P::INPUT_DATA_ATTR, "", None, false));
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
            {inner}
        }
    }
}

/// Monomorphized plain-fn input renderer consuming [`SegmentFieldContext<P>`].
pub(crate) fn render_segment_field_input<P: SegmentFieldUiPolicy>(
    spec: SegmentFieldInputSpec,
) -> Element {
    let runtime = use_segment_field_context::<P>();
    render_segment_field_input_with_runtime(runtime, spec)
}

/// Specification for [`render_segment_field_segment`].
pub(crate) struct SegmentFieldSegmentSpec {
    pub part: SegmentKind,
    pub index: Option<usize>,
    pub id: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    pub on_focus: Option<EventHandler<FocusEvent>>,
    pub on_blur: Option<EventHandler<FocusEvent>>,
    pub on_paste: Option<EventHandler<ClipboardEvent>>,
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    pub attributes: Vec<Attribute>,
    pub children: Option<Element>,
}

/// Monomorphized plain-fn segment renderer shared by `DateFieldSegment` and `TimeFieldSegment`.
pub(crate) fn render_segment_field_segment<P: SegmentFieldUiPolicy>(
    spec: SegmentFieldSegmentSpec,
) -> Element {
    let runtime = use_segment_field_context::<P>();
    let segments = runtime.segments();

    let item = spec
        .index
        .and_then(|idx| segments.iter().find(|s| s.index == idx).cloned())
        .or_else(|| segments.iter().find(|s| s.kind() == spec.part).cloned())
        .unwrap_or_else(|| FieldSegmentItem {
            index: spec.index.unwrap_or(0),
            id: runtime
                .rels
                .segment_id(spec.index.unwrap_or(0), spec.part),
            segment: FieldSegment {
                kind: spec.part,
                value: String::new(),
                is_placeholder: false,
                min: None,
                max: None,
                now: None,
                value_text: None,
            },
        });

    let attrs = runtime.segment_attrs(&item, spec.id);
    runtime.register_segment_dom_id(item.index, attrs.id.clone());
    let seg_index = item.index;
    let seg_kind = item.kind();
    let is_interactive = item.is_interactive();

    let base_on_key_down = {
        let mut rt = runtime.clone();
        EventHandler::new(move |evt: KeyboardEvent| {
            if is_interactive {
                rt.handle_segment_keydown(seg_index, seg_kind, &evt);
            }
        })
    };

    let base_on_blur = {
        let mut rt = runtime.clone();
        EventHandler::new(move |_: FocusEvent| {
            if is_interactive {
                rt.handle_segment_blur(seg_kind);
            }
        })
    };

    let base_on_paste = {
        let mut rt = runtime;
        EventHandler::new(move |evt: ClipboardEvent| {
            if is_interactive
                && let Some(text) = evt.data().data_transfer().get_as_text()
                && rt.handle_segment_paste(seg_kind, &text)
            {
                evt.prevent_default();
            }
        })
    };

    let rendered_children = match spec.children {
        Some(el) => el,
        None => rsx! { "{item.segment.value}" },
    };

    let mut base = Vec::with_capacity(24);
    base.push(Attribute::new("id", attrs.id, None, false));
    push_attr_opt(&mut base, "role", attrs.role);
    push_attr_opt(&mut base, "tabindex", attrs.tabindex);
    push_attr_opt(&mut base, "inputmode", attrs.inputmode);
    push_attr_opt(&mut base, "aria-label", attrs.aria_label);
    push_attr_opt(&mut base, "aria-labelledby", attrs.aria_labelledby);
    push_attr_opt(&mut base, "aria-describedby", attrs.aria_describedby);
    push_attr_opt(&mut base, "aria-valuemin", attrs.aria_valuemin);
    push_attr_opt(&mut base, "aria-valuemax", attrs.aria_valuemax);
    push_attr_opt(&mut base, "aria-valuenow", attrs.aria_valuenow);
    push_attr_opt(&mut base, "aria-valuetext", attrs.aria_valuetext);
    push_attr_opt(&mut base, "aria-invalid", attrs.aria_invalid);
    base.push(Attribute::new("aria-disabled", attrs.aria_disabled, None, false));
    push_attr_opt(&mut base, "aria-readonly", attrs.aria_readonly);
    push_attr_opt(&mut base, "aria-hidden", attrs.aria_hidden);
    base.push(Attribute::new("data-segment", attrs.data_segment, None, false));
    push_attr_opt(&mut base, "data-placeholder", attrs.data_placeholder);
    push_attr_opt(&mut base, "data-invalid", attrs.data_invalid);
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);
    push_attr_opt(&mut base, "data-readonly", attrs.data_readonly);
    base.push(Attribute::new(P::SEGMENT_DATA_ATTR, "", None, false));
    push_event_handler(&mut base, "onkeydown", Some(base_on_key_down));
    push_event_handler(&mut base, "onblur", Some(base_on_blur));
    push_event_handler(&mut base, "onpaste", Some(base_on_paste));

    let mut user = collect_user_attrs(spec.class, spec.style, spec.attributes);
    push_event_handler(&mut user, "onkeydown", spec.on_key_down);
    push_event_handler(&mut user, "onfocus", spec.on_focus);
    push_event_handler(&mut user, "onblur", spec.on_blur);
    push_event_handler(&mut user, "onpaste", spec.on_paste);
    push_event_handler(&mut user, "onmounted", spec.on_mounted);

    let merged = merge_props(base, user);

    if let Some(child) = spec.child {
        return child.call(merged);
    }

    rsx! {
        span {
            ..merged,
            {rendered_children}
        }
    }
}

/// Specification for [`render_segment_field_label`].
pub(crate) struct SegmentFieldLabelSpec {
    pub id: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    pub on_click: Option<EventHandler<MouseEvent>>,
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Monomorphized plain-fn label renderer shared by `DateFieldLabel` and `TimeFieldLabel`,
/// with equality-guarded `custom_label_id` drop cleanup preventing keyed swap clobber.
pub(crate) fn render_segment_field_label<P: SegmentFieldUiPolicy>(
    spec: SegmentFieldLabelSpec,
) -> Element {
    let mut runtime = use_segment_field_context::<P>();
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
    base.push(Attribute::new(P::LABEL_DATA_ATTR, "", None, false));
    push_event_handler(&mut base, "onclick", Some(base_on_click));

    let mut user = collect_user_attrs(spec.class, spec.style, spec.attributes);
    push_event_handler(&mut user, "onclick", spec.on_click);
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

/// Props for [`DateFieldRoot`].
#[derive(Clone, PartialEq, Props)]
pub struct DateFieldRootProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default)]
    pub value: Option<Option<DateValue>>,
    #[props(default)]
    pub default_value: Option<DateValue>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<Option<DateValue>>>,
    #[props(default)]
    pub on_value_change_details: Option<EventHandler<ChangeEvent<Option<DateValue>>>>,
    #[props(default)]
    pub placeholder: Option<DateValue>,
    #[props(default)]
    pub default_placeholder: Option<DateValue>,
    #[props(default)]
    pub on_placeholder_change: Option<EventHandler<DateValue>>,
    #[props(default)]
    pub on_placeholder_change_details: Option<EventHandler<ChangeEvent<DateValue>>>,
    #[props(default)]
    pub on_focus_out: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub granularity: Option<Granularity>,
    #[props(default)]
    pub hour_cycle: Option<HourCycle>,
    #[props(default)]
    pub locale: Option<CalendarLocale>,
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
    pub validate: Option<Callback<DateValue, Option<Vec<String>>>>,
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

/// Root provider for segmented date / date-time spinbutton fields.
#[component]
pub fn DateFieldRoot(props: DateFieldRootProps) -> Element {
    let resolved_locale = use_locale_context(props.locale);
    let today = use_hook(local_today);
    let initial_value = props.value.unwrap_or(props.default_value);
    let has_explicit_default_placeholder = props.default_placeholder.is_some();
    let initial_placeholder = props
        .placeholder
        .or(props.default_placeholder)
        .unwrap_or_else(|| {
            default_date_field_placeholder(
                initial_value,
                props.granularity,
                props.min_value,
                props.max_value,
                today,
            )
        });

    render_segment_field_root::<DateFieldPolicy>(
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

pub use DateFieldRoot as DateField;

/// Props for [`DateFieldInput`].
#[derive(Clone, PartialEq, Props)]
pub struct DateFieldInputProps {
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
    pub on_focus_out: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Container for `DateField` segments and automatic hidden `<input>` form synchronization.
#[component]
pub fn DateFieldInput(props: DateFieldInputProps) -> Element {
    render_segment_field_input::<DateFieldPolicy>(SegmentFieldInputSpec {
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

/// Props for [`DateFieldSegment`].
#[derive(Clone, PartialEq, Props)]
pub struct DateFieldSegmentProps {
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

/// Individual spinbutton or literal segment inside a `DateFieldInput`.
#[component]
pub fn DateFieldSegment(props: DateFieldSegmentProps) -> Element {
    render_segment_field_segment::<DateFieldPolicy>(SegmentFieldSegmentSpec {
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

/// Props for [`DateFieldLabel`].
#[derive(Clone, PartialEq, Props)]
pub struct DateFieldLabelProps {
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

/// Accessible label for `DateField`; clicking focuses the first interactive segment.
#[component]
pub fn DateFieldLabel(props: DateFieldLabelProps) -> Element {
    render_segment_field_label::<DateFieldPolicy>(SegmentFieldLabelSpec {
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
