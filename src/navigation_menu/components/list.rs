use dioxus::prelude::*;

use crate::foundation::compose::{collect_user_attrs, merge_props, push_event_handler};

use super::{
    super::runtime::{NavigationMenuContext, NavigationMenuInContent},
    indicator::NavigationMenuIndicatorSlot,
};

#[derive(Clone, PartialEq, Props)]
pub struct NavigationMenuListProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
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

    let base = vec![
        Attribute::new("id", list_attrs.id().to_owned(), None, false),
        Attribute::new("data-orientation", list_attrs.data_orientation(), None, false),
        Attribute::new(
            "data-monoxus-navigation-menu-list",
            list_attrs.data_monoxus_navigation_menu_list(),
            None,
            false,
        ),
    ];
    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler::<MountedData>(&mut user, "onmounted", props.on_mounted);
    let merged = merge_props(base, user);

    let list_el = if let Some(child) = props.child {
        child.call(merged)
    } else {
        rsx! {
            ul {
                ..merged,
                {props.children}
            }
        }
    };

    rsx! {
        div {
            id: "{track_attrs.id()}",
            style: "{track_attrs.style()}",
            "data-monoxus-navigation-menu-track": "{track_attrs.data_monoxus_navigation_menu_track()}",
            {list_el}
            NavigationMenuIndicatorSlot {}
        }
    }
}
