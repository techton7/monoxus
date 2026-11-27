use dioxus::core::{AttributeValue, Event, ListenerCallback};
use dioxus::html::PlatformEventData;
use dioxus::prelude::keyboard_types::Key;
use dioxus::prelude::*;

use crate::foundation::{
    browser::{click_element_by_id, is_element_focused_by_id},
    compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
    roving_focus::{RovingItemMeta, use_roving_focus_item},
    shared::use_stable_id,
};
use crate::toolbar::{components::ToolbarRootContext, state::Toolbar};

#[derive(Clone, PartialEq, Props)]
pub struct ToolbarLinkProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default, into)]
    pub href: Option<String>,
    #[props(default, into)]
    pub target: Option<String>,
    #[props(default, into)]
    pub rel: Option<String>,
    #[props(default, into)]
    pub download: Option<String>,
    #[props(default, into)]
    pub value: Option<String>,
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

    let roving_for_key = runtime.roving;
    let key_for_roving = roving_key.clone();
    let link_id_for_key = link_id;
    let roving_for_focus = runtime.roving;

    let mut base = Vec::with_capacity(12);
    base.push(Attribute::new("id", attrs.id().to_string(), None, false));
    push_attr_opt(&mut base, "href", attrs.href().map(str::to_owned));
    push_attr_opt(&mut base, "target", props.target);
    push_attr_opt(&mut base, "rel", props.rel);
    push_attr_opt(&mut base, "download", props.download);
    base.push(Attribute::new(
        "tabindex",
        attrs.tabindex().to_string(),
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
        "data-toolbar-link",
        attrs.data_toolbar_link(),
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
        "onkeydown",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                let kb_evt: KeyboardEvent = evt.map(|d| KeyboardData::from(d));
                if !kb_evt.default_action_enabled() {
                    return;
                }
                if let Key::Character(ref ch) = kb_evt.key() {
                    let is_focused_link = is_element_focused_by_id(&link_id_for_key);
                    if Toolbar::should_activate_link_on_key(ch, is_focused_link) {
                        kb_evt.prevent_default();
                        click_element_by_id(&link_id_for_key);
                        return;
                    }
                }
                roving_for_key.handle_keydown(
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
                roving_for_focus.handle_item_focus(&roving_key, true);
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
        a {
            ..merged,
            {props.children}
        }
    }
}
