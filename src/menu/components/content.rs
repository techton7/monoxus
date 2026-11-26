use std::rc::Rc;

use dioxus::prelude::*;

use crate::foundation::{
    compose::{join_styles, merge_attributes_with_part, merge_global_attributes},
    overlay::{
        FloatingPlacement, FloatingReadiness, OverlayBranchContext, PlacementAlign, PlacementSide,
        StickyStrategy, serialize_css_custom_properties, use_overlay_branch_provider,
        use_register_parent_overlay_branches,
    },
    shared::Direction,
    state::sync_signal,
};
use crate::menu::{
    MenuContentAttributes, MenuOutsideEvent, MenuPart, MenuPreventableEvent, MenuRootContext,
    MenuSurfaceContext, MenuSurfaceRuntime, MenuVariant,
};

/// Build floating wrapper style with canonical `--monoxus-*` and `--radix-*` geometry CSS custom properties.
pub fn build_menu_wrapper_style(
    root: &MenuRootContext,
    placement: Option<&FloatingPlacement>,
    readiness: FloatingReadiness,
    z_index: Option<&str>,
) -> String {
    let base_style = match placement {
        Some(pl) => pl.wrapper_style(readiness, z_index),
        None => FloatingReadiness::wrapper_measuring_style(z_index),
    };
    let vars_buf = placement
        .map(|pl| serialize_css_custom_properties(&root.css_vars(pl)))
        .unwrap_or_default();
    format!("{base_style}{vars_buf}")
}

/// Render props passed to `render_child` when `Content` uses `as_child: true`.
#[derive(Clone, PartialEq)]
pub struct MenuContentRenderProps {
    pub attrs: MenuContentAttributes,
    pub style: String,
    pub class: Option<String>,
    pub on_key_down: EventHandler<KeyboardEvent>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Plain Rust specification for [`render_menu_surface`] shared by `MenuContent`, `MenuContentStatic`,
/// `MenuSubContent`, and `MenuSubContentStatic` (zero internal `#[component]`, `Dioxus #5894`).
pub(crate) struct MenuSurfaceViewSpec {
    pub root: MenuRootContext,
    pub surface: MenuSurfaceRuntime,
    pub parent_surface: Option<MenuSurfaceRuntime>,
    pub close_this_submenu: Option<Rc<dyn Fn(bool)>>,
    pub is_floating: bool,
    pub is_open: bool,
    pub part: MenuPart,
    pub labelledby: Option<String>,
    pub default_static_side: PlacementSide,
    pub hide_when_detached: bool,
    pub class: Option<String>,
    pub style: Option<String>,
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    pub on_pointer_enter: Option<EventHandler<PointerEvent>>,
    pub on_pointer_move: Option<EventHandler<PointerEvent>>,
    pub on_pointer_leave: Option<EventHandler<PointerEvent>>,
    pub as_child: bool,
    pub render_child: Option<Callback<MenuContentRenderProps, Element>>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Plain Rust render helper unifying `MenuContent`, `MenuContentStatic`, `MenuSubContent`, and
/// `MenuSubContentStatic` (`Dioxus #5894`, `Dioxus #3717`).
pub(crate) fn render_menu_surface(spec: MenuSurfaceViewSpec) -> Element {
    let mut static_sig = spec.surface.is_static_surface;
    sync_signal(&mut static_sig, !spec.is_floating);

    let parent_branch = try_consume_context::<OverlayBranchContext>();
    let should_render = spec.surface.should_render_content();
    let content_id = spec.surface.current_content_id();
    let wrapper_id = spec.surface.current_wrapper_id();
    let portal_id = spec
        .surface
        .portal_id
        .read()
        .clone()
        .unwrap_or_else(|| format!("{}-portal", spec.surface.surface_key));

    let branch_ids = if spec.is_floating {
        vec![content_id.clone(), wrapper_id.clone(), portal_id]
    } else {
        vec![content_id.clone(), portal_id]
    };
    use_register_parent_overlay_branches(parent_branch, should_render, branch_ids);

    {
        let surf_reg = spec.surface.clone();
        let surf_unreg = spec.surface.clone();
        use_overlay_branch_provider(
            move |branch: String| surf_reg.register_branch(branch),
            move |branch: String| surf_unreg.unregister_branch(&branch),
        );
    }

    let ctx_surface = spec.surface.clone();
    let ctx_parent = spec.parent_surface;
    let ctx_close_sub = spec.close_this_submenu;
    use_context_provider(move || MenuSurfaceContext {
        surface: ctx_surface,
        parent_surface: ctx_parent,
        close_this_submenu: ctx_close_sub,
    });

    if !should_render {
        return rsx! {};
    }

    let placement = spec.is_floating.then(|| spec.surface.placement()).flatten();
    let readiness = if spec.is_floating {
        spec.surface.readiness()
    } else {
        FloatingReadiness::Ready
    };
    let hidden_detached = spec.is_floating
        && spec.hide_when_detached
        && placement
            .as_ref()
            .is_some_and(FloatingPlacement::reference_hidden);

    let data_side = if spec.is_floating {
        placement
            .as_ref()
            .map(|p| p.side().as_str())
            .unwrap_or_else(|| spec.surface.side.read().as_str())
    } else {
        spec.default_static_side.as_str()
    };
    let data_align = if spec.is_floating {
        placement
            .as_ref()
            .map(|p| p.align().as_str())
            .unwrap_or_else(|| spec.surface.align.read().as_str())
    } else {
        PlacementAlign::Start.as_str()
    };

    let attrs = MenuContentAttributes::new(
        content_id,
        wrapper_id.clone(),
        spec.labelledby,
        spec.root.direction(),
        spec.is_open,
        data_side,
        data_align,
        spec.root.variant,
        spec.part,
    );

    let base_content_style = if spec.is_floating {
        let vars_inline = placement
            .as_ref()
            .map(|pl| serialize_css_custom_properties(&spec.root.css_vars(pl)))
            .unwrap_or_default();
        let hidden_part = if hidden_detached {
            "visibility: hidden;"
        } else {
            ""
        };
        join_styles([
            vars_inline.as_str(),
            "pointer-events: auto; outline: none;",
            hidden_part,
            spec.style.as_deref().unwrap_or_default(),
        ])
    } else {
        spec.style.as_deref().and_then(|s| join_styles([s]))
    };

    let part_attrs = merge_attributes_with_part(
        spec.attributes,
        spec.part.static_data_attr_name(spec.root.variant),
    );
    let merged = merge_global_attributes(
        spec.class.as_deref(),
        base_content_style.as_deref(),
        part_attrs,
    );

    let kd_surface = spec.surface.clone();
    let kd_root = spec.root.clone();
    let user_kd = spec.on_key_down;
    let on_key_down = EventHandler::new(move |evt: KeyboardEvent| {
        if let Some(cb) = user_kd.as_ref() {
            cb.call(evt.clone());
        }
        kd_surface.handle_keydown(&evt, &kd_root);
    });

    let on_pointer_enter = spec.on_pointer_enter;
    let on_pointer_move = spec.on_pointer_move;
    let on_pointer_leave = spec.on_pointer_leave;

    let inner = if spec.as_child
        && let Some(render_child) = spec.render_child.as_ref()
    {
        render_child.call(MenuContentRenderProps {
            attrs,
            style: merged.style.unwrap_or_default(),
            class: merged.class,
            on_key_down,
            attributes: merged.rest,
            children: spec.children,
        })
    } else if spec.is_floating {
        let mount_surface = spec.surface.clone();
        rsx! {
            div {
                id: "{attrs.id}",
                class: merged.class.as_deref(),
                style: merged.style.as_deref(),
                role: "{attrs.role}",
                tabindex: "{attrs.tabindex}",
                dir: "{attrs.dir}",
                "aria-orientation": "{attrs.aria_orientation}",
                "aria-labelledby": attrs.aria_labelledby.as_deref(),
                "data-state": "{attrs.data_state.as_str()}",
                "data-side": "{attrs.data_side}",
                "data-align": "{attrs.data_align}",
                onmounted: move |evt: MountedEvent| {
                    mount_surface.capture_content_mount(evt.data());
                },
                onkeydown: move |evt| on_key_down.call(evt),
                onpointerenter: move |evt| {
                    if let Some(cb) = on_pointer_enter.as_ref() {
                        cb.call(evt);
                    }
                },
                onpointermove: move |evt| {
                    if let Some(cb) = on_pointer_move.as_ref() {
                        cb.call(evt);
                    }
                },
                onpointerleave: move |evt| {
                    if let Some(cb) = on_pointer_leave.as_ref() {
                        cb.call(evt);
                    }
                },
                ..merged.rest,
                {spec.children}
            }
        }
    } else {
        rsx! {
            div {
                id: "{attrs.id}",
                class: merged.class.as_deref(),
                style: merged.style.as_deref(),
                role: "{attrs.role}",
                tabindex: "{attrs.tabindex}",
                dir: "{attrs.dir}",
                "aria-orientation": "{attrs.aria_orientation}",
                "aria-labelledby": attrs.aria_labelledby.as_deref(),
                "data-state": "{attrs.data_state.as_str()}",
                "data-side": "{attrs.data_side}",
                "data-align": "{attrs.data_align}",
                onkeydown: move |evt| on_key_down.call(evt),
                ..merged.rest,
                {spec.children}
            }
        }
    };

    if spec.is_floating {
        let content_z_index = spec.surface.content_z_index();
        let wrapper_style = build_menu_wrapper_style(
            &spec.root,
            placement.as_ref(),
            readiness,
            content_z_index.as_deref(),
        );
        rsx! {
            div {
                id: "{wrapper_id}",
                "data-monoxus-floating-content-wrapper": "",
                style: "{wrapper_style}",
                "data-positioning": readiness.positioning_state(),
                {inner}
            }
        }
    } else {
        inner
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct MenuContentProps {
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
    #[props(default = false)]
    pub prevent_scroll: bool,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default)]
    pub on_escape_key_down: Option<EventHandler<MenuPreventableEvent>>,
    #[props(default)]
    pub on_pointer_down_outside: Option<EventHandler<MenuOutsideEvent>>,
    #[props(default)]
    pub on_focus_outside: Option<EventHandler<MenuOutsideEvent>>,
    #[props(default)]
    pub on_interact_outside: Option<EventHandler<MenuOutsideEvent>>,
    #[props(default)]
    pub on_open_auto_focus: Option<EventHandler<MenuPreventableEvent>>,
    #[props(default)]
    pub on_close_auto_focus: Option<EventHandler<MenuPreventableEvent>>,
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

/// Floating root menu surface implementing WAI-ARIA `role="menu"`, focus, dismiss, typeahead, and geometry vars.
#[component]
pub fn MenuContent(props: MenuContentProps) -> Element {
    let root = use_context::<MenuRootContext>();
    let surface = root.root_surface.clone();

    // Sync configurable surface options and non-reactive `CopyValue` callbacks (`Dioxus #5894`)
    {
        let resolved_id = props
            .id
            .clone()
            .unwrap_or_else(|| root.relationships.content_id().to_owned());
        let mut cid = surface.content_id;
        sync_signal(&mut cid, resolved_id);

        let resolved_anchor = root.resolved_anchor_id();
        let mut aid = surface.anchor_id;
        sync_signal(&mut aid, resolved_anchor);

        let default_side = match root.variant {
            MenuVariant::ContextMenu => PlacementSide::Right,
            _ => PlacementSide::Bottom,
        };
        let default_side_offset = match root.variant {
            MenuVariant::ContextMenu => crate::menu::MENU_CONTEXT_DEFAULT_SIDE_OFFSET,
            _ => crate::menu::MENU_DEFAULT_SIDE_OFFSET,
        };
        let next_side = props.side.unwrap_or(default_side);
        let next_align = props.align.unwrap_or(PlacementAlign::Start);
        let next_side_offset = props.side_offset.unwrap_or(default_side_offset);
        let next_align_offset = props.align_offset.unwrap_or(0.0);

        let mut side_sig = surface.side;
        sync_signal(&mut side_sig, next_side);
        let mut align_sig = surface.align;
        sync_signal(&mut align_sig, next_align);
        let mut so_sig = surface.side_offset;
        sync_signal(&mut so_sig, next_side_offset);
        let mut ao_sig = surface.align_offset;
        sync_signal(&mut ao_sig, next_align_offset);
        let mut ac_sig = surface.avoid_collisions;
        sync_signal(&mut ac_sig, props.avoid_collisions);
        let mut cp_sig = surface.collision_padding;
        sync_signal(&mut cp_sig, props.collision_padding);
        let mut ap_sig = surface.arrow_padding;
        sync_signal(&mut ap_sig, props.arrow_padding);
        let mut st_sig = surface.sticky;
        sync_signal(&mut st_sig, props.sticky);
        let mut hwd_sig = surface.hide_when_detached;
        sync_signal(&mut hwd_sig, props.hide_when_detached);
        let mut fm_sig = surface.force_mount;
        sync_signal(&mut fm_sig, props.force_mount);
        let effective_loop = props.loop_focus.unwrap_or_else(|| *root.loop_focus.read());
        let mut ps_sig = surface.prevent_scroll;
        sync_signal(&mut ps_sig, props.prevent_scroll);
        let mut lf_sig = surface.loop_focus;
        sync_signal(&mut lf_sig, effective_loop);
        if let Some(d) = props.dir {
            let mut dir_sig = root.dir;
            sync_signal(&mut dir_sig, d);
        }

        let mut esc = surface.on_escape_key_down;
        esc.set(props.on_escape_key_down);
        let mut pd = surface.on_pointer_down_outside;
        pd.set(props.on_pointer_down_outside);
        let mut fo = surface.on_focus_outside;
        fo.set(props.on_focus_outside);
        let mut io = surface.on_interact_outside;
        io.set(props.on_interact_outside);
        let mut oa = surface.on_open_auto_focus;
        oa.set(props.on_open_auto_focus);
        let mut ca = surface.on_close_auto_focus;
        ca.set(props.on_close_auto_focus);
    }

    let is_open = root.is_open();
    let labelledby = match root.variant {
        MenuVariant::ContextMenu => None,
        _ => Some(root.trigger_id.read().clone()),
    };

    render_menu_surface(MenuSurfaceViewSpec {
        root,
        surface,
        parent_surface: None,
        close_this_submenu: None,
        is_floating: true,
        is_open,
        part: MenuPart::Content,
        labelledby,
        default_static_side: PlacementSide::Bottom,
        hide_when_detached: props.hide_when_detached,
        class: props.class,
        style: props.style,
        on_key_down: props.on_key_down,
        on_pointer_enter: None,
        on_pointer_move: None,
        on_pointer_leave: None,
        as_child: props.as_child,
        render_child: props.render_child,
        attributes: props.attributes,
        children: props.children,
    })
}
