use dioxus::prelude::*;

use crate::foundation::compose::merge_global_attributes;

use super::{
    super::runtime::{NavigationMenuContext, NavigationMenuInContent, NavigationMenuListRenderProps},
    indicator::NavigationMenuIndicatorSlot,
};

#[derive(Clone, PartialEq, Props)]
pub struct NavigationMenuListProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<NavigationMenuListRenderProps, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn NavigationMenuList(props: NavigationMenuListProps) -> Element {
    let ctx = use_context::<NavigationMenuContext>();
    use_context_provider(|| NavigationMenuInContent(false));

    let list_attrs = ctx.list_attributes(props.id.clone());
    let track_attrs = ctx.list_track_attributes(None);
    let track_id = track_attrs.id().to_owned();
    let list_id = list_attrs.id().to_owned();

    ctx.register_track_id(track_id.clone());
    ctx.roving.set_root_dom_id(&list_id);

    let cleanup_ctx = ctx.clone();
    let cleanup_track_id = track_id;
    use_drop(move || {
        cleanup_ctx.unregister_track_id(&cleanup_track_id);
    });

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            let list_el = render_child.call(NavigationMenuListRenderProps {
                attrs: list_attrs,
                track_attrs: track_attrs.clone(),
                class: props.class,
                style: props.style,
                attributes: props.attributes,
                children: props.children,
                indicator: rsx! { NavigationMenuIndicatorSlot {} },
            });
            return rsx! {
                div {
                    id: "{track_attrs.id()}",
                    style: "{track_attrs.style()}",
                    "data-monoxus-navigation-menu-track": "{track_attrs.data_monoxus_navigation_menu_track()}",
                    {list_el}
                    NavigationMenuIndicatorSlot {}
                }
            };
        }
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::NavigationMenuList: as_child requires render_child; rendering the default ul"
        );
    }

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        div {
            id: "{track_attrs.id()}",
            style: "{track_attrs.style()}",
            "data-monoxus-navigation-menu-track": "{track_attrs.data_monoxus_navigation_menu_track()}",
            ul {
                id: "{list_attrs.id()}",
                class: merged.class.as_deref().unwrap_or_default(),
                style: merged.style.as_deref().unwrap_or_default(),
                "data-orientation": "{list_attrs.data_orientation()}",
                "data-monoxus-navigation-menu-list": "{list_attrs.data_monoxus_navigation_menu_list()}",
                onmounted: move |evt| {
                    if let Some(ref handler) = props.on_mounted {
                        handler.call(evt);
                    }
                },
                ..merged.rest,
                {props.children}
            }
            NavigationMenuIndicatorSlot {}
        }
    }
}
