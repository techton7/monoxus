use std::borrow::Cow;

use dioxus::prelude::*;

use crate::field::try_use_field_context;
use crate::foundation::{
    browser::focus::use_composite_focus_out,
    compose::{
        MergePropsConfig, collect_user_attrs, extract_aria_invalid_and_describedby,
        merge_props_with_config, push_attr_opt, push_event_handler,
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
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub r#type: ToggleGroupMode,
    #[props(default, into)]
    pub value: Option<String>,
    #[props(default, into)]
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
    #[props(default, into)]
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
    #[props(default, into)]
    pub role: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
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
    let resolved_id = props
        .id
        .clone()
        .unwrap_or_else(|| root_attrs.id().to_string());
    runtime.roving.set_root_dom_id(&resolved_id);
    let (handle_focus_in, handle_focus_out) =
        use_composite_focus_out(vec![resolved_id.clone()], props.on_focus_out);
    let resolved_role = props
        .role
        .clone()
        .unwrap_or_else(|| root_attrs.role().to_string());

    let mut base = Vec::with_capacity(12);
    base.push(Attribute::new("id", resolved_id, None, false));
    base.push(Attribute::new("role", resolved_role, None, false));
    base.push(Attribute::new("dir", root_attrs.dir, None, false));
    push_attr_opt(&mut base, "aria-invalid", root_attrs.aria_invalid);
    push_attr_opt(&mut base, "aria-describedby", root_attrs.aria_describedby);
    base.push(Attribute::new(
        "data-orientation",
        root_attrs.data_orientation,
        None,
        false,
    ));
    push_attr_opt(&mut base, "data-disabled", root_attrs.data_disabled);
    push_attr_opt(&mut base, "data-invalid", root_attrs.data_invalid);
    base.push(Attribute::new(
        "data-toggle-group-root",
        root_attrs.data_toggle_group_root,
        None,
        false,
    ));
    push_event_handler(&mut base, "onfocusin", Some(handle_focus_in));
    push_event_handler(&mut base, "onfocusout", Some(handle_focus_out));

    let user = collect_user_attrs(props.class, props.style, extracted.rest);
    let merged = merge_props_with_config(
        base,
        user,
        MergePropsConfig {
            invariant_attrs: &[],
            bookkeeping_events: &["onfocusin", "onfocusout"],
        },
    );

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ToggleGroupItemProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(into)]
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
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
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

    let item_attrs = runtime
        .item_attributes(&props.value, Some(item_id), props.disabled)
        .with_invalid(runtime.is_invalid());

    let mut base = Vec::with_capacity(18);
    base.push(Attribute::new("id", item_attrs.id, None, false));
    base.push(Attribute::new("type", "button", None, false));
    push_attr_opt(&mut base, "role", item_attrs.role);
    push_attr_opt(&mut base, "aria-checked", item_attrs.aria_checked);
    push_attr_opt(&mut base, "aria-pressed", item_attrs.aria_pressed);
    base.push(Attribute::new("tabindex", item_attrs.tabindex, None, false));
    if item_attrs.disabled {
        base.push(Attribute::new("disabled", true, None, false));
    }
    push_attr_opt(&mut base, "data-disabled", item_attrs.data_disabled);
    push_attr_opt(&mut base, "data-invalid", item_attrs.data_invalid);
    base.push(Attribute::new(
        "data-state",
        item_attrs.data_state.as_str(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-value",
        item_attrs.data_value,
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-orientation",
        item_attrs.data_orientation,
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-toggle-group-item",
        item_attrs.data_toggle_group_item,
        None,
        false,
    ));

    {
        let runtime = runtime.clone();
        let val = props.value.clone();
        push_event_handler(
            &mut base,
            "onclick",
            Some(EventHandler::new(move |_evt: MouseEvent| {
                if !effective_disabled {
                    runtime.toggle_item(&val);
                }
            })),
        );
    }
    {
        let runtime = runtime.clone();
        let val = props.value.clone();
        push_event_handler(
            &mut base,
            "onkeydown",
            Some(EventHandler::new(move |evt: KeyboardEvent| {
                if effective_disabled {
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
            })),
        );
    }
    {
        let runtime = runtime.clone();
        let val = props.value.clone();
        push_event_handler(
            &mut base,
            "onfocus",
            Some(EventHandler::new(move |_evt: FocusEvent| {
                runtime.roving.handle_item_focus(&val, !effective_disabled);
            })),
        );
    }

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    if !effective_disabled {
        push_event_handler(&mut user, "onclick", props.on_click);
    }
    push_event_handler(&mut user, "onkeydown", props.on_key_down);
    push_event_handler(&mut user, "onblur", props.on_blur);
    push_event_handler(&mut user, "onmounted", props.on_mounted);

    let merged = merge_props_with_config(
        base,
        user,
        MergePropsConfig {
            invariant_attrs: &[],
            bookkeeping_events: &["onfocus"],
        },
    );

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
