use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::foundation::{
    compose::{join_styles, merge_global_attributes},
    overlay::{OverlayBranchContext, use_register_parent_overlay_branches},
};
use crate::tooltip::components::TooltipRootContext;

#[derive(Props, Clone, PartialEq)]
pub struct TooltipContentProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default)]
    pub collision_boundary: Option<String>,
    #[props(default)]
    pub collision_padding: Option<f32>,
    #[props(default)]
    pub aria_label: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn TooltipContent(props: TooltipContentProps) -> Element {
    let ctx = use_context::<TooltipRootContext>();
    let runtime = ctx.runtime.read().clone();
    let should_render = props.force_mount || runtime.should_render_content();

    let content = runtime.content();
    let content_id = props.id.clone().unwrap_or_else(|| content.id().to_string());
    let wrapper_id = content.wrapper_id().to_string();
    let portal_id = runtime.relationships().scope().qualify("portal");

    let branch_context = try_consume_context::<OverlayBranchContext>();
    use_register_parent_overlay_branches(
        branch_context,
        should_render,
        vec![content_id.clone(), wrapper_id.clone(), portal_id],
    );

    let latest_cid = use_hook(|| Rc::new(RefCell::new(content_id.clone())));
    {
        let mut prev = latest_cid.borrow_mut();
        if *prev != content_id {
            runtime.unregister_content_id(&prev);
            *prev = content_id.clone();
        }
    }

    if should_render {
        runtime.register_content_id(content_id.clone());
    } else {
        runtime.unregister_content_id(&content_id);
    }
    let cleanup_cid = Rc::clone(&latest_cid);
    let cleanup_runtime = runtime.clone();
    use_drop(move || {
        let current_cid = cleanup_cid.borrow().clone();
        cleanup_runtime.unregister_content_id(&current_cid);
        cleanup_runtime.clear_content_z_index();
    });

    runtime.set_collision_boundary(props.collision_boundary.clone());
    runtime.set_collision_padding(props.collision_padding);

    if should_render && props.aria_label.is_some() {
        runtime.set_measurement_id(Some(wrapper_id.clone()));
    } else {
        runtime.set_measurement_id(None);
    }
    let cleanup_measurement_runtime = runtime.clone();
    use_drop(move || {
        cleanup_measurement_runtime.set_measurement_id(None);
    });

    if !should_render {
        return rsx! {};
    }

    let wrapper_style = runtime.wrapper_style();
    let css_vars = runtime.content_css_custom_properties();
    let base_style = join_styles([
        css_vars.as_str(),
        props.style.as_deref().unwrap_or_default(),
    ]);
    let merged = merge_global_attributes(
        props.class.as_deref(),
        base_style.as_deref(),
        props.attributes,
    );

    rsx! {
        div {
            id: "{wrapper_id}",
            "data-monoxus-floating-content-wrapper": "",
            style: "{wrapper_style}",
            div {
                id: if props.aria_label.is_some() { None } else { Some(content_id.clone()) },
                role: if props.aria_label.is_some() { None } else { Some(content.role()) },
                "aria-hidden": if props.aria_label.is_some() { Some("true") } else { None },
                class: merged.class.as_deref(),
                style: merged.style.as_deref(),
                "data-state": content.data_state().as_str(),
                "data-side": content.data_side(),
                "data-align": content.data_align(),
                tabindex: "-1",
                onmounted: runtime.mount_content(),
                onmouseenter: runtime.content_pointer_enter(),
                onmouseleave: runtime.content_pointer_leave(),
                onkeydown: runtime.escape_keydown(),
                ..merged.rest,
                if let Some(ref label) = props.aria_label {
                    span {
                        id: "{content_id}",
                        role: "tooltip",
                        style: "position: absolute; border: 0; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; word-wrap: normal;",
                        "{label}"
                    }
                }
                {props.children}
            }
        }
    }
}
