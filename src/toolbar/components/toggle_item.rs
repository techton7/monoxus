use dioxus::core::{AttributeValue, Event, ListenerCallback};
use dioxus::html::PlatformEventData;
use dioxus::prelude::*;
use ui_sdk::kernel::{ChangeContext, ChangeReason, ChangeSource};

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
    keyboard::is_activation_key,
    roving_focus::{RovingItemMeta, use_roving_focus_item},
    shared::use_stable_id,
};
use crate::toolbar::components::{ToolbarRootContext, ToolbarToggleGroupContext};

#[derive(Clone, PartialEq, Props)]
pub struct ToolbarToggleItemProps {
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
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn ToolbarToggleItem(props: ToolbarToggleItemProps) -> Element {
    let toolbar_ctx = use_context::<ToolbarRootContext>();
    let group_ctx = use_context::<ToolbarToggleGroupContext>();
    let toolbar_runtime = toolbar_ctx.runtime;
    let group_runtime = group_ctx.runtime;
    let toolbar_state = toolbar_runtime.state();

    let stable_id = use_stable_id(None);
    let group_dom_id = group_runtime.group_dom_id();
    let item_id = props.id.clone().unwrap_or_else(|| {
        format!(
            "{}-{}",
            toolbar_state
                .relationships()
                .toggle_item_id(&group_dom_id, &props.value),
            stable_id
        )
    });
    let roving_key = format!("tg:{group_dom_id}:{}", props.value);

    let effective_disabled = group_runtime.is_effective_disabled(props.disabled);
    let is_pressed = group_runtime.is_item_pressed(&props.value);

    use_roving_focus_item(
        toolbar_runtime.roving,
        &roving_key,
        RovingItemMeta::new(item_id.clone(), !effective_disabled, is_pressed),
    );

    let tabindex = toolbar_runtime
        .roving
        .tab_index_for(&roving_key, !effective_disabled);
    let attrs = group_runtime.item_attributes(
        &toolbar_state,
        &props.value,
        Some(item_id),
        props.disabled,
        tabindex,
    );

    let group_runtime_click = group_runtime.clone();
    let roving_click = toolbar_runtime.roving;
    let val_click = props.value.clone();
    let key_for_click = roving_key.clone();

    let group_runtime_key = group_runtime;
    let roving_key_down = toolbar_runtime.roving;
    let val_key = props.value.clone();
    let key_for_roving = roving_key.clone();

    let roving_focus = toolbar_runtime.roving;

    let mut base = Vec::with_capacity(16);
    base.push(Attribute::new("id", attrs.id().to_string(), None, false));
    base.push(Attribute::new("type", attrs.button_type(), None, false));
    push_attr_opt(&mut base, "role", attrs.role());
    push_attr_opt(&mut base, "aria-checked", attrs.aria_checked());
    push_attr_opt(&mut base, "aria-pressed", attrs.aria_pressed());
    base.push(Attribute::new(
        "tabindex",
        attrs.tabindex().to_string(),
        None,
        false,
    ));
    if attrs.is_disabled() {
        base.push(Attribute::new("disabled", true, None, false));
    }
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled());
    base.push(Attribute::new(
        "data-state",
        attrs.data_state().as_str(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-value",
        attrs.data_value().to_string(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-orientation",
        attrs.data_orientation(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-toolbar-group-item",
        attrs.data_toolbar_group_item(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-toolbar-item",
        attrs.data_toolbar_item(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onclick",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                if evt.default_action_enabled() && !effective_disabled {
                    let outcome = group_runtime_click.toggle_item_with_context(
                        &val_click,
                        ChangeContext::new(ChangeReason::Toggle, ChangeSource::Mouse),
                    );
                    if outcome.is_accepted() {
                        roving_click.set_current_tab_stop(Some(key_for_click.clone()));
                    }
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
                let kb_evt: KeyboardEvent = evt.map(|d| KeyboardData::from(d));
                if !kb_evt.default_action_enabled() || effective_disabled {
                    return;
                }
                if is_activation_key(&kb_evt.key()) {
                    kb_evt.prevent_default();
                    let outcome = group_runtime_key.toggle_item_with_context(
                        &val_key,
                        ChangeContext::new(ChangeReason::Toggle, ChangeSource::Keyboard),
                    );
                    if outcome.is_accepted() {
                        roving_key_down.set_current_tab_stop(Some(key_for_roving.clone()));
                    }
                    return;
                }
                roving_key_down.handle_keydown(
                    Some(&key_for_roving),
                    &kb_evt,
                    None::<fn(&str)>,
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
                roving_focus.handle_item_focus(&roving_key, !effective_disabled);
            })
            .erase(),
        ),
        None,
        false,
    ));

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler(&mut user, "onclick", props.on_click);
    push_event_handler(&mut user, "onkeydown", props.on_key_down);
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
