use std::borrow::Cow;
use std::rc::Rc;

use dioxus::core::{AttributeValue, Event, ListenerCallback};
use dioxus::html::PlatformEventData;
use dioxus::prelude::*;
use ui_sdk::kernel::{ChangeContext, ChangeEvent, ChangeReason, ChangeSource};

use crate::agent::{
    ActionDescriptor, AgentHints, AgentSurface, ComponentSnapshot, DispatchExecResult,
    SnapshotValue, SurfaceMetaDefaults, use_agent_surface,
};
use crate::foundation::{
    compose::{
        collect_user_attrs, merge_props, push_attr_opt, push_bool_data_attr, push_event_handler,
    },
    keyboard::is_activation_key,
    roving_focus::{RovingItemMeta, use_roving_focus_item},
    shared::{ScopeHandle, use_direction_context, use_mount_animation_guard, use_stable_id},
    state::{ControllableConfig, use_controllable_signal_config},
};

use super::{
    runtime::{TabsRuntime, use_tabs_runtime_controllable},
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
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default, into)]
    pub value: Option<String>,
    #[props(default, into)]
    pub default_value: Option<String>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<String>>,
    #[props(default)]
    pub on_value_change_details: Option<EventHandler<ChangeEvent<String>>>,
    #[props(default)]
    pub agent: Option<AgentHints>,
    #[props(default)]
    pub orientation: Option<TabsOrientation>,
    #[props(default)]
    pub dir: Option<TabsDirection>,
    #[props(default)]
    pub activation_mode: Option<TabsActivationMode>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
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
        on_value_change_details,
        agent,
        orientation,
        dir,
        activation_mode,
        disabled,
        child,
        attributes,
        children,
    } = props;
    let resolved_dir = use_direction_context(dir.map(Into::into));
    let tabs_dir = TabsDirection::from(resolved_dir);
    let has_explicit_default = default_value.is_some();
    let value_ctl = use_controllable_signal_config(
        ControllableConfig::new(value, default_value.unwrap_or_default())
            .with_explicit_default(has_explicit_default)
            .with_on_change(on_value_change)
            .with_on_change_details(on_value_change_details)
            .with_component("TabsRoot")
            .with_prop("value"),
    );
    let val_signal = value_ctl.signal();
    let scope_id = use_stable_id(id.clone().map(Cow::Owned));
    let scope = ScopeHandle::root("tabs").child(scope_id);

    let mut tabs_def = Tabs::new(scope, val_signal())
        .with_direction(tabs_dir)
        .with_disabled(disabled);
    if let Some(o) = orientation {
        tabs_def = tabs_def.with_orientation(o);
    }
    if let Some(am) = activation_mode {
        tabs_def = tabs_def.with_activation_mode(am);
    }

    let runtime = use_tabs_runtime_controllable(tabs_def, value_ctl);

    use_context_provider(|| TabsContext {
        runtime: runtime.clone(),
        value: val_signal,
        on_value_change,
    });

    let root_attrs = runtime.root();
    let active_now = runtime.active_value();
    let reveals: Vec<String> = runtime
        .roving()
        .registry
        .read()
        .iter()
        .map(|entry| entry.key().clone())
        .filter(|k| k != &active_now)
        .collect();
    let agent_label = root_attrs.id().to_string();
    let agent_rt_dispatch = runtime.clone();
    let agent_rt_reader = runtime.clone();
    use_agent_surface(
        agent.as_ref(),
        SurfaceMetaDefaults::new().with_reveals(reveals),
        || {
            AgentSurface::new("tabs", agent_label)
                .with_disabled(disabled)
                .with_snapshot(
                    ComponentSnapshot::new()
                        .with_field("value", SnapshotValue::text(active_now)),
                )
                .with_actions(vec![
                    ActionDescriptor::new("select", "Select a tab by value", true),
                    ActionDescriptor::new("set", "Set active tab value", true),
                ])
        },
        || {
            Rc::new(move |dispatch| match dispatch.action.as_str() {
                "select" | "set" => {
                    let Some(SnapshotValue::Text(val)) = dispatch.args.get("value") else {
                        return DispatchExecResult::ok()
                            .with_host_error("missing or invalid string argument 'value'");
                    };
                    let outcome = agent_rt_dispatch.select_tab_with_context(val, dispatch.ctx);
                    let mut res = DispatchExecResult::ok().with_canceled(outcome.is_canceled());
                    if !outcome.is_canceled() {
                        res = res.with_expected("value", SnapshotValue::text(val.clone()));
                    }
                    res
                }
                other => DispatchExecResult::ok()
                    .with_host_error(format!("unknown tabs action '{other}'")),
            })
        },
        || {
            Rc::new(move || {
                ComponentSnapshot::new().with_field(
                    "value",
                    SnapshotValue::text(agent_rt_reader.active_value_peek()),
                )
            })
        },
    );
    let mut base = Vec::with_capacity(5);
    base.push(Attribute::new("id", root_attrs.id().to_string(), None, false));
    base.push(Attribute::new("dir", resolved_dir.as_str(), None, false));
    base.push(Attribute::new(
        "data-orientation",
        root_attrs.data_orientation(),
        None,
        false,
    ));
    push_bool_data_attr(&mut base, "data-disabled", root_attrs.is_disabled());

    let user = collect_user_attrs(class, style, attributes);
    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            {children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct TabsListProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default = true)]
    pub loop_focus: bool,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn TabsList(props: TabsListProps) -> Element {
    let TabsListProps {
        id,
        class,
        style,
        loop_focus,
        child,
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

    let mut base = Vec::with_capacity(8);
    base.push(Attribute::new("id", list_id, None, false));
    base.push(Attribute::new("role", list_attrs.role(), None, false));
    base.push(Attribute::new(
        "aria-orientation",
        list_attrs.aria_orientation(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-orientation",
        list_attrs.data_orientation(),
        None,
        false,
    ));
    push_bool_data_attr(&mut base, "data-disabled", list_attrs.is_disabled());
    base.push(Attribute::new("tabindex", "-1", None, false));
    base.push(Attribute::new(
        "onkeydown",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                if runtime.tabs().is_disabled() {
                    return;
                }
                let kb_evt: KeyboardEvent = evt.map(|d| KeyboardData::from(d));
                if !kb_evt.default_action_enabled() {
                    return;
                }
                let automatic = runtime.tabs().activation_mode().is_automatic();
                let rt = runtime.clone();
                runtime.roving().handle_keydown(
                    None,
                    &kb_evt,
                    Some(|next_val: &str| {
                        if automatic {
                            let _ = rt.select_tab_with_context(
                                next_val,
                                ChangeContext::new(ChangeReason::TabChange, ChangeSource::Keyboard),
                            );
                        }
                    }),
                );
            })
            .erase(),
        ),
        None,
        false,
    ));

    let user = collect_user_attrs(class, style, attributes);
    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            {children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct TabsTriggerProps {
    #[props(into)]
    pub value: String,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
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
        on_click,
        on_key_down,
        on_mounted,
        child,
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
    let runtime_focus = ctx.runtime.clone();
    let val_focus = value.clone();
    let runtime_key = ctx.runtime.clone();
    let val_key = value.clone();

    let mut base = Vec::with_capacity(14);
    base.push(Attribute::new("id", trig_id, None, false));
    base.push(Attribute::new("type", "button", None, false));
    base.push(Attribute::new("role", trig_attrs.role(), None, false));
    base.push(Attribute::new(
        "tabindex",
        trig_attrs.tabindex().to_string(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "aria-selected",
        trig_attrs.aria_selected(),
        None,
        false,
    ));
    push_attr_opt(
        &mut base,
        "aria-controls",
        trig_attrs.aria_controls().map(str::to_owned),
    );
    if is_disabled {
        base.push(Attribute::new("disabled", true, None, false));
    }
    base.push(Attribute::new(
        "data-state",
        trig_attrs.data_state().as_str(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-orientation",
        trig_attrs.data_orientation(),
        None,
        false,
    ));
    push_bool_data_attr(&mut base, "data-disabled", is_disabled);
    base.push(Attribute::new(
        "onclick",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                if is_disabled {
                    return;
                }
                if !evt.default_action_enabled() {
                    return;
                }
                let _ = runtime_click.select_tab_with_context(
                    &val_click,
                    ChangeContext::new(ChangeReason::TabChange, ChangeSource::Mouse),
                );
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onfocus",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |_: Event<PlatformEventData>| {
                if is_disabled {
                    return;
                }
                runtime_focus.roving().handle_item_focus(&val_focus, true);
                if runtime_focus.tabs().activation_mode().is_automatic() {
                    let _ = runtime_focus.select_tab_with_context(
                        &val_focus,
                        ChangeContext::new(ChangeReason::Focus, ChangeSource::Keyboard),
                    );
                }
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onkeydown",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                if is_disabled {
                    return;
                }
                let kb_evt: KeyboardEvent = evt.map(|d| KeyboardData::from(d));
                if !kb_evt.default_action_enabled() {
                    return;
                }
                if is_activation_key(&kb_evt.key()) {
                    kb_evt.prevent_default();
                    let _ = runtime_key.select_tab_with_context(
                        &val_key,
                        ChangeContext::new(ChangeReason::TabChange, ChangeSource::Keyboard),
                    );
                    return;
                }
                let automatic = runtime_key.tabs().activation_mode().is_automatic();
                let rt = runtime_key.clone();
                runtime_key.roving().handle_keydown(
                    Some(&val_key),
                    &kb_evt,
                    Some(|next_val: &str| {
                        if automatic {
                            let _ = rt.select_tab_with_context(
                                next_val,
                                ChangeContext::new(ChangeReason::TabChange, ChangeSource::Keyboard),
                            );
                        }
                    }),
                );
            })
            .erase(),
        ),
        None,
        false,
    ));

    let mut user = collect_user_attrs(class, style, attributes);
    if !is_disabled {
        push_event_handler(&mut user, "onclick", on_click);
    }
    push_event_handler(&mut user, "onkeydown", on_key_down);
    push_event_handler(&mut user, "onmounted", on_mounted);

    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        button {
            ..merged,
            {children}
        }
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct TabsContentProps {
    #[props(into)]
    pub value: String,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub tabindex: Option<i32>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
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
        child,
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

    let tab_idx = tabindex.unwrap_or(0);
    let mut base = Vec::with_capacity(9);
    base.push(Attribute::new("id", content_id, None, false));
    push_attr_opt(&mut base, "style", guard_style);
    base.push(Attribute::new("role", content_attrs.role(), None, false));
    base.push(Attribute::new("aria-labelledby", labelledby, None, false));
    base.push(Attribute::new(
        "data-state",
        if is_active { "active" } else { "inactive" },
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-orientation",
        content_attrs.data_orientation(),
        None,
        false,
    ));
    base.push(Attribute::new("tabindex", tab_idx.to_string(), None, false));
    if !is_active && force_mount {
        base.push(Attribute::new("hidden", true, None, false));
    }

    let user = collect_user_attrs(class, style, attributes);
    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            {children}
        }
    }
}
