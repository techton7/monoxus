use dioxus::prelude::keyboard_types::Key;
use dioxus::prelude::*;

use crate::foundation::{
    browser::{click_element_by_id, is_element_focused_by_id},
    roving_focus::{RovingItemMeta, use_roving_focus_item},
    shared::use_stable_id,
};
use crate::toolbar::{components::ToolbarRootContext, state::Toolbar};

#[derive(Clone, PartialEq, Props)]
pub struct ToolbarLinkProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub href: Option<String>,
    #[props(default)]
    pub target: Option<String>,
    #[props(default)]
    pub rel: Option<String>,
    #[props(default)]
    pub download: Option<String>,
    #[props(default)]
    pub value: Option<String>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn ToolbarLink(props: ToolbarLinkProps) -> Element {
    let ctx = use_context::<ToolbarRootContext>();
    let runtime = ctx.runtime;
    let toolbar_state = runtime.state();

    let stable_id = use_stable_id(None);
    let link_id = props
        .id
        .clone()
        .unwrap_or_else(|| toolbar_state.relationships().link_id(&stable_id));
    let roving_key = props
        .value
        .clone()
        .unwrap_or_else(|| format!("link:{link_id}"));

    use_roving_focus_item(
        runtime.roving,
        &roving_key,
        RovingItemMeta::new(link_id.clone(), true, false),
    );

    let tabindex = runtime.roving.tab_index_for(&roving_key, true);
    let attrs = toolbar_state.link_attributes(link_id.clone(), props.href.clone(), tabindex);

    let on_click = {
        let consumer_on_click = props.on_click;
        move |evt: MouseEvent| {
            if let Some(ref handler) = consumer_on_click {
                handler.call(evt);
            }
        }
    };

    let on_key_down = {
        let roving = runtime.roving;
        let key_for_roving = roving_key.clone();
        let link_id_for_key = link_id.clone();
        let consumer_on_keydown = props.on_key_down;
        move |evt: KeyboardEvent| {
            if let Some(ref handler) = consumer_on_keydown {
                handler.call(evt.clone());
            }
            if !evt.default_action_enabled() {
                return;
            }
            if let Key::Character(ref ch) = evt.key() {
                let is_focused_link = is_element_focused_by_id(&link_id_for_key);
                if Toolbar::should_activate_link_on_key(ch, is_focused_link) {
                    evt.prevent_default();
                    click_element_by_id(&link_id_for_key);
                    return;
                }
            }
            roving.handle_keydown(Some(&key_for_roving), &evt, None::<fn(&str)>);
        }
    };

    let on_focus = {
        let roving = runtime.roving;
        let key_for_focus = roving_key;
        move |_| {
            roving.handle_item_focus(&key_for_focus, true);
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

    rsx! {
        a {
            id: "{attrs.id()}",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            href: attrs.href(),
            target: props.target.as_deref(),
            rel: props.rel.as_deref(),
            download: props.download.as_deref(),
            tabindex: attrs.tabindex(),
            "data-orientation": attrs.data_orientation(),
            "data-toolbar-link": attrs.data_toolbar_link(),
            "data-toolbar-item": attrs.data_toolbar_item(),
            onclick: on_click,
            onkeydown: on_key_down,
            onfocus: on_focus,
            onmounted: on_mounted,
            ..props.attributes,
            {props.children}
        }
    }
}
