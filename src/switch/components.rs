use std::borrow::Cow;
use std::rc::Rc;

use dioxus::prelude::*;

use crate::button::ButtonType;
use crate::foundation::{
    compose::{project_as_child, Slottable},
    form::{HiddenFormInput, HiddenInputKind, use_form_control_sync_with_epoch},
    shared::{ScopeHandle, use_stable_id},
    state::{use_controllable_state, ControllableStateProps},
};

use super::{
    attrs::{SwitchRootAttributes, SwitchThumbAttributes},
    relationships::SwitchRelationships,
    runtime::{use_switch_context, SwitchContext},
    types::SwitchCheckedChangeHandler,
};

pub fn project_switch_root_as_child<T, C>(
    attrs: &SwitchRootAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, SwitchRootAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

pub fn project_switch_thumb_as_child<T, C>(
    attrs: &SwitchThumbAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, SwitchThumbAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

/// Props for [`SwitchRoot`].
#[derive(Props, Clone, PartialEq)]
pub struct SwitchRootProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
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
    pub name: Option<String>,
    #[props(default = String::from("on"))]
    pub value: String,
    #[props(default)]
    pub form: Option<String>,
    #[props(default)]
    pub r#type: ButtonType,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Headless interactive `<button role="switch">` control element.
#[component]
pub fn SwitchRoot(props: SwitchRootProps) -> Element {
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned));
    let relationships = {
        let scope = ScopeHandle::root("switch").child(stable_id);
        if let Some(ref explicit_id) = props.id {
            SwitchRelationships::with_explicit_root_id(scope, explicit_id)
        } else {
            SwitchRelationships::new(scope)
        }
    };

    let root_id = relationships.root_id().to_string();
    let thumb_id = relationships.thumb_id().to_string();

    let on_checked_change_cb = props.on_checked_change.as_ref().map(|handler| {
        let handler = *handler;
        Rc::new(move |next: bool| {
            handler.call(next);
        }) as SwitchCheckedChangeHandler
    });

    let checked_handle = use_controllable_state(ControllableStateProps {
        value: props.checked,
        default_value: props.default_checked,
        on_change: on_checked_change_cb.clone().map(|cb| {
            move |v: bool| cb(v)
        }),
    });

    let is_checked = *checked_handle.value();

    let initial_checked_for_reset = props.default_checked;
    let reset_handle = checked_handle.clone();
    let (should_render_input, reset_epoch) = use_form_control_sync_with_epoch(
        &root_id,
        props.name.as_deref(),
        props.form.as_deref(),
        move || {
            reset_handle.set(initial_checked_for_reset);
        },
    );

    let mut checked_sig = use_signal(|| is_checked);
    if *checked_sig.peek() != is_checked {
        checked_sig.set(is_checked);
    }
    let mut disabled_sig = use_signal(|| props.disabled);
    if *disabled_sig.peek() != props.disabled {
        disabled_sig.set(props.disabled);
    }
    let mut required_sig = use_signal(|| props.required);
    if *required_sig.peek() != props.required {
        required_sig.set(props.required);
    }
    let mut read_only_sig = use_signal(|| props.read_only);
    if *read_only_sig.peek() != props.read_only {
        read_only_sig.set(props.read_only);
    }

    let context = SwitchContext {
        root_id: root_id.clone(),
        thumb_id: thumb_id.clone(),
        checked: checked_sig,
        disabled: disabled_sig,
        required: required_sig,
        read_only: read_only_sig,
    };
    use_context_provider(|| context);

    let on_click = {
        let on_click = props.on_click.clone();
        let disabled = props.disabled;
        let read_only = props.read_only;
        let handle = checked_handle.clone();
        let current = is_checked;
        move |evt: MouseEvent| {
            if let Some(ref handler) = on_click {
                handler.call(evt.clone());
            }
            if evt.default_action_enabled() && !disabled && !read_only {
                handle.set(!current);
            }
        }
    };

    let on_key_down = {
        let on_key_down = props.on_key_down.clone();
        move |evt: KeyboardEvent| {
            if let Some(ref handler) = on_key_down {
                handler.call(evt);
            }
            // Per WAI-ARIA 1.2 Switch Pattern, BOTH Enter AND Space toggle the switch.
            // We do NOT call evt.prevent_default() on Enter or Space so the native <button>
            // synthesizes Click on Enter (KeyDown) and Space (KeyUp) across Web and Blitz Native.
        }
    };

    let on_mounted = {
        let on_mounted = props.on_mounted.clone();
        move |evt: MountedEvent| {
            if let Some(ref handler) = on_mounted {
                handler.call(evt);
            }
        }
    };

    let attrs = SwitchRootAttributes::new(
        Some(root_id.clone()),
        props.r#type.as_str(),
        is_checked,
        props.disabled,
        props.required,
        props.read_only,
    );

    rsx! {
        button {
            id: attrs.id.as_deref(),
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            r#type: attrs.r#type,
            role: attrs.role,
            "aria-checked": attrs.aria_checked,
            "aria-required": attrs.aria_required,
            "aria-readonly": attrs.aria_readonly,
            "data-state": attrs.data_state_str(),
            "data-disabled": attrs.data_disabled,
            "data-required": attrs.data_required,
            "data-readonly": attrs.data_readonly,
            "data-switch-root": attrs.data_switch_root,
            disabled: attrs.disabled,
            value: "{props.value}",
            onclick: on_click,
            onkeydown: on_key_down,
            onmounted: on_mounted,
            ..props.attributes,
            {props.children}
        }
        if should_render_input {
            HiddenFormInput {
                kind: HiddenInputKind::Checkbox,
                name: props.name.clone(),
                value: props.value.clone(),
                checked: is_checked,
                disabled: props.disabled,
                required: props.required,
                read_only: props.read_only,
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
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
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

    let on_mounted = {
        let on_mounted = props.on_mounted.clone();
        move |evt: MountedEvent| {
            if let Some(ref handler) = on_mounted {
                handler.call(evt);
            }
        }
    };

    let attrs = SwitchThumbAttributes::new(
        Some(thumb_id),
        ctx.is_checked(),
        ctx.is_disabled(),
        ctx.is_required(),
        ctx.is_read_only(),
    );

    rsx! {
        span {
            id: attrs.id.as_deref(),
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            "data-state": attrs.data_state_str(),
            "data-disabled": attrs.data_disabled,
            "data-required": attrs.data_required,
            "data-readonly": attrs.data_readonly,
            "data-switch-thumb": attrs.data_switch_thumb,
            onmounted: on_mounted,
            ..props.attributes,
            {props.children}
        }
    }
}
