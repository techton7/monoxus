use dioxus::prelude::*;

use crate::foundation::shared::use_stable_id;

use super::super::runtime::{
    NavigationMenuContext, NavigationMenuItemContext, NavigationMenuItemRenderProps,
};

#[derive(Clone, PartialEq, Props)]
pub struct NavigationMenuItemProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub value: Option<String>,
    #[props(default)]
    pub open_on_hover: Option<bool>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<NavigationMenuItemRenderProps, Element>>,
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
    let is_open = ctx.current_value() == resolved_value;

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(NavigationMenuItemRenderProps {
                attrs: item_attrs,
                value: resolved_value,
                is_open,
                class: props.class,
                style: props.style,
                attributes: props.attributes,
                children: props.children,
            });
        }
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::NavigationMenuItem: as_child requires render_child; rendering the default li"
        );
    }

    rsx! {
        li {
            id: "{item_attrs.id()}",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            "data-monoxus-navigation-menu-item": "{item_attrs.data_monoxus_navigation_menu_item()}",
            onmounted: move |evt| {
                if let Some(ref handler) = props.on_mounted {
                    handler.call(evt);
                }
            },
            ..props.attributes,
            {props.children}
        }
    }
}
