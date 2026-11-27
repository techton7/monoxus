use dioxus::prelude::*;

use crate::field::{try_use_field_context, try_use_field_control_context};
use crate::foundation::{
    compose::{
        collect_user_attrs, extract_aria_invalid_and_describedby, merge_props, push_attr_opt,
        push_event_handler, resolve_invalid_and_describedby,
    },
    state::use_controllable_signal,
};

pub use ui_sdk::toggle::*;

#[derive(Props, Clone, PartialEq)]
pub struct ToggleProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
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
    #[props(default, into)]
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
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
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

    let mut base = Vec::with_capacity(12);
    push_attr_opt(&mut base, "id", attrs.id);
    base.push(Attribute::new("type", attrs.r#type, None, false));
    if attrs.disabled {
        base.push(Attribute::new("disabled", true, None, false));
    }
    base.push(Attribute::new("aria-pressed", attrs.aria_pressed, None, false));
    push_attr_opt(&mut base, "aria-invalid", attrs.aria_invalid);
    push_attr_opt(&mut base, "aria-describedby", attrs.aria_describedby);
    base.push(Attribute::new(
        "data-state",
        attrs.data_state.as_str(),
        None,
        false,
    ));
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);
    push_attr_opt(&mut base, "data-invalid", attrs.data_invalid);
    base.push(Attribute::new(
        "data-toggle-root",
        attrs.data_toggle_root,
        None,
        false,
    ));
    push_event_handler(
        &mut base,
        "onclick",
        Some(EventHandler::new(move |_evt: MouseEvent| {
            if !effective_disabled {
                let current = pressed_ctl.peek();
                pressed_ctl.set(!current);
            }
        })),
    );

    let mut user = collect_user_attrs(props.class, props.style, extracted.rest);
    if !effective_disabled {
        push_event_handler(&mut user, "onclick", props.on_click);
    }
    push_event_handler(&mut user, "onkeydown", props.on_key_down);
    push_event_handler(&mut user, "onfocus", props.on_focus);
    push_event_handler(&mut user, "onblur", props.on_blur);
    push_event_handler(&mut user, "onmounted", props.on_mounted);

    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        button {
            ..merged,
            {props.children}
        }
    }
}

pub use Toggle as ToggleRoot;
