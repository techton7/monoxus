use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::foundation::overlay::{PlacementAlign, PlacementSide};
use crate::popover::{
    components::PopoverContext,
    types::{
        CloseAutoFocusEvent, EscapeKeyDownEvent, InteractOutsideEvent, OpenAutoFocusEvent,
    },
};

#[derive(Props, Clone, PartialEq)]
pub struct PopoverContentRenderProps {
    pub id: String,
    pub wrapper_id: String,
    pub wrapper_style: String,
    pub role: String,
    pub aria_modal: bool,
    pub aria_labelledby: Option<String>,
    pub aria_describedby: Option<String>,
    pub data_state: &'static str,
    pub data_side: &'static str,
    pub data_align: &'static str,
    #[props(default)]
    pub class: Option<String>,
    pub style: String,
    pub on_mounted: EventHandler<MountedEvent>,
    pub on_key_down: EventHandler<KeyboardEvent>,
    pub on_click: EventHandler<MouseEvent>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

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
    #[props(default)]
    pub side: Option<PlacementSide>,
    #[props(default)]
    pub align: Option<PlacementAlign>,
    #[props(default)]
    pub side_offset: Option<f32>,
    #[props(default)]
    pub align_offset: Option<f32>,
    #[props(default)]
    pub avoid_collisions: Option<bool>,
    #[props(default)]
    pub collision_padding: Option<f32>,
    #[props(default)]
    pub sticky: Option<bool>,
    #[props(default)]
    pub on_open_auto_focus: Option<EventHandler<OpenAutoFocusEvent>>,
    #[props(default)]
    pub on_close_auto_focus: Option<EventHandler<CloseAutoFocusEvent>>,
    #[props(default)]
    pub on_escape_key_down: Option<EventHandler<EscapeKeyDownEvent>>,
    #[props(default)]
    pub on_interact_outside: Option<EventHandler<InteractOutsideEvent>>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<PopoverContentRenderProps, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn PopoverContent(props: PopoverContentProps) -> Element {
    let mut ctx = use_context::<PopoverContext>();
    let runtime = ctx.runtime.read().clone();
    let should_render = props.force_mount || runtime.should_render_content();
    let is_open = runtime.is_open();

    runtime.set_content_placement_overrides(
        props.side,
        props.align,
        props.side_offset,
        props.align_offset,
        props.avoid_collisions,
        props.collision_padding,
        props.sticky,
    );
    runtime.set_content_event_handlers(
        props.on_open_auto_focus,
        props.on_close_auto_focus,
        props.on_escape_key_down,
        props.on_interact_outside,
    );

    let content = runtime.content();
    let content_id = props.id.unwrap_or_else(|| content.id().to_string());
    let wrapper_id = format!("{}-wrapper", content_id);

    let latest_cid = use_hook(|| Rc::new(RefCell::new(content_id.clone())));
    {
        let mut prev = latest_cid.borrow_mut();
        if *prev != content_id {
            if ctx.content_id.peek().as_deref() == Some(prev.as_str()) {
                ctx.content_id.set(None);
            }
            runtime.unregister_content_id(&prev);
            crate::foundation::overlay::unregister_active_overlay_layer(&prev);
            *prev = content_id.clone();
        }
    }

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

    let cid_for_drop = latest_cid.clone();
    let mut cid_sig_drop = ctx.content_id;
    let runtime_for_drop = runtime.clone();
    use_drop(move || {
        let current_cid = cid_for_drop.borrow().clone();
        if let Ok(mut sig) = cid_sig_drop.try_write() {
            if sig.as_deref() == Some(current_cid.as_str()) {
                *sig = None;
            }
        }
        runtime_for_drop.unregister_content_id(&current_cid);
        crate::foundation::overlay::unregister_active_overlay_layer(&current_cid);
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

    let runtime_for_escape_guard = runtime.clone();
    use_context_provider(|| {
        let rt_reg = runtime_for_escape_guard.clone();
        let rt_unreg = runtime_for_escape_guard.clone();
        crate::foundation::overlay::OverlayEscapeGuardContext::new(
            move |id: String, guard: crate::foundation::overlay::OverlayEscapeGuardFn| {
                rt_reg.register_escape_guard(id, guard);
            },
            move |id: String| {
                rt_unreg.unregister_escape_guard(&id);
            },
        )
    });

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

    let on_mounted = {
        let mut cb = runtime.mount_content();
        EventHandler::new(move |evt: MountedEvent| cb(evt))
    };
    let on_key_down = {
        let mut cb = runtime.escape_keydown();
        EventHandler::new(move |evt: KeyboardEvent| cb(evt))
    };
    let on_click = EventHandler::new(move |evt: MouseEvent| evt.stop_propagation());

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(PopoverContentRenderProps {
                id: content_id,
                wrapper_id,
                wrapper_style,
                role: resolved_role.to_string(),
                aria_modal: content.aria_modal(),
                aria_labelledby: labelledby,
                aria_describedby: describedby,
                data_state: match content.data_state() {
                    crate::foundation::state::DataState::Open => "open",
                    _ => "closed",
                },
                data_side: content.data_side(),
                data_align: content.data_align(),
                class: props.class,
                style: inner_style,
                on_mounted,
                on_key_down,
                on_click,
                attributes: props.attributes,
                children: props.children,
            });
        }
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::PopoverContent: as_child requires render_child; rendering the default div"
        );
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
                onmounted: move |evt| on_mounted.call(evt),
                onkeydown: move |evt| on_key_down.call(evt),
                onclick: move |evt| on_click.call(evt),
                ..props.attributes,
                {props.children}
            }
        }
    }
}
