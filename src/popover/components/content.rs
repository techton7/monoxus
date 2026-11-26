use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use crate::foundation::{
    compose::{join_styles, merge_attributes_with_part, merge_global_attributes},
    overlay::{
        OverlayBranchContext, OverlayEscapeGuardContext, OverlayEscapeGuardFn, PlacementAlign,
        PlacementSide, use_overlay_branch_provider, use_register_parent_overlay_branches,
    },
    state::DataState,
};
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

/// Plain Rust specification for [`render_popover_content_surface`] (zero internal `#[component]`,
/// compliant with `Dioxus #5894`, shared by `PopoverContent`, `DatePickerContent*`, and
/// `DateRangePickerContent*`).
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

/// Plain Rust monomorphized render helper for floating and static popover content surfaces
/// (zero internal `#[component]` per `Dioxus #5894`, `merge_global_attributes` per `Dioxus #3717`,
/// render-time `pointer-events: auto` per `radix#4105`, and `use_drop` z-index cleanup per `B4`).
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

    if spec.floating {
        runtime.set_content_placement_overrides(
            spec.side,
            spec.align,
            spec.side_offset,
            spec.align_offset,
            spec.avoid_collisions,
            spec.collision_padding,
            spec.sticky,
        );
    }
    runtime.set_content_event_handlers(
        spec.on_open_auto_focus,
        spec.on_close_auto_focus,
        spec.on_escape_key_down,
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
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::PopoverContent: as_child requires render_child; rendering the default div"
        );
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
    let render_child = props.render_child;
    render_popover_content_surface(
        PopoverContentSurfaceSpec {
            id: props.id,
            class: props.class,
            style: props.style,
            role: props.role,
            aria_labelledby: props.aria_labelledby,
            aria_describedby: props.aria_describedby,
            force_mount: props.force_mount,
            floating: true,
            side: props.side,
            align: props.align,
            side_offset: props.side_offset,
            align_offset: props.align_offset,
            avoid_collisions: props.avoid_collisions,
            collision_padding: props.collision_padding,
            sticky: props.sticky,
            on_open_auto_focus: props.on_open_auto_focus,
            on_close_auto_focus: props.on_close_auto_focus,
            on_escape_key_down: props.on_escape_key_down,
            on_interact_outside: props.on_interact_outside,
            extra_data_attr: None,
            as_child: props.as_child,
            attributes: props.attributes,
            children: props.children,
        },
        move |view| {
            render_child.map(|cb| {
                cb.call(PopoverContentRenderProps {
                    id: view.id,
                    wrapper_id: view.wrapper_id.unwrap_or_default(),
                    wrapper_style: view.wrapper_style.unwrap_or_default(),
                    role: view.role,
                    aria_modal: view.aria_modal,
                    aria_labelledby: view.aria_labelledby,
                    aria_describedby: view.aria_describedby,
                    data_state: view.data_state,
                    data_side: view.data_side,
                    data_align: view.data_align,
                    class: view.class,
                    style: view.style,
                    on_mounted: view.on_mounted,
                    on_key_down: view.on_key_down,
                    on_click: view.on_click,
                    attributes: view.attributes,
                    children: view.children,
                })
            })
        },
    )
}
