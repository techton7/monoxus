use std::borrow::Cow;

use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_event_handler},
    shared::{Orientation, use_stable_id},
    state::use_controllable_signal,
};

use super::super::{
    relationships::NavigationMenuRelationships,
    runtime::{
        NavigationMenuContext, NavigationMenuInContent, NavigationMenuRootDismissContext,
        use_navigation_menu_sub_runtime,
    },
    types::NavigationMenuActivationMode,
};

#[derive(Clone, PartialEq, Props)]
pub struct NavigationMenuSubProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default, into)]
    pub value: Option<String>,
    #[props(default, into)]
    pub default_value: Option<String>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<String>>,
    #[props(default = Orientation::Horizontal)]
    pub orientation: Orientation,
    #[props(default)]
    pub activation_mode: Option<NavigationMenuActivationMode>,
    #[props(default = true)]
    pub disable_toggle: bool,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn NavigationMenuSub(props: NavigationMenuSubProps) -> Element {
    let parent_ctx = use_context::<NavigationMenuContext>();
    let root_dismiss = try_consume_context::<NavigationMenuRootDismissContext>();
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned));
    let scope = parent_ctx.relationships.sub_scope(&stable_id);
    let initial_rels = NavigationMenuRelationships::new(scope.clone());
    let dom_id = props
        .id
        .clone()
        .unwrap_or_else(|| initial_rels.root_id().to_owned());

    let value_state = use_controllable_signal(
        props.value,
        props.default_value.unwrap_or_default(),
        props.on_value_change,
    );

    let effective_mode = props
        .activation_mode
        .unwrap_or_else(|| parent_ctx.current_activation_mode());
    let inherited_dir = parent_ctx.current_dir();

    let ctx = use_navigation_menu_sub_runtime(
        &parent_ctx,
        scope,
        dom_id.clone(),
        value_state,
        props.orientation,
        Some(effective_mode),
        props.disable_toggle,
        root_dismiss.map(|d| d.on_dismiss),
    );

    ctx.sync_sub_props(
        &dom_id,
        inherited_dir,
        props.orientation,
        effective_mode,
        props.disable_toggle,
    );

    use_context_provider(|| ctx.clone());
    use_context_provider(|| NavigationMenuInContent(false));

    let sub_attrs = ctx.sub_attributes(Some(dom_id));

    let base = vec![
        Attribute::new("id", sub_attrs.id().to_owned(), None, false),
        Attribute::new("data-orientation", sub_attrs.data_orientation(), None, false),
        Attribute::new(
            "data-monoxus-navigation-menu-sub",
            sub_attrs.data_monoxus_navigation_menu_sub(),
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
        div {
            ..merged,
            {props.children}
        }
    }
}
