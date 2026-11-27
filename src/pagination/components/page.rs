use dioxus::core::{AttributeValue, Event, ListenerCallback};
use dioxus::html::PlatformEventData;
use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
    keyboard::is_activation_key,
    roving_focus::{RovingItemMeta, use_roving_focus_item},
};

use super::super::{
    relationships::page_roving_key,
    runtime::use_pagination_context,
    types::PageData,
};

#[derive(Clone, PartialEq, Props)]
pub struct PaginationPageProps {
    #[props(default, into)]
    pub id: Option<String>,
    pub page: PageData,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default, into)]
    pub aria_label: Option<String>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Option<Element>,
}

#[component]
pub fn PaginationPage(props: PaginationPageProps) -> Element {
    let ctx = use_pagination_context();
    let runtime = ctx.runtime;

    let mut page_attrs =
        runtime.page_attributes(props.page.value, props.id.clone(), props.disabled);
    if let Some(ref custom_label) = props.aria_label {
        page_attrs.aria_label = custom_label.clone();
    }

    let is_selected = page_attrs.is_selected();
    let effective_disabled = page_attrs.is_disabled();
    let page_value = props.page.value;
    let roving_key = page_roving_key(page_value);

    use_roving_focus_item(
        runtime.roving,
        &roving_key,
        RovingItemMeta::new(page_attrs.id.clone(), !effective_disabled, is_selected),
    );

    let runtime_click = runtime.clone();
    let runtime_key = runtime.clone();
    let roving_key_down = roving_key.clone();
    let runtime_focus = runtime;

    let mut base = Vec::with_capacity(14);
    base.push(Attribute::new("id", page_attrs.id, None, false));
    base.push(Attribute::new("type", page_attrs.r#type, None, false));
    base.push(Attribute::new("aria-label", page_attrs.aria_label, None, false));
    push_attr_opt(&mut base, "aria-current", page_attrs.aria_current);
    push_attr_opt(&mut base, "aria-disabled", page_attrs.aria_disabled);
    if page_attrs.disabled {
        base.push(Attribute::new("disabled", true, None, false));
    }
    base.push(Attribute::new("data-value", page_attrs.data_value, None, false));
    push_attr_opt(&mut base, "data-selected", page_attrs.data_selected);
    push_attr_opt(&mut base, "data-disabled", page_attrs.data_disabled);
    base.push(Attribute::new(
        "data-monoxus-pagination-page",
        page_attrs.data_monoxus_pagination_page,
        None,
        false,
    ));
    base.push(Attribute::new(
        "onclick",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                let mouse_evt: MouseEvent = evt.map(|d| MouseData::from(d));
                if mouse_evt
                    .trigger_button()
                    .is_some_and(|b| b != dioxus::html::input_data::MouseButton::Primary)
                {
                    return;
                }
                if effective_disabled {
                    mouse_evt.prevent_default();
                    return;
                }
                if mouse_evt.default_action_enabled() {
                    runtime_click.set_page(page_value);
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
                if is_activation_key(&kb_evt.key()) {
                    if effective_disabled {
                        kb_evt.prevent_default();
                        return;
                    }
                    if kb_evt.default_action_enabled() {
                        kb_evt.prevent_default();
                        runtime_key.set_page(page_value);
                    }
                    return;
                }
                if kb_evt.default_action_enabled() {
                    runtime_key.roving.handle_keydown(
                        Some(&roving_key_down),
                        &kb_evt,
                        None::<fn(&str)>,
                    );
                }
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
                runtime_focus
                    .roving
                    .handle_item_focus(&roving_key, !effective_disabled);
            })
            .erase(),
        ),
        None,
        false,
    ));

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    if !effective_disabled {
        push_event_handler(&mut user, "onclick", props.on_click);
    }
    push_event_handler(&mut user, "onkeydown", props.on_key_down);
    push_event_handler(&mut user, "onmounted", props.on_mounted);

    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        button {
            ..merged,
            if let Some(children) = props.children {
                {children}
            } else {
                "{page_attrs.value}"
            }
        }
    }
}
