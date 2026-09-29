use dioxus::prelude::*;

use crate::foundation::shared::ScopeHandle;

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

#[component]
pub fn TabsRoot(
    #[props(default)] id: Option<String>,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    #[props(default)] value: Option<Signal<String>>,
    #[props(default)] default_value: Option<String>,
    #[props(default)] on_value_change: Option<EventHandler<String>>,
    #[props(default)] orientation: Option<TabsOrientation>,
    #[props(default)] dir: Option<TabsDirection>,
    #[props(default)] activation_mode: Option<TabsActivationMode>,
    children: Element,
) -> Element {
    let initial_val = default_value.unwrap_or_default();
    let val_signal = value.unwrap_or_else(|| use_signal(|| initial_val.clone()));
    let scope_id = id.clone().unwrap_or_else(|| "tabs".to_string());
    let scope = ScopeHandle::root("tabs").child(scope_id);

    let mut tabs_def = Tabs::new(scope, val_signal());
    if let Some(o) = orientation {
        tabs_def = tabs_def.with_orientation(o);
    }
    if let Some(d) = dir {
        tabs_def = tabs_def.with_direction(d);
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

    rsx! {
        div {
            id: "{root_attrs.id()}",
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            "data-orientation": "{root_attrs.data_orientation()}",
            {children}
        }
    }
}

#[component]
pub fn TabsList(
    #[props(default)] id: Option<String>,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    #[props(default = true)] loop_focus: bool,
    children: Element,
) -> Element {
    let ctx = use_context::<TabsContext>();
    let list_attrs = ctx.runtime.list();
    let list_id = id.unwrap_or_else(|| list_attrs.id().to_string());

    let runtime = ctx.runtime.clone();
    let on_keydown = move |evt: KeyboardEvent| {
        let key = evt.key().to_string();
        if Tabs::is_navigation_key(&key) {
            evt.prevent_default();
            runtime.navigate_key(&key);
        }
    };

    rsx! {
        div {
            id: "{list_id}",
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            role: "{list_attrs.role()}",
            aria_orientation: "{list_attrs.aria_orientation()}",
            "data-orientation": "{list_attrs.data_orientation()}",
            tabindex: "-1",
            onkeydown: on_keydown,
            {children}
        }
    }
}

#[component]
pub fn TabsTrigger(
    value: String,
    #[props(default = false)] disabled: bool,
    #[props(default)] id: Option<String>,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    let ctx = use_context::<TabsContext>();
    let item_value = value.clone();

    use_effect(use_reactive((&item_value, &disabled), {
        let runtime = ctx.runtime.clone();
        move |(val, dis)| {
            runtime.register_trigger(&val, dis);
        }
    }));

    let trig_attrs = ctx.runtime.trigger(&value, disabled);
    let trig_id = id.unwrap_or_else(|| trig_attrs.id().to_string());

    let runtime_click = ctx.runtime.clone();
    let val_click = value.clone();
    let on_click = move |_| {
        if !disabled {
            runtime_click.select_tab(&val_click);
        }
    };

    let runtime_focus = ctx.runtime.clone();
    let val_focus = value.clone();
    let on_focus = move |_| {
        if !disabled && runtime_focus.tabs().activation_mode().is_automatic() {
            runtime_focus.select_tab(&val_focus);
        }
    };

    rsx! {
        button {
            id: "{trig_id}",
            r#type: "button",
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            role: "{trig_attrs.role()}",
            tabindex: "{trig_attrs.tabindex()}",
            aria_selected: "{trig_attrs.aria_selected()}",
            aria_controls: "{trig_attrs.aria_controls()}",
            "data-state": trig_attrs.data_state().as_str(),
            "data-orientation": "{trig_attrs.data_orientation()}",
            "data-disabled": if disabled { "true" } else { "false" },
            onclick: on_click,
            onfocus: on_focus,
            {children}
        }
    }
}

#[component]
pub fn TabsContent(
    value: String,
    #[props(default = false)] force_mount: bool,
    #[props(default)] id: Option<String>,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    let ctx = use_context::<TabsContext>();
    let is_active = ctx.runtime.active_value() == value;

    if !is_active && !force_mount {
        return rsx! {};
    }

    let content_attrs = ctx.runtime.content(&value);
    let content_id = id.unwrap_or_else(|| content_attrs.id().to_string());

    rsx! {
        div {
            id: "{content_id}",
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            role: "{content_attrs.role()}",
            aria_labelledby: "{content_attrs.aria_labelledby()}",
            "data-state": if is_active { "active" } else { "inactive" },
            "data-orientation": "{content_attrs.data_orientation()}",
            tabindex: "0",
            hidden: if !is_active && force_mount { true } else { false },
            {children}
        }
    }
}
