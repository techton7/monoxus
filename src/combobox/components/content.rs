use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;

use crate::foundation::{
    compose::{join_styles, merge_global_attributes},
    keyboard::{is_escape_key, is_ime_composing_key},
    overlay::{
        FloatingReadiness, OverlayBranchContext, PlacementAlign, PlacementSide, PresenceState,
        is_modal_pointer_lock_active, use_overlay_branch_provider,
        use_register_parent_overlay_branches,
    },
    shared::EscapeKeyDownEvent,
    state::sync_signal,
};

use crate::combobox::{
    attrs::ComboboxContentAttrs,
    monitors::ComboboxInteractOutsideEvent,
    runtime::use_combobox_context,
    types::ComboboxContext,
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

#[derive(Clone, PartialEq)]
pub struct ComboboxContentStaticRenderProps {
    pub attrs: ComboboxContentAttrs,
    pub class: Option<String>,
    pub style: String,
    pub on_key_down: EventHandler<KeyboardEvent>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

pub(crate) enum ComboboxAsChildRenderer {
    Floating(Callback<ComboboxContentRenderProps, Element>),
    Static(Callback<ComboboxContentStaticRenderProps, Element>),
}

/// Plain Rust specification for [`render_combobox_surface`] shared by `ComboboxContent` and
/// `ComboboxContentStatic` (zero internal `#[component]`, `Dioxus #5894`).
pub(crate) struct ComboboxSurfaceSpec {
    pub ctx: ComboboxContext,
    pub content_id: String,
    pub is_floating: bool,
    pub same_width: bool,
    pub force_mount: bool,
    pub class: Option<String>,
    pub style: Option<String>,
    pub as_child: Option<ComboboxAsChildRenderer>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Plain Rust render helper unifying `ComboboxContent` and `ComboboxContentStatic` (`Dioxus #5894`,
/// `Dioxus #3717`, `B10` Presence exit retention, `FM-OVL-1` IME Escape/Tab gating).
pub(crate) fn render_combobox_surface(spec: ComboboxSurfaceSpec) -> Element {
    let ctx = spec.ctx;
    ctx.runtime.set_is_static_content(!spec.is_floating);
    ctx.runtime.set_content_id(spec.content_id.clone());

    let is_open = ctx.runtime.is_open();
    let is_suspended = ctx.runtime.presence_state() == PresenceState::Suspended;
    let should_render = ctx.runtime.should_render_content() || spec.force_mount;
    let content_id = spec.content_id;
    let wrapper_id = format!("{content_id}-floating-wrapper");
    let portal_id = format!("{content_id}-portal-root");

    // Track current mounted state for `aria-controls` gating (4B D-6).
    {
        let rt = ctx.runtime.clone();
        use_effect(use_reactive((&should_render,), move |(mounted,)| {
            rt.set_content_mounted(mounted);
        }));
    }

    // Register with parent OverlayBranchContext (e.g. Dialog) & forward nested child branches (`FM-OVL-3`).
    let parent_branch = try_consume_context::<OverlayBranchContext>();
    let branch_ids = if spec.is_floating {
        vec![content_id.clone(), wrapper_id.clone(), portal_id]
    } else {
        vec![content_id.clone(), portal_id]
    };
    use_register_parent_overlay_branches(parent_branch, is_open, branch_ids);

    {
        let dismiss_handle = ctx.runtime.state.dismiss;
        use_overlay_branch_provider(
            move |branch: String| dismiss_handle.register_branch(branch),
            move |branch: String| dismiss_handle.unregister_branch(&branch),
        );
    }

    {
        let rt = ctx.runtime.clone();
        let is_floating = spec.is_floating;
        use_drop(move || {
            rt.set_content_mounted(false);
            if is_floating {
                rt.clear_content_z_index();
            }
        });
    }

    // Refresh scroll buttons when opening.
    {
        let rt = ctx.runtime.clone();
        use_effect(use_reactive((&is_open,), move |(open,)| {
            if open {
                rt.refresh_scroll_buttons();
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

    let key_rt = ctx.runtime.clone();
    let on_key_down = EventHandler::new(move |evt: KeyboardEvent| {
        if is_ime_composing_key(&evt) || key_rt.is_composing_untracked() {
            return;
        }
        if is_escape_key(&evt.key()) {
            let _ = key_rt.handle_escape_keydown(&evt);
        }
    });

    if !spec.is_floating {
        let attrs = ctx.runtime.content_attributes(
            content_id,
            PlacementSide::Bottom,
            PlacementAlign::Start,
        );
        let force_mount_style = if !is_open && !is_suspended && spec.force_mount {
            "display: none;"
        } else {
            ""
        };
        let pointer_events_style = if !is_open {
            "pointer-events: none;"
        } else if is_modal_pointer_lock_active() {
            "pointer-events: auto;"
        } else {
            ""
        };
        let base_style = join_styles([
            "display: flex; flex-direction: column; box-sizing: border-box; outline: none;",
            force_mount_style,
            pointer_events_style,
            spec.style.as_deref().unwrap_or_default(),
        ]);
        let merged = merge_global_attributes(
            spec.class.as_deref(),
            base_style.as_deref(),
            spec.attributes,
        );

        if let Some(ComboboxAsChildRenderer::Static(render_child)) = spec.as_child {
            return render_child.call(ComboboxContentStaticRenderProps {
                attrs,
                class: merged.class,
                style: merged.style.unwrap_or_default(),
                on_key_down,
                attributes: merged.rest,
                children: spec.children,
            });
        }

        return rsx! {
            div {
                id: "{attrs.id()}",
                role: "{attrs.role()}",
                tabindex: "{attrs.tabindex()}",
                "aria-multiselectable": attrs.aria_multiselectable(),
                "data-combobox-content": "",
                "data-state": "{attrs.data_state_str()}",
                class: merged.class.as_deref(),
                style: merged.style.as_deref(),
                onkeydown: move |evt| on_key_down.call(evt),
                ..merged.rest,
                {spec.children}
            }
        };
    }

    let eff_side = ctx.runtime.side();
    let eff_align = ctx.runtime.align();
    let attrs = ctx
        .runtime
        .content_attributes(content_id, eff_side, eff_align);

    let placement_opt = ctx.runtime.placement();
    let readiness = ctx.runtime.content_readiness();
    let is_ready = readiness == FloatingReadiness::Ready && placement_opt.is_some();
    let css_vars = ctx.runtime.content_css_custom_properties();
    let content_z_index = ctx.runtime.content_z_index();

    let (wrapper_style, width_style, ref_hidden_str) = if is_ready {
        let placement = placement_opt.unwrap();
        let anchor_w = placement.geometry().anchor_width();
        let w_style = if spec.same_width {
            format!("width: {anchor_w}px; min-width: {anchor_w}px;")
        } else {
            format!("min-width: {anchor_w}px;")
        };
        let w_pos_style = placement.wrapper_style(readiness, content_z_index.as_deref());
        let ref_hidden = if placement.reference_hidden() {
            "true"
        } else {
            "false"
        };
        (w_pos_style, w_style, ref_hidden)
    } else {
        let w_pos_style = FloatingReadiness::wrapper_measuring_style(content_z_index.as_deref());
        let w_style = if spec.same_width {
            "width: 100%; min-width: 100%;".to_string()
        } else {
            "min-width: 100%;".to_string()
        };
        (w_pos_style, w_style, "false")
    };

    let force_mount_style = if !is_open && !is_suspended && spec.force_mount {
        "display: none;"
    } else {
        ""
    };
    let pointer_events_style = if !is_open {
        "pointer-events: none;"
    } else if is_modal_pointer_lock_active() {
        "pointer-events: auto;"
    } else {
        ""
    };
    let base_inner_style = join_styles([
        "display: flex; flex-direction: column; box-sizing: border-box;",
        width_style.as_str(),
        css_vars.as_str(),
        force_mount_style,
        pointer_events_style,
        spec.style.as_deref().unwrap_or_default(),
    ]);
    let merged = merge_global_attributes(
        spec.class.as_deref(),
        base_inner_style.as_deref(),
        spec.attributes,
    );

    let mount_rt = ctx.runtime.clone();
    let on_mounted = EventHandler::new(move |evt: MountedEvent| {
        mount_rt.capture_content_mount(evt.data());
    });

    if let Some(ComboboxAsChildRenderer::Floating(render_child)) = spec.as_child {
        let child = render_child.call(ComboboxContentRenderProps {
            attrs,
            class: merged.class,
            style: merged.style.unwrap_or_default(),
            on_key_down,
            on_mounted,
            attributes: merged.rest,
            children: spec.children,
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
                class: merged.class.as_deref(),
                style: merged.style.as_deref(),
                onkeydown: move |evt| on_key_down.call(evt),
                onmounted: move |evt| on_mounted.call(evt),
                ..merged.rest,
                {spec.children}
            }
        }
    }
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
    pub on_escape_key_down: Option<EventHandler<EscapeKeyDownEvent>>,
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
    let content_id = props
        .id
        .clone()
        .unwrap_or_else(|| ctx.runtime.relationships().content_id().to_owned());

    // Sync floating configuration and non-reactive callbacks to runtime state.
    {
        let mut s = ctx.runtime.state.preferred_side;
        sync_signal(&mut s, props.side);
        let mut a = ctx.runtime.state.preferred_align;
        sync_signal(&mut a, props.align);
        let mut so = ctx.runtime.state.side_offset;
        sync_signal(&mut so, props.side_offset);
        let mut ao = ctx.runtime.state.align_offset;
        sync_signal(&mut ao, props.align_offset);
        let mut ac = ctx.runtime.state.avoid_collisions;
        sync_signal(&mut ac, props.avoid_collisions);
        let mut cb = ctx.runtime.state.collision_boundary;
        sync_signal(&mut cb, props.collision_boundary.clone());
        let mut cp = ctx.runtime.state.collision_padding;
        sync_signal(&mut cp, props.collision_padding);
        let mut ap = ctx.runtime.state.arrow_padding;
        sync_signal(&mut ap, props.arrow_padding);
        let mut hwd = ctx.runtime.state.hide_when_detached;
        sync_signal(&mut hwd, props.hide_when_detached);
        let mut st = ctx.runtime.state.sticky;
        sync_signal(&mut st, props.sticky.clone());
        let mut ca = ctx.runtime.state.custom_anchor;
        sync_signal(&mut ca, props.custom_anchor.clone());
        let mut fm = ctx.runtime.state.force_mount;
        sync_signal(&mut fm, props.force_mount);
        let mut oio = ctx.runtime.state.on_interact_outside;
        oio.set(props.on_interact_outside);
        let mut oek = ctx.runtime.state.on_escape_key_down;
        oek.set(props.on_escape_key_down);
    }

    let as_child = if props.as_child {
        props.render_child.map(ComboboxAsChildRenderer::Floating)
    } else {
        None
    };

    render_combobox_surface(ComboboxSurfaceSpec {
        ctx,
        content_id,
        is_floating: true,
        same_width: props.same_width,
        force_mount: props.force_mount,
        class: props.class,
        style: props.style,
        as_child,
        attributes: props.attributes,
        children: props.children,
    })
}
