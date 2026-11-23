use std::borrow::Cow;

use dioxus::prelude::*;

use crate::foundation::{
    roving_focus::RovingFocusConfig,
    shared::{Direction, Orientation, ScopeHandle, use_direction_context, use_stable_id},
    state::debug_assert_mode_stable,
};

use super::super::{
    attrs::PaginationRootAttributes,
    relationships::PaginationRelationships,
    runtime::{PaginationContext, PaginationRuntime},
    state::{clamp_page, compute_total_pages},
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
    pub as_child: Option<bool>,
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

    let initially_controlled = use_hook(|| props.page.is_some());
    let is_controlled = props.page.is_some();
    debug_assert_mode_stable(initially_controlled, is_controlled);

    let effective_per_page = props.per_page.max(1);
    let total_pages = compute_total_pages(props.count, effective_per_page);
    let initial_page = clamp_page(props.page.unwrap_or(props.default_page), total_pages);
    let resolved_aria_label = props
        .aria_label
        .clone()
        .unwrap_or_else(|| String::from("pagination"));

    let mut runtime = PaginationRuntime::new(
        props.count,
        effective_per_page,
        props.sibling_count,
        initial_page,
        is_controlled,
        props.loop_focus,
        props.orientation,
        resolved_dir,
        resolved_aria_label.clone(),
        scope_id.clone(),
        resolved_id.clone(),
        props.on_page_change,
    );

    // Keep reactive props synchronized with runtime signals.
    {
        runtime.on_page_change.set(props.on_page_change);

        if *runtime.count.peek() != props.count {
            runtime.count.set(props.count);
        }
        if *runtime.per_page.peek() != effective_per_page {
            runtime.per_page.set(effective_per_page);
        }
        if *runtime.sibling_count.peek() != props.sibling_count {
            runtime.sibling_count.set(props.sibling_count);
        }
        if *runtime.is_controlled.peek() != is_controlled {
            runtime.is_controlled.set(is_controlled);
        }

        if let Some(controlled_page) = props.page {
            let clamped = clamp_page(controlled_page, total_pages);
            if *runtime.page.peek() != clamped {
                runtime.page.set(clamped);
            }
        } else {
            let current = *runtime.page.peek();
            let clamped = clamp_page(current, total_pages);
            if clamped != current {
                runtime.page.set(clamped);
            }
        }

        if *runtime.loop_focus.peek() != props.loop_focus {
            runtime.loop_focus.set(props.loop_focus);
        }
        if *runtime.orientation.peek() != props.orientation {
            runtime.orientation.set(props.orientation);
        }
        if *runtime.dir.peek() != resolved_dir {
            runtime.dir.set(resolved_dir);
        }
        if *runtime.aria_label.peek() != resolved_aria_label {
            runtime.aria_label.set(resolved_aria_label);
        }
        if *runtime.scope_id.peek() != scope_id {
            runtime.scope_id.set(scope_id);
        }

        runtime.roving.set_config(
            RovingFocusConfig::new()
                .with_orientation(props.orientation)
                .with_direction(resolved_dir)
                .with_loop_focus(props.loop_focus)
                .with_enabled(true),
        );
        runtime.roving.set_root_dom_id(&resolved_id);
    }

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

    let as_child = props.as_child.unwrap_or(false);
    if as_child && props.render_child.is_none() {
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::PaginationRoot: as_child requires render_child; rendering the default nav"
        );
    }
    if let Some(render_child) = props.render_child.as_ref().filter(|_| as_child) {
        return render_child.call(PaginationRootRenderChildProps {
            attrs: root_attrs,
            snapshot,
            children: body,
        });
    }

    rsx! {
        nav {
            id: "{root_attrs.id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
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
            ..props.attributes,
            {body}
        }
    }
}
