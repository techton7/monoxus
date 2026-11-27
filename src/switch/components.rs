use std::borrow::Cow;

use dioxus::prelude::*;

use crate::button::ButtonType;
use crate::field::{try_use_field_context, try_use_field_control_context};
use crate::foundation::{
    compose::{
        collect_user_attrs, extract_aria_invalid_and_describedby, merge_props, push_attr_opt,
        push_event_handler, resolve_invalid_and_describedby,
    },
    form::{HiddenFormInput, HiddenInputKind, use_form_control_sync_with_epoch},
    shared::{ScopeHandle, use_stable_id},
    state::{sync_signal, use_controllable_signal},
};

use super::{
    attrs::{SwitchRootAttributes, SwitchThumbAttributes},
    relationships::SwitchRelationships,
    runtime::{SwitchContext, use_switch_context},
};

/// Props for [`SwitchRoot`].
#[derive(Props, Clone, PartialEq)]
pub struct SwitchRootProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub checked: Option<bool>,
    #[props(default = false)]
    pub default_checked: bool,
    #[props(default)]
    pub on_checked_change: Option<EventHandler<bool>>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub required: bool,
    #[props(default = false)]
    pub read_only: bool,
    #[props(default)]
    pub invalid: Option<bool>,
    #[props(default, into)]
    pub aria_describedby: Option<String>,
    #[props(default, into)]
    pub name: Option<String>,
    #[props(default = String::from("on"), into)]
    pub value: String,
    #[props(default, into)]
    pub form: Option<String>,
    #[props(default)]
    pub r#type: ButtonType,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_blur: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Headless interactive `<button role="switch">` control element.
#[component]
pub fn SwitchRoot(props: SwitchRootProps) -> Element {
    let field_ctx = try_use_field_context();
    let control_ctx = try_use_field_control_context();

    let effective_name = props
        .name
        .clone()
        .or_else(|| field_ctx.and_then(|f| f.name()));
    let effective_disabled = props.disabled || field_ctx.is_some_and(|f| f.is_disabled());
    let effective_required = props.required || field_ctx.is_some_and(|f| f.is_required());
    let effective_read_only = props.read_only || field_ctx.is_some_and(|f| f.is_read_only());

    let explicit_id = props.id.clone().filter(|id| !id.trim().is_empty());
    let bound_field_id = if let Some(ref explicit) = explicit_id {
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

    let stable_id = use_stable_id(bound_field_id.clone().map(Cow::Owned));
    let relationships = {
        let scope = ScopeHandle::root("switch").child(stable_id);
        if let Some(ref resolved_explicit_id) = bound_field_id {
            SwitchRelationships::with_explicit_root_id(scope, resolved_explicit_id)
        } else {
            SwitchRelationships::new(scope)
        }
    };

    let root_id = relationships.root_id().to_string();
    let thumb_id = relationships.thumb_id().to_string();

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

    let checked_ctl =
        use_controllable_signal(props.checked, props.default_checked, props.on_checked_change);
    let is_checked = checked_ctl.get();

    let initial_checked_for_reset =
        use_hook(|| props.checked.unwrap_or(props.default_checked));
    let (should_render_input, reset_epoch) = use_form_control_sync_with_epoch(
        &root_id,
        effective_name.as_deref(),
        props.form.as_deref(),
        move || {
            checked_ctl.set(initial_checked_for_reset);
        },
    );

    let mut disabled_sig = use_signal(|| effective_disabled);
    sync_signal(&mut disabled_sig, effective_disabled);
    let mut required_sig = use_signal(|| effective_required);
    sync_signal(&mut required_sig, effective_required);
    let mut read_only_sig = use_signal(|| effective_read_only);
    sync_signal(&mut read_only_sig, effective_read_only);
    let mut invalid_sig = use_signal(|| effective_invalid);
    sync_signal(&mut invalid_sig, effective_invalid);

    let context = SwitchContext {
        root_id: root_id.clone(),
        thumb_id: thumb_id.clone(),
        checked: checked_ctl.signal(),
        disabled: disabled_sig,
        required: required_sig,
        read_only: read_only_sig,
        invalid: invalid_sig,
    };
    use_context_provider(|| context);

    let attrs = SwitchRootAttributes::new(
        Some(root_id.clone()),
        props.r#type.as_str(),
        is_checked,
        effective_disabled,
        effective_required,
        effective_read_only,
    )
    .with_invalid(effective_invalid)
    .with_aria_describedby(effective_describedby);

    let data_state = attrs.data_state_str();
    let mut base = Vec::with_capacity(18);
    push_attr_opt(&mut base, "id", attrs.id);
    base.push(Attribute::new("type", attrs.r#type, None, false));
    base.push(Attribute::new("role", attrs.role, None, false));
    base.push(Attribute::new("aria-checked", attrs.aria_checked, None, false));
    push_attr_opt(&mut base, "aria-required", attrs.aria_required);
    push_attr_opt(&mut base, "aria-readonly", attrs.aria_readonly);
    push_attr_opt(&mut base, "aria-invalid", attrs.aria_invalid);
    push_attr_opt(&mut base, "aria-describedby", attrs.aria_describedby);
    base.push(Attribute::new("data-state", data_state, None, false));
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);
    push_attr_opt(&mut base, "data-required", attrs.data_required);
    push_attr_opt(&mut base, "data-readonly", attrs.data_readonly);
    push_attr_opt(&mut base, "data-invalid", attrs.data_invalid);
    base.push(Attribute::new(
        "data-switch-root",
        attrs.data_switch_root,
        None,
        false,
    ));
    if attrs.disabled {
        base.push(Attribute::new("disabled", true, None, false));
    }
    base.push(Attribute::new("value", props.value.clone(), None, false));
    push_event_handler(
        &mut base,
        "onclick",
        Some(EventHandler::new(move |_evt: MouseEvent| {
            if !effective_disabled && !effective_read_only {
                let current = checked_ctl.peek();
                checked_ctl.set(!current);
            }
        })),
    );

    let mut user = collect_user_attrs(props.class, props.style, extracted.rest);
    if !effective_disabled {
        push_event_handler(&mut user, "onclick", props.on_click);
    }
    // Per WAI-ARIA 1.2 Switch Pattern, BOTH Enter AND Space toggle the switch.
    // We do NOT call evt.prevent_default() on Enter or Space so the native <button>
    // synthesizes Click on Enter (KeyDown) and Space (KeyUp) across Web and Blitz Native.
    push_event_handler(&mut user, "onkeydown", props.on_key_down);
    push_event_handler(&mut user, "onblur", props.on_blur);
    push_event_handler(&mut user, "onmounted", props.on_mounted);

    let merged = merge_props(base, user);

    let root_el = if let Some(child) = props.child {
        child.call(merged)
    } else {
        rsx! {
            button {
                ..merged,
                {props.children}
            }
        }
    };

    rsx! {
        {root_el}
        if should_render_input {
            HiddenFormInput {
                kind: HiddenInputKind::Checkbox,
                name: effective_name,
                value: props.value.clone(),
                checked: is_checked,
                disabled: effective_disabled,
                required: effective_required,
                read_only: effective_read_only,
                form: props.form.clone(),
                focus_target_id: root_id.clone(),
                reset_epoch: reset_epoch,
            }
        }
    }
}

pub use SwitchRoot as Switch;

/// Props for [`SwitchThumb`].
#[derive(Props, Clone, PartialEq)]
pub struct SwitchThumbProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Permanently mounted slider thumb rendered inside [`SwitchRoot`].
#[component]
pub fn SwitchThumb(props: SwitchThumbProps) -> Element {
    let ctx = use_switch_context();
    let thumb_id = props.id.clone().unwrap_or_else(|| ctx.thumb_id().to_string());

    let attrs = SwitchThumbAttributes::new(
        Some(thumb_id),
        ctx.is_checked(),
        ctx.is_disabled(),
        ctx.is_required(),
        ctx.is_read_only(),
    )
    .with_invalid(ctx.is_invalid());

    let data_state = attrs.data_state_str();
    let mut base = Vec::with_capacity(8);
    push_attr_opt(&mut base, "id", attrs.id);
    base.push(Attribute::new("data-state", data_state, None, false));
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);
    push_attr_opt(&mut base, "data-required", attrs.data_required);
    push_attr_opt(&mut base, "data-readonly", attrs.data_readonly);
    push_attr_opt(&mut base, "data-invalid", attrs.data_invalid);
    base.push(Attribute::new(
        "data-switch-thumb",
        attrs.data_switch_thumb,
        None,
        false,
    ));

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler(&mut user, "onmounted", props.on_mounted);

    let merged = merge_props(base, user);

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
