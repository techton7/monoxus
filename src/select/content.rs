use dioxus::prelude::*;

use crate::foundation::{
    compose::{
        FloatingContentChildProps, collect_user_attrs, join_styles, merge_props, push_attr_opt,
        push_event_handler,
    },
    keyboard::{is_escape_key, is_ime_composing_key},
    overlay::{
        DismissBehavior, FloatingReadiness, OverlayBranchContext, PlacementAlign, PlacementSide,
        PresenceState, StickyStrategy, is_modal_pointer_lock_active, is_overlay_escape_consumed,
        is_topmost_overlay_layer, mark_overlay_escape_consumed, use_overlay_branch_provider,
        use_register_parent_overlay_branches,
    },
    shared::{EscapeKeyDownEvent, PointerDownOutsideEvent, PreventableEvent},
    state::sync_signal,
};

use super::types::SelectContext;

pub(crate) enum SelectSurfaceChild {
    Floating(Option<Callback<FloatingContentChildProps, Element>>),
    Static(Option<Callback<Vec<Attribute>, Element>>),
}

/// Plain Rust specification for [`render_select_surface`] shared by `SelectContent` and
/// `SelectContentStatic` (zero internal `#[component]`, `Dioxus #5894`).
pub(crate) struct SelectSurfaceSpec {
    pub ctx: SelectContext,
    pub id: Option<String>,
    pub is_floating: bool,
    pub class: Option<String>,
    pub style: Option<String>,
    pub same_width: bool,
    pub arrow_padding: f32,
    pub sticky: Option<StickyStrategy>,
    pub prevent_scroll: bool,
    pub prevent_overflow_text_selection: bool,
    pub force_mount: bool,
    pub custom_anchor: Option<String>,
    pub child: SelectSurfaceChild,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Plain Rust render helper unifying `SelectContent` and `SelectContentStatic` (`Dioxus #5894`,
/// `Dioxus #3717`, `B7` overlay branch & LIFO layer registration).
pub(crate) fn render_select_surface(spec: SelectSurfaceSpec) -> Element {
    let ctx = spec.ctx;
    ctx.runtime.set_is_static_content(!spec.is_floating);

    let is_open = ctx.runtime.is_open();
    let rels = ctx.runtime.select().relationships().clone();
    let default_content_id = rels.content_id().to_owned();
    let content_id = spec
        .id
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| default_content_id.clone());
    ctx.runtime.set_content_id(content_id.clone());

    let wrapper_id = format!("{content_id}-wrapper");
    let portal_id = format!("{default_content_id}-portal-root");

    let is_suspended = ctx.runtime.presence_state() == PresenceState::Suspended;
    let should_render = ctx.runtime.should_render_content() || spec.force_mount;
    let is_hidden = !spec.force_mount && ctx.runtime.is_hidden();

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
                    let vp_id = scroll_runtime.current_viewport_id();
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
        let drop_cid = content_id.clone();
        let drop_default_cid = default_content_id;
        use_drop(move || {
            drop_runtime.clear_content_z_index();
            drop_runtime.set_keep_mounted(false);
            if drop_runtime.current_content_id() == drop_cid {
                drop_runtime.set_content_id(drop_default_cid);
            }
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
    let kd_cid = content_id.clone();
    let on_key_down = EventHandler::new(move |evt: KeyboardEvent| {
        if is_ime_composing_key(&evt) {
            return;
        }
        if is_escape_key(&evt.key()) {
            let esc_behavior = kd_runtime
                .escape_key_down_behavior()
                .unwrap_or(DismissBehavior::Close);
            if !esc_behavior.dismisses() {
                return;
            }
            if !kd_runtime.is_open() {
                evt.prevent_default();
                evt.stop_propagation();
                return;
            }
            if let Some(dismiss_handle) = kd_runtime.state.dismiss {
                let _ = dismiss_handle.handle_keydown(&evt);
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
    });

    if !spec.is_floating {
        let attrs = ctx.runtime.content_attributes(activedescendant);
        let role_str = attrs.role();
        let tabindex_str = attrs.tabindex().to_string();
        let data_side_str = attrs.data_side_str();
        let data_align_str = attrs.data_align_str();
        let aria_activedescendant = attrs.aria_activedescendant().map(str::to_owned);

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
        ]);

        let focus_in_rt = ctx.runtime.clone();
        let focus_out_rt = ctx.runtime.clone();

        let mut base = Vec::with_capacity(14);
        base.push(Attribute::new("id", content_id, None, false));
        base.push(Attribute::new("role", role_str, None, false));
        base.push(Attribute::new("tabindex", tabindex_str, None, false));
        push_attr_opt(&mut base, "aria-activedescendant", aria_activedescendant);
        push_attr_opt(&mut base, "style", base_style);
        base.push(Attribute::new(
            "data-state",
            if is_open { "open" } else { "closed" },
            None,
            false,
        ));
        base.push(Attribute::new("data-side", data_side_str, None, false));
        base.push(Attribute::new("data-align", data_align_str, None, false));
        push_attr_opt(
            &mut base,
            "data-starting-style",
            ctx.runtime.data_starting_style(),
        );
        push_attr_opt(
            &mut base,
            "data-ending-style",
            ctx.runtime.data_ending_style(),
        );
        if is_hidden {
            base.push(Attribute::new("hidden", true, None, false));
        }
        push_event_handler(&mut base, "onkeydown", Some(on_key_down));
        push_event_handler(
            &mut base,
            "onfocusin",
            Some(EventHandler::new(move |evt: FocusEvent| {
                focus_in_rt.notify_composite_focus_in(evt);
            })),
        );
        push_event_handler(
            &mut base,
            "onfocusout",
            Some(EventHandler::new(move |evt: FocusEvent| {
                focus_out_rt.notify_composite_focus_out(evt);
            })),
        );

        let user = collect_user_attrs(spec.class, spec.style, spec.attributes);
        let merged = merge_props(base, user);

        if let SelectSurfaceChild::Static(Some(child)) = spec.child {
            return child.call(merged);
        }

        return rsx! {
            div {
                ..merged,
                {spec.children}
            }
        };
    }

    let eff_side = ctx.runtime.side();
    let eff_align = ctx.runtime.align();
    let attrs =
        ctx.runtime
            .content_attributes_with_side_and_align(activedescendant, eff_side, eff_align);
    let role_str = attrs.role();
    let tabindex_str = attrs.tabindex().to_string();
    let data_side_str = attrs.data_side_str();
    let data_align_str = attrs.data_align_str();
    let aria_activedescendant = attrs.aria_activedescendant().map(str::to_owned);

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
    let arrow_pad_style = format!(
        "--monoxus-select-arrow-padding: {}px; --bits-select-arrow-padding: {}px;",
        spec.arrow_padding, spec.arrow_padding
    );
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
    ]);

    let sticky_attr = match spec.sticky {
        Some(StickyStrategy::Always) => "always",
        Some(StickyStrategy::Partial) => "partial",
        None => "false",
    };

    let mut wrapper_props = Vec::with_capacity(5);
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
    wrapper_props.push(Attribute::new(
        "data-reference-hidden",
        ref_hidden_str,
        None,
        false,
    ));
    if is_hidden {
        wrapper_props.push(Attribute::new("hidden", true, None, false));
    }

    let mount_runtime = ctx.runtime.clone();
    let focus_in_rt = ctx.runtime.clone();
    let focus_out_rt = ctx.runtime.clone();

    let mut base = Vec::with_capacity(21);
    base.push(Attribute::new("id", content_id, None, false));
    base.push(Attribute::new("role", role_str, None, false));
    base.push(Attribute::new("tabindex", tabindex_str, None, false));
    push_attr_opt(&mut base, "aria-activedescendant", aria_activedescendant);
    push_attr_opt(&mut base, "style", base_inner_style);
    base.push(Attribute::new(
        "data-state",
        if is_open { "open" } else { "closed" },
        None,
        false,
    ));
    base.push(Attribute::new("data-side", data_side_str, None, false));
    base.push(Attribute::new("data-align", data_align_str, None, false));
    push_attr_opt(
        &mut base,
        "data-starting-style",
        ctx.runtime.data_starting_style(),
    );
    push_attr_opt(
        &mut base,
        "data-ending-style",
        ctx.runtime.data_ending_style(),
    );
    if is_hidden {
        base.push(Attribute::new("hidden", true, None, false));
    }
    base.push(Attribute::new(
        "data-positioning-state",
        ctx.runtime.content_positioning_state(),
        None,
        false,
    ));
    base.push(Attribute::new("data-sticky", sticky_attr, None, false));
    base.push(Attribute::new(
        "data-prevent-scroll",
        if spec.prevent_scroll { "true" } else { "false" },
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-prevent-overflow-text-selection",
        if spec.prevent_overflow_text_selection {
            "true"
        } else {
            "false"
        },
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-force-mount",
        if spec.force_mount { "true" } else { "false" },
        None,
        false,
    ));
    push_attr_opt(&mut base, "data-custom-anchor", spec.custom_anchor);
    push_event_handler(&mut base, "onkeydown", Some(on_key_down));
    push_event_handler(
        &mut base,
        "onfocusin",
        Some(EventHandler::new(move |evt: FocusEvent| {
            focus_in_rt.notify_composite_focus_in(evt);
        })),
    );
    push_event_handler(
        &mut base,
        "onfocusout",
        Some(EventHandler::new(move |evt: FocusEvent| {
            focus_out_rt.notify_composite_focus_out(evt);
        })),
    );
    push_event_handler(
        &mut base,
        "onmounted",
        Some(EventHandler::new(move |evt: MountedEvent| {
            mount_runtime.capture_content_mount(evt.data());
        })),
    );

    let user = collect_user_attrs(spec.class, spec.style, spec.attributes);
    let merged = merge_props(base, user);

    if let SelectSurfaceChild::Floating(Some(child)) = spec.child {
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
}

#[derive(Props, Clone, PartialEq)]
pub struct SelectContentProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
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
    #[props(default, into)]
    pub collision_boundary: Option<String>,
    #[props(default = 0.0)]
    pub collision_padding: f32,
    #[props(default = 0.0)]
    pub arrow_padding: f32,
    #[props(default)]
    pub sticky: Option<StickyStrategy>,
    #[props(default = false)]
    pub hide_when_detached: bool,
    #[props(default = false)]
    pub same_width: bool,
    #[props(default, into)]
    pub custom_anchor: Option<String>,
    #[props(default = false)]
    pub prevent_scroll: bool,
    #[props(default = false)]
    pub prevent_overflow_text_selection: bool,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default = false)]
    pub keep_mounted: bool,
    #[props(default)]
    pub escape_key_down_behavior: Option<DismissBehavior>,
    #[props(default)]
    pub interact_outside_behavior: Option<DismissBehavior>,
    #[props(default)]
    pub on_pointer_down_outside: Option<EventHandler<PointerDownOutsideEvent>>,
    #[props(default)]
    pub on_escape_key_down: Option<EventHandler<EscapeKeyDownEvent>>,
    #[props(default)]
    pub on_close_auto_focus: Option<EventHandler<()>>,
    #[props(default)]
    pub child: Option<Callback<FloatingContentChildProps, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn SelectContent(props: SelectContentProps) -> Element {
    let SelectContentProps {
        id,
        class,
        style,
        side,
        side_offset,
        align,
        align_offset,
        avoid_collisions,
        collision_boundary,
        collision_padding,
        arrow_padding,
        sticky,
        hide_when_detached,
        same_width,
        custom_anchor,
        prevent_scroll,
        prevent_overflow_text_selection,
        force_mount,
        keep_mounted,
        escape_key_down_behavior,
        interact_outside_behavior,
        on_pointer_down_outside,
        on_escape_key_down,
        on_close_auto_focus,
        child,
        attributes,
        children,
    } = props;
    let ctx = use_context::<SelectContext>();

    // Sync placement props and non-reactive callbacks to runtime state
    ctx.runtime.set_side(side);
    ctx.runtime.set_align(align);
    ctx.runtime.set_side_offset(side_offset);
    ctx.runtime.set_align_offset(align_offset);
    ctx.runtime.set_avoid_collisions(avoid_collisions);
    ctx.runtime.set_hide_when_detached(hide_when_detached);
    ctx.runtime.set_custom_anchor(custom_anchor.clone());
    ctx.runtime.set_prevent_scroll(prevent_scroll);
    ctx.runtime
        .set_prevent_overflow_text_selection(prevent_overflow_text_selection);
    ctx.runtime.set_force_mount(force_mount);
    ctx.runtime.set_keep_mounted(keep_mounted);
    ctx.runtime.set_collision_boundary(collision_boundary);
    ctx.runtime.set_collision_padding(collision_padding);
    ctx.runtime.set_arrow_padding(arrow_padding);
    ctx.runtime.set_sticky(sticky);
    ctx.runtime
        .set_escape_key_down_behavior(escape_key_down_behavior);
    ctx.runtime
        .set_interact_outside_behavior(interact_outside_behavior);
    ctx.runtime
        .set_on_pointer_down_outside(on_pointer_down_outside);
    ctx.runtime.set_on_escape_key_down(on_escape_key_down);
    ctx.runtime.set_on_close_auto_focus(on_close_auto_focus);

    render_select_surface(SelectSurfaceSpec {
        ctx,
        id,
        is_floating: true,
        class,
        style,
        same_width,
        arrow_padding,
        sticky,
        prevent_scroll,
        prevent_overflow_text_selection,
        force_mount,
        custom_anchor,
        child: SelectSurfaceChild::Floating(child),
        attributes,
        children,
    })
}
