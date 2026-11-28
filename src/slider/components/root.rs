use std::rc::Rc;

use dioxus::prelude::*;
use ui_sdk::kernel::ChangeEvent;

use crate::agent::{
    ActionDescriptor, AgentHints, AgentSurface, ComponentSnapshot, DispatchExecResult,
    SnapshotValue, SurfaceMetaDefaults, use_agent_surface,
};
use crate::field::try_use_field_context;
use crate::foundation::{
    browser::{
        focus::use_composite_focus_out,
        get_element_client_rect,
        pointer::{try_release_pointer_capture, try_set_pointer_capture},
    },
    compose::{
        MergePropsConfig, collect_user_attrs, extract_aria_invalid_and_describedby,
        merge_props_with_config, push_attr_opt, push_event_handler,
        resolve_invalid_and_describedby,
    },
    form::{HiddenFormInputs, use_form_control_sync_with_epoch},
    overlay::Rect2D,
    shared::{ScopeHandle, use_stable_id},
    state::sync_signal,
};

use crate::slider::{
    relationships::SliderRelationships,
    runtime::{
        SliderContext, SliderFieldState, find_closest_thumb, use_slider_runtime_with_details,
    },
    state::{SliderDefinition, percent_to_value},
    types::{Direction, Orientation, SliderThumbAlignment, SliderThumbOrderPolicy, SliderValue},
};

/// Props for [`SliderRoot`].
#[derive(Clone, PartialEq, Props)]
pub struct SliderRootProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default)]
    pub value: Option<SliderValue>,
    #[props(default)]
    pub default_value: Option<SliderValue>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<SliderValue>>,
    #[props(default)]
    pub on_value_change_details: Option<EventHandler<ChangeEvent<SliderValue>>>,
    #[props(default)]
    pub on_value_commit: Option<EventHandler<SliderValue>>,
    #[props(default)]
    pub agent: Option<AgentHints>,
    #[props(default = 0.0)]
    pub min: f64,
    #[props(default = 100.0)]
    pub max: f64,
    #[props(default = 1.0)]
    pub step: f64,
    #[props(default)]
    pub steps: Option<Vec<f64>>,
    #[props(default = 0)]
    pub min_steps_between_thumbs: usize,
    #[props(default)]
    pub order_policy: SliderThumbOrderPolicy,
    #[props(default)]
    pub thumb_alignment: SliderThumbAlignment,
    #[props(default)]
    pub track_padding: Option<f64>,
    #[props(default)]
    pub orientation: Orientation,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default = false)]
    pub inverted: bool,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub required: bool,
    #[props(default)]
    pub invalid: Option<bool>,
    #[props(default, into)]
    pub aria_describedby: Option<String>,
    #[props(default)]
    pub on_focus_out: Option<EventHandler<FocusEvent>>,
    #[props(default, into)]
    pub name: Option<String>,
    #[props(default, into)]
    pub form: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    pub children: Element,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

fn finish_slider_pointer(evt: &PointerEvent, runtime: &mut crate::slider::runtime::SliderRuntime) {
    try_release_pointer_capture(Some(evt), None, None);
    runtime.end_drag();
}

/// Headless slider container coordinating value collection, thumb positioning, and form sync.
#[component]
pub fn SliderRoot(props: SliderRootProps) -> Element {
    let field_ctx = try_use_field_context();
    let effective_name = props
        .name
        .clone()
        .or_else(|| field_ctx.and_then(|f| f.name()));
    let effective_disabled = props.disabled || field_ctx.is_some_and(|f| f.is_disabled());
    let effective_required = props.required || field_ctx.is_some_and(|f| f.is_required());

    let extracted = extract_aria_invalid_and_describedby(props.attributes);
    let field_describedby = field_ctx.and_then(|f| f.aria_describedby(None));
    let root_invalid_override = props.invalid.or(extracted.aria_invalid);
    let (effective_invalid, effective_describedby) = resolve_invalid_and_describedby(
        props.invalid,
        extracted.aria_invalid,
        field_ctx.map(|f| f.is_invalid()),
        false,
        props.aria_describedby.as_deref(),
        extracted.aria_describedby.as_deref(),
        field_describedby.as_deref(),
    );

    let root_id = use_stable_id(props.id.clone().map(std::borrow::Cow::Owned)).into_owned();
    let scope = ScopeHandle::root("slider").child(root_id.clone());
    let rels = SliderRelationships::new(scope.clone());

    let def = SliderDefinition::new(scope)
        .with_min(props.min)
        .with_max(props.max)
        .with_step(props.step)
        .with_steps(props.steps.clone())
        .with_min_steps_between_thumbs(props.min_steps_between_thumbs)
        .with_order_policy(props.order_policy)
        .with_thumb_alignment(props.thumb_alignment)
        .with_track_padding(props.track_padding)
        .with_orientation(props.orientation)
        .with_inverted(props.inverted)
        .with_disabled(effective_disabled);

    let runtime = use_slider_runtime_with_details(
        def,
        rels,
        props.value.clone(),
        props.default_value.clone(),
        props.dir,
        props.on_value_change,
        props.on_value_change_details,
        props.on_value_commit,
    );

    let mut req_sig = use_signal(|| effective_required);
    sync_signal(&mut req_sig, effective_required);
    let mut inv_sig = use_signal(|| effective_invalid);
    sync_signal(&mut inv_sig, effective_invalid);
    let mut inv_override_sig = use_signal(|| root_invalid_override);
    sync_signal(&mut inv_override_sig, root_invalid_override);
    let mut desc_sig = use_signal(|| effective_describedby.clone());
    sync_signal(&mut desc_sig, effective_describedby.clone());

    let attrs = runtime
        .root_attrs()
        .with_required(effective_required)
        .with_invalid(effective_invalid)
        .with_aria_describedby(effective_describedby);
    let dom_root_id = props.id.clone().unwrap_or_else(|| attrs.id().to_string());

    let initial_values_for_reset = use_hook(|| {
        if let Some(ref v) = props.value {
            v.to_vec()
        } else if let Some(ref dv) = props.default_value {
            dv.to_vec()
        } else {
            vec![props.min]
        }
    });

    let mut reset_runtime = runtime.clone();
    let on_reset_fn = move || {
        reset_runtime.reset_values(initial_values_for_reset.clone());
    };

    let (should_render_inputs, reset_epoch) = use_form_control_sync_with_epoch(
        &dom_root_id,
        effective_name.as_deref(),
        props.form.as_deref(),
        on_reset_fn,
    );

    use_context_provider(|| SliderContext(runtime.clone()));
    use_context_provider(|| SliderFieldState {
        required: req_sig,
        invalid: inv_sig,
        invalid_override: inv_override_sig,
        aria_describedby: desc_sig,
    });

    let agent_label = effective_name
        .clone()
        .unwrap_or_else(|| dom_root_id.clone());
    let agent_rt_surface = runtime.clone();
    let agent_rt_dispatch = runtime.clone();
    let agent_rt_reader = runtime.clone();
    let build_slider_snap = |rt: &crate::slider::runtime::SliderRuntime| {
        let def = rt.def.peek().clone();
        let vals = rt.values.peek().clone();
        let val_field = if *rt.is_single.peek() {
            SnapshotValue::Float(vals.first().copied().unwrap_or(def.min))
        } else {
            SnapshotValue::List(vals.into_iter().map(SnapshotValue::Float).collect())
        };
        ComponentSnapshot::new()
            .with_field("value", val_field)
            .with_field("min", SnapshotValue::Float(def.min))
            .with_field("max", SnapshotValue::Float(def.max))
            .with_field("step", SnapshotValue::Float(def.step))
    };
    use_agent_surface(
        props.agent.as_ref(),
        SurfaceMetaDefaults::new(),
        || {
            AgentSurface::new("slider", agent_label)
                .with_disabled(effective_disabled)
                .with_snapshot(build_slider_snap(&agent_rt_surface))
                .with_actions(vec![ActionDescriptor::new(
                    "set",
                    "Set slider value (clamps/snaps and commits)",
                    true,
                )])
        },
        || {
            Rc::new(move |dispatch| {
                let mut rt = agent_rt_dispatch.clone();
                match dispatch.action.as_str() {
                    "set" => {
                        let arg = dispatch
                            .args
                            .get("value")
                            .or_else(|| dispatch.args.get("values"));
                        let requested = match arg {
                            Some(SnapshotValue::Float(v)) if v.is_finite() => vec![*v],
                            Some(SnapshotValue::Int(n)) => vec![*n as f64],
                            Some(SnapshotValue::List(items)) if !items.is_empty() => {
                                let mut parsed = Vec::with_capacity(items.len());
                                for item in items {
                                    match item {
                                        SnapshotValue::Float(v) if v.is_finite() => {
                                            parsed.push(*v)
                                        }
                                        SnapshotValue::Int(n) => parsed.push(*n as f64),
                                        _ => {
                                            return DispatchExecResult::ok().with_host_error(
                                                "all elements of 'value' must be finite numbers",
                                            );
                                        }
                                    }
                                }
                                parsed
                            }
                            _ => {
                                return DispatchExecResult::ok().with_host_error(
                                    "missing or invalid numeric argument 'value'",
                                );
                            }
                        };
                        let (outcome, snapped) = rt.set_from_agent(requested, dispatch.ctx);
                        let mut res =
                            DispatchExecResult::ok().with_canceled(outcome.is_canceled());
                        if !outcome.is_canceled() {
                            let expected = if *rt.is_single.peek() {
                                SnapshotValue::Float(
                                    snapped.first().copied().unwrap_or(rt.def.peek().min),
                                )
                            } else {
                                SnapshotValue::List(
                                    snapped.into_iter().map(SnapshotValue::Float).collect(),
                                )
                            };
                            res = res.with_expected("value", expected);
                        }
                        res
                    }
                    other => DispatchExecResult::ok()
                        .with_host_error(format!("unknown slider action '{other}'")),
                }
            })
        },
        || Rc::new(move || build_slider_snap(&agent_rt_reader)),
    );

    let (handle_focus_in, handle_focus_out) =
        use_composite_focus_out(vec![dom_root_id.clone()], props.on_focus_out);

    let current_vals = runtime.current_values();
    let string_values: Vec<String> = current_vals.iter().map(|v| v.to_string()).collect();
    let is_single = *runtime.is_single.read();
    let root_dir = runtime.def.read().direction.as_str();
    let input_name = effective_name.map(|n| {
        if !is_single && !n.ends_with("[]") {
            format!("{n}[]")
        } else {
            n
        }
    });

    let base_style = attrs.style();
    let mut base = Vec::with_capacity(16);
    base.push(Attribute::new("id", dom_root_id.clone(), None, false));
    if !base_style.is_empty() {
        base.push(Attribute::new("style", base_style, None, false));
    }
    base.push(Attribute::new("dir", root_dir, None, false));
    base.push(Attribute::new(
        "data-orientation",
        attrs.data_orientation(),
        None,
        false,
    ));
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled());
    push_attr_opt(&mut base, "data-required", attrs.data_required());
    push_attr_opt(&mut base, "data-invalid", attrs.data_invalid());
    base.push(Attribute::new("data-monoxus-slider-root", "", None, false));
    push_attr_opt(&mut base, "aria-disabled", attrs.aria_disabled());

    {
        let mut runtime = runtime.clone();
        let dom_root_id = dom_root_id.clone();
        push_event_handler(
            &mut base,
            "onpointerdown",
            Some(EventHandler::new(move |evt: PointerEvent| {
                let def = runtime.def.peek().clone();
                if def.disabled {
                    return;
                }
                if evt
                    .trigger_button()
                    .is_some_and(|b| b != dioxus::html::input_data::MouseButton::Primary)
                {
                    return;
                }
                evt.prevent_default();
                let _ = try_set_pointer_capture(&evt.data(), None);
                if *runtime.thumb_drag_claimed.peek() {
                    runtime.thumb_drag_claimed.set(false);
                    return;
                }
                let coords = evt.data().client_coordinates();
                let rect = get_element_client_rect(&runtime.resolved_track_id())
                    .or_else(|| get_element_client_rect(&dom_root_id))
                    .unwrap_or(Rect2D::new(0.0, 0.0, 100.0, 100.0));

                let current_vals = runtime.values.peek().clone();
                let fraction = runtime.compute_pointer_fraction(coords.x, coords.y, &rect);
                let raw_val = percent_to_value(fraction * 100.0, def.min, def.max);
                let closest_idx = find_closest_thumb(&current_vals, raw_val);
                runtime.start_drag(closest_idx);
                runtime.update_pointer_coord(coords.x, coords.y, &rect);
            })),
        );
    }
    {
        let mut runtime = runtime.clone();
        let dom_root_id = dom_root_id.clone();
        push_event_handler(
            &mut base,
            "onpointermove",
            Some(EventHandler::new(move |evt: PointerEvent| {
                if !*runtime.is_dragging.peek() {
                    return;
                }
                let coords = evt.data().client_coordinates();
                let rect = get_element_client_rect(&runtime.resolved_track_id())
                    .or_else(|| get_element_client_rect(&dom_root_id))
                    .unwrap_or(Rect2D::new(0.0, 0.0, 100.0, 100.0));
                runtime.update_pointer_coord(coords.x, coords.y, &rect);
            })),
        );
    }
    {
        let mut runtime = runtime.clone();
        push_event_handler(
            &mut base,
            "onpointerup",
            Some(EventHandler::new(move |evt: PointerEvent| {
                finish_slider_pointer(&evt, &mut runtime);
            })),
        );
    }
    {
        let mut runtime = runtime.clone();
        push_event_handler(
            &mut base,
            "onpointercancel",
            Some(EventHandler::new(move |evt: PointerEvent| {
                finish_slider_pointer(&evt, &mut runtime);
            })),
        );
    }
    push_event_handler(&mut base, "onfocusin", Some(handle_focus_in));
    push_event_handler(&mut base, "onfocusout", Some(handle_focus_out));

    let user = collect_user_attrs(props.class, props.style, extracted.rest);
    let merged = merge_props_with_config(
        base,
        user,
        MergePropsConfig {
            invariant_attrs: &[],
            bookkeeping_events: &[
                "onpointermove",
                "onpointerup",
                "onpointercancel",
                "onfocusin",
                "onfocusout",
            ],
        },
    );

    let root_el = if let Some(child) = props.child {
        child.call(merged)
    } else {
        rsx! {
            div {
                ..merged,
                {props.children}
            }
        }
    };

    rsx! {
        {root_el}
        if should_render_inputs {
            HiddenFormInputs {
                name: input_name,
                values: string_values,
                disabled: effective_disabled,
                required: effective_required,
                form: props.form.clone(),
                focus_target_id: Some(runtime.resolved_thumb_id(0)),
                reset_epoch: reset_epoch,
            }
        }
    }
}
