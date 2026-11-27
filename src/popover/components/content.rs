use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::foundation::{
    compose::{
        FloatingContentChildProps, collect_user_attrs, join_styles, merge_attributes_with_part,
        merge_global_attributes, merge_props, push_attr_opt, push_event_handler,
    },
    overlay::{
        DismissBehavior, OverlayBranchContext, OverlayEscapeGuardContext, OverlayEscapeGuardFn,
        PlacementAlign, PlacementSide, StickyStrategy, use_overlay_branch_provider,
        use_register_parent_overlay_branches,
    },
    state::DataState,
};
use crate::popover::{
    components::PopoverContext,
    types::{
        CloseAutoFocusEvent, EscapeKeyDownEvent, InteractOutsideEvent, OpenAutoFocusEvent,
        PointerDownOutsideEvent, PopoverCloseFocusPolicy, PopoverOpenFocusPolicy,
        PopoverOutsideInteractionPolicy, PopoverScrollLockPolicy,
    },
};

#[derive(Props, Clone, PartialEq)]
pub struct PopoverContentProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default, into)]
    pub role: Option<String>,
    #[props(default, into)]
    pub aria_labelledby: Option<String>,
    #[props(default, into)]
    pub aria_describedby: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default = false)]
    pub keep_mounted: bool,
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
    #[props(default, into)]
    pub collision_boundary: Option<String>,
    #[props(default)]
    pub collision_padding: Option<f32>,
    #[props(default)]
    pub arrow_padding: Option<f32>,
    #[props(default)]
    pub sticky: Option<StickyStrategy>,
    #[props(default)]
    pub hide_when_detached: Option<bool>,
    #[props(default, into)]
    pub custom_anchor: Option<String>,
    #[props(default)]
    pub open_focus: Option<PopoverOpenFocusPolicy>,
    #[props(default)]
    pub close_focus: Option<PopoverCloseFocusPolicy>,
    #[props(default)]
    pub scroll_lock: Option<PopoverScrollLockPolicy>,
    #[props(default)]
    pub outside_interaction: Option<PopoverOutsideInteractionPolicy>,
    #[props(default)]
    pub prevent_body_pointer_events: Option<bool>,
    #[props(default)]
    pub escape_key_down_behavior: Option<DismissBehavior>,
    #[props(default)]
    pub interact_outside_behavior: Option<DismissBehavior>,
    #[props(default)]
    pub on_open_auto_focus: Option<EventHandler<OpenAutoFocusEvent>>,
    #[props(default)]
    pub on_close_auto_focus: Option<EventHandler<CloseAutoFocusEvent>>,
    #[props(default)]
    pub on_escape_key_down: Option<EventHandler<EscapeKeyDownEvent>>,
    #[props(default)]
    pub on_pointer_down_outside: Option<EventHandler<PointerDownOutsideEvent>>,
    #[props(default)]
    pub on_interact_outside: Option<EventHandler<InteractOutsideEvent>>,
    #[props(default)]
    pub child: Option<Callback<FloatingContentChildProps, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Plain Rust specification for [`render_popover_content_surface`] (zero internal `#[component]`,
/// compliant with `Dioxus #5894`, shared by `DatePickerContent*` and `DateRangePickerContent*`).
#[allow(dead_code)]
pub(crate) struct PopoverContentSurfaceSpec {
    pub id: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub role: Option<String>,
    pub aria_labelledby: Option<String>,
    pub aria_describedby: Option<String>,
    pub force_mount: bool,
    pub floating: bool,
    pub side: Option<PlacementSide>,
    pub align: Option<PlacementAlign>,
    pub side_offset: Option<f32>,
    pub align_offset: Option<f32>,
    pub avoid_collisions: Option<bool>,
    pub collision_padding: Option<f32>,
    pub sticky: Option<bool>,
    pub on_open_auto_focus: Option<EventHandler<OpenAutoFocusEvent>>,
    pub on_close_auto_focus: Option<EventHandler<CloseAutoFocusEvent>>,
    pub on_escape_key_down: Option<EventHandler<EscapeKeyDownEvent>>,
    pub on_interact_outside: Option<EventHandler<InteractOutsideEvent>>,
    pub on_focus_in: Option<EventHandler<FocusEvent>>,
    pub on_focus_out: Option<EventHandler<FocusEvent>>,
    pub extra_data_attr: Option<&'static str>,
    pub as_child: bool,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// View passed to `render_as_child` inside [`render_popover_content_surface`].
#[allow(dead_code)]
pub(crate) struct PopoverContentSurfaceView {
    pub id: String,
    pub wrapper_id: Option<String>,
    pub wrapper_style: Option<String>,
    pub role: String,
    pub aria_modal: bool,
    pub aria_labelledby: Option<String>,
    pub aria_describedby: Option<String>,
    pub is_open: bool,
    pub data_state: &'static str,
    pub data_side: &'static str,
    pub data_align: &'static str,
    pub class: Option<String>,
    pub style: String,
    pub on_mounted: EventHandler<MountedEvent>,
    pub on_key_down: EventHandler<KeyboardEvent>,
    pub on_click: EventHandler<MouseEvent>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Plain Rust monomorphized render helper for floating and static picker content surfaces
/// (zero internal `#[component]` per `Dioxus #5894`, `merge_global_attributes` per `Dioxus #3717`,
/// render-time `pointer-events: auto` per `radix#4105`, and `use_drop` z-index cleanup per `B4`).
#[allow(dead_code)]
pub(crate) fn render_popover_content_surface<F>(
    spec: PopoverContentSurfaceSpec,
    render_as_child: F,
) -> Element
where
    F: FnOnce(PopoverContentSurfaceView) -> Option<Element>,
{
    let mut ctx = use_context::<PopoverContext>();
    let runtime = ctx.runtime.read().clone();
    let should_render = spec.force_mount || runtime.should_render_content();
    let is_open = runtime.is_open();

    let mut focus_in_slot = use_hook(|| CopyValue::new(spec.on_focus_in));
    focus_in_slot.set(spec.on_focus_in);
    let mut focus_out_slot = use_hook(|| CopyValue::new(spec.on_focus_out));
    focus_out_slot.set(spec.on_focus_out);

    if spec.floating {
        runtime.set_content_placement_overrides(
            spec.side,
            spec.align,
            spec.side_offset,
            spec.align_offset,
            spec.avoid_collisions,
            None,
            spec.collision_padding,
            None,
            spec.sticky.map(|s| {
                if s {
                    StickyStrategy::Always
                } else {
                    StickyStrategy::Partial
                }
            }),
            None,
            None,
        );
    }
    runtime.set_content_event_handlers(
        spec.on_open_auto_focus,
        spec.on_close_auto_focus,
        spec.on_escape_key_down,
        None,
        spec.on_interact_outside,
    );

    let content = runtime.content();
    let content_id = spec.id.unwrap_or_else(|| content.id().to_string());
    let wrapper_id = format!("{content_id}-wrapper");
    let portal_id = runtime.relationships().scope().qualify("portal");

    let parent_branch = try_consume_context::<OverlayBranchContext>();
    let branch_ids = if spec.floating {
        vec![content_id.clone(), wrapper_id.clone(), portal_id]
    } else {
        vec![content_id.clone(), portal_id]
    };
    use_register_parent_overlay_branches(parent_branch, is_open, branch_ids);

    let latest_cid = use_hook(|| Rc::new(RefCell::new(content_id.clone())));
    {
        let mut prev = latest_cid.borrow_mut();
        if *prev != content_id {
            if ctx.content_id.peek().as_deref() == Some(prev.as_str()) {
                ctx.content_id.set(None);
            }
            runtime.unregister_content_id(&prev);
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

    let cid_for_drop = Rc::clone(&latest_cid);
    let mut cid_sig_drop = ctx.content_id;
    let runtime_for_drop = runtime.clone();
    use_drop(move || {
        let current_cid = cid_for_drop.borrow().clone();
        if let Ok(mut sig) = cid_sig_drop.try_write()
            && sig.as_deref() == Some(current_cid.as_str())
        {
            *sig = None;
        }
        runtime_for_drop.unregister_content_id(&current_cid);
        runtime_for_drop.clear_content_z_index();
    });

    let effect_wrapper_id = wrapper_id.clone();
    let runtime_for_shield = runtime.clone();
    let is_floating = spec.floating;
    use_effect(use_reactive(
        (&should_render, &is_floating, &effect_wrapper_id),
        move |(render_val, floating_val, wid)| {
            if render_val && floating_val {
                runtime_for_shield.sync_modal_shield_with_wrapper(Some(&wid));
            } else {
                runtime_for_shield.sync_modal_shield_with_wrapper(None);
            }
        },
    ));

    let rt_reg_branch = runtime.clone();
    let rt_unreg_branch = runtime.clone();
    use_overlay_branch_provider(
        move |branch: String| rt_reg_branch.register_branch(branch),
        move |branch: String| rt_unreg_branch.unregister_branch(&branch),
    );

    let runtime_for_escape_guard = runtime.clone();
    use_context_provider(|| {
        let rt_reg = runtime_for_escape_guard.clone();
        let rt_unreg = runtime_for_escape_guard.clone();
        OverlayEscapeGuardContext::new(
            move |id: String, guard: OverlayEscapeGuardFn| {
                rt_reg.register_escape_guard(id, guard);
            },
            move |id: String| {
                rt_unreg.unregister_escape_guard(&id);
            },
        )
    });

    if !should_render {
        return rsx! {};
    }

    let wrapper_style = spec.floating.then(|| runtime.wrapper_style());
    let css_vars = if spec.floating {
        runtime.content_css_custom_properties()
    } else {
        String::new()
    };

    let title_id = (ctx.title_id)();
    let desc_id = (ctx.description_id)();
    let labelledby = spec.aria_labelledby.or(title_id);
    let describedby = spec.aria_describedby.or(desc_id);
    let resolved_role = spec.role.as_deref().unwrap_or(content.role());

    let mut style_parts: Vec<&str> = Vec::with_capacity(3);
    if !css_vars.trim().is_empty() {
        style_parts.push(css_vars.as_str());
    }
    if let Some(custom_style) = spec.style.as_deref()
        && !custom_style.trim().is_empty()
    {
        style_parts.push(custom_style);
    }
    let has_pointer_events = spec
        .style
        .as_deref()
        .is_some_and(|s| s.contains("pointer-events:"));
    if !is_open || content.data_state() == &DataState::Closed {
        style_parts.push("pointer-events: none;");
    } else if runtime.should_force_pointer_events_auto() && !has_pointer_events {
        style_parts.push("pointer-events: auto;");
    }
    let base_style = join_styles(style_parts).unwrap_or_default();

    let data_state = match content.data_state() {
        DataState::Open => "open",
        _ => "closed",
    };

    let attrs_with_part = match spec.extra_data_attr {
        Some(part_attr) => merge_attributes_with_part(spec.attributes, part_attr),
        None => spec.attributes,
    };

    if spec.as_child {
        let on_mounted = {
            let mut cb = runtime.mount_content();
            EventHandler::new(move |evt: MountedEvent| cb(evt))
        };
        let on_key_down = {
            let mut cb = runtime.escape_keydown();
            EventHandler::new(move |evt: KeyboardEvent| cb(evt))
        };
        let on_click = EventHandler::new(move |evt: MouseEvent| evt.stop_propagation());
        let view = PopoverContentSurfaceView {
            id: content_id.clone(),
            wrapper_id: spec.floating.then(|| wrapper_id.clone()),
            wrapper_style: wrapper_style.clone(),
            role: resolved_role.to_string(),
            aria_modal: content.aria_modal(),
            aria_labelledby: labelledby.clone(),
            aria_describedby: describedby.clone(),
            is_open,
            data_state,
            data_side: content.data_side(),
            data_align: content.data_align(),
            class: spec.class.clone(),
            style: base_style.clone(),
            on_mounted,
            on_key_down,
            on_click,
            attributes: attrs_with_part.clone(),
            children: spec.children.clone(),
        };
        if let Some(el) = render_as_child(view) {
            return el;
        }
    }

    let merged = merge_global_attributes(
        spec.class.as_deref(),
        (!base_style.is_empty()).then_some(base_style.as_str()),
        attrs_with_part,
    );

    let mut mount_cb = runtime.mount_content();
    let mut keydown_cb = runtime.escape_keydown();

    let inner = rsx! {
        div {
            id: "{content_id}",
            role: resolved_role,
            aria_modal: content.aria_modal(),
            aria_labelledby: labelledby.as_deref(),
            aria_describedby: describedby.as_deref(),
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            "data-state": data_state,
            "data-side": spec.floating.then_some(content.data_side()),
            "data-align": spec.floating.then_some(content.data_align()),
            tabindex: "-1",
            onmounted: move |evt| mount_cb(evt),
            onkeydown: move |evt| keydown_cb(evt),
            onclick: move |evt: MouseEvent| evt.stop_propagation(),
            onfocusin: move |evt: FocusEvent| {
                if let Some(handler) = focus_in_slot.try_read().ok().and_then(|s| *s) {
                    handler.call(evt);
                }
            },
            onfocusout: move |evt: FocusEvent| {
                if let Some(handler) = focus_out_slot.try_read().ok().and_then(|s| *s) {
                    handler.call(evt);
                }
            },
            ..merged.rest,
            {spec.children}
        }
    };

    if let Some(ws) = wrapper_style {
        rsx! {
            div {
                id: "{wrapper_id}",
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
pub fn PopoverContent(props: PopoverContentProps) -> Element {
    let PopoverContentProps {
        id,
        class,
        style,
        role,
        aria_labelledby,
        aria_describedby,
        force_mount,
        keep_mounted,
        side,
        align,
        side_offset,
        align_offset,
        avoid_collisions,
        collision_boundary,
        collision_padding,
        arrow_padding,
        sticky,
        hide_when_detached,
        custom_anchor,
        open_focus,
        close_focus,
        scroll_lock,
        outside_interaction,
        prevent_body_pointer_events,
        escape_key_down_behavior,
        interact_outside_behavior,
        on_open_auto_focus,
        on_close_auto_focus,
        on_escape_key_down,
        on_pointer_down_outside,
        on_interact_outside,
        child,
        attributes,
        children,
    } = props;

    let mut ctx = use_context::<PopoverContext>();
    let runtime = ctx.runtime.read().clone();

    runtime.set_keep_mounted(keep_mounted);
    runtime.set_content_placement_overrides(
        side,
        align,
        side_offset,
        align_offset,
        avoid_collisions,
        collision_boundary,
        collision_padding,
        arrow_padding,
        sticky,
        hide_when_detached,
        custom_anchor,
    );
    runtime.set_content_policy_overrides(
        open_focus,
        close_focus,
        scroll_lock,
        outside_interaction,
        prevent_body_pointer_events,
        escape_key_down_behavior,
        interact_outside_behavior,
    );
    runtime.set_content_event_handlers(
        on_open_auto_focus,
        on_close_auto_focus,
        on_escape_key_down,
        on_pointer_down_outside,
        on_interact_outside,
    );

    let should_render = force_mount || runtime.should_render_content();
    let is_open = runtime.is_open();

    let content = runtime.content();
    let content_id = id.unwrap_or_else(|| content.id().to_string());
    let wrapper_id = format!("{content_id}-wrapper");
    let portal_id = runtime.relationships().scope().qualify("portal");

    let parent_branch = try_consume_context::<OverlayBranchContext>();
    use_register_parent_overlay_branches(
        parent_branch,
        is_open,
        vec![content_id.clone(), wrapper_id.clone(), portal_id],
    );

    let latest_cid = use_hook(|| Rc::new(RefCell::new(content_id.clone())));
    {
        let mut prev = latest_cid.borrow_mut();
        if *prev != content_id {
            if ctx.content_id.peek().as_deref() == Some(prev.as_str()) {
                ctx.content_id.set(None);
            }
            runtime.unregister_content_id(&prev);
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

    let cid_for_drop = Rc::clone(&latest_cid);
    let mut cid_sig_drop = ctx.content_id;
    let runtime_for_drop = runtime.clone();
    use_drop(move || {
        let current_cid = cid_for_drop.borrow().clone();
        if let Ok(mut sig) = cid_sig_drop.try_write()
            && sig.as_deref() == Some(current_cid.as_str())
        {
            *sig = None;
        }
        runtime_for_drop.unregister_content_id(&current_cid);
        runtime_for_drop.clear_content_z_index();
        runtime_for_drop.set_keep_mounted(false);
        runtime_for_drop.set_content_placement_overrides(
            None, None, None, None, None, None, None, None, None, None, None,
        );
        runtime_for_drop.set_content_policy_overrides(None, None, None, None, None, None, None);
        runtime_for_drop.set_content_event_handlers(None, None, None, None, None);
    });

    let effect_wrapper_id = wrapper_id.clone();
    let runtime_for_shield = runtime.clone();
    use_effect(use_reactive(
        (&should_render, &effect_wrapper_id),
        move |(render_val, wid)| {
            if render_val {
                runtime_for_shield.sync_modal_shield_with_wrapper(Some(&wid));
            } else {
                runtime_for_shield.sync_modal_shield_with_wrapper(None);
            }
        },
    ));

    let rt_reg_branch = runtime.clone();
    let rt_unreg_branch = runtime.clone();
    use_overlay_branch_provider(
        move |branch: String| rt_reg_branch.register_branch(branch),
        move |branch: String| rt_unreg_branch.unregister_branch(&branch),
    );

    let runtime_for_escape_guard = runtime.clone();
    use_context_provider(|| {
        let rt_reg = runtime_for_escape_guard.clone();
        let rt_unreg = runtime_for_escape_guard.clone();
        OverlayEscapeGuardContext::new(
            move |id: String, guard: OverlayEscapeGuardFn| {
                rt_reg.register_escape_guard(id, guard);
            },
            move |id: String| {
                rt_unreg.unregister_escape_guard(&id);
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
    let labelledby = aria_labelledby.or(title_id);
    let describedby = aria_describedby.or(desc_id);
    let resolved_role = role.unwrap_or_else(|| content.role().to_string());
    let aria_modal = if content.aria_modal() {
        "true"
    } else {
        "false"
    };
    let data_state = match content.data_state() {
        DataState::Open => "open",
        _ => "closed",
    };
    let data_side = content.data_side();
    let data_align = content.data_align();
    let is_closed_state = *content.data_state() == DataState::Closed;
    let is_hidden = !force_mount && runtime.is_hidden();

    let mut style_parts: Vec<&str> = Vec::with_capacity(2);
    if !css_vars.trim().is_empty() {
        style_parts.push(css_vars.as_str());
    }
    let has_pointer_events = style
        .as_deref()
        .is_some_and(|s| s.contains("pointer-events:"));
    if !is_open || is_closed_state {
        style_parts.push("pointer-events: none;");
    } else if runtime.should_force_pointer_events_auto() && !has_pointer_events {
        style_parts.push("pointer-events: auto;");
    }
    let base_style = join_styles(style_parts);

    let mut mount_cb = runtime.mount_content();
    let mut keydown_cb = runtime.escape_keydown();

    let mut wrapper_props = Vec::with_capacity(4);
    wrapper_props.push(Attribute::new("id", wrapper_id, None, false));
    wrapper_props.push(Attribute::new(
        "data-monoxus-floating-content-wrapper",
        "",
        None,
        false,
    ));
    if !wrapper_style.is_empty() {
        wrapper_props.push(Attribute::new("style", wrapper_style, None, false));
    }
    if is_hidden {
        wrapper_props.push(Attribute::new("hidden", true, None, false));
    }

    let mut base = Vec::with_capacity(17);
    base.push(Attribute::new("id", content_id, None, false));
    base.push(Attribute::new("role", resolved_role, None, false));
    base.push(Attribute::new("aria-modal", aria_modal, None, false));
    push_attr_opt(&mut base, "aria-labelledby", labelledby);
    push_attr_opt(&mut base, "aria-describedby", describedby);
    push_attr_opt(&mut base, "style", base_style);
    base.push(Attribute::new("data-state", data_state, None, false));
    base.push(Attribute::new("data-side", data_side, None, false));
    base.push(Attribute::new("data-align", data_align, None, false));
    push_attr_opt(
        &mut base,
        "data-starting-style",
        runtime.data_starting_style(),
    );
    push_attr_opt(&mut base, "data-ending-style", runtime.data_ending_style());
    if is_hidden {
        base.push(Attribute::new("hidden", true, None, false));
    }
    base.push(Attribute::new("tabindex", "-1", None, false));
    push_event_handler(
        &mut base,
        "onmounted",
        Some(EventHandler::new(move |evt: MountedEvent| mount_cb(evt))),
    );
    push_event_handler(
        &mut base,
        "onkeydown",
        Some(EventHandler::new(move |evt: KeyboardEvent| {
            keydown_cb(evt)
        })),
    );
    push_event_handler(
        &mut base,
        "onclick",
        Some(EventHandler::new(move |evt: MouseEvent| {
            evt.stop_propagation();
        })),
    );

    let user = collect_user_attrs(class, style, attributes);
    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(FloatingContentChildProps::new(wrapper_props, merged));
    }

    rsx! {
        div {
            ..wrapper_props,
            div {
                ..merged,
                {children}
            }
        }
    }
}
