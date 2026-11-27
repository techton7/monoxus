use dioxus::prelude::*;

use crate::field::try_use_field_context;
use crate::foundation::{
    browser::{
        focus::use_composite_focus_out,
        get_element_client_rect,
        pointer::{try_release_pointer_capture, try_set_pointer_capture},
    },
    compose::{
        extract_aria_invalid_and_describedby, join_styles, merge_global_attributes,
        resolve_invalid_and_describedby,
    },
    form::{HiddenFormInputs, use_form_control_sync_with_epoch},
    overlay::Rect2D,
    shared::{ScopeHandle, use_stable_id},
    state::sync_signal,
};

use crate::slider::{
    relationships::SliderRelationships,
    runtime::{SliderContext, SliderFieldState, find_closest_thumb, use_slider_runtime},
    state::{SliderDefinition, percent_to_value},
    types::{Direction, Orientation, SliderThumbAlignment, SliderThumbOrderPolicy, SliderValue},
};

/// Props for [`SliderRoot`].
#[derive(Clone, PartialEq, Props)]
pub struct SliderRootProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub value: Option<SliderValue>,
    #[props(default)]
    pub default_value: Option<SliderValue>,
    pub on_value_change: Option<EventHandler<SliderValue>>,
    pub on_value_commit: Option<EventHandler<SliderValue>>,
    #[props(default = 0.0)]
    pub min: f64,
    #[props(default = 100.0)]
    pub max: f64,
    #[props(default = 1.0)]
    pub step: f64,
    pub steps: Option<Vec<f64>>,
    #[props(default = 0)]
    pub min_steps_between_thumbs: usize,
    #[props(default)]
    pub order_policy: SliderThumbOrderPolicy,
    #[props(default)]
    pub thumb_alignment: SliderThumbAlignment,
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
    #[props(default)]
    pub aria_describedby: Option<String>,
    #[props(default)]
    pub on_focus_out: Option<EventHandler<FocusEvent>>,
    pub name: Option<String>,
    pub form: Option<String>,
    pub children: Element,
    pub class: Option<String>,
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

    let runtime = use_slider_runtime(
        def,
        rels,
        props.value.clone(),
        props.default_value.clone(),
        props.dir,
        props.on_value_change,
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

    let (handle_focus_in, handle_focus_out) =
        use_composite_focus_out(vec![dom_root_id.clone()], props.on_focus_out);

    let on_pointer_down = {
        let mut runtime = runtime.clone();
        let dom_root_id = dom_root_id.clone();
        move |evt: PointerEvent| {
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
        }
    };

    let on_pointer_move = {
        let mut runtime = runtime.clone();
        let dom_root_id = dom_root_id.clone();
        move |evt: PointerEvent| {
            if !*runtime.is_dragging.peek() {
                return;
            }
            let coords = evt.data().client_coordinates();
            let rect = get_element_client_rect(&runtime.resolved_track_id())
                .or_else(|| get_element_client_rect(&dom_root_id))
                .unwrap_or(Rect2D::new(0.0, 0.0, 100.0, 100.0));
            runtime.update_pointer_coord(coords.x, coords.y, &rect);
        }
    };

    let on_pointer_up = {
        let mut runtime = runtime.clone();
        move |evt: PointerEvent| finish_slider_pointer(&evt, &mut runtime)
    };

    let on_pointer_cancel = {
        let mut runtime = runtime.clone();
        move |evt: PointerEvent| finish_slider_pointer(&evt, &mut runtime)
    };

    let combined_style = join_styles([attrs.style(), props.style.as_deref().unwrap_or("")]);
    let merged = merge_global_attributes(
        props.class.as_deref(),
        combined_style.as_deref(),
        extracted.rest,
    );

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

    rsx! {
        div {
            id: "{dom_root_id}",
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref().unwrap_or_default(),
            dir: "{root_dir}",
            "data-orientation": "{attrs.data_orientation()}",
            "data-disabled": attrs.data_disabled(),
            "data-required": attrs.data_required(),
            "data-invalid": attrs.data_invalid(),
            "data-monoxus-slider-root": "",
            aria_disabled: attrs.aria_disabled(),
            onpointerdown: on_pointer_down,
            onpointermove: on_pointer_move,
            onpointerup: on_pointer_up,
            onpointercancel: on_pointer_cancel,
            onfocusin: handle_focus_in,
            onfocusout: handle_focus_out,
            ..merged.rest,
            {props.children}
        }
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
