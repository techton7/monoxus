use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_event_handler},
    shared::use_stable_id,
};

use super::super::runtime::{NavigationMenuContext, NavigationMenuItemContext};

#[derive(Clone, PartialEq, Props)]
pub struct NavigationMenuItemProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default, into)]
    pub value: Option<String>,
    #[props(default)]
    pub open_on_hover: Option<bool>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn NavigationMenuItem(props: NavigationMenuItemProps) -> Element {
    let ctx = use_context::<NavigationMenuContext>();
    let stable_id = use_stable_id(None);
    let resolved_value = props
        .value
        .clone()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| stable_id.into_owned());

    let mut value_sig = use_signal(|| resolved_value.clone());
    if *value_sig.peek() != resolved_value {
        value_sig.set(resolved_value.clone());
    }

    let mut open_on_hover_sig = use_signal(|| props.open_on_hover);
    if *open_on_hover_sig.peek() != props.open_on_hover {
        open_on_hover_sig.set(props.open_on_hover);
    }

    let focus_proxy_default = ctx.relationships.focus_proxy_id(&resolved_value);
    let item_ctx = NavigationMenuItemContext {
        value: value_sig,
        open_on_hover: open_on_hover_sig,
        trigger_id: use_signal(|| None),
        content_id: use_signal(|| None),
        focus_proxy_id: use_signal(|| focus_proxy_default),
        was_escape_close: use_signal(|| false),
    };

    use_context_provider(|| item_ctx);

    let item_attrs = ctx.item_attributes(&resolved_value, props.id.clone());

    let base = vec![
        Attribute::new("id", item_attrs.id().to_owned(), None, false),
        Attribute::new(
            "data-monoxus-navigation-menu-item",
            item_attrs.data_monoxus_navigation_menu_item(),
            None,
            false,
        ),
    ];
    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler::<MountedData>(&mut user, "onmounted", props.on_mounted);
    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        li {
            ..merged,
            {props.children}
        }
    }
}
