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
        is_modal_pointer_lock_active, use_overlay_branch_provider,
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
    types::HOVER_CARD_DEFAULT_SIDE_OFFSET,
};

#[derive(Clone, PartialEq, Props)]
pub struct HoverCardContentProps {
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
    #[props(default = PlacementSide::Bottom)]
    pub side: PlacementSide,
    #[props(default = HOVER_CARD_DEFAULT_SIDE_OFFSET)]
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
    #[props(default = 0.0)]
    pub arrow_padding: f32,
    #[props(default)]
    pub sticky: StickyStrategy,
    #[props(default = false)]
    pub hide_when_detached: bool,
    #[props(default, into)]
    pub custom_anchor: Option<String>,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default)]
    pub on_escape_key_down: Option<EventHandler<EscapeKeyDownEvent>>,
    #[props(default)]
    pub on_pointer_down_outside: Option<EventHandler<PointerDownOutsideEvent>>,
    #[props(default)]
    pub on_interact_outside: Option<EventHandler<InteractOutsideEvent>>,
    #[props(default)]
    pub escape_key_down_behavior: DismissBehavior,
    #[props(default)]
    pub interact_outside_behavior: DismissBehavior,
    #[props(default)]
    pub child: Option<Callback<FloatingContentChildProps, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

pub(crate) enum HoverCardSurfaceChild {
    Floating(Callback<FloatingContentChildProps, Element>),
    Static(Callback<Vec<Attribute>, Element>),
}

/// Plain Rust specification for [`render_hover_card_surface`] (zero internal `#[component]`,
/// compliant with `Dioxus #5894` and `Dioxus #3717`).
pub(crate) struct HoverCardSurfaceSpec {
    pub id: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub force_mount: bool,
    pub keep_mounted: bool,
    pub floating: bool,
    pub floating_overrides: Option<HoverCardFloatingOverrides>,
    pub dir: Option<Direction>,
    pub outside_config: HoverCardOutsideConfig,
    pub child: Option<HoverCardSurfaceChild>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Plain Rust render helper unifying `HoverCardContent` and `HoverCardContentStatic`.
pub(crate) fn render_hover_card_surface(spec: HoverCardSurfaceSpec) -> Element {
    let ctx = use_context::<HoverCardRootContext>();
    let runtime = ctx.runtime.read().clone();

    runtime.set_keep_mounted(spec.keep_mounted);
    if let Some(overrides) = spec.floating_overrides {
        runtime.set_floating_overrides(overrides);
    }
    runtime.set_outside_config(spec.outside_config);

    let should_render = spec.force_mount || spec.keep_mounted || runtime.should_render_content();
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
    use_register_parent_overlay_branches(parent_branch, is_open, branch_ids);

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
        cleanup_runtime.set_keep_mounted(false);
    });

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

    let mut style_parts: Vec<&str> = Vec::with_capacity(3);
    if !css_vars.trim().is_empty() {
        style_parts.push(css_vars.as_str());
    }
    let has_user_select = spec
        .style
        .as_deref()
        .is_some_and(|s| s.contains("user-select:"));
    if runtime.is_contain_selection() && !has_user_select {
        style_parts.push("user-select: text;");
    }
    let has_pointer_events = spec
        .style
        .as_deref()
        .is_some_and(|s| s.contains("pointer-events:"));
    if !is_open || data_state == DataState::Closed {
        style_parts.push("pointer-events: none;");
    } else if is_modal_pointer_lock_active() && !has_pointer_events {
        style_parts.push("pointer-events: auto;");
    }
    let base_style = join_styles(style_parts);

    let is_hidden = runtime.is_presence_hidden() && !spec.force_mount;
    let dir_str = spec.dir.map(|d| d.as_str());
    let rt_mount = runtime.clone();
    let mut on_pointer_down = runtime.content_pointer_down();
    let mut on_pointer_enter = runtime.content_pointer_enter();
    let mut on_pointer_leave = runtime.content_pointer_leave();
    let mut on_key_down = runtime.escape_keydown();

    let mut base = Vec::with_capacity(18);
    base.push(Attribute::new("id", content_id, None, false));
    push_attr_opt(&mut base, "style", base_style);
    push_attr_opt(&mut base, "dir", dir_str);
    base.push(Attribute::new("tabindex", tabindex.to_string(), None, false));
    if is_hidden {
        base.push(Attribute::new("hidden", true, None, false));
    }
    base.push(Attribute::new("data-state", data_state.as_str(), None, false));
    push_attr_opt(&mut base, "data-side", data_side);
    push_attr_opt(&mut base, "data-align", data_align);
    push_attr_opt(&mut base, "data-starting-style", runtime.data_starting_style());
    push_attr_opt(&mut base, "data-ending-style", runtime.data_ending_style());
    base.push(Attribute::new(
        "data-monoxus-hover-card-content",
        part_attr,
        None,
        false,
    ));
    base.push(Attribute::new(
        "onmounted",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                let mounted_evt = evt.map(|d| MountedData::from(d));
                if is_floating {
                    rt_mount.mount_content()(mounted_evt);
                } else {
                    rt_mount.mount_content_static()(mounted_evt);
                }
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onpointerdown",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                on_pointer_down(evt.map(|d| PointerData::from(d)));
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onpointerenter",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                on_pointer_enter(evt.map(|d| PointerData::from(d)));
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onpointerleave",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                on_pointer_leave(evt.map(|d| PointerData::from(d)));
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

    let user = collect_user_attrs(spec.class, spec.style, spec.attributes);
    let merged = merge_props(base, user);

    if let (Some(wid), Some(ws)) = (wrapper_id_opt, wrapper_style) {
        let wrapper_props = vec![
            Attribute::new("id", wid, None, false),
            Attribute::new("data-monoxus-floating-content-wrapper", "", None, false),
            Attribute::new("style", ws, None, false),
        ];
        if let Some(HoverCardSurfaceChild::Floating(child)) = spec.child {
            return child.call(FloatingContentChildProps::new(wrapper_props, merged));
        }
        rsx! {
            div {
                ..wrapper_props,
                div {
                    ..merged,
                    {spec.children}
                }
            }
        }
    } else {
        if let Some(HoverCardSurfaceChild::Static(child)) = spec.child {
            return child.call(merged);
        }
        rsx! {
            div {
                ..merged,
                {spec.children}
            }
        }
    }
}

#[component]
pub fn HoverCardContent(props: HoverCardContentProps) -> Element {
    render_hover_card_surface(HoverCardSurfaceSpec {
        id: props.id,
        class: props.class,
        style: props.style,
        force_mount: props.force_mount,
        keep_mounted: props.keep_mounted,
        floating: true,
        floating_overrides: Some(HoverCardFloatingOverrides {
            side: Some(props.side),
            side_offset: Some(props.side_offset),
            align: Some(props.align),
            align_offset: Some(props.align_offset),
            avoid_collisions: Some(props.avoid_collisions),
            collision_boundary: props.collision_boundary,
            collision_padding: Some(props.collision_padding),
            arrow_padding: Some(props.arrow_padding),
            sticky: Some(props.sticky),
            hide_when_detached: Some(props.hide_when_detached),
            custom_anchor: props.custom_anchor,
            dir: props.dir,
        }),
        dir: props.dir,
        outside_config: HoverCardOutsideConfig {
            escape_key_down_behavior: props.escape_key_down_behavior,
            interact_outside_behavior: props.interact_outside_behavior,
            on_escape_key_down: props.on_escape_key_down,
            on_pointer_down_outside: props.on_pointer_down_outside,
            on_interact_outside: props.on_interact_outside,
        },
        child: props.child.map(HoverCardSurfaceChild::Floating),
        attributes: props.attributes,
        children: props.children,
    })
}
