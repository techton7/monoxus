use std::borrow::Cow;
use std::rc::Rc;

use dioxus::prelude::*;
use ui_sdk::kernel::ChangeEvent;

use crate::agent::{
    ActionDescriptor, AgentHints, AgentSurface, ComponentSnapshot, DispatchExecResult,
    SnapshotValue, SurfaceMetaDefaults, use_agent_surface,
};
use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_event_handler},
    shared::{Direction, Orientation, ScopeHandle, use_direction_context, use_stable_id},
};

use super::super::{
    relationships::PaginationRelationships,
    runtime::{PaginationContext, use_pagination_runtime_with_details},
    state::{clamp_page, compute_total_pages},
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
    #[props(default)]
    pub on_page_change_details: Option<EventHandler<ChangeEvent<usize>>>,
    #[props(default)]
    pub agent: Option<AgentHints>,
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

    let runtime = use_pagination_runtime_with_details(
        props.count,
        props.per_page,
        props.sibling_count,
        props.page,
        props.default_page,
        props.on_page_change,
        props.on_page_change_details,
        props.loop_focus,
        props.orientation,
        resolved_dir,
        resolved_aria_label.clone(),
        scope_id,
        resolved_id.clone(),
    );

    use_context_provider(|| PaginationContext {
        runtime: runtime.clone(),
    });

    let snapshot = runtime.snapshot();
    let total_pages = snapshot.total_pages;
    let reveals = if total_pages > 1 {
        vec![format!("page {}/{}", snapshot.current_page, total_pages)]
    } else {
        Vec::new()
    };
    let agent_rt_dispatch = runtime.clone();
    let agent_rt_reader = runtime.clone();
    let build_page_snap = |rt: &crate::pagination::runtime::PaginationRuntime| {
        let count = *rt.count.peek();
        let per_page = *rt.per_page.peek();
        let total = compute_total_pages(count, per_page);
        let page = rt.page_ctl.try_peek_cloned().unwrap_or(1);
        ComponentSnapshot::new()
            .with_field("page", SnapshotValue::Int(page as i64))
            .with_field("total_pages", SnapshotValue::Int(total as i64))
            .with_field("count", SnapshotValue::Int(count as i64))
            .with_field("per_page", SnapshotValue::Int(per_page as i64))
    };
    use_agent_surface(
        props.agent.as_ref(),
        SurfaceMetaDefaults::new().with_reveals(reveals),
        || {
            AgentSurface::new("pagination", resolved_aria_label)
                .with_snapshot(build_page_snap(&runtime))
                .with_actions(vec![
                    ActionDescriptor::new("set", "Navigate to a specific page number", true),
                    ActionDescriptor::new("next", "Navigate to the next page", false),
                    ActionDescriptor::new("prev", "Navigate to the previous page", false),
                ])
        },
        || {
            Rc::new(move |dispatch| match dispatch.action.as_str() {
                "set" | "goto" => {
                    let Some(SnapshotValue::Int(page_i64)) = dispatch
                        .args
                        .get("page")
                        .or_else(|| dispatch.args.get("value"))
                    else {
                        return DispatchExecResult::ok()
                            .with_host_error("missing or invalid integer argument 'page'");
                    };
                    if *page_i64 < 1 {
                        return DispatchExecResult::ok()
                            .with_host_error("argument 'page' must be >= 1");
                    }
                    let total = compute_total_pages(
                        *agent_rt_dispatch.count.peek(),
                        *agent_rt_dispatch.per_page.peek(),
                    );
                    let target = clamp_page(*page_i64 as usize, total);
                    let outcome = agent_rt_dispatch.set_page_with_context(target, dispatch.ctx);
                    let mut res = DispatchExecResult::ok().with_canceled(outcome.is_canceled());
                    if !outcome.is_canceled() {
                        res = res.with_expected("page", SnapshotValue::Int(target as i64));
                    }
                    res
                }
                "next" => {
                    let total = compute_total_pages(
                        *agent_rt_dispatch.count.peek(),
                        *agent_rt_dispatch.per_page.peek(),
                    );
                    let cur = *agent_rt_dispatch.page.peek();
                    let target = clamp_page(cur.saturating_add(1), total);
                    let outcome = agent_rt_dispatch.next_page_with_context(dispatch.ctx);
                    let mut res = DispatchExecResult::ok().with_canceled(outcome.is_canceled());
                    if !outcome.is_canceled() {
                        res = res.with_expected("page", SnapshotValue::Int(target as i64));
                    }
                    res
                }
                "prev" => {
                    let cur = *agent_rt_dispatch.page.peek();
                    let target = cur.saturating_sub(1).max(1);
                    let outcome = agent_rt_dispatch.prev_page_with_context(dispatch.ctx);
                    let mut res = DispatchExecResult::ok().with_canceled(outcome.is_canceled());
                    if !outcome.is_canceled() {
                        res = res.with_expected("page", SnapshotValue::Int(target as i64));
                    }
                    res
                }
                other => DispatchExecResult::ok()
                    .with_host_error(format!("unknown pagination action '{other}'")),
            })
        },
        || Rc::new(move || build_page_snap(&agent_rt_reader)),
    );
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
