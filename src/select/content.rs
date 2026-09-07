use dioxus::prelude::*;

use crate::foundation::{
    browser::restore_focus_element_by_id,
    overlay::{PlacementAlign, PlacementSide},
};

use super::types::SelectContext;

#[component]
pub fn SelectContentStatic(
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    let ctx = use_context::<SelectContext>();
    let is_open = ctx.runtime.is_open();
    let rels = ctx.runtime.relationships();
    let content_id = rels.content_id().to_owned();

    let runtime = ctx.runtime.clone();
    use_effect(use_reactive((&is_open,), {
        let cid = content_id.clone();
        let rt = runtime.clone();
        move |(open,)| {
            if open {
                restore_focus_element_by_id(&cid);
                rt.sync_dom_order();
            }
        }
    }));

    if !is_open {
        return rsx! {};
    }

    let hl = ctx.runtime.highlighted_value();
    let activedescendant = hl.map(|v| rels.item_id(&v));
    // Live content attributes reading live runtime.is_open()
    let attrs = ctx.runtime.content_attributes(activedescendant);

    rsx! {
        div {
            id: "{attrs.id()}",
            role: "{attrs.role()}",
            tabindex: "{attrs.tabindex()}",
            aria_activedescendant: attrs.aria_activedescendant(),
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            "data-state": "{attrs.data_state_str()}",
            "data-side": "{attrs.data_side_str()}",
            "data-align": "{attrs.data_align_str()}",
            onkeydown: {
                let runtime = ctx.runtime.clone();
                move |evt| {
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
    #[props(default)] on_pointer_down_outside: Option<EventHandler<PointerEvent>>,
    #[props(default)] on_escape_keydown: Option<EventHandler<KeyboardEvent>>,
    #[props(default)] on_close_auto_focus: Option<EventHandler<()>>,
    children: Element,
) -> Element {
    let ctx = use_context::<SelectContext>();
    let is_open = ctx.runtime.is_open();
    let rels = ctx.runtime.relationships();
    let content_id = rels.content_id().to_owned();

    // Sync placement props to runtime state
    ctx.runtime.set_side_offset(side_offset);
    ctx.runtime.set_align_offset(align_offset);

    let runtime = ctx.runtime.clone();
    use_effect(use_reactive((&is_open,), {
        let cid = content_id.clone();
        let rt = runtime.clone();
        move |(open,)| {
            if open {
                restore_focus_element_by_id(&cid);
                rt.sync_dom_order();
                rt.start_position_monitor();
            } else {
                rt.stop_position_monitor();
            }
        }
    }));

    if !is_open {
        return rsx! {};
    }

    let hl = ctx.runtime.highlighted_value();
    let activedescendant = hl.map(|v| rels.item_id(&v));
    let eff_side = ctx.runtime.side();
    let eff_align = ctx.runtime.align();
    // Read live runtime state for content attributes
    let attrs = ctx.runtime.content_attributes_with_side_and_align(
        activedescendant,
        eff_side,
        eff_align,
    );

    let side_placement_style = match eff_side {
        PlacementSide::Top => format!(
            "bottom: calc(100% + {}px) !important; top: auto !important;",
            side_offset
        ),
        _ => format!(
            "top: calc(100% + {}px) !important; bottom: auto !important;",
            side_offset
        ),
    };
    let align_placement_style = match eff_align {
        PlacementAlign::End => "right: 0; left: auto;",
        PlacementAlign::Center => "left: 50%; transform: translateX(-50%);",
        PlacementAlign::Start => "left: 0; right: auto;",
    };

    let width_style = if same_width {
        "width: 100%; min-width: 100%;"
    } else {
        "min-width: 100%;"
    };

    let transform_origin = match (eff_side, eff_align) {
        (PlacementSide::Top, PlacementAlign::Start) => "bottom left",
        (PlacementSide::Top, PlacementAlign::Center) => "bottom center",
        (PlacementSide::Top, PlacementAlign::End) => "bottom right",
        (PlacementSide::Bottom, PlacementAlign::Start) => "top left",
        (PlacementSide::Bottom, PlacementAlign::Center) => "top center",
        (PlacementSide::Bottom, PlacementAlign::End) => "top right",
        (PlacementSide::Left, _) => "center right",
        (PlacementSide::Right, _) => "center left",
    };

    let base_position_style = format!(
        "position: absolute; {align_placement_style} {width_style} --bits-select-content-transform-origin: {transform_origin}; --bits-select-anchor-width: 100%;"
    );
    let merged_style = if let Some(custom) = style {
        format!("{base_position_style} {side_placement_style} {custom}")
    } else {
        format!("{base_position_style} {side_placement_style}")
    };

    rsx! {
        div {
            id: "{attrs.id()}",
            role: "{attrs.role()}",
            tabindex: "{attrs.tabindex()}",
            aria_activedescendant: attrs.aria_activedescendant(),
            class: class.as_deref().unwrap_or_default(),
            style: "{merged_style}",
            "data-state": "{attrs.data_state_str()}",
            "data-side": "{attrs.data_side_str()}",
            "data-align": "{attrs.data_align_str()}",
            onkeydown: {
                let runtime = ctx.runtime.clone();
                let esc_cb = on_escape_keydown;
                let close_focus_cb = on_close_auto_focus;
                move |evt| {
                    if evt.key() == Key::Escape {
                        if let Some(ref cb) = esc_cb {
                            cb.call(evt.clone());
                        }
                        if !evt.default_action_enabled() {
                            return;
                        }
                        runtime.close_dropdown();
                        if let Some(ref cb) = close_focus_cb {
                            cb.call(());
                        }
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

#[component]
pub fn SelectViewport(
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        div {
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or("overflow-y: auto;"),
            {children}
        }
    }
}

#[component]
pub fn SelectArrow(
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    #[props(default = 10.0)] width: f32,
    #[props(default = 5.0)] height: f32,
    children: Option<Element>,
) -> Element {
    if let Some(c) = children {
        return rsx! {
            div {
                class: class.as_deref().unwrap_or_default(),
                style: style.as_deref().unwrap_or_default(),
                aria_hidden: "true",
                {c}
            }
        };
    }

    rsx! {
        svg {
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            width: "{width}",
            height: "{height}",
            view_box: "0 0 {width} {height}",
            "aria-hidden": "true",
            polygon {
                points: "0,{height} {width / 2.0},0 {width},{height}",
                fill: "currentColor",
            }
        }
    }
}
