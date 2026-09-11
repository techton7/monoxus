use dioxus::prelude::*;
use monoxus::{
    foundation::{
        overlay::{
            FloatingLayer, FloatingPlacement, FloatingReadiness, PlacementAlign, PlacementSide,
            PortalHost,
        },
        shared::ScopeHandle,
        state::DataState,
    },
    popover::{
        use_popover_runtime, Popover, PopoverCloseFocusPolicy, PopoverOpenFocusPolicy,
        PopoverScrollLockPolicy,
    },
};

const CARD_STYLE: &str = "display: grid; gap: 1rem; padding: 1.25rem; border-radius: 0.75rem; border: 1px solid #d8b4fe; background-color: white; box-shadow: 0 10px 30px rgba(88, 28, 135, 0.08);";
const MUTED_STYLE: &str = "margin: 0; color: #6b21a8;";
const CANVAS_STYLE: &str = "position: relative; min-height: 20rem; padding: 1.25rem; border-radius: 0.85rem; border: 1px dashed #d8b4fe; background: linear-gradient(135deg, #faf5ff, #f5f3ff); overflow: hidden;";
const POPOVER_PLAYGROUND_CSS: &str = r#"
@keyframes monoxus-popover-content-in {
    from {
        opacity: 0;
        transform: translate3d(var(--monoxus-popover-motion-x), var(--monoxus-popover-motion-y), 0) scale(0.96);
    }

    to {
        opacity: 1;
        transform: translate3d(0, 0, 0) scale(1);
    }
}

@keyframes monoxus-popover-content-out {
    from {
        opacity: 1;
        transform: translate3d(0, 0, 0) scale(1);
    }

    to {
        opacity: 0;
        transform: translate3d(var(--monoxus-popover-motion-x), var(--monoxus-popover-motion-y), 0) scale(0.96);
    }
}

[data-playground-popover-content='true'] {
    --monoxus-popover-motion-x: 0px;
    --monoxus-popover-motion-y: -10px;
    transform-origin: var(--radix-popover-content-transform-origin, var(--monoxus-popover-transform-origin-x, 0px) var(--monoxus-popover-transform-origin-y, 0px));
}

[data-playground-popover-content='true'][data-side='top'] {
    --monoxus-popover-motion-y: 10px;
}

[data-playground-popover-content='true'][data-side='bottom'] {
    --monoxus-popover-motion-y: -10px;
}

[data-playground-popover-content='true'][data-side='left'] {
    --monoxus-popover-motion-x: 10px;
    --monoxus-popover-motion-y: 0px;
}

[data-playground-popover-content='true'][data-side='right'] {
    --monoxus-popover-motion-x: -10px;
    --monoxus-popover-motion-y: 0px;
}

[data-playground-popover-content='true'][data-state='open'][data-positioning-state='positioned'] {
    animation: monoxus-popover-content-in 220ms cubic-bezier(0.16, 1, 0.3, 1);
}

[data-playground-popover-content='true'][data-state='closed'] {
    animation: monoxus-popover-content-out 320ms cubic-bezier(0.16, 1, 0.3, 1) forwards;
}
"#;

#[component]
pub fn PopoverPlayground() -> Element {
    let open = use_signal(|| false);
    let mut scroll_lock_enabled = use_signal(|| false);
    let scope = ScopeHandle::root("playground").child("popover");
    let restore_focus_id = scope.qualify("restore-focus");
    let open_focus_id = scope.qualify("primary-action");
    let popover = use_popover_runtime(
        Popover::new(scope.clone(), open())
            .with_modal(scroll_lock_enabled())
            .with_portal_host(PortalHost::inline())
            .with_floating(
                FloatingLayer::new(PlacementSide::Bottom)
                    .with_align(PlacementAlign::Start)
                    .with_side_offset(10.0),
            )
            .with_scroll_lock_policy(if scroll_lock_enabled() {
                PopoverScrollLockPolicy::enabled()
            } else {
                PopoverScrollLockPolicy::disabled()
            })
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
    let content_positioning_state = popover.content_positioning_state();
    let content_css_custom_properties = popover.content_css_custom_properties();
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
    let (wrapper_style, ref_hidden_str) = if content_positioning_state == "positioned" {
        if let Some(ref p) = placement {
            let w_pos_style = p.wrapper_style(FloatingReadiness::Ready, Some("20"));
            let ref_hidden = if p.reference_hidden() { "true" } else { "false" };
            (w_pos_style, ref_hidden)
        } else {
            (FloatingReadiness::wrapper_measuring_style(Some("20")), "false")
        }
    } else {
        (FloatingReadiness::wrapper_measuring_style(Some("20")), "false")
    };
    let content_style = popover_content_style(
        content.data_state(),
        content_css_custom_properties.as_str(),
    );
    let arrow_style = popover_arrow_style(placement.as_ref());

    rsx! {
        div {
            style: "min-height: 100vh; padding: 1rem;",
            section {
                style: CARD_STYLE,
                style { "{POPOVER_PLAYGROUND_CSS}" }
                h2 {
                    style: "margin: 0;",
                    "Popover"
                }
                p {
                    style: MUTED_STYLE,
                    "This example renders a positioned card from "
                    code { "monoxus::popover" }
                    " runtime data. Trigger toggling, focus management, scroll lock, dismiss decisions, and retained close continuity now come from "
                    code { "use_popover_runtime" }
                    "."
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
                        style: "display: flex; gap: 1rem; align-items: center; padding: 0.75rem 1rem; background-color: #faf5ff; border: 1px solid #e9d5ff; border-radius: 0.5rem;",
                        label {
                            style: "display: flex; align-items: center; gap: 0.5rem; font-size: 0.8125rem; color: #581c87; font-weight: 600; cursor: pointer;",
                            input {
                                r#type: "checkbox",
                                id: "popover-scroll-lock-checkbox",
                                checked: scroll_lock_enabled(),
                                onchange: move |evt| scroll_lock_enabled.set(evt.value().parse().unwrap_or(false)),
                            }
                            span { "Enable Scroll Lock (prevent_scroll)" }
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
                        if popover.should_render_content() {
                            div {
                                "data-monoxus-floating-content-wrapper": "",
                                style: "{wrapper_style}",
                                "data-reference-hidden": "{ref_hidden_str}",

                                div {
                                    id: content.id(),
                                    role: content.role(),
                                    aria_modal: content.aria_modal(),
                                    "data-state": content.data_state().as_str(),
                                    "data-side": content.data_side(),
                                    "data-align": content.data_align(),
                                    "data-positioning-state": content_positioning_state,
                                    "data-playground-popover-content": "true",
                                    onmounted: popover.mount_content(),
                                    style: "{content_style}",
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
                                        "The list above shows the configured open/close focus policies; close focus restores the external button in this harness, and close retention now keeps the measured placement stable until the primitive-owned unmount completes."
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

fn popover_content_style(
    state: &DataState,
    css_custom_properties: &str,
) -> String {
    let mut style = String::from(
        "position: relative; width: 250px; max-width: calc(100vw - 2rem); padding: 1rem; border-radius: 0.85rem; border: 1px solid #c084fc; background-color: white; box-shadow: 0 18px 40px rgba(88, 28, 135, 0.18); display: grid; gap: 0.75rem;",
    );

    style.push_str(css_custom_properties);
    style.push_str(
        " transform-origin: var(--radix-popover-content-transform-origin, var(--monoxus-popover-transform-origin-x, 0px) var(--monoxus-popover-transform-origin-y, 0px)); will-change: opacity, transform;",
    );
    if let DataState::Closed = state {
        style.push_str(" pointer-events: none;");
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
