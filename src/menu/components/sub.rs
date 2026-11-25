use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use dioxus::prelude::*;
use futures_timer::Delay;

use crate::foundation::{
    browser::spawn_root_task,
    overlay::{FloatingPlacement, PlacementAlign, PlacementSide, Point2D, StickyStrategy},
    shared::{Direction, use_stable_id},
};
use crate::menu::{
    ActiveCorridorSubmenu, MenuContentAttributes, MenuItemKind, MenuKey, MenuOpenFocusTarget,
    MenuPart, MenuRootContext, MenuSelectEvent, MenuSubContext, MenuSubRelationships,
    MenuSubTriggerAttributes, MenuSurfaceContext, MenuSurfaceRuntime,
    merge_attributes_with_part, translate_menu_key, use_menu_surface_lifecycle,
};

use super::{
    content::{MenuContentRenderProps, build_menu_wrapper_style},
    item::{is_descendant_owning_focus, use_menu_item_registration},
    portal::MenuActivePortalSurface,
};

#[derive(Props, Clone, PartialEq)]
pub struct MenuSubProps {
    #[props(default)]
    pub open: Option<bool>,
    #[props(default = false)]
    pub default_open: bool,
    #[props(default)]
    pub on_open_change: Option<EventHandler<bool>>,
    #[props(default)]
    pub open_delay_ms: Option<u64>,
    pub children: Element,
}

/// Nested submenu state container supporting configurable hover delay, keyboard tree nav, and bidirectional corridors (`M-10`, `M-11`).
#[component]
pub fn MenuSub(props: MenuSubProps) -> Element {
    let root = use_context::<MenuRootContext>();
    let parent_ctx = use_context::<MenuSurfaceContext>();
    let parent_surface = parent_ctx.surface;
    let sub_token = use_stable_id(None);
    let rels = use_hook(|| MenuSubRelationships::new(root.relationships.scope().child(sub_token)));

    let is_controlled = props.open.is_some();
    let open_sig = use_signal(|| props.open.unwrap_or(props.default_open));
    if let Some(controlled) = props.open {
        let mut s = open_sig;
        if s.try_peek().is_ok_and(|cur| *cur != controlled) {
            let _ = s.try_write().map(|mut w| *w = controlled);
        }
    }

    let default_side = match root.direction() {
        Direction::Ltr => PlacementSide::Right,
        Direction::Rtl => PlacementSide::Left,
    };
    let loop_focus = *root.loop_focus.read();
    let open_delay_ms = props
        .open_delay_ms
        .unwrap_or_else(|| *root.submenu_open_delay_ms.read());

    let surface = MenuSurfaceRuntime::new(
        rels.root_id().to_owned(),
        true,
        open_sig,
        rels.content_id().to_owned(),
        rels.wrapper_id().to_owned(),
        rels.trigger_id().to_owned(),
        default_side,
        PlacementAlign::Start,
        0.0,
        0.0,
        loop_focus,
    );

    let hover_task = use_hook(|| Rc::new(RefCell::new(None)));
    let hover_gen = use_hook(|| Rc::new(Cell::new(0)));
    let opened_by_pointer_move = use_signal(|| false);

    let sub_ctx = MenuSubContext {
        sub_id: rels.root_id().to_owned(),
        trigger_id: rels.trigger_id().to_owned(),
        content_id: rels.content_id().to_owned(),
        wrapper_id: rels.wrapper_id().to_owned(),
        open: open_sig,
        is_controlled,
        on_open_change: props.on_open_change,
        open_delay_ms,
        parent_surface: parent_surface.clone(),
        surface: surface.clone(),
        hover_task,
        hover_gen,
        opened_by_pointer_move,
    };

    use_menu_surface_lifecycle(surface.clone(), root.clone());

    // Wire `ArrowLeft`/`ArrowRight` submenu close-back-to-trigger into this surface's key handler
    {
        let close_sub = sub_ctx.clone();
        let using_kb_sig = root.using_keyboard;
        let mut close_cb_sig = surface.on_close_submenu;
        if close_cb_sig.try_peek().is_ok_and(|cur| cur.is_none()) {
            let _ = close_cb_sig.try_write().map(|mut w| {
                *w = Some(Rc::new(move || {
                    let kb = using_kb_sig.try_read().map(|g| *g).unwrap_or(true);
                    close_sub.close_and_focus_trigger(true, kb);
                }))
            });
        }
    }

    // Register with root close-tree handler and drop cleanly when unmounted
    {
        let root_reg = root.clone();
        let sub_reg = sub_ctx.clone();
        use_hook(move || {
            let sub_id = sub_reg.sub_id.clone();
            let close_ctx = sub_reg.clone();
            root_reg.register_sub_close_handler(
                sub_id,
                Rc::new(move || {
                    close_ctx.request_open(false, MenuOpenFocusTarget::None);
                }),
            );
        });
    }

    // Close submenu automatically if parent surface closes
    let sync_sub = sub_ctx.clone();
    let sync_parent = parent_surface.clone();
    use_effect(move || {
        let parent_open = (sync_parent.open)();
        if !parent_open && sync_sub.is_open() {
            sync_sub.request_open(false, MenuOpenFocusTarget::None);
        }
    });

    let drop_sub = sub_ctx.clone();
    let drop_root = root.clone();
    let drop_parent = parent_surface;
    dioxus::core::use_drop(move || {
        drop_sub.cancel_hover_timer();
        drop_root.unregister_sub_close_handler(&drop_sub.sub_id);
        drop_parent.unregister_child_subcontent(&drop_sub.content_id);
    });

    use_context_provider(|| sub_ctx);
    use_context_provider(|| MenuActivePortalSurface(surface));

    rsx! { {props.children} }
}

/// Render props passed to `render_child` when `SubTrigger` uses `as_child: true`.
#[derive(Clone, PartialEq)]
pub struct MenuSubTriggerRenderProps {
    pub attrs: MenuSubTriggerAttributes,
    pub class: Option<String>,
    pub style: Option<String>,
    pub on_click: EventHandler<MouseEvent>,
    pub on_pointer_move: EventHandler<PointerEvent>,
    pub on_pointer_leave: EventHandler<PointerEvent>,
    pub on_key_down: EventHandler<KeyboardEvent>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Props, Clone, PartialEq)]
pub struct MenuSubTriggerProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub text_value: Option<String>,
    #[props(default)]
    pub open_delay_ms: Option<u64>,
    #[props(default)]
    pub on_select: Option<EventHandler<MenuSelectEvent>>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<MenuSubTriggerRenderProps, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Trigger item opening an enclosing [`MenuSub`] on hover (`100ms`), click, `Enter`/`Space`, or directional arrow (`M-10`, `3B D-1`, `3B D-6`).
#[component]
pub fn MenuSubTrigger(props: MenuSubTriggerProps) -> Element {
    let root = use_context::<MenuRootContext>();
    let sub = use_context::<MenuSubContext>();
    let id = props.id.unwrap_or_else(|| sub.trigger_id.clone());

    {
        let mut aid = sub.surface.anchor_id;
        if aid.try_peek().is_ok_and(|g| *g != id) {
            let _ = aid.try_write().map(|mut w| *w = id.clone());
        }
    }

    use_menu_item_registration(
        &sub.parent_surface,
        &id,
        props.text_value.clone(),
        &props.children,
        props.disabled,
        MenuItemKind::SubTrigger,
    );

    // Register trigger open hooks so parent surface can open via keyboard arrow or corridor exit
    {
        let hover_sub = sub.clone();
        let kb_sub = sub.clone();
        let disabled = props.disabled;
        sub.parent_surface.register_sub_trigger_open(
            id.clone(),
            Rc::new(move || {
                if !disabled {
                    hover_sub.request_open(true, MenuOpenFocusTarget::ContentContainer);
                }
            }),
            Rc::new(move || {
                if !disabled {
                    kb_sub.request_open(true, MenuOpenFocusTarget::FirstItem);
                }
            }),
        );
    }
    let drop_parent = sub.parent_surface.clone();
    let drop_id = id.clone();
    dioxus::core::use_drop(move || {
        drop_parent.unregister_sub_trigger_open(&drop_id);
    });

    let is_open = sub.is_open();
    let content_id = sub.surface.current_content_id();
    let highlighted =
        !props.disabled && sub.parent_surface.current_highlighted_id().as_deref() == Some(id.as_str());
    let attrs = MenuSubTriggerAttributes::from_parts(
        id.clone(),
        &content_id,
        is_open,
        props.disabled,
        highlighted,
        root.variant,
    );
    let merged_attrs = merge_attributes_with_part(
        props.attributes,
        MenuPart::SubTrigger.static_data_attr_name(root.variant),
    );

    let open_delay_ms = props.open_delay_ms.unwrap_or(sub.open_delay_ms);

    let pm_id = id.clone();
    let pm_sub = sub.clone();
    let on_pointer_move = EventHandler::new(move |evt: PointerEvent| {
        if props.disabled || evt.pointer_type() == "touch" {
            return;
        }
        if pm_sub.parent_surface.is_submenu_grace_blocking(Some(&pm_id)) {
            return;
        }
        pm_sub.parent_surface.clear_corridor();
        if pm_sub.parent_surface.current_highlighted_id().as_deref() != Some(pm_id.as_str()) {
            pm_sub
                .parent_surface
                .highlight_item(Some(pm_id.clone()), true, false);
        }
        pm_sub
            .parent_surface
            .close_other_child_submenus(Some(&pm_sub.sub_id));

        if pm_sub.is_open() {
            return;
        }

        if open_delay_ms == 0 {
            pm_sub.request_open(true, MenuOpenFocusTarget::ContentContainer);
            return;
        }

        if pm_sub.hover_task.borrow().is_some() {
            return;
        }

        let next_gen = pm_sub.hover_gen.get().wrapping_add(1);
        pm_sub.hover_gen.set(next_gen);
        let gen_ref = pm_sub.hover_gen.clone();
        let timer_sub = pm_sub.clone();
        let task = spawn_root_task(async move {
            Delay::new(Duration::from_millis(open_delay_ms)).await;
            let still_open_parent = (timer_sub.parent_surface.open)();
            if gen_ref.get() == next_gen && still_open_parent {
                *timer_sub.hover_task.borrow_mut() = None;
                timer_sub.request_open(true, MenuOpenFocusTarget::ContentContainer);
            }
        });
        *pm_sub.hover_task.borrow_mut() = Some(task);
    });

    let pl_id = id.clone();
    let pl_sub = sub.clone();
    let on_pointer_leave = EventHandler::new(move |evt: PointerEvent| {
        if props.disabled || evt.pointer_type() == "touch" {
            return;
        }
        pl_sub.cancel_hover_timer();
        if pl_sub.is_open() {
            let coords = evt.data().client_coordinates();
            let close_ctx = pl_sub.clone();
            pl_sub.parent_surface.start_corridor(
                ActiveCorridorSubmenu {
                    sub_id: pl_sub.sub_id.clone(),
                    trigger_id: pl_id.clone(),
                    content_id: pl_sub.surface.current_content_id(),
                    side: pl_sub.surface.side,
                    close_sub: Rc::new(move || {
                        close_ctx.request_open(false, MenuOpenFocusTarget::None);
                    }),
                },
                Point2D::new(coords.x, coords.y),
                false,
            );
        } else if pl_sub.parent_surface.current_highlighted_id().as_deref() == Some(pl_id.as_str())
        {
            pl_sub.parent_surface.highlight_item(None, true, false);
        }
    });

    let clk_sub = sub.clone();
    let clk_on_select = props.on_select;
    let on_click = EventHandler::new(move |_evt: MouseEvent| {
        if props.disabled {
            return;
        }
        let select_evt = MenuSelectEvent::new();
        if let Some(cb) = clk_on_select.as_ref() {
            cb.call(select_evt.clone());
        }
        if !select_evt.is_default_prevented() {
            clk_sub.request_open(true, MenuOpenFocusTarget::ContentContainer);
        }
    });

    let kd_id = id;
    let kd_sub = sub;
    let kd_root = root.clone();
    let kd_on_select = props.on_select;
    let on_key_down = EventHandler::new(move |evt: KeyboardEvent| {
        if props.disabled || is_descendant_owning_focus(&kd_id) {
            return;
        }
        let Some(key) = translate_menu_key(&evt.key()) else {
            return;
        };
        let open_dir_key = key.is_submenu_open_key(kd_root.direction());
        let select_key = matches!(key, MenuKey::Enter)
            || (matches!(key, MenuKey::Space) && !kd_sub.parent_surface.is_typeahead_active());

        if open_dir_key || select_key {
            evt.prevent_default();
            evt.stop_propagation();
            if select_key {
                let select_evt = MenuSelectEvent::new();
                if let Some(cb) = kd_on_select.as_ref() {
                    cb.call(select_evt.clone());
                }
                if select_evt.is_default_prevented() {
                    return;
                }
            }
            kd_sub.request_open(true, MenuOpenFocusTarget::FirstItem);
        }
    });

    if props.as_child
        && let Some(render_child) = props.render_child.as_ref()
    {
        return render_child.call(MenuSubTriggerRenderProps {
            attrs,
            class: props.class,
            style: props.style,
            on_click,
            on_pointer_move,
            on_pointer_leave,
            on_key_down,
            attributes: merged_attrs,
            children: props.children,
        });
    }

    rsx! {
        div {
            id: "{attrs.id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            role: "{attrs.role}",
            tabindex: "{attrs.tabindex}",
            "aria-haspopup": "{attrs.aria_haspopup}",
            "aria-expanded": "{attrs.aria_expanded}",
            "aria-controls": attrs.aria_controls.as_deref(),
            "aria-disabled": attrs.aria_disabled,
            "data-state": "{attrs.data_state.as_str()}",
            "data-disabled": attrs.data_disabled,
            "data-highlighted": attrs.data_highlighted,
            "data-orientation": "{attrs.data_orientation}",
            onclick: move |evt| on_click.call(evt),
            onpointermove: move |evt| on_pointer_move.call(evt),
            onpointerleave: move |evt| on_pointer_leave.call(evt),
            onkeydown: move |evt| on_key_down.call(evt),
            ..merged_attrs,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct MenuSubContentProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub side: Option<PlacementSide>,
    #[props(default)]
    pub align: Option<PlacementAlign>,
    #[props(default)]
    pub side_offset: Option<f32>,
    #[props(default)]
    pub align_offset: Option<f32>,
    #[props(default = true)]
    pub avoid_collisions: bool,
    #[props(default = 0.0)]
    pub collision_padding: f32,
    #[props(default = 0.0)]
    pub arrow_padding: f32,
    #[props(default = StickyStrategy::Partial)]
    pub sticky: StickyStrategy,
    #[props(default = false)]
    pub hide_when_detached: bool,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default)]
    pub loop_focus: Option<bool>,
    #[props(default)]
    pub on_escape_keydown: Option<EventHandler<crate::menu::MenuPreventableEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<MenuContentRenderProps, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Floating submenu content panel anchored to [`MenuSubTrigger`] with bidirectional grace-corridor protection.
#[component]
pub fn MenuSubContent(props: MenuSubContentProps) -> Element {
    let root = use_context::<MenuRootContext>();
    let sub = use_context::<MenuSubContext>();
    let surface = sub.surface.clone();

    {
        let resolved_id = props.id.clone().unwrap_or_else(|| sub.content_id.clone());
        let mut cid = surface.content_id;
        if cid.try_peek().is_ok_and(|g| *g != resolved_id) {
            let _ = cid.try_write().map(|mut w| *w = resolved_id.clone());
        }
        if sub.is_open()
            && !sub
                .parent_surface
                .child_subcontents
                .try_peek()
                .is_ok_and(|list| list.contains(&resolved_id))
        {
            sub.parent_surface.register_child_subcontent(resolved_id);
        }

        let default_side = match root.direction() {
            Direction::Ltr => PlacementSide::Right,
            Direction::Rtl => PlacementSide::Left,
        };
        let next_side = props.side.unwrap_or(default_side);
        let next_align = props.align.unwrap_or(PlacementAlign::Start);
        let next_side_offset = props.side_offset.unwrap_or(0.0);
        let next_align_offset = props.align_offset.unwrap_or(0.0);

        macro_rules! sync_sub_sig {
            ($sig:expr, $val:expr) => {{
                let mut s = $sig;
                if s.try_peek().is_ok_and(|g| *g != $val) {
                    let _ = s.try_write().map(|mut w| *w = $val);
                }
            }};
        }

        sync_sub_sig!(surface.side, next_side);
        sync_sub_sig!(surface.align, next_align);
        sync_sub_sig!(surface.side_offset, next_side_offset);
        sync_sub_sig!(surface.align_offset, next_align_offset);
        sync_sub_sig!(surface.avoid_collisions, props.avoid_collisions);
        sync_sub_sig!(surface.collision_padding, props.collision_padding);
        sync_sub_sig!(surface.arrow_padding, props.arrow_padding);
        sync_sub_sig!(surface.sticky, props.sticky);
        sync_sub_sig!(surface.hide_when_detached, props.hide_when_detached);
        let effective_loop = props.loop_focus.unwrap_or_else(|| *root.loop_focus.read());
        sync_sub_sig!(surface.force_mount, props.force_mount);
        sync_sub_sig!(surface.loop_focus, effective_loop);
        let mut esc = surface.on_escape_keydown;
        let _ = esc.try_write().map(|mut w| *w = props.on_escape_keydown.clone());
    }

    let parent_branch = try_consume_context::<crate::foundation::overlay::OverlayBranchContext>();
    let should_render = surface.should_render_content();
    let is_open = sub.is_open();
    let content_id = surface.current_content_id();
    let wrapper_id = surface.current_wrapper_id();
    let portal_id = surface
        .portal_id
        .read()
        .clone()
        .unwrap_or_else(|| format!("{}-portal", surface.surface_key));

    crate::foundation::overlay::use_register_parent_overlay_branches(
        parent_branch.clone(),
        should_render,
        vec![content_id.clone(), wrapper_id.clone(), portal_id],
    );

    {
        let surf_reg = surface.clone();
        let surf_unreg = surface.clone();
        let parent_for_reg = parent_branch.clone();
        let parent_for_unreg = parent_branch;
        use_context_provider(|| {
            crate::foundation::overlay::OverlayBranchContext::new(
                move |branch: String| {
                    surf_reg.register_branch(branch.clone());
                    if let Some(ref pb) = parent_for_reg {
                        pb.register_branch(branch);
                    }
                },
                move |branch: String| {
                    surf_unreg.unregister_branch(&branch);
                    if let Some(ref pb) = parent_for_unreg {
                        pb.unregister_branch(branch);
                    }
                },
            )
        });
    }

    let close_fn_sub = sub.clone();
    let root_kb = root.using_keyboard;
    use_context_provider(|| MenuSurfaceContext {
        surface: surface.clone(),
        parent_surface: Some(sub.parent_surface.clone()),
        close_this_submenu: Some(Rc::new(move |focus_trigger| {
            let kb = root_kb.try_read().map(|g| *g).unwrap_or(true);
            close_fn_sub.close_and_focus_trigger(focus_trigger, kb);
        })),
    });

    if !should_render {
        return rsx! {};
    }

    let placement = surface.placement.read().clone();
    let readiness = *surface.readiness.read();
    let hidden_detached = props.hide_when_detached
        && placement.as_ref().is_some_and(FloatingPlacement::reference_hidden);

    let data_side = placement
        .as_ref()
        .map(|p| p.side().as_str())
        .unwrap_or_else(|| surface.side.read().as_str());
    let data_align = placement
        .as_ref()
        .map(|p| p.align().as_str())
        .unwrap_or_else(|| surface.align.read().as_str());

    let attrs = MenuContentAttributes::new(
        content_id.clone(),
        wrapper_id.clone(),
        Some(surface.anchor_id.read().clone()),
        root.direction(),
        is_open,
        data_side,
        data_align,
        root.variant,
        MenuPart::SubContent,
    );

    let content_z_index = surface.content_z_index.read();
    let wrapper_style = build_menu_wrapper_style(
        &root,
        placement.as_ref(),
        readiness,
        content_z_index.as_deref(),
    );
    let hidden_style = if hidden_detached {
        " visibility: hidden;"
    } else {
        ""
    };
    let mut vars_inline = String::new();
    if let Some(pl) = placement.as_ref() {
        for (k, v) in root.css_vars(pl) {
            vars_inline.push_str(&k);
            vars_inline.push_str(": ");
            vars_inline.push_str(&v);
            vars_inline.push_str("; ");
        }
    }
    let content_style = match props.style.as_deref() {
        Some(custom) if !custom.is_empty() => {
            format!("{vars_inline}pointer-events: auto; outline: none;{hidden_style} {custom}")
        }
        _ => format!("{vars_inline}pointer-events: auto; outline: none;{hidden_style}"),
    };

    let merged_attrs = merge_attributes_with_part(
        props.attributes.clone(),
        MenuPart::SubContent.static_data_attr_name(root.variant),
    );

    let kd_surface = surface.clone();
    let kd_root = root.clone();
    let user_kd = props.on_key_down.clone();
    let on_key_down = EventHandler::new(move |evt: KeyboardEvent| {
        if let Some(cb) = user_kd.as_ref() {
            cb.call(evt.clone());
        }
        kd_surface.handle_keydown(&evt, &kd_root);
    });

    let mount_surface = surface.clone();
    let mount_root = root.clone();
    let on_mounted = move |_evt: MountedEvent| {
        mount_surface.ensure_position_tracking(&mount_root);
    };

    let pm_parent = sub.parent_surface.clone();
    let on_pointer_move = EventHandler::new(move |_evt: PointerEvent| {
        pm_parent.clear_corridor();
    });

    let pl_sub = sub.clone();
    let pl_content_id = content_id;
    let on_pointer_leave = EventHandler::new(move |evt: PointerEvent| {
        if evt.pointer_type() == "touch" || !pl_sub.is_open() {
            return;
        }
        let coords = evt.data().client_coordinates();
        let close_ctx = pl_sub.clone();
        pl_sub.parent_surface.start_corridor(
            ActiveCorridorSubmenu {
                sub_id: pl_sub.sub_id.clone(),
                trigger_id: pl_sub.surface.anchor_id.read().clone(),
                content_id: pl_content_id.clone(),
                side: pl_sub.surface.side,
                close_sub: Rc::new(move || {
                    close_ctx.request_open(false, MenuOpenFocusTarget::None);
                }),
            },
            Point2D::new(coords.x, coords.y),
            true,
        );
    });

    if props.as_child
        && let Some(render_child) = props.render_child.as_ref()
    {
        let rendered = render_child.call(MenuContentRenderProps {
            attrs,
            style: content_style,
            class: props.class,
            on_key_down,
            attributes: merged_attrs,
            children: props.children,
        });
        return rsx! {
            div {
                id: "{wrapper_id}",
                "data-monoxus-floating-content-wrapper": "",
                style: "{wrapper_style}",
                "data-positioning": readiness.positioning_state(),
                {rendered}
            }
        };
    }

    rsx! {
        div {
            id: "{wrapper_id}",
            "data-monoxus-floating-content-wrapper": "",
            style: "{wrapper_style}",
            "data-positioning": readiness.positioning_state(),
            div {
                id: "{attrs.id}",
                class: props.class.as_deref().unwrap_or_default(),
                style: "{content_style}",
                role: "{attrs.role}",
                tabindex: "{attrs.tabindex}",
                dir: "{attrs.dir}",
                "aria-orientation": "{attrs.aria_orientation}",
                "aria-labelledby": attrs.aria_labelledby.as_deref(),
                "data-state": "{attrs.data_state.as_str()}",
                "data-side": "{attrs.data_side}",
                "data-align": "{attrs.data_align}",
                onmounted: on_mounted,
                onkeydown: move |evt| on_key_down.call(evt),
                onpointermove: move |evt| on_pointer_move.call(evt),
                onpointerleave: move |evt| on_pointer_leave.call(evt),
                ..merged_attrs,
                {props.children}
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct MenuSubContentStaticProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default)]
    pub loop_focus: Option<bool>,
    #[props(default)]
    pub on_escape_keydown: Option<EventHandler<crate::menu::MenuPreventableEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<MenuContentRenderProps, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Inline static submenu surface retaining keyboard navigation and `role="menu"` (`3B D-6`).
#[component]
pub fn MenuSubContentStatic(props: MenuSubContentStaticProps) -> Element {
    let root = use_context::<MenuRootContext>();
    let sub = use_context::<MenuSubContext>();
    let surface = sub.surface.clone();

    {
        let resolved_id = props.id.clone().unwrap_or_else(|| sub.content_id.clone());
        let mut cid = surface.content_id;
        if cid.try_peek().is_ok_and(|g| *g != resolved_id) {
            let _ = cid.try_write().map(|mut w| *w = resolved_id.clone());
        }
        if sub.is_open()
            && !sub
                .parent_surface
                .child_subcontents
                .try_peek()
                .is_ok_and(|list| list.contains(&resolved_id))
        {
            sub.parent_surface.register_child_subcontent(resolved_id);
        }
        let mut fm = surface.force_mount;
        if fm.try_peek().is_ok_and(|g| *g != props.force_mount) {
            let _ = fm.try_write().map(|mut w| *w = props.force_mount);
        }
        let effective_loop = props.loop_focus.unwrap_or_else(|| *root.loop_focus.read());
        let mut lf = surface.loop_focus;
        if lf.try_peek().is_ok_and(|g| *g != effective_loop) {
            let _ = lf.try_write().map(|mut w| *w = effective_loop);
        }
        let mut esc = surface.on_escape_keydown;
        let _ = esc.try_write().map(|mut w| *w = props.on_escape_keydown.clone());
    }

    let parent_branch = try_consume_context::<crate::foundation::overlay::OverlayBranchContext>();
    let should_render = surface.should_render_content();
    let content_id = surface.current_content_id();
    let portal_id = surface
        .portal_id
        .read()
        .clone()
        .unwrap_or_else(|| format!("{}-portal", surface.surface_key));

    crate::foundation::overlay::use_register_parent_overlay_branches(
        parent_branch.clone(),
        should_render,
        vec![content_id.clone(), portal_id],
    );

    {
        let surf_reg = surface.clone();
        let surf_unreg = surface.clone();
        let parent_for_reg = parent_branch.clone();
        let parent_for_unreg = parent_branch;
        use_context_provider(|| {
            crate::foundation::overlay::OverlayBranchContext::new(
                move |branch: String| {
                    surf_reg.register_branch(branch.clone());
                    if let Some(ref pb) = parent_for_reg {
                        pb.register_branch(branch);
                    }
                },
                move |branch: String| {
                    surf_unreg.unregister_branch(&branch);
                    if let Some(ref pb) = parent_for_unreg {
                        pb.unregister_branch(branch);
                    }
                },
            )
        });
    }

    let close_fn_sub = sub.clone();
    let root_kb = root.using_keyboard;
    use_context_provider(|| MenuSurfaceContext {
        surface: surface.clone(),
        parent_surface: Some(sub.parent_surface.clone()),
        close_this_submenu: Some(Rc::new(move |focus_trigger| {
            let kb = root_kb.try_read().map(|g| *g).unwrap_or(true);
            close_fn_sub.close_and_focus_trigger(focus_trigger, kb);
        })),
    });

    if !should_render {
        return rsx! {};
    }

    let attrs = MenuContentAttributes::new(
        content_id,
        surface.current_wrapper_id(),
        Some(surface.anchor_id.read().clone()),
        root.direction(),
        sub.is_open(),
        PlacementSide::Right.as_str(),
        PlacementAlign::Start.as_str(),
        root.variant,
        MenuPart::SubContentStatic,
    );

    let merged_attrs = merge_attributes_with_part(
        props.attributes,
        MenuPart::SubContentStatic.static_data_attr_name(root.variant),
    );
    let content_style = props.style.unwrap_or_default();

    let kd_surface = surface.clone();
    let kd_root = root.clone();
    let user_kd = props.on_key_down.clone();
    let on_key_down = EventHandler::new(move |evt: KeyboardEvent| {
        if let Some(cb) = user_kd.as_ref() {
            cb.call(evt.clone());
        }
        kd_surface.handle_keydown(&evt, &kd_root);
    });

    if props.as_child
        && let Some(render_child) = props.render_child.as_ref()
    {
        return render_child.call(MenuContentRenderProps {
            attrs,
            style: content_style,
            class: props.class,
            on_key_down,
            attributes: merged_attrs,
            children: props.children,
        });
    }

    rsx! {
        div {
            id: "{attrs.id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: "{content_style}",
            role: "{attrs.role}",
            tabindex: "{attrs.tabindex}",
            dir: "{attrs.dir}",
            "aria-orientation": "{attrs.aria_orientation}",
            "aria-labelledby": attrs.aria_labelledby.as_deref(),
            "data-state": "{attrs.data_state.as_str()}",
            "data-side": "{attrs.data_side}",
            "data-align": "{attrs.data_align}",
            onkeydown: move |evt| on_key_down.call(evt),
            ..merged_attrs,
            {props.children}
        }
    }
}
