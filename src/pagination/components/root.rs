use std::borrow::Cow;

use dioxus::prelude::*;

use crate::foundation::{
    compose::merge_global_attributes,
    shared::{Direction, Orientation, ScopeHandle, use_direction_context, use_stable_id},
};

use super::super::{
    attrs::PaginationRootAttributes,
    relationships::PaginationRelationships,
    runtime::{PaginationContext, use_pagination_runtime},
    types::PaginationSnapshot,
};

#[derive(Clone)]
pub struct PaginationRootRenderChildProps {
    pub attrs: PaginationRootAttributes,
    pub snapshot: PaginationSnapshot,
    pub children: Element,
}

#[derive(Clone, PartialEq, Props)]
pub struct PaginationRootProps {
    #[props(default)]
    pub id: Option<String>,
    pub count: usize,
    #[props(default = 1)]
    pub per_page: usize,
    #[props(default = 1)]
    pub sibling_count: usize,
    #[props(default)]
    pub page: Option<usize>,
    #[props(default = 1)]
    pub default_page: usize,
    #[props(default)]
    pub on_page_change: Option<EventHandler<usize>>,
    #[props(default = false)]
    pub loop_focus: bool,
    #[props(default)]
    pub orientation: Orientation,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default)]
    pub aria_label: Option<String>,
    #[props(default)]
    pub role: Option<String>,
    #[props(default)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<PaginationRootRenderChildProps, Element>>,
    #[props(default)]
    pub render_children: Option<Callback<PaginationSnapshot, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn PaginationRoot(props: PaginationRootProps) -> Element {
    let resolved_dir = use_direction_context(props.dir);
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned));
    let scope_id = stable_id.into_owned();
    let scope = ScopeHandle::root("pagination").child(scope_id.clone());
    let initial_rels = PaginationRelationships::new(scope);
    let resolved_id = props
        .id
        .clone()
        .unwrap_or_else(|| initial_rels.root_id().to_string());

    let resolved_aria_label = props
        .aria_label
        .clone()
        .unwrap_or_else(|| String::from("pagination"));

    let runtime = use_pagination_runtime(
        props.count,
        props.per_page,
        props.sibling_count,
        props.page,
        props.default_page,
        props.on_page_change,
        props.loop_focus,
        props.orientation,
        resolved_dir,
        resolved_aria_label,
        scope_id,
        resolved_id.clone(),
    );

    use_context_provider(|| PaginationContext {
        runtime: runtime.clone(),
    });

    let snapshot = runtime.snapshot();
    let root_attrs = runtime.root_attributes(Some(resolved_id));
    let resolved_role = props.role.as_deref().unwrap_or(root_attrs.role());

    let body = if let Some(ref render_children) = props.render_children {
        render_children.call(snapshot.clone())
    } else {
        props.children
    };

    if props.as_child && props.render_child.is_none() {
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::PaginationRoot: as_child requires render_child; rendering the default nav"
        );
    }
    if let Some(render_child) = props.render_child.as_ref().filter(|_| props.as_child) {
        return render_child.call(PaginationRootRenderChildProps {
            attrs: root_attrs,
            snapshot,
            children: body,
        });
    }

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        nav {
            id: "{root_attrs.id}",
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref().unwrap_or_default(),
            role: "{resolved_role}",
            aria_label: "{root_attrs.aria_label}",
            dir: "{root_attrs.dir}",
            "data-orientation": "{root_attrs.data_orientation}",
            "data-monoxus-pagination-root": "{root_attrs.data_monoxus_pagination_root}",
            onmounted: move |evt| {
                if let Some(ref handler) = props.on_mounted {
                    handler.call(evt);
                }
            },
            ..merged.rest,
            {body}
        }
    }
}
