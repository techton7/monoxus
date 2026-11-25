use dioxus::prelude::*;

use crate::foundation::{overlay::OverlayBranchContext, shared::Direction, state::DataState};
use crate::hover_card::{
    components::HoverCardRootContext,
    runtime::HoverCardOutsideConfig,
    types::HoverCardOutsideBehavior,
};

#[derive(Clone, PartialEq, Props)]
pub struct HoverCardContentStaticProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
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
pub fn HoverCardContentStatic(props: HoverCardContentStaticProps) -> Element {
    let ctx = use_context::<HoverCardRootContext>();
    let runtime = ctx.runtime.read();
    let should_render = props.force_mount || runtime.should_render_content();
    let is_open = runtime.is_open();

    let content = runtime.content_static();
    let content_id = props.id.clone().unwrap_or_else(|| content.id().to_string());
    let portal_id = runtime.relationships().portal_id().to_string();

    let branch_context = try_consume_context::<OverlayBranchContext>();
    crate::foundation::overlay::use_register_parent_overlay_branches(
        branch_context.clone(),
        should_render,
        vec![content_id.clone(), portal_id],
    );

    let latest_cid = use_hook(|| std::rc::Rc::new(std::cell::RefCell::new(content_id.clone())));
    {
        let mut prev = latest_cid.borrow_mut();
        if *prev != content_id {
            runtime.unregister_content_id(&prev);
            crate::foundation::overlay::unregister_active_overlay_layer(&prev);
            *prev = content_id.clone();
        }
    }

    let mut registered_layer_id = use_signal(|| Option::<String>::None);
    if is_open {
        let prev = registered_layer_id.peek().clone();
        if prev.as_deref() != Some(content_id.as_str()) {
            if let Some(prev_id) = prev.as_deref() {
                crate::foundation::overlay::unregister_active_overlay_layer(prev_id);
            }
            crate::foundation::overlay::register_active_overlay_layer(&content_id);
            registered_layer_id.set(Some(content_id.clone()));
        }
    } else {
        let prev = registered_layer_id.peek().clone();
        if let Some(prev_id) = prev {
            crate::foundation::overlay::unregister_active_overlay_layer(&prev_id);
            registered_layer_id.set(None);
        }
    }

    if should_render {
        runtime.register_content_id(content_id.clone());
    } else {
        runtime.unregister_content_id(&content_id);
    }
    let cleanup_cid = latest_cid.clone();
    let cleanup_runtime = runtime.clone();
    use_drop(move || {
        let current_cid = cleanup_cid.borrow().clone();
        cleanup_runtime.unregister_content_id(&current_cid);
        crate::foundation::overlay::unregister_active_overlay_layer(&current_cid);
        cleanup_runtime.set_outside_config(HoverCardOutsideConfig::default());
    });

    runtime.set_outside_config(HoverCardOutsideConfig {
        escape_keydown_behavior: props.escape_keydown_behavior,
        interact_outside_behavior: props.interact_outside_behavior,
        on_escape_key_down: props.on_escape_key_down,
        on_pointer_down_outside: props.on_pointer_down_outside,
        on_interact_outside: props.on_interact_outside,
    });

    let runtime_for_branch = runtime.clone();
    let parent_for_reg = branch_context.clone();
    let parent_for_unreg = branch_context;
    use_context_provider(|| {
        let rt_reg = runtime_for_branch.clone();
        let rt_unreg = runtime_for_branch.clone();
        OverlayBranchContext::new(
            move |branch: String| {
                rt_reg.register_branch(branch.clone());
                if let Some(ref pb) = parent_for_reg {
                    pb.register_branch(branch);
                }
            },
            move |branch: String| {
                rt_unreg.unregister_branch(&branch);
                if let Some(ref pb) = parent_for_unreg {
                    pb.unregister_branch(branch);
                }
            },
        )
    });

    if !should_render {
        return rsx! {};
    }

    let mut inner_style = props.style.clone().unwrap_or_default();
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

    let dir_str = props.dir.map(|d| d.as_str());

    rsx! {
        div {
            id: "{content_id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: "{inner_style}",
            dir: dir_str,
            tabindex: "{content.tabindex()}",
            "data-state": content.data_state().as_str(),
            "data-monoxus-hover-card-content": content.data_monoxus_hover_card_content(),
            onmounted: runtime.mount_content_static(),
            onpointerdown: runtime.content_pointer_down(),
            onpointerenter: runtime.content_pointer_enter(),
            onpointerleave: runtime.content_pointer_leave(),
            onkeydown: runtime.escape_keydown(),
            ..props.attributes,
            {props.children}
        }
    }
}
