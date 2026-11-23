use dioxus::prelude::*;

use crate::foundation::{
    overlay::{OverlayBranchContext, PlacementAlign, PlacementSide},
    shared::Direction,
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
    pub on_escape_key_down: Option<EventHandler<()>>,
    #[props(default)]
    pub on_pointer_down_outside: Option<EventHandler<()>>,
    #[props(default)]
    pub on_interact_outside: Option<EventHandler<()>>,
    #[props(default)]
    pub escape_keydown_behavior: HoverCardOutsideBehavior,
    #[props(default)]
    pub interact_outside_behavior: HoverCardOutsideBehavior,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn HoverCardContent(props: HoverCardContentProps) -> Element {
    let ctx = use_context::<HoverCardRootContext>();
    let runtime = ctx.runtime.read();
    let should_render = props.force_mount || runtime.should_render_content();
    let is_open = runtime.is_open();

    let content = runtime.content();
    let content_id = props.id.clone().unwrap_or_else(|| content.id().to_string());
    let wrapper_id = content.wrapper_id().to_string();

    let branch_context = try_consume_context::<OverlayBranchContext>();
    if let Some(ref bctx) = branch_context {
        if should_render {
            bctx.register_branch(&content_id);
        } else {
            bctx.unregister_branch(&content_id);
        }
    }
    let branch_cleanup_id = content_id.clone();
    let branch_ctx_cleanup = branch_context.clone();
    use_drop(move || {
        if let Some(ref bctx) = branch_ctx_cleanup {
            bctx.unregister_branch(&branch_cleanup_id);
        }
    });

    if should_render {
        runtime.register_content_id(content_id.clone());
    } else {
        runtime.unregister_content_id(&content_id);
    }
    let cleanup_content_id = content_id.clone();
    let cleanup_runtime = runtime.clone();
    use_drop(move || {
        cleanup_runtime.unregister_content_id(&cleanup_content_id);
        cleanup_runtime.set_floating_overrides(HoverCardFloatingOverrides::default());
        cleanup_runtime.set_outside_config(HoverCardOutsideConfig::default());
    });

    runtime.set_floating_overrides(HoverCardFloatingOverrides {
        side: props.side,
        side_offset: props.side_offset,
        align: props.align,
        align_offset: props.align_offset,
        avoid_collisions: Some(props.avoid_collisions),
        collision_boundary: props.collision_boundary.clone(),
        collision_padding: props.collision_padding,
        arrow_padding: Some(props.arrow_padding),
        hide_when_detached: Some(props.hide_when_detached),
        dir: props.dir,
    });

    runtime.set_outside_config(HoverCardOutsideConfig {
        escape_keydown_behavior: props.escape_keydown_behavior,
        interact_outside_behavior: props.interact_outside_behavior,
        on_escape_key_down: props.on_escape_key_down,
        on_pointer_down_outside: props.on_pointer_down_outside,
        on_interact_outside: props.on_interact_outside,
    });

    let runtime_for_branch = runtime.clone();
    use_context_provider(|| {
        let rt_reg = runtime_for_branch.clone();
        let rt_unreg = runtime_for_branch.clone();
        OverlayBranchContext::new(
            move |branch: String| {
                rt_reg.register_branch(branch);
            },
            move |branch: String| {
                rt_unreg.unregister_branch(&branch);
            },
        )
    });

    if !should_render {
        return rsx! {};
    }

    let wrapper_style = runtime.wrapper_style();
    let css_vars = runtime.content_css_custom_properties();

    let mut inner_style = format!("{css_vars} {}", props.style.as_deref().unwrap_or_default());
    if runtime.is_contain_selection() && !inner_style.contains("user-select:") {
        if !inner_style.trim().is_empty() && !inner_style.trim_end().ends_with(';') {
            inner_style.push(';');
        }
        inner_style.push_str(" user-select: text;");
    }
    if !is_open || content.data_state() == &DataState::Closed {
        if !inner_style.trim().is_empty() && !inner_style.trim_end().ends_with(';') {
            inner_style.push(';');
        }
        inner_style.push_str(" pointer-events: none;");
    }

    rsx! {
        div {
            id: "{wrapper_id}",
            "data-monoxus-floating-content-wrapper": "",
            style: "{wrapper_style}",
            div {
                id: "{content_id}",
                class: props.class.as_deref().unwrap_or_default(),
                style: "{inner_style}",
                tabindex: "{content.tabindex()}",
                "data-state": content.data_state().as_str(),
                "data-side": content.data_side(),
                "data-align": content.data_align(),
                "data-monoxus-hover-card-content": content.data_monoxus_hover_card_content(),
                onmounted: runtime.mount_content(),
                onpointerdown: runtime.content_pointer_down(),
                onpointerenter: runtime.content_pointer_enter(),
                onpointerleave: runtime.content_pointer_leave(),
                onkeydown: runtime.escape_keydown(),
                ..props.attributes,
                {props.children}
            }
        }
    }
}
