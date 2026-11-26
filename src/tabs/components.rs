use std::borrow::Cow;

use dioxus::prelude::*;

use crate::foundation::{
    compose::merge_global_attributes,
    keyboard::is_activation_key,
    roving_focus::{RovingItemMeta, use_roving_focus_item},
    shared::{
        ScopeHandle, data_disabled_attr, use_direction_context, use_mount_animation_guard,
        use_stable_id,
    },
};

use super::{
    runtime::{TabsRuntime, use_tabs_runtime},
    state::Tabs,
    types::{TabsActivationMode, TabsDirection, TabsOrientation},
};

#[derive(Clone)]
pub struct TabsContext {
    pub runtime: TabsRuntime,
    pub value: Signal<String>,
    pub on_value_change: Option<EventHandler<String>>,
}

#[derive(Clone, PartialEq, Props)]
pub struct TabsRootProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub value: Option<Signal<String>>,
    #[props(default)]
    pub default_value: Option<String>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<String>>,
    #[props(default)]
    pub orientation: Option<TabsOrientation>,
    #[props(default)]
    pub dir: Option<TabsDirection>,
    #[props(default)]
    pub activation_mode: Option<TabsActivationMode>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn TabsRoot(props: TabsRootProps) -> Element {
    let TabsRootProps {
        id,
        class,
        style,
        value,
        default_value,
        on_value_change,
        orientation,
        dir,
        activation_mode,
        attributes,
        children,
    } = props;
    let resolved_dir = use_direction_context(dir.map(Into::into));
    let tabs_dir = TabsDirection::from(resolved_dir);
    let initial_val = default_value.unwrap_or_default();
    let fallback_val = use_signal(|| initial_val.clone());
    let val_signal = value.unwrap_or(fallback_val);
    let scope_id = use_stable_id(id.clone().map(Cow::Owned));
    let scope = ScopeHandle::root("tabs").child(scope_id);

    let mut tabs_def = Tabs::new(scope, val_signal()).with_direction(tabs_dir);
    if let Some(o) = orientation {
        tabs_def = tabs_def.with_orientation(o);
    }
    if let Some(am) = activation_mode {
        tabs_def = tabs_def.with_activation_mode(am);
    }

    let change_handler = {
        let val_sig = val_signal;
        let on_change = on_value_change;
        move |new_val: String| {
            let mut sig = val_sig;
            sig.set(new_val.clone());
            if let Some(h) = on_change {
                h.call(new_val);
            }
        }
    };

    let runtime = use_tabs_runtime(tabs_def, Some(change_handler));

    use_context_provider(|| TabsContext {
        runtime: runtime.clone(),
        value: val_signal,
        on_value_change,
    });

    let root_attrs = runtime.root();
    let merged = merge_global_attributes(class.as_deref(), style.as_deref(), attributes);

    rsx! {
        div {
            id: "{root_attrs.id()}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            dir: "{resolved_dir.as_str()}",
            "data-orientation": "{root_attrs.data_orientation()}",
            ..merged.rest,
            {children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct TabsListProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = true)]
    pub loop_focus: bool,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn TabsList(props: TabsListProps) -> Element {
    let TabsListProps {
        id,
        class,
        style,
        loop_focus,
        attributes,
        children,
    } = props;
    let ctx = use_context::<TabsContext>();
    let list_attrs = ctx.runtime.list();
    let list_id = id.unwrap_or_else(|| list_attrs.id().to_string());

    ctx.runtime.set_loop_focus(loop_focus);
    let runtime = ctx.runtime.clone();
    use_effect(use_reactive((&loop_focus,), {
        let runtime = ctx.runtime.clone();
        move |(loop_f,)| {
            runtime.set_loop_focus(loop_f);
        }
    }));

    let on_keydown = move |evt: KeyboardEvent| {
        let automatic = runtime.tabs().activation_mode().is_automatic();
        let rt = runtime.clone();
        runtime.roving().handle_keydown(
            None,
            &evt,
            Some(|next_val: &str| {
                if automatic {
                    rt.select_tab(next_val);
                }
            }),
        );
    };

    let merged = merge_global_attributes(class.as_deref(), style.as_deref(), attributes);

    rsx! {
        div {
            id: "{list_id}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            role: "{list_attrs.role()}",
            aria_orientation: "{list_attrs.aria_orientation()}",
            "data-orientation": "{list_attrs.data_orientation()}",
            "data-disabled": data_disabled_attr(list_attrs.is_disabled()),
            tabindex: "-1",
            onkeydown: on_keydown,
            ..merged.rest,
            {children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct TabsTriggerProps {
    pub value: String,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn TabsTrigger(props: TabsTriggerProps) -> Element {
    let TabsTriggerProps {
        value,
        disabled,
        id,
        class,
        style,
        attributes,
        children,
    } = props;
    let ctx = use_context::<TabsContext>();
    let trig_id = id.unwrap_or_else(|| ctx.runtime.tabs().relationships().trigger_id(&value));
    let effective_disabled = ctx.runtime.tabs().is_disabled() || disabled;
    let is_active = ctx.runtime.active_value() == value;

    use_roving_focus_item(
        ctx.runtime.roving(),
        &value,
        RovingItemMeta::new(trig_id.clone(), !effective_disabled, is_active),
    );

    let trig_attrs = ctx.runtime.trigger(&value, disabled);
    let is_disabled = trig_attrs.is_disabled();

    let runtime_click = ctx.runtime.clone();
    let val_click = value.clone();
    let on_click = move |_| {
        if is_disabled {
            return;
        }
        runtime_click.select_tab(&val_click);
    };

    let runtime_focus = ctx.runtime.clone();
    let val_focus = value.clone();
    let on_focus = move |_| {
        if is_disabled {
            return;
        }
        runtime_focus.roving().handle_item_focus(&val_focus, true);
        if runtime_focus.tabs().activation_mode().is_automatic() {
            runtime_focus.select_tab(&val_focus);
        }
    };

    let runtime_key = ctx.runtime.clone();
    let val_key = value.clone();
    let on_keydown = move |evt: KeyboardEvent| {
        if is_disabled {
            return;
        }
        if is_activation_key(&evt.key()) {
            evt.prevent_default();
            runtime_key.select_tab(&val_key);
            return;
        }
        let automatic = runtime_key.tabs().activation_mode().is_automatic();
        let rt = runtime_key.clone();
        runtime_key.roving().handle_keydown(
            Some(&val_key),
            &evt,
            Some(|next_val: &str| {
                if automatic {
                    rt.select_tab(next_val);
                }
            }),
        );
    };

    let merged = merge_global_attributes(class.as_deref(), style.as_deref(), attributes);

    rsx! {
        button {
            id: "{trig_id}",
            r#type: "button",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            role: "{trig_attrs.role()}",
            tabindex: "{trig_attrs.tabindex()}",
            aria_selected: "{trig_attrs.aria_selected()}",
            aria_controls: trig_attrs.aria_controls(),
            disabled: is_disabled,
            "data-state": trig_attrs.data_state().as_str(),
            "data-orientation": "{trig_attrs.data_orientation()}",
            "data-disabled": data_disabled_attr(is_disabled),
            onclick: on_click,
            onfocus: on_focus,
            onkeydown: on_keydown,
            ..merged.rest,
            {children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct TabsContentProps {
    pub value: String,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub tabindex: Option<i32>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn TabsContent(props: TabsContentProps) -> Element {
    let TabsContentProps {
        value,
        force_mount,
        id,
        class,
        style,
        tabindex,
        attributes,
        children,
    } = props;
    let ctx = use_context::<TabsContext>();
    let is_active = ctx.runtime.active_value() == value;

    let guard_style = use_mount_animation_guard(is_active);
    let should_render = is_active || force_mount;

    let content_attrs = ctx.runtime.content(&value);
    let content_id = id.unwrap_or_else(|| content_attrs.id().to_string());
    let labelledby = ctx.runtime.trigger_element_id(&value);

    let mut last_registered_val: CopyValue<Option<String>> = use_hook(|| CopyValue::new(None));
    let prev_registered = last_registered_val.try_read().ok().and_then(|g| g.clone());
    if should_render {
        if let Some(prev) = prev_registered.as_deref()
            && prev != value.as_str()
        {
            ctx.runtime.unregister_content_id(prev);
            ctx.runtime.unregister_content_mounted(prev);
        }
        ctx.runtime.register_content_id(&value, &content_id);
        ctx.runtime.register_content_mounted(&value);
        if prev_registered.as_deref() != Some(value.as_str()) {
            last_registered_val.set(Some(value.clone()));
        }
    } else {
        if let Some(prev) = prev_registered.as_deref() {
            ctx.runtime.unregister_content_id(prev);
            ctx.runtime.unregister_content_mounted(prev);
            last_registered_val.set(None);
        }
        ctx.runtime.unregister_content_id(&value);
        ctx.runtime.unregister_content_mounted(&value);
    }

    let drop_runtime = ctx.runtime.clone();
    use_drop(move || {
        if let Some(prev) = last_registered_val.try_read().ok().and_then(|g| g.clone()) {
            drop_runtime.unregister_content_id(&prev);
            drop_runtime.unregister_content_mounted(&prev);
        }
    });

    if !should_render {
        return rsx! {};
    }

    let effective_style = match (style.as_deref(), guard_style) {
        (Some(s), Some(g)) => Some(format!("{s}; {g}")),
        (Some(s), None) => Some(s.to_string()),
        (None, Some(g)) => Some(g.to_string()),
        (None, None) => None,
    };

    let tab_idx = tabindex.unwrap_or(0);
    let merged = merge_global_attributes(class.as_deref(), effective_style.as_deref(), attributes);

    rsx! {
        div {
            id: "{content_id}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            role: "{content_attrs.role()}",
            aria_labelledby: "{labelledby}",
            "data-state": if is_active { "active" } else { "inactive" },
            "data-orientation": "{content_attrs.data_orientation()}",
            tabindex: "{tab_idx}",
            hidden: if !is_active && force_mount { true } else { false },
            ..merged.rest,
            {children}
        }
    }
}
