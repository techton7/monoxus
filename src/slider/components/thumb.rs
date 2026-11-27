use dioxus::prelude::*;

use crate::field::{try_use_field_context, try_use_field_control_context};
use crate::foundation::{
    browser::{ResizeObserverBox, use_element_size_with_mount},
    compose::{
        MergePropsConfig, collect_user_attrs, extract_aria_invalid_and_describedby,
        merge_props_with_config, push_attr_opt, push_event_handler,
        resolve_invalid_and_describedby,
    },
    roving_focus::key_to_roving_str,
};

use crate::slider::{
    runtime::{try_use_slider_field_state, use_slider_context},
    types::Orientation,
};

/// Props for [`SliderThumb`].
#[derive(Clone, PartialEq, Props)]
pub struct SliderThumbProps {
    #[props(default = 0)]
    pub index: usize,
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub children: Option<Element>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default, into)]
    pub aria_label: Option<String>,
    #[props(default, into)]
    pub aria_labelledby: Option<String>,
    #[props(default, into)]
    pub aria_describedby: Option<String>,
    #[props(default, into)]
    pub aria_valuetext: Option<String>,
    #[props(default)]
    pub invalid: Option<bool>,
    #[props(default)]
    pub on_blur: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub on_focus: Option<EventHandler<FocusEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// Headless interactive thumb handle supporting keyboard navigation, pointer dragging, and ARIA attributes.
#[component]
pub fn SliderThumb(props: SliderThumbProps) -> Element {
    let ctx = use_slider_context();
    let field_ctx = try_use_field_context();
    let control_ctx = try_use_field_control_context();
    let slider_field_state = try_use_slider_field_state();

    let extracted = extract_aria_invalid_and_describedby(props.attributes);

    let root_invalid_override = slider_field_state.and_then(|s| *s.invalid_override.read());
    let root_or_field_describedby = slider_field_state
        .and_then(|s| s.aria_describedby.read().clone())
        .or_else(|| field_ctx.and_then(|f| f.aria_describedby(None)));
    let fallback_invalid = root_invalid_override.or_else(|| field_ctx.map(|f| f.is_invalid()));
    let (effective_invalid, effective_describedby) = resolve_invalid_and_describedby(
        props.invalid,
        extracted.aria_invalid,
        fallback_invalid,
        false,
        props.aria_describedby.as_deref(),
        extracted.aria_describedby.as_deref(),
        root_or_field_describedby.as_deref(),
    );
    let effective_required = slider_field_state.is_some_and(|s| *s.required.read())
        || field_ctx.is_some_and(|f| f.is_required());

    let field_labelledby = if props.index == 0 {
        if let Some(ctrl_ctx) = control_ctx {
            ctrl_ctx.mounted_label_id().or_else(|| {
                field_ctx
                    .filter(|f| !f.is_fieldset())
                    .and_then(|f| f.aria_labelledby())
            })
        } else {
            field_ctx.and_then(|f| f.aria_labelledby())
        }
    } else {
        None
    };
    let effective_labelledby = props.aria_labelledby.clone().or_else(|| {
        if props.aria_label.is_none() && *ctx.0.is_single.read() {
            field_labelledby
        } else {
            None
        }
    });

    let attrs = ctx
        .0
        .thumb_attrs(
            props.index,
            props.aria_label.clone(),
            effective_labelledby,
            props.aria_valuetext.clone(),
        )
        .with_required(effective_required)
        .with_invalid(effective_invalid)
        .with_aria_describedby(effective_describedby);

    let explicit_id = props.id.clone().filter(|id| !id.trim().is_empty());
    let thumb_id = if props.index == 0 {
        if let Some(ref explicit) = explicit_id {
            if let Some(ctrl_ctx) = control_ctx {
                if !ctrl_ctx.is_explicit_id_locked() {
                    ctrl_ctx.sync_control_id(explicit.clone());
                }
            } else if let Some(f_ctx) = field_ctx
                && !f_ctx.is_fieldset()
            {
                f_ctx.set_primary_control_id(Some(explicit.clone()));
            }
            explicit.clone()
        } else if let Some(ctrl_ctx) = control_ctx {
            ctrl_ctx.control_id()
        } else if let Some(f_ctx) = field_ctx
            && !f_ctx.is_fieldset()
        {
            f_ctx.resolved_control_id()
        } else {
            attrs.id().to_string()
        }
    } else {
        explicit_id.unwrap_or_else(|| attrs.id().to_string())
    };

    use_effect(use_reactive((&props.index, &thumb_id), {
        let mut runtime = ctx.0.clone();
        move |(idx, tid)| {
            runtime.register_thumb_id(idx, tid);
        }
    }));

    let mut mounted_rev = use_signal(|| 0u64);
    let thumb_size_sig =
        use_element_size_with_mount(&thumb_id, ResizeObserverBox::BorderBox, mounted_rev);
    use_effect({
        let mut runtime = ctx.0.clone();
        let index = props.index;
        move || {
            let orientation = runtime.def.read().orientation;
            if let Some(size) = *thumb_size_sig.read() {
                let s = match orientation {
                    Orientation::Vertical => size.height,
                    _ => size.width,
                };
                let mut sizes = runtime.thumb_sizes.peek().clone();
                if sizes.len() <= index {
                    sizes.resize(index + 1, None);
                }
                if sizes[index] != Some(s) {
                    sizes[index] = Some(s);
                    runtime.thumb_sizes.set(sizes);
                }
            }
        }
    });

    let base_style = attrs.style();
    let mut base = Vec::with_capacity(26);
    base.push(Attribute::new("id", thumb_id, None, false));
    base.push(Attribute::new("role", attrs.role(), None, false));
    base.push(Attribute::new("tabindex", attrs.tabindex(), None, false));
    base.push(Attribute::new(
        "aria-valuenow",
        attrs.aria_valuenow,
        None,
        false,
    ));
    base.push(Attribute::new(
        "aria-valuemin",
        attrs.aria_valuemin,
        None,
        false,
    ));
    base.push(Attribute::new(
        "aria-valuemax",
        attrs.aria_valuemax,
        None,
        false,
    ));
    push_attr_opt(&mut base, "aria-valuetext", attrs.aria_valuetext.as_deref());
    base.push(Attribute::new(
        "aria-orientation",
        attrs.aria_orientation,
        None,
        false,
    ));
    push_attr_opt(&mut base, "aria-disabled", attrs.aria_disabled);
    push_attr_opt(&mut base, "aria-invalid", attrs.aria_invalid());
    push_attr_opt(&mut base, "aria-describedby", attrs.aria_describedby());
    push_attr_opt(&mut base, "aria-label", attrs.aria_label.as_deref());
    push_attr_opt(
        &mut base,
        "aria-labelledby",
        attrs.aria_labelledby.as_deref(),
    );
    if !base_style.is_empty() {
        base.push(Attribute::new("style", base_style, None, false));
    }
    base.push(Attribute::new(
        "data-orientation",
        attrs.data_orientation(),
        None,
        false,
    ));
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled());
    push_attr_opt(&mut base, "data-required", attrs.data_required());
    push_attr_opt(&mut base, "data-invalid", attrs.data_invalid());
    base.push(Attribute::new(
        "data-value",
        attrs.data_value().to_string(),
        None,
        false,
    ));
    push_attr_opt(&mut base, "data-active", attrs.data_active());
    base.push(Attribute::new("data-monoxus-slider-thumb", "", None, false));

    {
        let mut runtime = ctx.0.clone();
        let index = props.index;
        push_event_handler(
            &mut base,
            "onpointerdown",
            Some(EventHandler::new(move |evt: PointerEvent| {
                if runtime.def.peek().disabled {
                    return;
                }
                if evt
                    .trigger_button()
                    .is_some_and(|b| b != dioxus::html::input_data::MouseButton::Primary)
                {
                    return;
                }
                evt.prevent_default();
                runtime.thumb_drag_claimed.set(true);
                runtime.start_drag(index);
            })),
        );
    }
    {
        let mut runtime = ctx.0.clone();
        let index = props.index;
        push_event_handler(
            &mut base,
            "onkeydown",
            Some(EventHandler::new(move |evt: KeyboardEvent| {
                if runtime.def.peek().disabled {
                    return;
                }
                let event_key = evt.key();
                let Some(key_name) = key_to_roving_str(&event_key) else {
                    return;
                };
                evt.prevent_default();
                let shift = evt.modifiers().contains(Modifiers::SHIFT);
                runtime.handle_keydown(index, key_name, shift);
            })),
        );
    }
    {
        let mut runtime = ctx.0.clone();
        let index = props.index;
        let on_focus = props.on_focus;
        push_event_handler(
            &mut base,
            "onfocus",
            Some(EventHandler::new(move |evt: FocusEvent| {
                runtime.focused_thumb.set(Some(index));
                if let Some(handler) = on_focus {
                    handler.call(evt);
                }
            })),
        );
    }
    {
        let mut runtime = ctx.0.clone();
        let on_blur = props.on_blur;
        push_event_handler(
            &mut base,
            "onblur",
            Some(EventHandler::new(move |evt: FocusEvent| {
                runtime.focused_thumb.set(None);
                if let Some(handler) = on_blur {
                    handler.call(evt);
                }
            })),
        );
    }
    push_event_handler(
        &mut base,
        "onmounted",
        Some(EventHandler::new(move |_evt: MountedEvent| {
            mounted_rev += 1;
        })),
    );

    let user = collect_user_attrs(props.class, props.style, extracted.rest);

    let merged = merge_props_with_config(
        base,
        user,
        MergePropsConfig {
            invariant_attrs: &[],
            bookkeeping_events: &["onfocus", "onblur"],
        },
    );

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        span {
            ..merged,
            {props.children}
        }
    }
}
