use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::foundation::{
    compose::{join_styles, merge_global_attributes},
    overlay::{
        OverlayBranchContext, PlacementAlign, PlacementSide, use_overlay_branch_provider,
        use_register_parent_overlay_branches,
    },
    shared::{
        Direction, EscapeKeyDownEvent, InteractOutsideEvent, PointerDownOutsideEvent,
    },
    state::DataState,
};
use crate::hover_card::{
    components::HoverCardRootContext,
    runtime::{HoverCardFloatingOverrides, HoverCardOutsideConfig},
    types::HoverCardOutsideBehavior,
};

#[derive(Clone, PartialEq, Props)]
pub struct HoverCardContentProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default)]
    pub side: Option<PlacementSide>,
    #[props(default)]
    pub side_offset: Option<f32>,
    #[props(default)]
    pub align: Option<PlacementAlign>,
    #[props(default)]
    pub align_offset: Option<f32>,
    #[props(default = true)]
    pub avoid_collisions: bool,
    #[props(default)]
    pub collision_boundary: Option<String>,
    #[props(default)]
    pub collision_padding: Option<f32>,
    #[props(default = 0.0)]
    pub arrow_padding: f32,
    #[props(default = false)]
    pub hide_when_detached: bool,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default)]
    pub on_escape_key_down: Option<EventHandler<EscapeKeyDownEvent>>,
    #[props(default)]
    pub on_pointer_down_outside: Option<EventHandler<PointerDownOutsideEvent>>,
    #[props(default)]
    pub on_interact_outside: Option<EventHandler<InteractOutsideEvent>>,
    #[props(default)]
    pub escape_keydown_behavior: HoverCardOutsideBehavior,
    #[props(default)]
    pub interact_outside_behavior: HoverCardOutsideBehavior,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Plain Rust specification for [`render_hover_card_surface`] (zero internal `#[component]`,
/// compliant with `Dioxus #5894` and `Dioxus #3717`).
pub(crate) struct HoverCardSurfaceSpec {
    pub id: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub force_mount: bool,
    pub floating: bool,
    pub floating_overrides: Option<HoverCardFloatingOverrides>,
    pub dir: Option<Direction>,
    pub outside_config: HoverCardOutsideConfig,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Plain Rust render helper unifying `HoverCardContent` and `HoverCardContentStatic`.
pub(crate) fn render_hover_card_surface(spec: HoverCardSurfaceSpec) -> Element {
    let ctx = use_context::<HoverCardRootContext>();
    let runtime = ctx.runtime.read().clone();
    let should_render = spec.force_mount || runtime.should_render_content();
    let is_open = runtime.is_open();

    let (default_id, wrapper_id_opt, data_state, data_side, data_align, tabindex, part_attr) =
        if spec.floating {
            let content = runtime.content();
            (
                content.id().to_string(),
                Some(content.wrapper_id().to_string()),
                content.data_state().clone(),
                Some(content.data_side()),
                Some(content.data_align()),
                content.tabindex(),
                content.data_monoxus_hover_card_content(),
            )
        } else {
            let content = runtime.content_static();
            (
                content.id().to_string(),
                None,
                content.data_state().clone(),
                None,
                None,
                content.tabindex(),
                content.data_monoxus_hover_card_content(),
            )
        };

    let content_id = spec.id.unwrap_or(default_id);
    let portal_id = runtime.relationships().portal_id().to_string();

    let parent_branch = try_consume_context::<OverlayBranchContext>();
    let mut branch_ids = vec![content_id.clone(), portal_id];
    if let Some(ref wid) = wrapper_id_opt {
        branch_ids.push(wid.clone());
    }
    use_register_parent_overlay_branches(parent_branch, should_render, branch_ids);

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
    let is_floating = spec.floating;
    use_drop(move || {
        let current_cid = cleanup_cid.borrow().clone();
        cleanup_runtime.unregister_content_id(&current_cid);
        if is_floating {
            cleanup_runtime.clear_content_z_index();
            cleanup_runtime.set_floating_overrides(HoverCardFloatingOverrides::default());
        }
        cleanup_runtime.set_outside_config(HoverCardOutsideConfig::default());
    });

    if let Some(overrides) = spec.floating_overrides {
        runtime.set_floating_overrides(overrides);
    }
    runtime.set_outside_config(spec.outside_config);

    let rt_reg = runtime.clone();
    let rt_unreg = runtime.clone();
    use_overlay_branch_provider(
        move |branch: String| rt_reg.register_branch(branch),
        move |branch: String| rt_unreg.unregister_branch(&branch),
    );

    if !should_render {
        return rsx! {};
    }

    let wrapper_style = spec.floating.then(|| runtime.wrapper_style());
    let css_vars = if spec.floating {
        runtime.content_css_custom_properties()
    } else {
        String::new()
    };

    let mut style_parts: Vec<&str> = Vec::with_capacity(4);
    if !css_vars.trim().is_empty() {
        style_parts.push(css_vars.as_str());
    }
    if let Some(custom_style) = spec.style.as_deref()
        && !custom_style.trim().is_empty()
    {
        style_parts.push(custom_style);
    }
    let has_user_select = spec
        .style
        .as_deref()
        .is_some_and(|s| s.contains("user-select:"));
    if runtime.is_contain_selection() && !has_user_select {
        style_parts.push("user-select: text;");
    }
    if !is_open || data_state == DataState::Closed {
        style_parts.push("pointer-events: none;");
    }
    let base_style = join_styles(style_parts);

    let merged = merge_global_attributes(
        spec.class.as_deref(),
        base_style.as_deref(),
        spec.attributes,
    );

    let dir_str = (!spec.floating).then(|| spec.dir.map(|d| d.as_str())).flatten();
    let is_floating = spec.floating;
    let rt_mount = runtime.clone();

    let inner = rsx! {
        div {
            id: "{content_id}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            dir: dir_str,
            tabindex: "{tabindex}",
            "data-state": data_state.as_str(),
            "data-side": data_side,
            "data-align": data_align,
            "data-monoxus-hover-card-content": part_attr,
            onmounted: move |evt| {
                if is_floating {
                    rt_mount.mount_content()(evt);
                } else {
                    rt_mount.mount_content_static()(evt);
                }
            },
            onpointerdown: runtime.content_pointer_down(),
            onpointerenter: runtime.content_pointer_enter(),
            onpointerleave: runtime.content_pointer_leave(),
            onkeydown: runtime.escape_keydown(),
            ..merged.rest,
            {spec.children}
        }
    };

    if let (Some(wid), Some(ws)) = (wrapper_id_opt, wrapper_style) {
        rsx! {
            div {
                id: "{wid}",
                "data-monoxus-floating-content-wrapper": "",
                style: "{ws}",
                {inner}
            }
        }
    } else {
        inner
    }
}

#[component]
pub fn HoverCardContent(props: HoverCardContentProps) -> Element {
    render_hover_card_surface(HoverCardSurfaceSpec {
        id: props.id,
        class: props.class,
        style: props.style,
        force_mount: props.force_mount,
        floating: true,
        floating_overrides: Some(HoverCardFloatingOverrides {
            side: props.side,
            side_offset: props.side_offset,
            align: props.align,
            align_offset: props.align_offset,
            avoid_collisions: Some(props.avoid_collisions),
            collision_boundary: props.collision_boundary,
            collision_padding: props.collision_padding,
            arrow_padding: Some(props.arrow_padding),
            hide_when_detached: Some(props.hide_when_detached),
            dir: props.dir,
        }),
        dir: props.dir,
        outside_config: HoverCardOutsideConfig {
            escape_keydown_behavior: props.escape_keydown_behavior,
            interact_outside_behavior: props.interact_outside_behavior,
            on_escape_key_down: props.on_escape_key_down,
            on_pointer_down_outside: props.on_pointer_down_outside,
            on_interact_outside: props.on_interact_outside,
        },
        attributes: props.attributes,
        children: props.children,
    })
}
