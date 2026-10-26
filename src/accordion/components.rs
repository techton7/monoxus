use dioxus::html::{AnimationData, TransitionData};
use dioxus::prelude::*;

use crate::foundation::{
    browser::{start_presence_monitor, PresenceMonitorEvent, WatcherGuard},
    overlay::PresenceController,
    shared::ScopeHandle,
};

use super::{
    runtime::{use_accordion_runtime, AccordionRuntime},
    state::Accordion,
    types::{AccordionDirection, AccordionMode, AccordionOrientation},
};

#[derive(Clone)]
pub struct AccordionContext {
    pub runtime: AccordionRuntime,
}

#[component]
pub fn AccordionRoot(
    #[props(default)] id: Option<String>,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    #[props(default)] mode: AccordionMode,
    #[props(default)] orientation: AccordionOrientation,
    #[props(default)] dir: AccordionDirection,
    #[props(default = true)] loop_focus: bool,
    #[props(default = false)] disabled: bool,
    #[props(default)] default_value: Option<String>,
    #[props(default)] default_values: Option<Vec<String>>,
    #[props(default)] on_value_change: Option<EventHandler<Vec<String>>>,
    children: Element,
) -> Element {
    let scope_id = id.clone().unwrap_or_else(|| "accordion".to_string());
    let scope = ScopeHandle::root("accordion").child(scope_id);

    let mut accordion = Accordion::new(scope, mode)
        .with_orientation(orientation)
        .with_direction(dir)
        .with_loop_focus(loop_focus)
        .with_disabled(disabled);

    if let Some(vals) = default_values {
        accordion = accordion.with_values(vals);
    } else if let Some(val) = default_value {
        accordion = accordion.with_value(val);
    }

    let change_handler = on_value_change.map(|handler| move |vals| handler.call(vals));
    let runtime = use_accordion_runtime(accordion, change_handler);

    use_context_provider(|| AccordionContext {
        runtime: runtime.clone(),
    });

    let attrs = runtime.root();

    rsx! {
        div {
            id: "{attrs.id()}",
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            "data-orientation": "{attrs.data_orientation()}",
            "data-disabled": if attrs.is_disabled() { "true" } else { "false" },
            {children}
        }
    }
}

#[derive(Clone)]
pub struct AccordionItemContext {
    pub value: String,
    pub disabled: bool,
}

#[component]
pub fn AccordionItem(
    value: String,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    #[props(default = false)] disabled: bool,
    children: Element,
) -> Element {
    let ctx = use_context::<AccordionContext>();
    let item_value = value.clone();

    use_context_provider(|| AccordionItemContext {
        value: item_value.clone(),
        disabled,
    });

    use_effect(use_reactive((&item_value, &disabled), {
        let runtime = ctx.runtime.clone();
        move |(val, dis)| {
            runtime.register_item(&val, dis);
        }
    }));

    let attrs = ctx.runtime.item(&value, disabled);

    rsx! {
        div {
            id: "{attrs.id()}",
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            "data-state": "{attrs.data_state_str()}",
            "data-orientation": "{attrs.data_orientation()}",
            "data-value": "{attrs.data_value()}",
            "data-disabled": if attrs.is_disabled() { "true" } else { "false" },
            {children}
        }
    }
}

#[component]
pub fn AccordionHeader(
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    let ctx = use_context::<AccordionContext>();
    let item_ctx = use_context::<AccordionItemContext>();
    let attrs = ctx.runtime.header(&item_ctx.value, item_ctx.disabled);

    rsx! {
        h3 {
            id: "{attrs.id()}",
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            role: "{attrs.role()}",
            "aria-level": "{attrs.aria_level()}",
            "data-heading-level": "{attrs.data_heading_level()}",
            "data-state": "{attrs.data_state_str()}",
            "data-orientation": "{attrs.data_orientation()}",
            "data-value": "{attrs.data_value()}",
            "data-disabled": if attrs.is_disabled() { "true" } else { "false" },
            {children}
        }
    }
}

#[component]
pub fn AccordionTrigger(
    #[props(default)] id: Option<String>,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    let ctx = use_context::<AccordionContext>();
    let item_ctx = use_context::<AccordionItemContext>();
    let attrs = ctx.runtime.trigger(&item_ctx.value, item_ctx.disabled);
    let trig_id = id.unwrap_or_else(|| attrs.id().to_string());

    let val = item_ctx.value.clone();
    let trig_id_for_effect = trig_id.clone();
    use_effect(use_reactive((&val, &trig_id_for_effect), {
        let runtime = ctx.runtime.clone();
        move |(v, el_id)| {
            runtime.register_trigger_id(&v, &el_id);
        }
    }));

    let click_runtime = ctx.runtime.clone();
    let click_val = item_ctx.value.clone();

    let key_runtime = ctx.runtime.clone();

    rsx! {
        button {
            r#type: "button",
            id: "{trig_id}",
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            "aria-expanded": "{attrs.aria_expanded()}",
            "aria-controls": "{attrs.aria_controls()}",
            "aria-disabled": attrs.aria_disabled().unwrap_or_default(),
            tabindex: attrs.tabindex(),
            disabled: attrs.is_disabled(),
            "data-state": "{attrs.data_state_str()}",
            "data-orientation": "{attrs.data_orientation()}",
            "data-value": "{attrs.data_value()}",
            onclick: move |_| {
                click_runtime.toggle_item(&click_val);
            },
            onkeydown: move |evt: KeyboardEvent| {
                let key = evt.key();
                let key_str = key.to_string();
                if Accordion::is_navigation_key(&key_str) {
                    evt.prevent_default();
                    key_runtime.navigate_key(&key_str);
                }
            },
            {children}
        }
    }
}

#[component]
pub fn AccordionContent(
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    let ctx = use_context::<AccordionContext>();
    let item_ctx = use_context::<AccordionItemContext>();
    let attrs = ctx.runtime.content(&item_ctx.value);

    let is_open = ctx.runtime.is_open(&item_ctx.value);
    let mut presence = use_signal(|| PresenceController::new(is_open).with_retained_mount(true));
    let mut monitor = use_signal(|| None::<WatcherGuard>);

    use_effect(use_reactive((&is_open,), {
        let content_id = attrs.id().to_string();
        move |(open,)| {
            let update = presence.write().sync(open);
            if let Some(cycle_id) = update.started_close_cycle() {
                let watcher =
                    start_presence_monitor(
                        content_id.as_str(),
                        cycle_id,
                        move |event| match event {
                            PresenceMonitorEvent::TransitionEnd {
                                cycle_id: event_cycle,
                                ..
                            }
                            | PresenceMonitorEvent::TransitionCancel {
                                cycle_id: event_cycle,
                                ..
                            }
                            | PresenceMonitorEvent::AnimationEnd {
                                cycle_id: event_cycle,
                                ..
                            }
                            | PresenceMonitorEvent::AnimationCancel {
                                cycle_id: event_cycle,
                                ..
                            }
                            | PresenceMonitorEvent::Fallback {
                                cycle_id: event_cycle,
                                ..
                            } => {
                                presence.write().complete_close_cycle(event_cycle);
                            }
                            _ => {}
                        },
                    );
                monitor.set(Some(watcher));
            } else if !update.should_render() || update.invalidated_close_cycle().is_some() {
                monitor.set(None);
            }
        }
    }));

    let hidden = (!presence.read().should_render()).then_some(true);

    rsx! {
        div {
            id: "{attrs.id()}",
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            role: "{attrs.role()}",
            "aria-labelledby": "{attrs.aria_labelledby()}",
            hidden: hidden,
            "data-state": "{attrs.data_state_str()}",
            "data-orientation": "{attrs.data_orientation()}",
            "data-value": "{attrs.data_value()}",
            ontransitionend: move |_evt: Event<TransitionData>| {
                let cycle = presence.read().active_close_cycle();
                if let Some(cycle) = cycle {
                    presence.write().complete_close_cycle(cycle);
                }
            },
            onanimationend: move |_evt: Event<AnimationData>| {
                let cycle = presence.read().active_close_cycle();
                if let Some(cycle) = cycle {
                    presence.write().complete_close_cycle(cycle);
                }
            },
            {children}
        }
    }
}
