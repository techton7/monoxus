use dioxus::prelude::*;

use crate::foundation::{
    overlay::{FloatingPlacement, FloatingReadiness, PlacementAlign, PlacementSide, StickyStrategy},
    shared::Direction,
};
use crate::menu::{
    MenuContentAttributes, MenuOutsideEvent, MenuPart, MenuPreventableEvent, MenuRootContext,
    MenuSurfaceContext, MenuVariant, merge_attributes_with_part,
};

/// Build floating wrapper style with canonical `--monoxus-*` and `--radix-*` geometry CSS custom properties.
pub fn build_menu_wrapper_style(
    root: &MenuRootContext,
    placement: Option<&FloatingPlacement>,
    readiness: FloatingReadiness,
) -> String {
    let base_style = match placement {
        Some(pl) => pl.wrapper_style(readiness, None),
        None => FloatingReadiness::wrapper_measuring_style(None).to_owned(),
    };
    let mut vars_buf = String::new();
    if let Some(pl) = placement {
        for (name, val) in root.css_vars(pl) {
            vars_buf.push(' ');
            vars_buf.push_str(&name);
            vars_buf.push_str(": ");
            vars_buf.push_str(&val);
            vars_buf.push(';');
        }
    }
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
    pub on_escape_keydown: Option<EventHandler<MenuPreventableEvent>>,
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

    // Sync configurable surface options and handlers
    {
        let resolved_id = props
            .id
            .clone()
            .unwrap_or_else(|| root.relationships.content_id().to_owned());
        let mut cid = surface.content_id;
        if cid.try_peek().is_ok_and(|g| *g != resolved_id) {
            let _ = cid.try_write().map(|mut w| *w = resolved_id);
        }

        let resolved_anchor = root.resolved_anchor_id();
        let mut aid = surface.anchor_id;
        if aid.try_peek().is_ok_and(|g| *g != resolved_anchor) {
            let _ = aid.try_write().map(|mut w| *w = resolved_anchor);
        }

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

        macro_rules! sync_sig {
            ($sig:expr, $val:expr) => {{
                let mut s = $sig;
                if s.try_peek().is_ok_and(|g| *g != $val) {
                    let _ = s.try_write().map(|mut w| *w = $val);
                }
            }};
        }

        sync_sig!(surface.side, next_side);
        sync_sig!(surface.align, next_align);
        sync_sig!(surface.side_offset, next_side_offset);
        sync_sig!(surface.align_offset, next_align_offset);
        sync_sig!(surface.avoid_collisions, props.avoid_collisions);
        sync_sig!(surface.collision_padding, props.collision_padding);
        sync_sig!(surface.arrow_padding, props.arrow_padding);
        sync_sig!(surface.sticky, props.sticky);
        sync_sig!(surface.hide_when_detached, props.hide_when_detached);
        sync_sig!(surface.force_mount, props.force_mount);
        let effective_loop = props.loop_focus.unwrap_or_else(|| *root.loop_focus.read());
        sync_sig!(surface.prevent_scroll, props.prevent_scroll);
        sync_sig!(surface.loop_focus, effective_loop);
        if let Some(d) = props.dir {
            sync_sig!(root.dir, d);
        }

        let mut esc = surface.on_escape_keydown;
        let _ = esc.try_write().map(|mut w| *w = props.on_escape_keydown.clone());
        let mut pd = surface.on_pointer_down_outside;
        let _ = pd.try_write().map(|mut w| *w = props.on_pointer_down_outside.clone());
        let mut fo = surface.on_focus_outside;
        let _ = fo.try_write().map(|mut w| *w = props.on_focus_outside.clone());
        let mut io = surface.on_interact_outside;
        let _ = io.try_write().map(|mut w| *w = props.on_interact_outside.clone());
        let mut oa = surface.on_open_auto_focus;
        let _ = oa.try_write().map(|mut w| *w = props.on_open_auto_focus.clone());
        let mut ca = surface.on_close_auto_focus;
        let _ = ca.try_write().map(|mut w| *w = props.on_close_auto_focus.clone());
    }

    use_context_provider(|| MenuSurfaceContext {
        surface: surface.clone(),
        parent_surface: None,
        close_this_submenu: None,
    });

    if !surface.should_render_content() {
        return rsx! {};
    }

    let is_open = root.is_open();
    let content_id = surface.current_content_id();
    let wrapper_id = surface.current_wrapper_id();
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

    let labelledby = match root.variant {
        MenuVariant::ContextMenu => None,
        _ => Some(root.trigger_id.read().clone()),
    };

    let attrs = MenuContentAttributes::new(
        content_id.clone(),
        wrapper_id.clone(),
        labelledby,
        root.direction(),
        is_open,
        data_side,
        data_align,
        root.variant,
        MenuPart::Content,
    );

    let wrapper_style = build_menu_wrapper_style(&root, placement.as_ref(), readiness);
    let hidden_style = if hidden_detached {
        " visibility: hidden;"
    } else {
        ""
    };
    let content_style = match props.style.as_deref() {
        Some(custom) if !custom.is_empty() => format!("pointer-events: auto; outline: none;{hidden_style} {custom}"),
        _ => format!("pointer-events: auto; outline: none;{hidden_style}"),
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
    let content_style_with_vars = if vars_inline.is_empty() {
        content_style
    } else {
        format!("{vars_inline}{content_style}")
    };

    let merged_attrs = merge_attributes_with_part(
        props.attributes.clone(),
        MenuPart::Content.static_data_attr_name(root.variant),
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

    if props.as_child
        && let Some(render_child) = props.render_child.as_ref()
    {
        let rendered = render_child.call(MenuContentRenderProps {
            attrs: attrs.clone(),
            style: content_style_with_vars.clone(),
            class: props.class.clone(),
            on_key_down,
            attributes: merged_attrs,
            children: props.children,
        });
        return rsx! {
            div {
                id: "{wrapper_id}",
                style: "{wrapper_style}",
                "data-positioning": readiness.positioning_state(),
                {rendered}
            }
        };
    }

    rsx! {
        div {
            id: "{wrapper_id}",
            style: "{wrapper_style}",
            "data-positioning": readiness.positioning_state(),
            div {
                id: "{attrs.id}",
                class: props.class.as_deref().unwrap_or_default(),
                style: "{content_style_with_vars}",
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
                ..merged_attrs,
                {props.children}
            }
        }
    }
}
