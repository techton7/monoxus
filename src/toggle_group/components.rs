use std::borrow::Cow;

use dioxus::prelude::*;

use crate::field::try_use_field_context;
use crate::foundation::{
    browser::focus::use_composite_focus_out,
    compose::{
        extract_aria_invalid_and_describedby, merge_global_attributes,
        resolve_invalid_and_describedby,
    },
    keyboard::is_activation_key,
    roving_focus::{RovingItemMeta, use_roving_focus_item},
    shared::{Direction, ScopeHandle, use_direction_context, use_stable_id},
    state::sync_signal,
};

use super::{
    runtime::{ToggleGroupRuntime, use_toggle_group_runtime},
    types::{ToggleGroupMode, ToggleGroupOrientation},
};

#[derive(Clone)]
pub struct ToggleGroupContext {
    pub runtime: ToggleGroupRuntime,
}

#[derive(Props, Clone, PartialEq)]
pub struct ToggleGroupRootProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub r#type: ToggleGroupMode,
    #[props(default)]
    pub value: Option<String>,
    #[props(default)]
    pub default_value: Option<String>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<String>>,
    #[props(default)]
    pub values: Option<Vec<String>>,
    #[props(default)]
    pub default_values: Option<Vec<String>>,
    #[props(default)]
    pub on_values_change: Option<EventHandler<Vec<String>>>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub invalid: Option<bool>,
    #[props(default)]
    pub aria_describedby: Option<String>,
    #[props(default)]
    pub on_focus_out: Option<EventHandler<FocusEvent>>,
    #[props(default = true)]
    pub roving_focus: bool,
    #[props(default = true)]
    pub loop_focus: bool,
    #[props(default)]
    pub orientation: ToggleGroupOrientation,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default)]
    pub role: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn ToggleGroupRoot(props: ToggleGroupRootProps) -> Element {
    let field_ctx = try_use_field_context();
    let effective_disabled = props.disabled || field_ctx.is_some_and(|f| f.is_disabled());

    let resolved_dir = use_direction_context(props.dir);
    let scope_id = use_stable_id(props.id.clone().map(Cow::Owned)).into_owned();
    let initial_root_dom_id = props.id.clone().unwrap_or_else(|| {
        ScopeHandle::root("toggle-group")
            .child(scope_id.clone())
            .qualify("root")
    });

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

    let mut runtime = use_toggle_group_runtime(
        props.r#type,
        props.value.clone(),
        props.default_value.clone(),
        props.on_value_change,
        props.values.clone(),
        props.default_values.clone(),
        props.on_values_change,
        props.orientation,
        resolved_dir,
        props.roving_focus,
        props.loop_focus,
        effective_disabled,
        scope_id,
        initial_root_dom_id,
    );
    sync_signal(&mut runtime.invalid, effective_invalid);

    use_context_provider(|| ToggleGroupContext {
        runtime: runtime.clone(),
    });

    let root_attrs = runtime
        .root_attributes(None)
        .with_invalid(effective_invalid)
        .with_aria_describedby(effective_describedby);
    let resolved_id = props.id.as_deref().unwrap_or(root_attrs.id());
    runtime.roving.set_root_dom_id(resolved_id);
    let (handle_focus_in, handle_focus_out) =
        use_composite_focus_out(vec![resolved_id.to_string()], props.on_focus_out);
    let resolved_role = props.role.as_deref().unwrap_or(root_attrs.role());
    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        extracted.rest,
    );

    rsx! {
        div {
            id: "{resolved_id}",
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref().unwrap_or_default(),
            role: "{resolved_role}",
            dir: "{root_attrs.dir}",
            "aria-invalid": root_attrs.aria_invalid,
            "aria-describedby": root_attrs.aria_describedby.as_deref(),
            "data-orientation": "{root_attrs.data_orientation}",
            "data-disabled": root_attrs.data_disabled,
            "data-invalid": root_attrs.data_invalid,
            "data-toggle-group-root": "{root_attrs.data_toggle_group_root}",
            onfocusin: handle_focus_in,
            onfocusout: handle_focus_out,
            ..merged.rest,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ToggleGroupItemProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    pub value: String,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_blur: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn ToggleGroupItem(props: ToggleGroupItemProps) -> Element {
    let ctx = use_context::<ToggleGroupContext>();
    let runtime = ctx.runtime;

    let item_id = props
        .id
        .clone()
        .unwrap_or_else(|| runtime.state().relationships().item_id(&props.value));
    let effective_disabled = runtime.is_effective_disabled(props.disabled);
    let is_pressed = runtime.is_item_pressed(&props.value);

    use_roving_focus_item(
        runtime.roving,
        &props.value,
        RovingItemMeta::new(item_id.clone(), !effective_disabled, is_pressed),
    );

    let mut on_blur_slot = use_hook(|| CopyValue::new(props.on_blur));
    on_blur_slot.set(props.on_blur);

    let item_attrs = runtime
        .item_attributes(&props.value, Some(item_id), props.disabled)
        .with_invalid(runtime.is_invalid());

    let on_click = {
        let runtime = runtime.clone();
        let val = props.value.clone();
        let consumer_on_click = props.on_click;
        move |evt: MouseEvent| {
            if let Some(ref handler) = consumer_on_click {
                handler.call(evt.clone());
            }
            if evt.default_action_enabled() && !effective_disabled {
                runtime.toggle_item(&val);
            }
        }
    };

    let on_key_down = {
        let runtime = runtime.clone();
        let val = props.value.clone();
        let consumer_on_keydown = props.on_key_down;
        move |evt: KeyboardEvent| {
            if let Some(ref handler) = consumer_on_keydown {
                handler.call(evt.clone());
            }

            if !evt.default_action_enabled() || effective_disabled {
                return;
            }

            if is_activation_key(&evt.key()) {
                evt.prevent_default();
                runtime.toggle_item(&val);
                return;
            }

            runtime
                .roving
                .handle_keydown(Some(&val), &evt, None::<fn(&str)>);
        }
    };

    let on_focus = {
        let runtime = runtime.clone();
        let val = props.value.clone();
        move |_| {
            runtime.roving.handle_item_focus(&val, !effective_disabled);
        }
    };

    let on_blur = move |evt: FocusEvent| {
        if let Some(handler) = on_blur_slot.try_read().ok().and_then(|s| *s) {
            handler.call(evt);
        }
    };

    let on_mounted = {
        let consumer_on_mounted = props.on_mounted;
        move |evt: MountedEvent| {
            if let Some(ref handler) = consumer_on_mounted {
                handler.call(evt);
            }
        }
    };

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        button {
            id: "{item_attrs.id}",
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref().unwrap_or_default(),
            r#type: "button",
            role: item_attrs.role,
            "aria-checked": item_attrs.aria_checked,
            "aria-pressed": item_attrs.aria_pressed,
            tabindex: item_attrs.tabindex,
            disabled: item_attrs.disabled,
            "data-disabled": item_attrs.data_disabled,
            "data-invalid": item_attrs.data_invalid,
            "data-state": item_attrs.data_state.as_str(),
            "data-value": "{item_attrs.data_value}",
            "data-orientation": "{item_attrs.data_orientation}",
            "data-toggle-group-item": "{item_attrs.data_toggle_group_item}",
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
