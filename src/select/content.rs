use dioxus::prelude::*;

use crate::foundation::{
    compose::{join_styles, merge_global_attributes},
    keyboard::{is_escape_key, is_ime_composing_key},
    overlay::{
        FloatingReadiness, OverlayBranchContext, PlacementAlign, PlacementSide, PresenceState,
        is_modal_pointer_lock_active, is_overlay_escape_consumed, is_topmost_overlay_layer,
        mark_overlay_escape_consumed, use_overlay_branch_provider,
        use_register_parent_overlay_branches,
    },
    shared::{EscapeKeyDownEvent, PointerDownOutsideEvent, PreventableEvent},
    state::sync_signal,
};

use super::types::SelectContext;

/// Plain Rust specification for [`render_select_surface`] shared by `SelectContent` and
/// `SelectContentStatic` (zero internal `#[component]`, `Dioxus #5894`).
pub(crate) struct SelectSurfaceSpec {
    pub ctx: SelectContext,
    pub is_floating: bool,
    pub class: Option<String>,
    pub style: Option<String>,
    pub same_width: bool,
    pub arrow_padding: f32,
    pub sticky: Option<String>,
    pub prevent_scroll: bool,
    pub prevent_overflow_text_selection: bool,
    pub force_mount: bool,
    pub custom_anchor: Option<String>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Plain Rust render helper unifying `SelectContent` and `SelectContentStatic` (`Dioxus #5894`,
/// `Dioxus #3717`, `B7` overlay branch & LIFO layer registration).
pub(crate) fn render_select_surface(spec: SelectSurfaceSpec) -> Element {
    let ctx = spec.ctx;
    ctx.runtime.set_is_static_content(!spec.is_floating);

    let is_open = ctx.runtime.is_open();
    let rels = ctx.runtime.relationships();
    let content_id = rels.content_id().to_owned();
    let wrapper_id = format!("{content_id}-wrapper");
    let portal_id = format!("{content_id}-portal-root");

    let is_suspended = ctx.runtime.presence_state() == PresenceState::Suspended;
    let should_render = ctx.runtime.should_render_content() || spec.force_mount;

    // Register with enclosing OverlayBranchContext (e.g. Dialog) & provide child branch forwarding (`B7`).
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
            move |branch: String| {
                if let Some(d) = dismiss_handle {
                    d.register_branch(branch);
                }
            },
            move |branch: String| {
                if let Some(d) = dismiss_handle {
                    d.unregister_branch(&branch);
                }
            },
        );
    }

    // Static readiness synchronization.
    let is_floating = spec.is_floating;
    let readiness_runtime = ctx.runtime.clone();
    use_effect(use_reactive((&is_open, &should_render), move |(open, renderable)| {
        if !is_floating && open && renderable {
            let mut readiness = readiness_runtime.state.fallback_readiness;
            sync_signal(&mut readiness, FloatingReadiness::Ready);
        }
    }));

    // Align focus & scroll highlighted item into view once open and ready.
    let readiness_now = ctx.runtime.content_readiness();
    let has_placement = !spec.is_floating || ctx.runtime.placement().is_some();
    let focus_cid = content_id.clone();
    let scroll_runtime = ctx.runtime.clone();
    let mut aligned_for_open = use_signal(|| false);
    use_effect(use_reactive(
        (&is_open, &readiness_now, &has_placement),
        move |(open, readiness, positioned)| {
            if !open {
                if *aligned_for_open.peek() {
                    aligned_for_open.set(false);
                }
                return;
            }
            if readiness == FloatingReadiness::Ready && positioned && !*aligned_for_open.peek() {
                aligned_for_open.set(true);
                crate::foundation::browser::restore_focus_element_by_id(&focus_cid);
                if let Some(ref v) = scroll_runtime.highlighted_value_untracked() {
                    let item_id = scroll_runtime.relationships().item_id(v);
                    let vp_id = scroll_runtime.relationships().viewport_id();
                    super::viewport::scroll_item_within_container(&vp_id, &focus_cid, &item_id);
                }
            }
        },
    ));

    // Prevent overflow text selection while open (floating SelectContent).
    let prev_sel = spec.is_floating && spec.prevent_overflow_text_selection;
    use_effect(use_reactive((&is_open, &prev_sel), move |(open, enabled)| {
        if open && enabled {
            crate::foundation::browser::set_body_user_select_none();
        } else {
            crate::foundation::browser::restore_body_user_select();
        }
    }));

    // Synchronize mounted DOM order when open or registry changes.
    let order_cid = content_id.clone();
    let order_runtime = ctx.runtime.clone();
    let reg_sig = ctx.runtime.state.registry;
    use_effect(use_reactive((&is_open, &reg_sig), move |(open, _)| {
        if open {
            order_runtime.sync_dom_order(&order_cid);
        }
    }));

    {
        let drop_runtime = ctx.runtime.clone();
        use_drop(move || {
            drop_runtime.clear_content_z_index();
            if is_floating {
                crate::foundation::browser::restore_body_user_select();
            }
        });
    }

    if !should_render {
        return rsx! {};
    }

    let hl = ctx.runtime.highlighted_value();
    let activedescendant = hl.as_ref().map(|v| rels.item_id(v));

    let kd_runtime = ctx.runtime.clone();
    let kd_cid = content_id;
    let on_key_down = move |evt: KeyboardEvent| {
        if is_ime_composing_key(&evt) {
            return;
        }
        if is_escape_key(&evt.key()) {
            if !kd_runtime.is_open() {
                evt.prevent_default();
                evt.stop_propagation();
                return;
            }
            if let Some(dismiss_handle) = kd_runtime.state.dismiss {
                if dismiss_handle.handle_keydown(&evt) {
                    evt.prevent_default();
                }
                return;
            }
            if is_overlay_escape_consumed() || !is_topmost_overlay_layer(&kd_cid) {
                return;
            }
            let esc_evt: EscapeKeyDownEvent = PreventableEvent::empty();
            if let Some(cb) = kd_runtime
                .state
                .on_escape_key_down
                .try_read()
                .ok()
                .and_then(|s| *s)
            {
                cb.call(esc_evt.clone());
            }
            mark_overlay_escape_consumed();
            evt.prevent_default();
            evt.stop_propagation();
            if !esc_evt.is_default_prevented() {
                kd_runtime.close_dropdown();
            }
            return;
        }

        let now_ms = oxidase::time::monotonic_now_ms();
        kd_runtime.handle_content_keydown(&evt, now_ms);
    };

    if !spec.is_floating {
        let attrs = ctx.runtime.content_attributes(activedescendant);
        let pointer_events_style = if !is_open {
            "pointer-events: none;"
        } else if is_modal_pointer_lock_active() {
            "pointer-events: auto;"
        } else {
            ""
        };
        let force_mount_style = if !is_open && !is_suspended && spec.force_mount {
            "display: none;"
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

        let focus_in_rt = ctx.runtime.clone();
        let focus_out_rt = ctx.runtime.clone();

        return rsx! {
            div {
                id: "{attrs.id()}",
                role: "{attrs.role()}",
                tabindex: "{attrs.tabindex()}",
                aria_activedescendant: attrs.aria_activedescendant(),
                class: merged.class.as_deref(),
                style: merged.style.as_deref(),
                "data-state": if is_open { "open" } else { "closed" },
                "data-side": "{attrs.data_side_str()}",
                "data-align": "{attrs.data_align_str()}",
                onkeydown: on_key_down,
                onfocusin: move |evt| focus_in_rt.notify_composite_focus_in(evt),
                onfocusout: move |evt| focus_out_rt.notify_composite_focus_out(evt),
                ..merged.rest,
                {spec.children}
            }
        };
    }

    let eff_side = ctx.runtime.side();
    let eff_align = ctx.runtime.align();
    let attrs =
        ctx.runtime
            .content_attributes_with_side_and_align(activedescendant, eff_side, eff_align);

    let placement_opt = ctx.runtime.placement();
    let readiness = ctx.runtime.content_readiness();
    let is_ready = readiness == FloatingReadiness::Ready && placement_opt.is_some();

    let css_vars = ctx.runtime.content_css_custom_properties();
    let content_z_index = ctx.runtime.content_z_index();

    let (wrapper_style, width_style, ref_hidden_str) = if is_ready {
        let placement = placement_opt.unwrap();
        let geom = placement.geometry();
        let anchor_w = geom.anchor_width();

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
    let arrow_pad_style = format!("--bits-select-arrow-padding: {}px;", spec.arrow_padding);
    let pointer_events_style = if !is_open {
        "pointer-events: none;"
    } else if spec.prevent_scroll || is_modal_pointer_lock_active() {
        "pointer-events: auto;"
    } else {
        ""
    };

    let base_inner_style = join_styles([
        "display: flex; flex-direction: column; box-sizing: border-box;",
        width_style.as_str(),
        css_vars.as_str(),
        arrow_pad_style.as_str(),
        force_mount_style,
        pointer_events_style,
        spec.style.as_deref().unwrap_or_default(),
    ]);
    let merged = merge_global_attributes(
        spec.class.as_deref(),
        base_inner_style.as_deref(),
        spec.attributes,
    );

    let mount_runtime = ctx.runtime.clone();
    let focus_in_rt = ctx.runtime.clone();
    let focus_out_rt = ctx.runtime.clone();

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
                aria_activedescendant: attrs.aria_activedescendant(),
                class: merged.class.as_deref(),
                style: merged.style.as_deref(),
                "data-state": if is_open { "open" } else { "closed" },
                "data-side": "{attrs.data_side_str()}",
                "data-align": "{attrs.data_align_str()}",
                "data-positioning-state": ctx.runtime.content_positioning_state(),
                "data-sticky": spec.sticky.as_deref().unwrap_or("false"),
                "data-prevent-scroll": if spec.prevent_scroll { "true" } else { "false" },
                "data-prevent-overflow-text-selection": if spec.prevent_overflow_text_selection { "true" } else { "false" },
                "data-force-mount": if spec.force_mount { "true" } else { "false" },
                "data-custom-anchor": spec.custom_anchor.as_deref().unwrap_or(""),
                onkeydown: on_key_down,
                onfocusin: move |evt| focus_in_rt.notify_composite_focus_in(evt),
                onfocusout: move |evt| focus_out_rt.notify_composite_focus_out(evt),
                onmounted: move |evt: Event<MountedData>| {
                    mount_runtime.capture_content_mount(evt.data());
                },
                ..merged.rest,
                {spec.children}
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct SelectContentProps {
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
    pub prevent_scroll: bool,
    #[props(default = false)]
    pub prevent_overflow_text_selection: bool,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default)]
    pub on_pointer_down_outside: Option<EventHandler<PointerDownOutsideEvent>>,
    #[props(default)]
    pub on_escape_key_down: Option<EventHandler<EscapeKeyDownEvent>>,
    #[props(default)]
    pub on_close_auto_focus: Option<EventHandler<()>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn SelectContent(props: SelectContentProps) -> Element {
    let ctx = use_context::<SelectContext>();

    // Sync placement props and non-reactive callbacks to runtime state
    ctx.runtime.set_side(props.side);
    ctx.runtime.set_align(props.align);
    ctx.runtime.set_side_offset(props.side_offset);
    ctx.runtime.set_align_offset(props.align_offset);
    ctx.runtime.set_avoid_collisions(props.avoid_collisions);
    ctx.runtime.set_hide_when_detached(props.hide_when_detached);
    ctx.runtime.set_custom_anchor(props.custom_anchor.clone());
    ctx.runtime.set_prevent_scroll(props.prevent_scroll);
    ctx.runtime
        .set_prevent_overflow_text_selection(props.prevent_overflow_text_selection);
    ctx.runtime.set_force_mount(props.force_mount);
    ctx.runtime.set_collision_boundary(props.collision_boundary);
    ctx.runtime.set_collision_padding(props.collision_padding);
    ctx.runtime.set_arrow_padding(props.arrow_padding);
    ctx.runtime.set_sticky(props.sticky.clone());
    ctx.runtime
        .set_on_pointer_down_outside(props.on_pointer_down_outside);
    ctx.runtime.set_on_escape_key_down(props.on_escape_key_down);
    ctx.runtime.set_on_close_auto_focus(props.on_close_auto_focus);

    render_select_surface(SelectSurfaceSpec {
        ctx,
        is_floating: true,
        class: props.class,
        style: props.style,
        same_width: props.same_width,
        arrow_padding: props.arrow_padding,
        sticky: props.sticky,
        prevent_scroll: props.prevent_scroll,
        prevent_overflow_text_selection: props.prevent_overflow_text_selection,
        force_mount: props.force_mount,
        custom_anchor: props.custom_anchor,
        attributes: props.attributes,
        children: props.children,
    })
}
