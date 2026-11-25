use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::keyboard_types::{Key, NamedKey};
use dioxus::prelude::*;

use crate::foundation::overlay::{
    FloatingReadiness, OverlayBranchContext, PlacementAlign, PlacementSide,
    is_topmost_overlay_layer, register_active_overlay_layer, unregister_active_overlay_layer,
};

use crate::combobox::{
    attrs::ComboboxContentAttrs, monitors::ComboboxInteractOutsideEvent,
    runtime::use_combobox_context,
};

#[derive(Clone, PartialEq)]
pub struct ComboboxContentRenderProps {
    pub attrs: ComboboxContentAttrs,
    pub class: Option<String>,
    pub style: String,
    pub on_key_down: EventHandler<KeyboardEvent>,
    pub on_mounted: EventHandler<MountedEvent>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxContentProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = PlacementSide::Bottom)]
    pub side: PlacementSide,
    #[props(default = 4.0)]
    pub side_offset: f32,
    #[props(default = PlacementAlign::Start)]
    pub align: PlacementAlign,
    #[props(default = 0.0)]
    pub align_offset: f32,
    #[props(default = true)]
    pub avoid_collisions: bool,
    #[props(default)]
    pub collision_boundary: Option<String>,
    #[props(default = 0.0)]
    pub collision_padding: f32,
    #[props(default = 0.0)]
    pub arrow_padding: f32,
    #[props(default)]
    pub sticky: Option<String>,
    #[props(default = false)]
    pub hide_when_detached: bool,
    #[props(default = false)]
    pub same_width: bool,
    #[props(default)]
    pub custom_anchor: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default)]
    pub on_interact_outside: Option<EventHandler<ComboboxInteractOutsideEvent>>,
    #[props(default)]
    pub on_escape_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<ComboboxContentRenderProps, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn ComboboxContent(props: ComboboxContentProps) -> Element {
    let ctx = use_combobox_context();
    let is_open = ctx.runtime.is_open();
    let content_id = props
        .id
        .clone()
        .unwrap_or_else(|| ctx.runtime.relationships().content_id().to_owned());
    ctx.runtime.set_content_id(content_id.clone());

    // Sync floating configuration to runtime signals.
    {
        let mut s = ctx.runtime.state.preferred_side;
        if s.try_peek().is_ok_and(|g| *g != props.side) {
            let _ = s.try_write().map(|mut w| *w = props.side);
        }
        let mut a = ctx.runtime.state.preferred_align;
        if a.try_peek().is_ok_and(|g| *g != props.align) {
            let _ = a.try_write().map(|mut w| *w = props.align);
        }
        let mut so = ctx.runtime.state.side_offset;
        if so.try_peek().is_ok_and(|g| *g != props.side_offset) {
            let _ = so.try_write().map(|mut w| *w = props.side_offset);
        }
        let mut ao = ctx.runtime.state.align_offset;
        if ao.try_peek().is_ok_and(|g| *g != props.align_offset) {
            let _ = ao.try_write().map(|mut w| *w = props.align_offset);
        }
        let mut ac = ctx.runtime.state.avoid_collisions;
        if ac.try_peek().is_ok_and(|g| *g != props.avoid_collisions) {
            let _ = ac.try_write().map(|mut w| *w = props.avoid_collisions);
        }
        let mut cb = ctx.runtime.state.collision_boundary;
        if cb.try_peek().ok().as_deref() != Some(&props.collision_boundary) {
            let _ = cb
                .try_write()
                .map(|mut w| *w = props.collision_boundary.clone());
        }
        let mut cp = ctx.runtime.state.collision_padding;
        if cp.try_peek().is_ok_and(|g| *g != props.collision_padding) {
            let _ = cp.try_write().map(|mut w| *w = props.collision_padding);
        }
        let mut ap = ctx.runtime.state.arrow_padding;
        if ap.try_peek().is_ok_and(|g| *g != props.arrow_padding) {
            let _ = ap.try_write().map(|mut w| *w = props.arrow_padding);
        }
        let mut hwd = ctx.runtime.state.hide_when_detached;
        if hwd.try_peek().is_ok_and(|g| *g != props.hide_when_detached) {
            let _ = hwd.try_write().map(|mut w| *w = props.hide_when_detached);
        }
        let mut st = ctx.runtime.state.sticky;
        if st.try_peek().ok().as_deref() != Some(&props.sticky) {
            let _ = st.try_write().map(|mut w| *w = props.sticky.clone());
        }
        let mut ca = ctx.runtime.state.custom_anchor;
        if ca.try_peek().ok().as_deref() != Some(&props.custom_anchor) {
            let _ = ca.try_write().map(|mut w| *w = props.custom_anchor.clone());
        }
        let mut fm = ctx.runtime.state.force_mount;
        if fm.try_peek().is_ok_and(|g| *g != props.force_mount) {
            let _ = fm.try_write().map(|mut w| *w = props.force_mount);
        }
        let mut oio = ctx.runtime.state.on_interact_outside;
        let _ = oio.try_write().map(|mut w| *w = props.on_interact_outside);
        let mut oek = ctx.runtime.state.on_escape_key_down;
        let _ = oek.try_write().map(|mut w| *w = props.on_escape_key_down);
    }

    // Track current mounted state for `aria-controls` gating (4B D-6).
    let should_render = is_open || props.force_mount;
    {
        let rt = ctx.runtime.clone();
        use_effect(use_reactive((&should_render,), move |(mounted,)| {
            rt.set_content_mounted(mounted);
        }));
    }

    // Register with parent OverlayBranchContext (e.g. when nested inside Dialog/Popover)
    // and with the shared LIFO overlay stack, tracking dynamic content_id safely (DEF-CBX-12).
    let parent_branch = try_consume_context::<OverlayBranchContext>();
    let wrapper_id = format!("{content_id}-floating-wrapper");
    let tracked_ids: Rc<RefCell<Option<(String, String)>>> =
        use_hook(|| Rc::new(RefCell::new(None)));
    {
        let cid = content_id.clone();
        let wid = wrapper_id.clone();
        let branch = parent_branch.clone();
        let tracked = Rc::clone(&tracked_ids);
        use_effect(use_reactive(
            (&is_open, &cid, &wid),
            move |(open, cur_cid, cur_wid)| {
                if let Some((prev_cid, prev_wid)) = tracked.borrow_mut().take() {
                    if prev_cid != cur_cid || !open {
                        unregister_active_overlay_layer(&prev_cid);
                        if let Some(ref b) = branch {
                            b.unregister_branch(prev_cid);
                            b.unregister_branch(prev_wid);
                        }
                    }
                }
                if open {
                    register_active_overlay_layer(&cur_cid);
                    if let Some(ref b) = branch {
                        b.register_branch(cur_cid.clone());
                        b.register_branch(cur_wid.clone());
                    }
                    *tracked.borrow_mut() = Some((cur_cid, cur_wid));
                }
            },
        ));
    }

    {
        let tracked = Rc::clone(&tracked_ids);
        let branch = parent_branch;
        let rt = ctx.runtime.clone();
        use_drop(move || {
            rt.set_content_mounted(false);
            rt.stop_position_monitor();
            if let Some((cid, wid)) = tracked.borrow_mut().take() {
                unregister_active_overlay_layer(&cid);
                if let Some(ref b) = branch {
                    b.unregister_branch(cid);
                    b.unregister_branch(wid);
                }
            }
        });
    }

    // Floating position monitor lifecycle (no focus stealing on open/close!).
    {
        let rt = ctx.runtime.clone();
        use_effect(use_reactive((&is_open,), move |(open,)| {
            if open {
                rt.start_position_monitor();
                rt.refresh_scroll_buttons();
            } else {
                rt.stop_position_monitor();
            }
        }));
    }

    // DEF-CBX-01/02/04: Tracked DOM order sync + post-render open/input highlight reconciliation.
    let reg_ver = *ctx.runtime.state.registry_version.read();
    let reconcile_ver = *ctx.runtime.state.pending_input_reconcile.read();
    let seen_reconcile: Rc<RefCell<u64>> = use_hook(|| Rc::new(RefCell::new(0)));
    {
        let cid = content_id.clone();
        let rt = ctx.runtime.clone();
        let seen = Rc::clone(&seen_reconcile);
        use_effect(use_reactive(
            (&is_open, &reg_ver, &reconcile_ver, &cid),
            move |(open, _ver, r_ver, cur_cid)| {
                if open {
                    rt.sync_dom_order(&cur_cid);
                    rt.flush_pending_open_and_input_reconcile(&mut seen.borrow_mut(), r_ver);
                    rt.refresh_scroll_buttons();
                }
            },
        ));
    }

    if !should_render {
        return rsx! {};
    }

    let eff_side = ctx.runtime.side();
    let eff_align = ctx.runtime.align();
    let attrs = ctx
        .runtime
        .content_attributes(content_id.clone(), eff_side, eff_align);

    let placement_opt = ctx.runtime.placement();
    let readiness = ctx.runtime.content_readiness();
    let is_ready = readiness == FloatingReadiness::Ready && placement_opt.is_some();
    let css_vars = ctx.runtime.content_css_custom_properties();

    const DEFAULT_OVERLAY_Z_INDEX: &str = "50";
    let (wrapper_style, width_style, ref_hidden_str) = if is_ready {
        let placement = placement_opt.unwrap();
        let anchor_w = placement.geometry().anchor_width();
        let w_style = if props.same_width {
            format!("width: {anchor_w}px; min-width: {anchor_w}px;")
        } else {
            format!("min-width: {anchor_w}px;")
        };
        let w_pos_style = placement.wrapper_style(readiness, Some(DEFAULT_OVERLAY_Z_INDEX));
        let ref_hidden = if placement.reference_hidden() {
            "true"
        } else {
            "false"
        };
        (w_pos_style, w_style, ref_hidden)
    } else {
        let w_pos_style =
            FloatingReadiness::wrapper_measuring_style(Some(DEFAULT_OVERLAY_Z_INDEX));
        let w_style = if props.same_width {
            "width: 100%; min-width: 100%;".to_string()
        } else {
            "min-width: 100%;".to_string()
        };
        (w_pos_style, w_style, "false")
    };

    let force_mount_style = if !is_open && props.force_mount {
        "display: none;"
    } else {
        ""
    };
    let pointer_events_style = if !is_open {
        "pointer-events: none;"
    } else {
        ""
    };
    let inner_style = format!(
        "display: flex; flex-direction: column; box-sizing: border-box; {width_style} {css_vars} {force_mount_style} {pointer_events_style} {}",
        props.style.as_deref().unwrap_or_default()
    );

    let key_rt = ctx.runtime.clone();
    let esc_cb = props.on_escape_key_down;
    let esc_cid = content_id.clone();
    let on_key_down = EventHandler::new(move |evt: KeyboardEvent| {
        if evt.key() == Key::Named(NamedKey::Escape) {
            if !is_topmost_overlay_layer(&esc_cid) {
                return;
            }
            // DEF-CBX-03: Invoke consumer callback BEFORE `evt.prevent_default()`.
            let mut consumer_prevented = false;
            if let Some(ref cb) = esc_cb {
                cb.call(evt.clone());
                consumer_prevented = !evt.default_action_enabled();
            }
            evt.prevent_default();
            if consumer_prevented {
                return;
            }
            key_rt.close_menu();
        }
    });

    let mount_rt = ctx.runtime.clone();
    let on_mounted = EventHandler::new(move |evt: MountedEvent| {
        let mut ch = mount_rt.state.content_handle;
        let _ = ch.try_write().map(|mut w| *w = Some(evt.data()));
        mount_rt.start_position_monitor();
    });

    if props.as_child {
        if let Some(ref render_child) = props.render_child {
            let child = render_child.call(ComboboxContentRenderProps {
                attrs,
                class: props.class,
                style: inner_style,
                on_key_down,
                on_mounted,
                attributes: props.attributes,
                children: props.children,
            });
            return rsx! {
                div {
                    id: "{wrapper_id}",
                    "data-monoxus-floating-content-wrapper": "",
                    style: "{wrapper_style}",
                    "data-reference-hidden": "{ref_hidden_str}",
                    {child}
                }
            };
        }
    }

    rsx! {
        div {
            id: "{wrapper_id}",
            "data-monoxus-floating-content-wrapper": "",
            style: "{wrapper_style}",
            "data-reference-hidden": "{ref_hidden_str}",
            div {
                id: "{attrs.id()}",
                role: "{attrs.role()}",
                tabindex: "{attrs.tabindex()}",
                "aria-multiselectable": attrs.aria_multiselectable(),
                "data-combobox-content": "",
                "data-state": "{attrs.data_state_str()}",
                "data-side": "{attrs.data_side_str()}",
                "data-align": "{attrs.data_align_str()}",
                class: props.class.as_deref().unwrap_or_default(),
                style: "{inner_style}",
                onkeydown: move |evt| on_key_down.call(evt),
                onmounted: move |evt| on_mounted.call(evt),
                ..props.attributes,
                {props.children}
            }
        }
    }
}
