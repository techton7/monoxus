use dioxus::prelude::keyboard_types::Modifiers;
use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
    roving_focus::{RovingItemMeta, use_roving_focus_item},
    shared::use_stable_id,
};

use super::super::{
    runtime::{NavigationMenuContext, NavigationMenuInContent, NavigationMenuItemContext},
    types::{NavigationMenuLinkSelectDetail, NavigationMenuLinkSelectEvent},
};

#[derive(Clone, PartialEq, Props)]
pub struct NavigationMenuLinkProps {
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
    #[props(default = false)]
    pub active: bool,
    #[props(default)]
    pub on_select: Option<EventHandler<NavigationMenuLinkSelectEvent>>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn NavigationMenuLink(props: NavigationMenuLinkProps) -> Element {
    let ctx = use_context::<NavigationMenuContext>();
    let item_ctx = try_consume_context::<NavigationMenuItemContext>();
    let in_content = try_consume_context::<NavigationMenuInContent>()
        .map(|marker| marker.0)
        .unwrap_or(false);

    let stable_id = use_stable_id(None);
    let link_key = stable_id.into_owned();
    let mut is_focused = use_signal(|| false);

    let link_attrs = ctx.link_attributes(
        &link_key,
        props.href.clone(),
        props.target.clone(),
        props.rel.clone(),
        props.active,
        *is_focused.read(),
        props.id.clone(),
    );
    let link_id = link_attrs.id().to_owned();
    let roving_key = format!("link:{link_id}");

    if !in_content {
        use_roving_focus_item(
            ctx.roving,
            &roving_key,
            RovingItemMeta::new(link_id.clone(), true, false),
        );
    }

    let on_click = {
        let ctx = ctx.clone();
        let on_select = props.on_select;
        let href = props.href.clone();
        EventHandler::new(move |evt: MouseEvent| {
            let select_event = NavigationMenuLinkSelectEvent::new(NavigationMenuLinkSelectDetail {
                href: href.clone(),
                meta_key: evt.modifiers().contains(Modifiers::META),
            });
            if let Some(ref handler) = on_select {
                handler.call(select_event.clone());
            }
            if !select_event.is_default_prevented() && !select_event.detail().meta_key {
                if let Some(ref item) = item_ctx {
                    item.on_root_content_close(&ctx.relationships);
                }
                ctx.dismiss_root();
            }
        })
    };

    let on_key_down = {
        let ctx = ctx.clone();
        let roving_key = roving_key.clone();
        EventHandler::new(move |evt: KeyboardEvent| {
            if in_content {
                return;
            }
            ctx.roving
                .handle_keydown(Some(&roving_key), &evt, None::<fn(&str)>);
        })
    };

    let on_focus = {
        let ctx = ctx.clone();
        let link_id = link_id.clone();
        let roving_key = roving_key.clone();
        EventHandler::new(move |_: FocusEvent| {
            is_focused.set(true);
            ctx.record_element_focus(&link_id);
            if !in_content {
                ctx.roving.handle_item_focus(&roving_key, true);
            }
        })
    };

    let on_blur = EventHandler::new(move |_: FocusEvent| {
        is_focused.set(false);
    });

    let handle_pointer_dismiss = {
        let ctx = ctx.clone();
        move || {
            let open_val = ctx
                .value
                .try_peek()
                .map(|v| v.clone())
                .unwrap_or_default();
            if open_val.is_empty() {
                return;
            }
            let inside_open_item = in_content
                && item_ctx
                    .as_ref()
                    .is_some_and(|item| item.value_str() == open_val);
            if !inside_open_item && ctx.active_item_open_on_hover() {
                ctx.on_item_dismiss();
            }
        }
    };

    let on_pointer_enter = {
        let dismiss = handle_pointer_dismiss.clone();
        EventHandler::new(move |_: PointerEvent| {
            dismiss();
        })
    };

    let on_pointer_move = {
        let dismiss = handle_pointer_dismiss;
        EventHandler::new(move |evt: PointerEvent| {
            if evt.pointer_type() == "mouse" {
                dismiss();
            }
        })
    };

    let mut base = Vec::with_capacity(16);
    base.push(Attribute::new("id", link_attrs.id().to_owned(), None, false));
    push_attr_opt(&mut base, "href", link_attrs.href().map(str::to_owned));
    push_attr_opt(&mut base, "target", link_attrs.target().map(str::to_owned));
    push_attr_opt(&mut base, "rel", link_attrs.rel().map(str::to_owned));
    base.push(Attribute::new("tabindex", link_attrs.tabindex(), None, false));
    push_attr_opt(&mut base, "aria-current", link_attrs.aria_current());
    push_attr_opt(&mut base, "data-active", link_attrs.data_active());
    push_attr_opt(&mut base, "data-focused", link_attrs.data_focused());
    base.push(Attribute::new(
        "data-monoxus-navigation-menu-link",
        link_attrs.data_monoxus_navigation_menu_link(),
        None,
        false,
    ));
    push_event_handler::<MouseData>(&mut base, "onclick", Some(on_click));
    push_event_handler::<KeyboardData>(&mut base, "onkeydown", Some(on_key_down));
    push_event_handler::<FocusData>(&mut base, "onfocus", Some(on_focus));
    push_event_handler::<FocusData>(&mut base, "onblur", Some(on_blur));
    push_event_handler::<PointerData>(&mut base, "onpointerenter", Some(on_pointer_enter));
    push_event_handler::<PointerData>(&mut base, "onpointermove", Some(on_pointer_move));

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler::<MouseData>(&mut user, "onclick", props.on_click);
    push_event_handler::<MountedData>(&mut user, "onmounted", props.on_mounted);
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
