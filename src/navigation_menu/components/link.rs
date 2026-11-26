use dioxus::prelude::keyboard_types::Modifiers;
use dioxus::prelude::*;

use crate::foundation::{
    compose::merge_global_attributes,
    roving_focus::{RovingItemMeta, use_roving_focus_item},
    shared::use_stable_id,
};

use super::super::{
    runtime::{
        NavigationMenuContext, NavigationMenuInContent, NavigationMenuItemContext,
        NavigationMenuLinkRenderProps,
    },
    types::{NavigationMenuLinkSelectDetail, NavigationMenuLinkSelectEvent},
};

#[derive(Clone, PartialEq, Props)]
pub struct NavigationMenuLinkProps {
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
    #[props(default = false)]
    pub active: bool,
    #[props(default)]
    pub on_select: Option<EventHandler<NavigationMenuLinkSelectEvent>>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<NavigationMenuLinkRenderProps, Element>>,
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
        let consumer_on_click = props.on_click;
        let on_select = props.on_select;
        let href = props.href.clone();
        move |evt: MouseEvent| {
            if let Some(ref handler) = consumer_on_click {
                handler.call(evt.clone());
            }
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
        }
    };

    let on_key_down = {
        let ctx = ctx.clone();
        let roving_key = roving_key.clone();
        move |evt: KeyboardEvent| {
            if in_content {
                return;
            }
            ctx.roving
                .handle_keydown(Some(&roving_key), &evt, None::<fn(&str)>);
        }
    };

    let on_focus = {
        let ctx = ctx.clone();
        let link_id = link_id.clone();
        let roving_key = roving_key.clone();
        move |_| {
            is_focused.set(true);
            ctx.record_element_focus(&link_id);
            if !in_content {
                ctx.roving.handle_item_focus(&roving_key, true);
            }
        }
    };

    let on_blur = move |_| {
        is_focused.set(false);
    };

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
        move |_: Event<PointerData>| {
            dismiss();
        }
    };

    let on_pointer_move = {
        let dismiss = handle_pointer_dismiss;
        move |evt: Event<PointerData>| {
            if evt.pointer_type() == "mouse" {
                dismiss();
            }
        }
    };

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(NavigationMenuLinkRenderProps {
                attrs: link_attrs,
                active: props.active,
                is_focused: *is_focused.read(),
                class: props.class,
                style: props.style,
                attributes: props.attributes,
                children: props.children,
            });
        }
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::NavigationMenuLink: as_child requires render_child; rendering the default a"
        );
    }

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        a {
            id: "{link_attrs.id()}",
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref().unwrap_or_default(),
            href: link_attrs.href(),
            target: link_attrs.target(),
            rel: link_attrs.rel(),
            tabindex: "{link_attrs.tabindex()}",
            "aria-current": link_attrs.aria_current(),
            "data-active": link_attrs.data_active(),
            "data-focused": link_attrs.data_focused(),
            "data-monoxus-navigation-menu-link": "{link_attrs.data_monoxus_navigation_menu_link()}",
            onclick: on_click,
            onkeydown: on_key_down,
            onfocus: on_focus,
            onblur: on_blur,
            onpointerenter: on_pointer_enter,
            onpointermove: on_pointer_move,
            onmounted: move |evt| {
                if let Some(ref handler) = props.on_mounted {
                    handler.call(evt);
                }
            },
            ..merged.rest,
            {props.children}
        }
    }
}
