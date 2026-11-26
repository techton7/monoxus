use dioxus::prelude::*;

use crate::foundation::{
    browser::{
        get_element_client_rect,
        pointer::{try_release_pointer_capture, try_set_pointer_capture},
    },
    compose::{Slottable, join_styles, merge_global_attributes, project_as_child},
    form::{HiddenFormInputs, use_form_control_sync_with_epoch},
    overlay::Rect2D,
    shared::{ScopeHandle, use_stable_id},
};

use crate::slider::{
    attrs::SliderRootAttributes,
    relationships::SliderRelationships,
    runtime::{SliderContext, find_closest_thumb, use_slider_runtime},
    state::{SliderDefinition, percent_to_value},
    types::{Direction, Orientation, SliderThumbAlignment, SliderThumbOrderPolicy, SliderValue},
};

/// Projects child targets through [`AsChildSlot`] for `SliderRoot`.
pub fn project_slider_root_as_child<T, C>(
    attrs: &SliderRootAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, SliderRootAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

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
        .with_disabled(props.disabled);

    let runtime = use_slider_runtime(
        def,
        rels,
        props.value.clone(),
        props.default_value.clone(),
        props.dir,
        props.on_value_change,
        props.on_value_commit,
    );

    let attrs = runtime.root_attrs();
    let dom_root_id = props.id.clone().unwrap_or_else(|| attrs.id().to_string());

    let initial_values_for_reset = if let Some(ref v) = props.value {
        v.to_vec()
    } else if let Some(ref dv) = props.default_value {
        dv.to_vec()
    } else {
        vec![props.min]
    };

    let mut reset_runtime = runtime.clone();
    let on_reset_fn = move || {
        reset_runtime.reset_values(initial_values_for_reset.clone());
    };

    let (should_render_inputs, reset_epoch) = use_form_control_sync_with_epoch(
        &dom_root_id,
        props.name.as_deref(),
        props.form.as_deref(),
        on_reset_fn,
    );

    use_context_provider(|| SliderContext(runtime.clone()));

    let on_pointer_down = {
        let mut runtime = runtime.clone();
        let dom_root_id = dom_root_id.clone();
        move |evt: PointerEvent| {
            if runtime.def.disabled {
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
            let raw_val = percent_to_value(fraction * 100.0, runtime.def.min, runtime.def.max);
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
        props.attributes,
    );

    let current_vals = runtime.current_values();
    let string_values: Vec<String> = current_vals.iter().map(|v| v.to_string()).collect();
    let input_name = props.name.clone().map(|n| {
        if !runtime.is_single && !n.ends_with("[]") {
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
            dir: "{runtime.def.direction.as_str()}",
            "data-orientation": "{attrs.data_orientation()}",
            "data-disabled": attrs.data_disabled(),
            "data-monoxus-slider-root": "",
            aria_disabled: attrs.aria_disabled(),
            onpointerdown: on_pointer_down,
            onpointermove: on_pointer_move,
            onpointerup: on_pointer_up,
            onpointercancel: on_pointer_cancel,
            ..merged.rest,
            {props.children}
        }
        if should_render_inputs {
            HiddenFormInputs {
                name: input_name,
                values: string_values,
                disabled: props.disabled,
                form: props.form.clone(),
                reset_epoch: reset_epoch,
            }
        }
    }
}
