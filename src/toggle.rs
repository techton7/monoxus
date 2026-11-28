use std::rc::Rc;

use dioxus::prelude::*;
use ui_sdk::kernel::{ChangeContext, ChangeEvent, ChangeReason, ChangeSource};

use crate::agent::{
    ActionDescriptor, AgentHints, AgentSurface, ComponentSnapshot, DispatchExecResult,
    SnapshotValue, SurfaceMetaDefaults, use_agent_surface,
};
use crate::field::{try_use_field_context, try_use_field_control_context};
use crate::foundation::{
    compose::{
        collect_user_attrs, extract_aria_invalid_and_describedby, merge_props, push_attr_opt,
        push_event_handler, resolve_invalid_and_describedby,
    },
    state::{ControllableConfig, use_controllable_signal_config},
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
    #[props(default)]
    pub on_pressed_change_details: Option<EventHandler<ChangeEvent<bool>>>,
    #[props(default)]
    pub agent: Option<AgentHints>,
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

    let pressed_ctl = use_controllable_signal_config(
        ControllableConfig::new(props.pressed, props.default_pressed)
            .with_explicit_default(props.default_pressed)
            .with_on_change(props.on_pressed_change)
            .with_on_change_details(props.on_pressed_change_details)
            .with_component("Toggle")
            .with_prop("pressed"),
    );
    let is_pressed = pressed_ctl.get();

    let agent_label = resolved_id
        .clone()
        .unwrap_or_else(|| String::from("Toggle"));
    use_agent_surface(
        props.agent.as_ref(),
        SurfaceMetaDefaults::new(),
        || {
            AgentSurface::new("toggle", agent_label)
                .with_disabled(effective_disabled)
                .with_snapshot(
                    ComponentSnapshot::new()
                        .with_field("pressed", SnapshotValue::Bool(is_pressed)),
                )
                .with_actions(vec![
                    ActionDescriptor::new("toggle", "Toggle pressed state", false),
                    ActionDescriptor::new("set", "Set pressed state", true),
                ])
        },
        || {
            Rc::new(move |dispatch| match dispatch.action.as_str() {
                "toggle" => {
                    let next = !pressed_ctl.peek();
                    let outcome = pressed_ctl.set_with_context(next, dispatch.ctx);
                    let mut res = DispatchExecResult::ok().with_canceled(outcome.is_canceled());
                    if !outcome.is_canceled() {
                        res = res.with_expected("pressed", SnapshotValue::Bool(next));
                    }
                    res
                }
                "set" => {
                    let Some(SnapshotValue::Bool(next)) = dispatch
                        .args
                        .get("pressed")
                        .or_else(|| dispatch.args.get("value"))
                    else {
                        return DispatchExecResult::ok().with_host_error(
                            "missing or invalid boolean argument 'pressed'",
                        );
                    };
                    let next = *next;
                    let outcome = pressed_ctl.set_with_context(next, dispatch.ctx);
                    let mut res = DispatchExecResult::ok().with_canceled(outcome.is_canceled());
                    if !outcome.is_canceled() {
                        res = res.with_expected("pressed", SnapshotValue::Bool(next));
                    }
                    res
                }
                other => DispatchExecResult::ok()
                    .with_host_error(format!("unknown toggle action '{other}'")),
            })
        },
        || {
            Rc::new(move || {
                ComponentSnapshot::new().with_field(
                    "pressed",
                    SnapshotValue::Bool(pressed_ctl.try_peek_cloned().unwrap_or(false)),
                )
            })
        },
    );

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
                pressed_ctl.set_with_context(
                    !current,
                    ChangeContext::new(ChangeReason::Toggle, ChangeSource::Mouse),
                );
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
