use std::borrow::Cow;

use dioxus::prelude::*;

use crate::foundation::{
    compose::merge_global_attributes,
    shared::{Orientation, use_stable_id},
};

use super::super::{
    relationships::NavigationMenuRelationships,
    runtime::{
        NavigationMenuContext, NavigationMenuInContent, NavigationMenuRootDismissContext,
        NavigationMenuSubRenderProps, use_navigation_menu_sub_runtime,
    },
    types::NavigationMenuActivationMode,
};

#[derive(Clone, PartialEq, Props)]
pub struct NavigationMenuSubProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub value: Option<Signal<String>>,
    #[props(default)]
    pub default_value: Option<String>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<String>>,
    #[props(default = Orientation::Horizontal)]
    pub orientation: Orientation,
    #[props(default)]
    pub activation_mode: Option<NavigationMenuActivationMode>,
    #[props(default = true)]
    pub disable_toggle: bool,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<NavigationMenuSubRenderProps, Element>>,
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

    let fallback_value = use_signal(|| props.default_value.clone().unwrap_or_default());
    let value_signal = props.value.unwrap_or(fallback_value);

    let effective_mode = props
        .activation_mode
        .unwrap_or_else(|| parent_ctx.current_activation_mode());
    let inherited_dir = parent_ctx.current_dir();

    let ctx = use_navigation_menu_sub_runtime(
        &parent_ctx,
        scope,
        dom_id.clone(),
        value_signal,
        props.on_value_change,
        props.orientation,
        Some(effective_mode),
        props.disable_toggle,
        root_dismiss.map(|d| d.on_dismiss),
    );

    ctx.sync_sub_props(
        &dom_id,
        props.on_value_change,
        inherited_dir,
        props.orientation,
        effective_mode,
        props.disable_toggle,
    );

    use_context_provider(|| ctx.clone());
    use_context_provider(|| NavigationMenuInContent(false));

    let sub_attrs = ctx.sub_attributes(Some(dom_id));

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(NavigationMenuSubRenderProps {
                attrs: sub_attrs,
                class: props.class,
                style: props.style,
                attributes: props.attributes,
                children: props.children,
            });
        }
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::NavigationMenuSub: as_child requires render_child; rendering the default div"
        );
    }

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        div {
            id: "{sub_attrs.id()}",
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref().unwrap_or_default(),
            "data-orientation": "{sub_attrs.data_orientation()}",
            "data-monoxus-navigation-menu-sub": "{sub_attrs.data_monoxus_navigation_menu_sub()}",
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
