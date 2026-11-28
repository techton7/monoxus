use dioxus::prelude::*;

use crate::field::{try_use_field_context, try_use_field_control_context};
use crate::foundation::{
    compose::{
        extract_aria_invalid_and_describedby, merge_global_attributes,
        resolve_invalid_and_describedby,
    },
    state::use_controllable_signal,
};

pub use ui_sdk::toggle::*;

#[derive(Props, Clone, PartialEq)]
pub struct ToggleProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub pressed: Option<bool>,
    #[props(default = false)]
    pub default_pressed: bool,
    #[props(default)]
    pub on_pressed_change: Option<EventHandler<bool>>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub invalid: Option<bool>,
    #[props(default)]
    pub aria_describedby: Option<String>,
    #[props(default)]
    pub r#type: ButtonType,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_focus: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub on_blur: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn Toggle(props: ToggleProps) -> Element {
    let field_ctx = try_use_field_context();
    let control_ctx = try_use_field_control_context();

    let effective_disabled = props.disabled || field_ctx.is_some_and(|f| f.is_disabled());

    let explicit_id = props.id.clone().filter(|id| !id.trim().is_empty());
    let resolved_id = if let Some(ref explicit) = explicit_id {
        if let Some(ctrl_ctx) = control_ctx {
            if !ctrl_ctx.is_explicit_id_locked() {
                ctrl_ctx.sync_control_id(explicit.clone());
            }
        } else if let Some(f_ctx) = field_ctx
            && !f_ctx.is_fieldset()
        {
            f_ctx.set_primary_control_id(Some(explicit.clone()));
        }
        Some(explicit.clone())
    } else if let Some(ctrl_ctx) = control_ctx {
        Some(ctrl_ctx.control_id())
    } else if let Some(f_ctx) = field_ctx
        && !f_ctx.is_fieldset()
    {
        Some(f_ctx.resolved_control_id())
    } else {
        None
    };

    let extracted = extract_aria_invalid_and_describedby(props.attributes);
    let field_describedby = field_ctx.and_then(|f| f.aria_describedby(None));
    let (effective_invalid, effective_describedby) = resolve_invalid_and_describedby(
        props.invalid,
        extracted.aria_invalid,
        field_ctx.map(|f| f.is_invalid()),
        false,
        props.aria_describedby.as_deref(),
        extracted.aria_describedby.as_deref(),
        field_describedby.as_deref(),
    );

    let pressed_ctl =
        use_controllable_signal(props.pressed, props.default_pressed, props.on_pressed_change);
    let is_pressed = pressed_ctl.get();

    let mut toggle_state = Toggle::new()
        .with_pressed(is_pressed)
        .with_disabled(effective_disabled)
        .with_invalid(effective_invalid)
        .with_aria_describedby(effective_describedby)
        .with_button_type(props.r#type);
    if let Some(id) = resolved_id {
        toggle_state = toggle_state.with_id(id);
    }
    let attrs = toggle_state.attributes();

    let mut on_focus_slot = use_hook(|| CopyValue::new(props.on_focus));
    on_focus_slot.set(props.on_focus);
    let mut on_blur_slot = use_hook(|| CopyValue::new(props.on_blur));
    on_blur_slot.set(props.on_blur);

    let on_click = {
        let on_click = props.on_click;
        let disabled = effective_disabled;
        move |evt: MouseEvent| {
            if let Some(ref handler) = on_click {
                handler.call(evt.clone());
            }
            if evt.default_action_enabled() && !disabled {
                let current = pressed_ctl.peek();
                pressed_ctl.set(!current);
            }
        }
    };

    let on_key_down = {
        let on_key_down = props.on_key_down;
        move |evt: KeyboardEvent| {
            if let Some(ref handler) = on_key_down {
                handler.call(evt);
            }
        }
    };

    let on_focus = move |evt: FocusEvent| {
        if let Some(handler) = on_focus_slot.try_read().ok().and_then(|s| *s) {
            handler.call(evt);
        }
    };

    let on_blur = move |evt: FocusEvent| {
        if let Some(handler) = on_blur_slot.try_read().ok().and_then(|s| *s) {
            handler.call(evt);
        }
    };

    let on_mounted = {
        let on_mounted = props.on_mounted;
        move |evt: MountedEvent| {
            if let Some(ref handler) = on_mounted {
                handler.call(evt);
            }
        }
    };

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        extracted.rest,
    );

    rsx! {
        button {
            id: attrs.id.as_deref(),
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref().unwrap_or_default(),
            r#type: attrs.r#type,
            disabled: attrs.disabled,
            "aria-pressed": attrs.aria_pressed,
            "aria-invalid": attrs.aria_invalid,
            "aria-describedby": attrs.aria_describedby.as_deref(),
            "data-state": attrs.data_state.as_str(),
            "data-disabled": attrs.data_disabled,
            "data-invalid": attrs.data_invalid,
            "data-toggle-root": attrs.data_toggle_root,
            onclick: on_click,
            onkeydown: on_key_down,
            onfocus: on_focus,
            onblur: on_blur,
            onmounted: on_mounted,
            ..merged.rest,
            {props.children}
        }
    }
}

pub use Toggle as ToggleRoot;
