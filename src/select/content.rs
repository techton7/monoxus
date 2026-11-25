use dioxus::prelude::*;
use dioxus::prelude::keyboard_types::{Key, NamedKey};

use crate::foundation::{
    browser::restore_focus_element_by_id,
    overlay::{PlacementAlign, PlacementSide},
};

use super::types::SelectContext;

#[component]
pub fn SelectContent(
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    #[props(default = PlacementSide::Bottom)] side: PlacementSide,
    #[props(default = 4.0)] side_offset: f32,
    #[props(default = PlacementAlign::Start)] align: PlacementAlign,
    #[props(default = 0.0)] align_offset: f32,
    #[props(default = true)] avoid_collisions: bool,
    #[props(default)] collision_boundary: Option<String>,
    #[props(default = 0.0)] collision_padding: f32,
    #[props(default = 0.0)] arrow_padding: f32,
    #[props(default)] sticky: Option<String>,
    #[props(default = false)] hide_when_detached: bool,
    #[props(default = false)] same_width: bool,
    #[props(default)] custom_anchor: Option<String>,
    #[props(default = false)] prevent_scroll: bool,
    #[props(default = false)] prevent_overflow_text_selection: bool,
    #[props(default = false)] force_mount: bool,
    #[props(default)] on_pointer_down_outside: Option<
        EventHandler<super::runtime::PointerDownOutsideEvent>,
    >,
    #[props(default)] on_escape_keydown: Option<EventHandler<KeyboardEvent>>,
    #[props(default)] on_close_auto_focus: Option<EventHandler<()>>,
    children: Element,
) -> Element {
    let ctx = use_context::<SelectContext>();
    let is_open = ctx.runtime.is_open();
    let rels = ctx.runtime.relationships();
    let content_id = rels.content_id().to_owned();
    let wrapper_id = format!("{content_id}-wrapper");
    let portal_id = format!("{content_id}-portal-root");

    let parent_branch = try_consume_context::<crate::foundation::overlay::OverlayBranchContext>();
    let latest_ids = use_hook(|| {
        std::rc::Rc::new(std::cell::RefCell::new((
            content_id.clone(),
            wrapper_id.clone(),
            portal_id.clone(),
        )))
    });
    {
        let mut ids = latest_ids.borrow_mut();
        if ids.0 != content_id {
            crate::foundation::overlay::unregister_active_overlay_layer(&ids.0);
            if let Some(ref branch_ctx) = parent_branch {
                branch_ctx.unregister_branch(ids.0.clone());
                branch_ctx.unregister_branch(ids.1.clone());
                branch_ctx.unregister_branch(ids.2.clone());
            }
            *ids = (content_id.clone(), wrapper_id.clone(), portal_id.clone());
        }
    }
    let mut registered_layer_id = use_signal(|| Option::<String>::None);
    if is_open {
        if registered_layer_id.peek().as_deref() != Some(content_id.as_str()) {
            if let Some(prev) = registered_layer_id.peek().as_deref() {
                crate::foundation::overlay::unregister_active_overlay_layer(prev);
            }
            crate::foundation::overlay::register_active_overlay_layer(&content_id);
            registered_layer_id.set(Some(content_id.clone()));
        }
        if let Some(ref branch_ctx) = parent_branch {
            branch_ctx.register_branch(content_id.clone());
            branch_ctx.register_branch(wrapper_id.clone());
            branch_ctx.register_branch(portal_id.clone());
        }
    } else {
        let prev = registered_layer_id.peek().clone();
        if let Some(prev_id) = prev {
            crate::foundation::overlay::unregister_active_overlay_layer(&prev_id);
            registered_layer_id.set(None);
        }
        if let Some(ref branch_ctx) = parent_branch {
            branch_ctx.unregister_branch(content_id.clone());
            branch_ctx.unregister_branch(wrapper_id.clone());
            branch_ctx.unregister_branch(portal_id.clone());
        }
    }
    use_effect(use_reactive((&is_open,), {
        let parent_branch = parent_branch.clone();
        let cid = content_id.clone();
        let wid = wrapper_id.clone();
        let pid = portal_id.clone();
        move |(open,)| {
            if let Some(ref branch_ctx) = parent_branch {
                if open {
                    branch_ctx.register_branch(cid.clone());
                    branch_ctx.register_branch(wid.clone());
                    branch_ctx.register_branch(pid.clone());
                } else {
                    branch_ctx.unregister_branch(cid.clone());
                    branch_ctx.unregister_branch(wid.clone());
                    branch_ctx.unregister_branch(pid.clone());
                }
            }
        }
    }));

    // Sync placement props and behaviors to runtime state
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
    ctx.runtime.set_collision_boundary(collision_boundary);
    ctx.runtime.set_collision_padding(collision_padding);
    ctx.runtime.set_arrow_padding(arrow_padding);
    ctx.runtime.set_sticky(sticky.clone());
    ctx.runtime
        .set_on_pointer_down_outside(on_pointer_down_outside);
    ctx.runtime.set_on_escape_keydown(on_escape_keydown);
    ctx.runtime.set_on_close_auto_focus(on_close_auto_focus);

    let readiness_sig = ctx.runtime.state.content_readiness;
    let placement_sig = ctx.runtime.state.placement;

    let runtime = ctx.runtime.clone();
    use_effect(use_reactive((&is_open,), {
        let rt = runtime.clone();
        move |(open,)| {
            if !open {
                rt.stop_position_monitor();
            } else if rt.should_render_content() {
                rt.start_position_monitor();
            }
        }
    }));

    let focus_cid = content_id.clone();
    let scroll_runtime = runtime.clone();
    let mut aligned_for_open = use_signal(|| false);
    use_effect(use_reactive((&is_open, &readiness_sig, &placement_sig), {
        let cid = focus_cid.clone();
        let rt = scroll_runtime.clone();
        move |(open, readiness, placement)| {
            if !open {
                if *aligned_for_open.peek() {
                    aligned_for_open.set(false);
                }
                return;
            }

            if readiness() == crate::foundation::overlay::FloatingReadiness::Ready
                && placement().is_some()
                && !*aligned_for_open.peek()
            {
                aligned_for_open.set(true);
                restore_focus_element_by_id(&cid);
                if let Some(ref v) = rt.highlighted_value_untracked() {
                    let item_id = rt.relationships().item_id(v);
                    let vp_id = rt.relationships().viewport_id();
                    super::viewport::scroll_item_within_container(&vp_id, &cid, &item_id);
                }
            }
        }
    }));

    let scroll_cid = content_id.clone();
    use_effect(use_reactive(
        (&is_open, &prevent_scroll),
        move |(open, prev_scroll)| {
            let lock_id = format!("select-scroll-lock-{}", scroll_cid);
            if open && prev_scroll {
                crate::foundation::browser::acquire_scroll_lock(&lock_id);
            } else {
                crate::foundation::browser::release_scroll_lock(&lock_id, Some(24));
            }
        },
    ));

    use_effect(use_reactive(
        (&is_open, &prevent_overflow_text_selection),
        |(open, prev_sel)| {
            if open && prev_sel {
                crate::foundation::browser::set_body_user_select_none();
            } else {
                crate::foundation::browser::restore_body_user_select();
            }
        },
    ));

    let order_cid = content_id.clone();
    let order_runtime = ctx.runtime.clone();
    let reg_sig = ctx.runtime.state.registry;
    use_effect(use_reactive((&is_open, &reg_sig), move |(open, _)| {
        if open {
            order_runtime.sync_dom_order(&order_cid);
        }
    }));

    let drop_ids = latest_ids.clone();
    let drop_parent_branch = parent_branch.clone();
    let drop_runtime = ctx.runtime.clone();
    use_drop(move || {
        let (drop_cid, drop_wid, drop_pid) = drop_ids.borrow().clone();
        drop_runtime.clear_content_z_index();
        crate::foundation::overlay::unregister_active_overlay_layer(&drop_cid);
        if let Some(ref branch_ctx) = drop_parent_branch {
            branch_ctx.unregister_branch(drop_cid.clone());
            branch_ctx.unregister_branch(drop_wid);
            branch_ctx.unregister_branch(drop_pid);
        }
        let lock_id = format!("select-scroll-lock-{}", drop_cid);
        crate::foundation::browser::release_scroll_lock(&lock_id, Some(24));
        crate::foundation::browser::restore_body_user_select();
    });

    let is_suspended = ctx.runtime.presence_state() == crate::foundation::overlay::PresenceState::Suspended;
    let should_render = ctx.runtime.should_render_content() || force_mount;
    if !should_render {
        return rsx! {};
    }

    let hl = ctx.runtime.highlighted_value();
    let activedescendant = hl.as_ref().map(|v| rels.item_id(v));
    let eff_side = ctx.runtime.side();
    let eff_align = ctx.runtime.align();
    // Read live runtime state for content attributes
    let attrs =
        ctx.runtime
            .content_attributes_with_side_and_align(activedescendant, eff_side, eff_align);

    let placement_opt = ctx.runtime.placement();
    let readiness = ctx.runtime.content_readiness();
    let is_ready = readiness == crate::foundation::overlay::FloatingReadiness::Ready && placement_opt.is_some();

    let css_vars = ctx.runtime.content_css_custom_properties();
    let content_z_index = ctx.runtime.content_z_index();

    let (wrapper_style, width_style, ref_hidden_str) = if is_ready {
        let placement = placement_opt.unwrap();
        let geom = placement.geometry();
        let anchor_w = geom.anchor_width();

        let w_style = if same_width {
            format!("width: {}px; min-width: {}px;", anchor_w, anchor_w)
        } else {
            format!("min-width: {}px;", anchor_w)
        };
        let w_pos_style = placement.wrapper_style(readiness, content_z_index.as_deref());
        let ref_hidden = if placement.reference_hidden() { "true" } else { "false" };
        (w_pos_style, w_style, ref_hidden)
    } else {
        let w_pos_style =
            crate::foundation::overlay::FloatingReadiness::wrapper_measuring_style(
                content_z_index.as_deref(),
            );
        let w_style = if same_width {
            "width: 100%; min-width: 100%;".to_string()
        } else {
            "min-width: 100%;".to_string()
        };
        (w_pos_style, w_style, "false")
    };

    let force_mount_style = if !is_open && !is_suspended && force_mount {
        "display: none;"
    } else {
        ""
    };

    let arrow_pad_style = format!("--bits-select-arrow-padding: {}px;", arrow_padding);

    let pointer_events_style = if !is_open {
        "pointer-events: none;"
    } else if prevent_scroll || crate::foundation::browser::is_body_pointer_events_locked() {
        "pointer-events: auto;"
    } else {
        ""
    };

    let inner_style = format!(
        "display: flex; flex-direction: column; box-sizing: border-box; {width_style} {css_vars} {arrow_pad_style} {force_mount_style} {pointer_events_style} {}",
        style.as_deref().unwrap_or_default()
    );

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
                class: class.as_deref().unwrap_or_default(),
                style: "{inner_style}",
                "data-state": if is_open { "open" } else { "closed" },
                "data-side": "{attrs.data_side_str()}",
                "data-align": "{attrs.data_align_str()}",
                "data-positioning-state": ctx.runtime.content_positioning_state(),
                "data-sticky": sticky.as_deref().unwrap_or("false"),
                "data-prevent-scroll": if prevent_scroll { "true" } else { "false" },
                "data-prevent-overflow-text-selection": if prevent_overflow_text_selection { "true" } else { "false" },
                "data-force-mount": if force_mount { "true" } else { "false" },
                "data-custom-anchor": custom_anchor.as_deref().unwrap_or(""),
                onkeydown: {
                    let runtime = ctx.runtime.clone();
                    let esc_cb = on_escape_keydown;
                    let keydown_cid = content_id.clone();
                    move |evt| {
                        if evt.key() == Key::Named(NamedKey::Escape) {
                            if !runtime.is_open() {
                                evt.prevent_default();
                                evt.stop_propagation();
                                return;
                            }
                            if crate::foundation::overlay::is_overlay_escape_consumed()
                                || !crate::foundation::overlay::is_topmost_overlay_layer(&keydown_cid)
                            {
                                return;
                            }
                            if let Some(ref cb) = esc_cb {
                                cb.call(evt.clone());
                            }
                            if !evt.default_action_enabled() {
                                runtime.set_escape_prevented(true);
                                evt.stop_propagation();
                                return;
                            }
                            crate::foundation::overlay::mark_overlay_escape_consumed();
                            evt.prevent_default();
                            evt.stop_propagation();
                            runtime.close_dropdown();
                            return;
                        }

                        let now_ms = oxidase::time::monotonic_now_ms();

                        runtime.handle_content_keydown(&evt, now_ms);
                    }
                },
                onmounted: {
                    let runtime = ctx.runtime.clone();
                    move |evt: Event<MountedData>| {
                        let mut ch = runtime.state.content_handle;
                        ch.set(Some(evt.data()));
                        runtime.sync_content_z_index();
                        runtime.start_position_monitor();
                    }
                },
                {children}
            }
        }
    }
}

