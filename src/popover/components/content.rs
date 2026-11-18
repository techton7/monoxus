use dioxus::prelude::*;

use crate::popover::components::PopoverContext;

#[derive(Props, Clone, PartialEq)]
pub struct PopoverContentProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub role: Option<String>,
    #[props(default)]
    pub aria_labelledby: Option<String>,
    #[props(default)]
    pub aria_describedby: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    pub children: Element,
}

#[component]
pub fn PopoverContent(props: PopoverContentProps) -> Element {
    let mut ctx = use_context::<PopoverContext>();
    let runtime = ctx.runtime.read();
    let should_render = props.force_mount || runtime.should_render_content();
    let is_open = runtime.is_open();

    let content = runtime.content();
    let content_id = props.id.unwrap_or_else(|| content.id().to_string());
    let wrapper_id = format!("{}-wrapper", content_id);

    if should_render {
        if ctx.content_id.peek().as_deref() != Some(content_id.as_str()) {
            ctx.content_id.set(Some(content_id.clone()));
        }
        runtime.register_content_id(&content_id);
    } else {
        if ctx.content_id.peek().as_deref() == Some(content_id.as_str()) {
            ctx.content_id.set(None);
        }
        runtime.unregister_content_id(&content_id);
    }
    let cid_for_drop = content_id.clone();
    let mut cid_sig_drop = ctx.content_id;
    let runtime_for_drop = runtime.clone();
    use_drop(move || {
        if let Ok(mut sig) = cid_sig_drop.try_write() {
            if sig.as_deref() == Some(cid_for_drop.as_str()) {
                *sig = None;
            }
        }
        runtime_for_drop.unregister_content_id(&cid_for_drop);
    });

    let effect_wrapper_id = wrapper_id.clone();
    let runtime_for_shield = runtime.clone();
    let branches_sig = runtime.branches_signal();
    use_effect(use_reactive(
        (&should_render, &is_open, &effect_wrapper_id),
        move |(render_val, _open_val, wid)| {
            let _ = branches_sig.read();
            if render_val {
                runtime_for_shield.sync_modal_shield_with_wrapper(Some(&wid));
            } else {
                runtime_for_shield.sync_modal_shield_with_wrapper(None);
            }
        },
    ));

    let runtime_for_branch = runtime.clone();
    use_context_provider(|| {
        let rt_reg = runtime_for_branch.clone();
        let rt_unreg = runtime_for_branch.clone();
        crate::foundation::overlay::OverlayBranchContext::new(
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

    let title_id = (ctx.title_id)();
    let desc_id = (ctx.description_id)();

    let labelledby = props.aria_labelledby.or(title_id);
    let describedby = props.aria_describedby.or(desc_id);

    let resolved_role = props.role.as_deref().unwrap_or(content.role());

    let mut inner_style = format!("{css_vars} {}", props.style.as_deref().unwrap_or_default());
    if !is_open || content.data_state() == &crate::foundation::state::DataState::Closed {
        if !inner_style.trim().is_empty() && !inner_style.trim_end().ends_with(';') {
            inner_style.push(';');
        }
        inner_style.push_str(" pointer-events: none;");
    } else if runtime.should_force_pointer_events_auto() && !inner_style.contains("pointer-events:")
    {
        if !inner_style.trim().is_empty() && !inner_style.trim_end().ends_with(';') {
            inner_style.push(';');
        }
        inner_style.push_str(" pointer-events: auto;");
    }

    rsx! {
        div {
            id: "{wrapper_id}",
            "data-monoxus-floating-content-wrapper": "",
            style: "{wrapper_style}",
            div {
                id: "{content_id}",
                role: resolved_role,
                aria_modal: content.aria_modal(),
                aria_labelledby: labelledby.as_deref(),
                aria_describedby: describedby.as_deref(),
                class: props.class.as_deref().unwrap_or_default(),
                style: "{inner_style}",
                "data-state": content.data_state().as_str(),
                "data-side": content.data_side(),
                "data-align": content.data_align(),
                tabindex: "-1",
                onmounted: runtime.mount_content(),
                onkeydown: runtime.escape_keydown(),
                onclick: move |evt: MouseEvent| evt.stop_propagation(),
                {props.children}
            }
        }
    }
}
