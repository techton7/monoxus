use dioxus::prelude::*;

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
            if open {
                rt.sync_dom_order();
                rt.start_position_monitor();
            } else {
                rt.stop_position_monitor();
            }
        }
    }));

    let focus_cid = content_id.clone();
    use_effect(use_reactive((&is_open, &readiness_sig, &placement_sig), {
        let cid = focus_cid.clone();
        move |(open, readiness, placement)| {
            if open && readiness() == crate::foundation::overlay::FloatingReadiness::Ready && placement().is_some() {
                restore_focus_element_by_id(&cid);
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

    let drop_cid = content_id.clone();
    use_drop(move || {
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
    let activedescendant = hl.map(|v| rels.item_id(&v));
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

    const DEFAULT_OVERLAY_Z_INDEX: &str = "50";

    let (wrapper_style, width_style, ref_hidden_str) = if is_ready {
        let placement = placement_opt.unwrap();
        let geom = placement.geometry();
        let anchor_w = geom.anchor_width();

        let w_style = if same_width {
            format!("width: {}px; min-width: {}px;", anchor_w, anchor_w)
        } else {
            format!("min-width: {}px;", anchor_w)
        };
        let w_pos_style = placement.wrapper_style(readiness, Some(DEFAULT_OVERLAY_Z_INDEX));
        let ref_hidden = if placement.reference_hidden() { "true" } else { "false" };
        (w_pos_style, w_style, ref_hidden)
    } else {
        let w_pos_style = crate::foundation::overlay::FloatingReadiness::wrapper_measuring_style(Some(DEFAULT_OVERLAY_Z_INDEX));
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

    let inner_style = format!(
        "{width_style} {css_vars} {arrow_pad_style} {force_mount_style} {}",
        style.as_deref().unwrap_or_default()
    );

    rsx! {
        div {
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
                    move |evt| {
                        if evt.key() == Key::Escape {
                            if let Some(ref cb) = esc_cb {
                                cb.call(evt.clone());
                            }
                            if !evt.default_action_enabled() {
                                runtime.set_escape_prevented(true);
                                return;
                            }
                            runtime.close_dropdown();
                            return;
                        }

                        #[cfg(target_arch = "wasm32")]
                        let now_ms = js_sys::Date::now();
                        #[cfg(not(target_arch = "wasm32"))]
                        let now_ms = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs_f64() * 1000.0)
                            .unwrap_or(0.0);

                        runtime.handle_content_keydown(&evt, now_ms);
                    }
                },
                {children}
            }
        }
    }
}

