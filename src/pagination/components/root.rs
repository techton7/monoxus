use std::borrow::Cow;

use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_event_handler},
    shared::{Direction, Orientation, ScopeHandle, use_direction_context, use_stable_id},
};

use super::super::{
    relationships::PaginationRelationships,
    runtime::{PaginationContext, use_pagination_runtime},
    types::PaginationSnapshot,
};

#[derive(Clone, PartialEq, Props)]
pub struct PaginationRootProps {
    #[props(default, into)]
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
    #[props(default, into)]
    pub aria_label: Option<String>,
    #[props(default, into)]
    pub role: Option<String>,
    #[props(default)]
    pub render_children: Option<Callback<PaginationSnapshot, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
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
    let resolved_role = props
        .role
        .unwrap_or_else(|| root_attrs.role().to_string());

    let body = if let Some(ref render_children) = props.render_children {
        render_children.call(snapshot)
    } else {
        props.children
    };

    let mut base = Vec::with_capacity(6);
    base.push(Attribute::new("id", root_attrs.id, None, false));
    base.push(Attribute::new("role", resolved_role, None, false));
    base.push(Attribute::new("aria-label", root_attrs.aria_label, None, false));
    base.push(Attribute::new("dir", root_attrs.dir, None, false));
    base.push(Attribute::new(
        "data-orientation",
        root_attrs.data_orientation,
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-monoxus-pagination-root",
        root_attrs.data_monoxus_pagination_root,
        None,
        false,
    ));

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler(&mut user, "onmounted", props.on_mounted);

    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        nav {
            ..merged,
            {body}
        }
    }
}
