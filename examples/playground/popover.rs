use dioxus::prelude::*;
use monoxus::{
    foundation::{
        overlay::{FloatingLayer, FloatingPlacement, PlacementAlign, PlacementSide, PortalHost},
        shared::ScopeHandle,
    },
    popover::{Popover, PopoverCloseFocusPolicy, PopoverOpenFocusPolicy, use_popover_runtime},
};

const CARD_STYLE: &str = "display: grid; gap: 1rem; padding: 1.25rem; border-radius: 0.75rem; border: 1px solid #d8b4fe; background-color: white; box-shadow: 0 10px 30px rgba(88, 28, 135, 0.08);";
const MUTED_STYLE: &str = "margin: 0; color: #6b21a8;";
const CANVAS_STYLE: &str = "position: relative; min-height: 20rem; padding: 1.25rem; border-radius: 0.85rem; border: 1px dashed #d8b4fe; background: linear-gradient(135deg, #faf5ff, #f5f3ff); overflow: hidden;";

#[component]
pub fn PopoverPlayground() -> Element {
    let open = use_signal(|| false);
    let scope = ScopeHandle::root("playground").child("popover");
    let restore_focus_id = scope.qualify("restore-focus");
    let open_focus_id = scope.qualify("primary-action");
    let popover = use_popover_runtime(
        Popover::new(scope.clone(), open())
            .with_portal_host(PortalHost::inline())
            .with_floating(
                FloatingLayer::new(PlacementSide::Bottom)
                    .with_align(PlacementAlign::Start)
                    .with_side_offset(10.0),
            )
            .with_open_focus_policy(PopoverOpenFocusPolicy::Target(open_focus_id.clone()))
            .with_close_focus_policy(PopoverCloseFocusPolicy::Target(restore_focus_id.clone())),
        move |next_open| {
            let mut open = open;
            open.set(next_open);
        },
    );
    let placement = popover.placement();
    let root = popover.root();
    let trigger = popover.trigger();
    let anchor = popover.anchor();
    let portal = popover.portal();
    let content = popover.content();
    let arrow = popover.arrow();
    let close = popover.close();
    let lifecycle = popover.lifecycle();

    let portal_host = if portal.host().is_inline() {
        "inline"
    } else {
        portal.host().id().unwrap_or("document.body")
    };
    let open_focus = open_focus_policy_label(lifecycle.open_focus_policy());
    let close_focus = close_focus_policy_label(lifecycle.close_focus_policy());
    let scroll_lock = if lifecycle.scroll_lock_policy().is_enabled() {
        match lifecycle.scroll_lock_policy().restore_delay() {
            Some(delay) => format!("enabled (restore delay: {delay}ms)"),
            None => String::from("enabled"),
        }
    } else {
        String::from("disabled")
    };
    let outside_pointer = outside_behavior_label(
        lifecycle
            .outside_interaction_policy()
            .pointer_down_outside()
            .dismisses(),
    );
    let outside_focus = outside_behavior_label(
        lifecycle
            .outside_interaction_policy()
            .focus_outside()
            .dismisses(),
    );
    let geometry_summary = placement
        .as_ref()
        .map(|placement| {
            format!(
                "x={:.1}, y={:.1}",
                placement.geometry().x(),
                placement.geometry().y()
            )
        })
        .unwrap_or_else(|| String::from("pending live measurement"));
    let content_style = popover_content_style(placement.as_ref());
    let arrow_style = popover_arrow_style(placement.as_ref());

    rsx! {
        div {
            style: "min-height: 100vh; padding: 1rem;",
            section {
            style: CARD_STYLE,
            h2 {
                style: "margin: 0;",
                "Popover"
            }
            p {
                style: MUTED_STYLE,
                "This example renders a positioned card from "
                code { "monoxus::popover" }
                " runtime data. Trigger toggling, focus management, scroll lock, and dismiss decisions now come from "
                code { "use_popover_runtime" }
                ", and the runtime owns document-level outside dismissal directly."
            }
            div {
                id: root.id(),
                "data-state": root.data_state().as_str(),
                style: "display: grid; gap: 1rem;",
                ul {
                    style: "margin: 0; padding-left: 1.25rem; color: #6b21a8;",
                    li {
                        "portal host: "
                        code { "{portal_host}" }
                    }
                    li {
                        "placement: "
                        code { "{content.data_side()} / {content.data_align()}" }
                    }
                    li {
                        "open focus: "
                        code { "{open_focus}" }
                    }
                    li {
                        "close focus: "
                        code { "{close_focus}" }
                    }
                    li {
                        "scroll lock: "
                        code { "{scroll_lock}" }
                    }
                    li {
                        "outside pointer: "
                        code { "{outside_pointer}" }
                        " / focus outside: "
                        code { "{outside_focus}" }
                    }
                    li {
                        "modal: "
                        code { "{content.aria_modal()}" }
                    }
                }
                div {
                    style: CANVAS_STYLE,
                    div {
                        id: anchor.id(),
                        onmounted: popover.mount_anchor(),
                        style: "position: absolute; left: 24px; top: 44px; width: 180px; padding: 0.7rem 0.85rem; border-radius: 0.75rem; border: 1px dashed #a855f7; color: #7e22ce; font-weight: 700; background-color: rgba(255, 255, 255, 0.72);",
                        "Custom anchor lane"
                    }
                    button {
                        id: restore_focus_id.clone(),
                        r#type: "button",
                        onmounted: popover.mount_focus_target(restore_focus_id.clone()),
                        style: "position: absolute; left: 24px; top: 120px; padding: 0.6rem 0.8rem; border: 1px solid #c084fc; border-radius: 0.65rem; background-color: white; color: #6b21a8; font-weight: 600;",
                        "Restore focus target"
                    }
                    div {
                        style: "position: absolute; left: 232px; top: 44px; display: grid; gap: 0.6rem;",
                        button {
                            id: trigger.id(),
                            r#type: "button",
                            aria_controls: trigger.aria_controls(),
                            aria_expanded: trigger.aria_expanded(),
                            "data-state": trigger.data_state().as_str(),
                            onmounted: popover.mount_trigger(),
                            onclick: popover.trigger_click(),
                            style: "padding: 0.7rem 0.95rem; border: 0; border-radius: 0.65rem; background-color: #9333ea; color: white; font-weight: 600; cursor: pointer;",
                            "Toggle popover"
                        }
                    }
                    p {
                        style: "position: absolute; left: 24px; top: 172px; max-width: 18rem; margin: 0; color: #7e22ce;",
                        "The trigger lives away from the anchor lane, so the floating card proves the dedicated "
                        code { "Anchor" }
                        " surface instead of assuming trigger-only positioning."
                    }
                    if popover.is_open() {
                        div {
                            id: content.id(),
                            role: content.role(),
                            aria_modal: content.aria_modal(),
                            "data-state": content.data_state().as_str(),
                            "data-side": content.data_side(),
                            "data-align": content.data_align(),
                            onmounted: popover.mount_content(),
                            style: content_style,
                            div {
                                id: arrow.id(),
                                "data-state": arrow.data_state().as_str(),
                                "data-side": arrow.data_side(),
                                "data-align": arrow.data_align(),
                                style: arrow_style,
                            }
                            strong { "Runtime-owned positioned content" }
                            p {
                                style: MUTED_STYLE,
                                "Geometry vars come from the shared floating backbone: "
                                code { "{geometry_summary}" }
                            }
                            p {
                                style: MUTED_STYLE,
                                "The list above shows the configured open/close focus policies; close focus restores the external button in this harness, and pointer/focus/escape dismissal is exercised here through the runtime-owned dismiss decisions."
                            }
                            p {
                                style: MUTED_STYLE,
                                "This harness now mirrors the default non-modal reference lane, so body scroll stays available while the popover is open and outside interactions still collapse it."
                            }
                            button {
                                id: open_focus_id.clone(),
                                r#type: "button",
                                onmounted: popover.mount_focus_target(open_focus_id.clone()),
                                style: "justify-self: start; padding: 0.55rem 0.8rem; border-radius: 0.6rem; border: 1px solid #c084fc; background-color: #faf5ff; color: #6b21a8; cursor: pointer; font-weight: 700;",
                                "Open focus target"
                            }
                            button {
                                id: close.id(),
                                r#type: "button",
                                "data-state": close.data_state().as_str(),
                                onmounted: popover.mount_close(),
                                onclick: popover.close_click(),
                                style: "justify-self: end; padding: 0.55rem 0.8rem; border-radius: 0.6rem; border: 1px solid #c084fc; background-color: white; color: #6b21a8; cursor: pointer; font-weight: 600;",
                                "Close"
                            }
                        }
                    } else {
                        p {
                            style: "position: absolute; left: 24px; top: 220px; margin: 0; color: #7e22ce;",
                            "Closed. Open the popover, then click anywhere outside the card, focus the restore target, or press Escape while focused inside the card to exercise the primitive-owned runtime dismissal path."
                            }
                        }
                    }
                }
            }
        }
    }
}

fn open_focus_policy_label(policy: &PopoverOpenFocusPolicy) -> String {
    match policy {
        PopoverOpenFocusPolicy::FirstFocusable => String::from("first focusable"),
        PopoverOpenFocusPolicy::Target(target) => format!("target:{target}"),
        PopoverOpenFocusPolicy::Suppress => String::from("suppressed"),
    }
}

fn close_focus_policy_label(policy: &PopoverCloseFocusPolicy) -> String {
    match policy {
        PopoverCloseFocusPolicy::Trigger => String::from("trigger"),
        PopoverCloseFocusPolicy::Target(target) => format!("target:{target}"),
        PopoverCloseFocusPolicy::None => String::from("none"),
    }
}

fn outside_behavior_label(dismisses: bool) -> &'static str {
    if dismisses { "dismisses" } else { "ignored" }
}

fn popover_content_style(placement: Option<&FloatingPlacement>) -> String {
    let mut style = String::from(
        "position: fixed; width: 250px; max-width: calc(100vw - 2rem); padding: 1rem; border-radius: 0.85rem; border: 1px solid #c084fc; background-color: white; box-shadow: 0 18px 40px rgba(88, 28, 135, 0.18); display: grid; gap: 0.75rem; z-index: 20;",
    );

    match placement {
        Some(placement) => {
            let visibility = if placement.reference_hidden() {
                "hidden"
            } else {
                "visible"
            };
            style.push_str(&format!(
                " left: {}px; top: {}px; visibility: {visibility}; pointer-events: {};",
                placement.geometry().x(),
                placement.geometry().y(),
                if placement.reference_hidden() {
                    "none"
                } else {
                    "auto"
                }
            ));
        }
        None => style.push_str(" left: -9999px; top: -9999px; visibility: visible;"),
    }

    style
}

fn popover_arrow_style(placement: Option<&FloatingPlacement>) -> String {
    const ARROW_SIZE_REM: f32 = 0.9;
    const ARROW_SIZE_PX: f32 = 14.4;
    const ARROW_HALF_PX: f32 = ARROW_SIZE_PX / 2.0;

    let mut style = format!(
        "position: absolute; width: {ARROW_SIZE_REM}rem; height: {ARROW_SIZE_REM}rem; rotate: 45deg; background-color: white; border: 1px solid #c084fc; pointer-events: none; display: block; left: auto; right: auto; top: auto; bottom: auto;"
    );
    let Some(placement) = placement else {
        style.push_str(" display: none;");
        return style;
    };
    if placement.arrow().hidden() {
        style.push_str(" display: none;");
        return style;
    }

    match placement.side() {
        PlacementSide::Bottom => {
            if let Some(center_x) = placement.arrow().x() {
                style.push_str(&format!(
                    " left: {}px; top: -{}px;",
                    center_x - ARROW_HALF_PX,
                    ARROW_HALF_PX
                ));
            }
        }
        PlacementSide::Top => {
            if let Some(center_x) = placement.arrow().x() {
                style.push_str(&format!(
                    " left: {}px; bottom: -{}px;",
                    center_x - ARROW_HALF_PX,
                    ARROW_HALF_PX
                ));
            }
        }
        PlacementSide::Right => {
            if let Some(center_y) = placement.arrow().y() {
                style.push_str(&format!(
                    " left: -{}px; top: {}px;",
                    ARROW_HALF_PX,
                    center_y - ARROW_HALF_PX
                ));
            }
        }
        PlacementSide::Left => {
            if let Some(center_y) = placement.arrow().y() {
                style.push_str(&format!(
                    " right: -{}px; top: {}px;",
                    ARROW_HALF_PX,
                    center_y - ARROW_HALF_PX
                ));
            }
        }
    }

    style
}
