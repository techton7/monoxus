use std::{cell::RefCell, rc::Rc};

use dioxus::core::{AttributeValue, Event, ListenerCallback};
use dioxus::html::PlatformEventData;
use dioxus::prelude::*;

use crate::foundation::{
    compose::{
        FloatingContentChildProps, collect_user_attrs, join_styles, merge_props, push_attr_opt,
    },
    overlay::{
        DismissBehavior, OverlayBranchContext, PlacementAlign, PlacementSide, StickyStrategy,
        is_modal_pointer_lock_active, use_register_parent_overlay_branches,
    },
    shared::{EscapeKeyDownEvent, InteractOutsideEvent, PointerDownOutsideEvent},
    state::DataState,
};
use crate::tooltip::{
    components::TooltipRootContext,
    runtime::{FloatingTooltipRootContext, TooltipDismissConfig, TooltipFloatingOverrides},
};

#[derive(Props, Clone, PartialEq)]
pub struct TooltipContentProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default = false)]
    pub keep_mounted: bool,
    #[props(default = PlacementSide::Top)]
    pub side: PlacementSide,
    #[props(default = 8.0)]
    pub side_offset: f32,
    #[props(default = PlacementAlign::Center)]
    pub align: PlacementAlign,
    #[props(default = 0.0)]
    pub align_offset: f32,
    #[props(default = true)]
    pub avoid_collisions: bool,
    #[props(default, into)]
    pub collision_boundary: Option<String>,
    #[props(default = 0.0)]
    pub collision_padding: f32,
    #[props(default = 4.0)]
    pub arrow_padding: f32,
    #[props(default)]
    pub sticky: StickyStrategy,
    #[props(default = false)]
    pub hide_when_detached: bool,
    #[props(default, into)]
    pub custom_anchor: Option<String>,
    #[props(default)]
    pub escape_key_down_behavior: DismissBehavior,
    #[props(default)]
    pub interact_outside_behavior: DismissBehavior,
    #[props(default)]
    pub on_escape_key_down: Option<EventHandler<EscapeKeyDownEvent>>,
    #[props(default)]
    pub on_pointer_down_outside: Option<EventHandler<PointerDownOutsideEvent>>,
    #[props(default)]
    pub on_interact_outside: Option<EventHandler<InteractOutsideEvent>>,
    #[props(default, into)]
    pub aria_label: Option<String>,
    #[props(default)]
    pub child: Option<Callback<FloatingContentChildProps, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn TooltipContent(props: TooltipContentProps) -> Element {
    let ctx = use_context::<TooltipRootContext>();
    let floating_ctx = use_context::<FloatingTooltipRootContext>();
    let runtime = ctx.runtime.read().clone();

    runtime.set_keep_mounted(props.keep_mounted);
    floating_ctx.set_custom_anchor(props.custom_anchor.clone());
    runtime.set_floating_overrides(TooltipFloatingOverrides {
        side: Some(props.side),
        side_offset: Some(props.side_offset),
        align: Some(props.align),
        align_offset: Some(props.align_offset),
        avoid_collisions: Some(props.avoid_collisions),
        collision_boundary: props.collision_boundary.clone(),
        collision_padding: Some(props.collision_padding),
        arrow_padding: Some(props.arrow_padding),
        sticky: Some(props.sticky),
        hide_when_detached: Some(props.hide_when_detached),
        custom_anchor: props.custom_anchor.clone(),
    });
    runtime.set_dismiss_config(TooltipDismissConfig {
        escape_key_down_behavior: props.escape_key_down_behavior,
        interact_outside_behavior: props.interact_outside_behavior,
        on_escape_key_down: props.on_escape_key_down,
        on_pointer_down_outside: props.on_pointer_down_outside,
        on_interact_outside: props.on_interact_outside,
    });

    let should_render = props.force_mount || props.keep_mounted || runtime.should_render_content();
    let is_open = runtime.is_open();

    let content = runtime.content();
    let content_id = props.id.clone().unwrap_or_else(|| content.id().to_string());
    let visible_surface_id = props
        .aria_label
        .as_ref()
        .map(|_| format!("{content_id}-surface"));
    let wrapper_id = content.wrapper_id().to_string();
    let portal_id = runtime.relationships().scope().qualify("portal");

    let branch_context = try_consume_context::<OverlayBranchContext>();
    let mut branch_ids = vec![content_id.clone(), wrapper_id.clone(), portal_id];
    if let Some(ref surface_id) = visible_surface_id {
        branch_ids.push(surface_id.clone());
    }
    use_register_parent_overlay_branches(branch_context, is_open, branch_ids);

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
        cleanup_runtime.set_floating_overrides(TooltipFloatingOverrides::default());
        cleanup_runtime.set_dismiss_config(TooltipDismissConfig::default());
        cleanup_runtime.set_keep_mounted(false);
        floating_ctx.set_custom_anchor(None);
    });

    if should_render && let Some(ref surface_id) = visible_surface_id {
        runtime.set_measurement_id(Some(surface_id.clone()));
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
    let mut style_parts: Vec<&str> = Vec::with_capacity(2);
    if !css_vars.trim().is_empty() {
        style_parts.push(css_vars.as_str());
    }
    let has_pointer_events = props
        .style
        .as_deref()
        .is_some_and(|s| s.contains("pointer-events:"));
    if !is_open || content.data_state() == &DataState::Closed {
        style_parts.push("pointer-events: none;");
    } else if is_modal_pointer_lock_active() && !has_pointer_events {
        style_parts.push("pointer-events: auto;");
    }
    let base_style = join_styles(style_parts);

    let outer_div_id = visible_surface_id.unwrap_or_else(|| content_id.clone());
    let is_hidden = runtime.is_presence_hidden() && !props.force_mount;

    let mut on_mount = runtime.mount_content();
    let mut on_mouse_enter = runtime.content_pointer_enter();
    let mut on_mouse_leave = runtime.content_pointer_leave();
    let mut on_key_down = runtime.escape_keydown();

    let mut base = Vec::with_capacity(16);
    base.push(Attribute::new("id", outer_div_id, None, false));
    if props.aria_label.is_some() {
        base.push(Attribute::new("aria-hidden", "true", None, false));
    } else {
        base.push(Attribute::new("role", content.role(), None, false));
    }
    push_attr_opt(&mut base, "style", base_style);
    if is_hidden {
        base.push(Attribute::new("hidden", true, None, false));
    }
    base.push(Attribute::new(
        "data-state",
        content.data_state().as_str(),
        None,
        false,
    ));
    base.push(Attribute::new("data-side", content.data_side(), None, false));
    base.push(Attribute::new("data-align", content.data_align(), None, false));
    push_attr_opt(&mut base, "data-starting-style", runtime.data_starting_style());
    push_attr_opt(&mut base, "data-ending-style", runtime.data_ending_style());
    base.push(Attribute::new("tabindex", "-1", None, false));
    base.push(Attribute::new(
        "onmounted",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                on_mount(evt.map(|d| MountedData::from(d)));
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onmouseenter",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                on_mouse_enter(evt.map(|d| MouseData::from(d)));
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onmouseleave",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                on_mouse_leave(evt.map(|d| MouseData::from(d)));
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onkeydown",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                on_key_down(evt.map(|d| KeyboardData::from(d)));
            })
            .erase(),
        ),
        None,
        false,
    ));

    let user = collect_user_attrs(props.class, props.style, props.attributes);
    let merged = merge_props(base, user);

    let wrapper_props = vec![
        Attribute::new("id", wrapper_id, None, false),
        Attribute::new("data-monoxus-floating-content-wrapper", "", None, false),
        Attribute::new("style", wrapper_style, None, false),
    ];

    if let Some(child) = props.child {
        return child.call(FloatingContentChildProps::new(wrapper_props, merged));
    }

    rsx! {
        div {
            ..wrapper_props,
            div {
                ..merged,
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
