use dioxus::prelude::*;

use crate::field::{try_use_field_context, try_use_field_control_context};
use crate::foundation::{
    browser::{ResizeObserverBox, use_element_size_with_mount},
    compose::{
        extract_aria_invalid_and_describedby, join_styles, merge_global_attributes,
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
    #[props(default)]
    pub id: Option<String>,
    pub children: Option<Element>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub aria_label: Option<String>,
    pub aria_labelledby: Option<String>,
    #[props(default)]
    pub aria_describedby: Option<String>,
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

    let mut on_focus_slot = use_hook(|| CopyValue::new(props.on_focus));
    on_focus_slot.set(props.on_focus);
    let mut on_blur_slot = use_hook(|| CopyValue::new(props.on_blur));
    on_blur_slot.set(props.on_blur);

    let on_pointer_down = {
        let mut runtime = ctx.0.clone();
        let index = props.index;
        move |evt: PointerEvent| {
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
        }
    };

    let on_key_down = {
        let mut runtime = ctx.0.clone();
        let index = props.index;
        move |evt: KeyboardEvent| {
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
        }
    };

    let on_focus = {
        let mut runtime = ctx.0.clone();
        let index = props.index;
        move |evt: FocusEvent| {
            runtime.focused_thumb.set(Some(index));
            if let Some(handler) = on_focus_slot.try_read().ok().and_then(|s| *s) {
                handler.call(evt);
            }
        }
    };

    let on_blur = {
        let mut runtime = ctx.0.clone();
        move |evt: FocusEvent| {
            runtime.focused_thumb.set(None);
            if let Some(handler) = on_blur_slot.try_read().ok().and_then(|s| *s) {
                handler.call(evt);
            }
        }
    };

    let combined_style = join_styles([attrs.style(), props.style.as_deref().unwrap_or("")]);
    let merged = merge_global_attributes(
        props.class.as_deref(),
        combined_style.as_deref(),
        extracted.rest,
    );

    rsx! {
        span {
            id: "{thumb_id}",
            role: "{attrs.role()}",
            tabindex: attrs.tabindex(),
            aria_valuenow: attrs.aria_valuenow,
            aria_valuemin: attrs.aria_valuemin,
            aria_valuemax: attrs.aria_valuemax,
            aria_valuetext: attrs.aria_valuetext.as_deref(),
            aria_orientation: attrs.aria_orientation,
            aria_disabled: attrs.aria_disabled,
            aria_invalid: attrs.aria_invalid(),
            aria_describedby: attrs.aria_describedby(),
            aria_label: attrs.aria_label.as_deref(),
            aria_labelledby: attrs.aria_labelledby.as_deref(),
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref().unwrap_or_default(),
            "data-orientation": "{attrs.data_orientation()}",
            "data-disabled": attrs.data_disabled(),
            "data-required": attrs.data_required(),
            "data-invalid": attrs.data_invalid(),
            "data-value": "{attrs.data_value()}",
            "data-active": attrs.data_active(),
            "data-monoxus-slider-thumb": "",
            onpointerdown: on_pointer_down,
            onkeydown: on_key_down,
            onfocus: on_focus,
            onblur: on_blur,
            onmounted: move |_| mounted_rev += 1,
            ..merged.rest,
            {props.children}
        }
    }
}
