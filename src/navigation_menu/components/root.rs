use std::borrow::Cow;

use dioxus::prelude::*;

use crate::foundation::{
    compose::merge_global_attributes,
    shared::{Direction, Orientation, ScopeHandle, use_direction_context, use_stable_id},
};

use super::super::{
    runtime::{
        NavigationMenuInContent, NavigationMenuRootRenderProps, use_navigation_menu_root_runtime,
    },
    types::{
        NAVIGATION_MENU_DEFAULT_DELAY_MS, NAVIGATION_MENU_DEFAULT_SKIP_DELAY_MS,
        NavigationMenuActivationMode,
    },
};

#[derive(Clone, PartialEq, Props)]
pub struct NavigationMenuRootProps {
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
    #[props(default = NAVIGATION_MENU_DEFAULT_DELAY_MS)]
    pub delay_duration: u64,
    #[props(default = NAVIGATION_MENU_DEFAULT_SKIP_DELAY_MS)]
    pub skip_delay_duration: u64,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default = Orientation::Horizontal)]
    pub orientation: Orientation,
    #[props(default)]
    pub activation_mode: NavigationMenuActivationMode,
    #[props(default = Some("Main".to_string()))]
    pub aria_label: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<NavigationMenuRootRenderProps, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn NavigationMenuRoot(props: NavigationMenuRootProps) -> Element {
    let resolved_dir = use_direction_context(props.dir);
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned));
    let scope_id = stable_id.into_owned();
    let scope = ScopeHandle::root("navigation-menu").child(scope_id);

    let fallback_value = use_signal(|| props.default_value.clone().unwrap_or_default());
    let value_signal = props.value.unwrap_or(fallback_value);

    let initial_rels = super::super::relationships::NavigationMenuRelationships::new(scope.clone());
    let dom_id = props
        .id
        .clone()
        .unwrap_or_else(|| initial_rels.root_id().to_owned());

    let ctx = use_navigation_menu_root_runtime(
        scope,
        dom_id.clone(),
        value_signal,
        props.on_value_change,
        props.delay_duration,
        props.skip_delay_duration,
        resolved_dir,
        props.orientation,
        props.activation_mode,
        props.aria_label.clone(),
    );

    ctx.sync_root_props(
        &dom_id,
        props.on_value_change,
        props.delay_duration,
        props.skip_delay_duration,
        resolved_dir,
        props.orientation,
        props.activation_mode,
        props.aria_label.clone(),
    );

    use_context_provider(|| ctx.clone());
    use_context_provider(|| NavigationMenuInContent(false));

    let root_attrs = ctx.root_attributes(Some(dom_id));

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(NavigationMenuRootRenderProps {
                attrs: root_attrs,
                class: props.class,
                style: props.style,
                attributes: props.attributes,
                children: props.children,
            });
        }
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::NavigationMenuRoot: as_child requires render_child; rendering the default nav"
        );
    }

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        nav {
            id: "{root_attrs.id()}",
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref().unwrap_or_default(),
            aria_label: root_attrs.aria_label(),
            dir: "{root_attrs.dir()}",
            "data-orientation": "{root_attrs.data_orientation()}",
            "data-monoxus-navigation-menu-root": "{root_attrs.data_monoxus_navigation_menu_root()}",
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

pub use NavigationMenuRoot as NavigationMenu;
