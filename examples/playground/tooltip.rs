use dioxus::prelude::*;
use monoxus::{
    foundation::{
        overlay::{FloatingLayer, FloatingPlacement, PlacementAlign, PlacementSide, PortalHost},
        shared::ScopeHandle,
    },
    tooltip::{Tooltip, TooltipProvider, use_tooltip_provider_runtime, use_tooltip_runtime},
};

const CARD_STYLE: &str = "display: grid; gap: 1rem; padding: 1.25rem; border-radius: 0.75rem; border: 1px solid #a5f3fc; background-color: white; box-shadow: 0 10px 30px rgba(8, 145, 178, 0.08);";
const MUTED_STYLE: &str = "margin: 0; color: #0f766e;";
const CANVAS_STYLE: &str = "position: relative; min-height: 15rem; padding: 1.25rem; border-radius: 0.85rem; border: 1px dashed #67e8f9; background: linear-gradient(135deg, #ecfeff, #f0fdfa);";

#[component]
pub fn TooltipPlayground() -> Element {
    let first_open = use_signal(|| false);
    let second_open = use_signal(|| false);
    let provider = TooltipProvider::new(ScopeHandle::root("playground").child("tooltip-provider"))
        .with_delay_duration_ms(450)
        .with_skip_delay_duration_ms(600)
        .with_close_on_trigger_click(true)
        .with_ignore_non_keyboard_focus(true);
    let provider_runtime = use_tooltip_provider_runtime(provider.clone());
    let first = use_tooltip_runtime(
        Tooltip::new(
            ScopeHandle::root("playground").child("tooltip-first"),
            first_open(),
        )
        .with_provider(provider.clone())
        .with_portal_host(PortalHost::inline())
        .with_floating(
            FloatingLayer::new(PlacementSide::Top)
                .with_align(PlacementAlign::Start)
                .with_side_offset(8.0),
        ),
        Some(provider_runtime.clone()),
        move |next_open| {
            let mut first_open = first_open;
            first_open.set(next_open);
        },
    );
    let second = use_tooltip_runtime(
        Tooltip::new(
            ScopeHandle::root("playground").child("tooltip-second"),
            second_open(),
        )
        .with_provider(provider.clone())
        .with_portal_host(PortalHost::inline())
        .with_floating(
            FloatingLayer::new(PlacementSide::Right)
                .with_align(PlacementAlign::Center)
                .with_side_offset(10.0),
        ),
        Some(provider_runtime.clone()),
        move |next_open| {
            let mut second_open = second_open;
            second_open.set(next_open);
        },
    );
    let first_placement = first.placement();
    let second_placement = second.placement();

    let first_root = first.root();
    let first_trigger = first.trigger();
    let first_content = first.content();
    let first_arrow = first.arrow();
    let second_root = second.root();
    let second_trigger = second.trigger();
    let second_content = second.content();
    let second_arrow = second.arrow();

    let provider_delay = format!(
        "{}ms / {}ms",
        provider.delay_duration_ms(),
        provider.skip_delay_duration_ms()
    );
    let active_tooltip = provider_runtime
        .active_tooltip_id()
        .unwrap_or_else(|| String::from("none"));
    let provider_phase = if provider_runtime.opens_instantly() {
        "instant"
    } else {
        "delayed"
    };
    let first_content_style = tooltip_content_style(first_placement.as_ref(), "#0f172a", "#ffffff");
    let second_content_style =
        tooltip_content_style(second_placement.as_ref(), "#115e59", "#ffffff");
    let first_arrow_style = tooltip_arrow_style(first_placement.as_ref(), "#0f172a");
    let second_arrow_style = tooltip_arrow_style(second_placement.as_ref(), "#115e59");

    rsx! {
        div {
            style: "min-height: 100vh; padding: 1rem;",
            section {
                style: CARD_STYLE,
                h2 {
                    style: "margin: 0;",
                    "Tooltip"
                }
                p {
                    style: MUTED_STYLE,
                    "Two tooltip roots share one "
                    code { "TooltipProvider" }
                    " contract and one "
                    code { "use_tooltip_provider_runtime" }
                    " controller. Delay, skip-delay, grouped-trigger coordination, and close-on-click now live in primitive-owned runtime surfaces, and this playground now serves as the live-proof harness for them."
                }
                div {
                    style: "display: grid; gap: 1rem;",
                    ul {
                        style: "margin: 0; padding-left: 1.25rem; color: #0f766e;",
                        li {
                            "provider id: "
                            code { "{provider.id()}" }
                        }
                        li {
                            "delay / skip delay: "
                            code { "{provider_delay}" }
                        }
                        li {
                            "close on trigger click: "
                            code { "{provider.close_on_trigger_click()}" }
                        }
                        li {
                            "ignore non-keyboard focus: "
                            code { "{provider.ignore_non_keyboard_focus()}" }
                        }
                        li {
                            "content autofocus suppressed: "
                            code { "{first.content().autofocus_suppressed()}" }
                        }
                        li {
                            "active tooltip: "
                            code { "{active_tooltip}" }
                            " / provider phase: "
                            code { "{provider_phase}" }
                        }
                    }
                    div {
                        style: CANVAS_STYLE,
                        div {
                            id: first_root.id(),
                            "data-state": first_root.data_state().as_str(),
                            style: "position: absolute; left: 24px; top: 72px;",
                            button {
                                id: first_trigger.id(),
                                r#type: "button",
                                aria_describedby: first_trigger.aria_describedby(),
                                "data-state": first_trigger.data_state().as_str(),
                                onmounted: first.mount_trigger(),
                                onpointerdown: first.trigger_pointer_down(),
                                onclick: first.trigger_click(),
                                onmouseenter: first.trigger_pointer_enter(),
                                onmouseleave: first.trigger_pointer_leave(),
                                onfocus: first.trigger_focus(),
                                onblur: first.trigger_blur(),
                                onkeydown: first.escape_keydown(),
                                style: "padding: 0.6rem 0.85rem; border: 1px solid #06b6d4; border-radius: 0.65rem; background-color: white; color: #0f766e; font-weight: 600; cursor: pointer;",
                                "Primary trigger"
                            }
                        }
                        div {
                            id: second_root.id(),
                            "data-state": second_root.data_state().as_str(),
                            style: "position: absolute; left: 168px; top: 72px;",
                            button {
                                id: second_trigger.id(),
                                r#type: "button",
                                aria_describedby: second_trigger.aria_describedby(),
                                "data-state": second_trigger.data_state().as_str(),
                                onmounted: second.mount_trigger(),
                                onpointerdown: second.trigger_pointer_down(),
                                onclick: second.trigger_click(),
                                onmouseenter: second.trigger_pointer_enter(),
                                onmouseleave: second.trigger_pointer_leave(),
                                onfocus: second.trigger_focus(),
                                onblur: second.trigger_blur(),
                                onkeydown: second.escape_keydown(),
                                style: "padding: 0.6rem 0.85rem; border: 1px solid #06b6d4; border-radius: 0.65rem; background-color: white; color: #0f766e; font-weight: 600; cursor: pointer;",
                                "Shared provider trigger"
                            }
                        }
                        if first.is_open() {
                            div {
                                id: first_content.id(),
                                role: first_content.role(),
                                "data-state": first_content.data_state().as_str(),
                                "data-side": first_content.data_side(),
                                "data-align": first_content.data_align(),
                                onmounted: first.mount_content(),
                                onmouseenter: first.content_pointer_enter(),
                                onmouseleave: first.content_pointer_leave(),
                                style: first_content_style.clone(),
                                div {
                                    id: first_arrow.id(),
                                    "data-state": first_arrow.data_state().as_str(),
                                    "data-side": first_arrow.data_side(),
                                    "data-align": first_arrow.data_align(),
                                    style: first_arrow_style,
                                }
                                strong { "Delayed provider open" }
                                p {
                                    style: "margin: 0; color: #cffafe;",
                                    "Shared provider id: "
                                    code { "{first_trigger.provider_id().unwrap_or(\"none\")}" }
                                }
                            }
                        }
                        if second.is_open() {
                            div {
                                id: second_content.id(),
                                role: second_content.role(),
                                "data-state": second_content.data_state().as_str(),
                                "data-side": second_content.data_side(),
                                "data-align": second_content.data_align(),
                                onmounted: second.mount_content(),
                                onmouseenter: second.content_pointer_enter(),
                                onmouseleave: second.content_pointer_leave(),
                                style: second_content_style.clone(),
                                div {
                                    id: second_arrow.id(),
                                    "data-state": second_arrow.data_state().as_str(),
                                    "data-side": second_arrow.data_side(),
                                    "data-align": second_arrow.data_align(),
                                    style: second_arrow_style,
                                }
                                strong { "Skip-delay instant handoff" }
                                p {
                                    style: "margin: 0; color: #ccfbf1;",
                                    "Move from the first trigger to this one within the skip-delay window to observe grouped-provider instant open. The content stays descriptive and unfocused while rendered through "
                                    code { "aria-describedby" }
                                    "."
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn tooltip_content_style(
    placement: Option<&FloatingPlacement>,
    background: &str,
    foreground: &str,
) -> String {
    let mut style = format!(
        "position: fixed; width: 184px; max-width: calc(100vw - 2rem); padding: 0.7rem 0.85rem; border-radius: 0.7rem; background-color: {background}; color: {foreground}; display: grid; gap: 0.45rem; z-index: 20;"
    );

    match placement {
        Some(placement) => {
            style.push_str(&format!(
                " left: {}px; top: {}px;",
                placement.geometry().x(),
                placement.geometry().y()
            ));
        }
        None => style.push_str(" left: -9999px; top: -9999px; visibility: visible;"),
    }

    style
}

fn tooltip_arrow_style(placement: Option<&FloatingPlacement>, background: &str) -> String {
    const ARROW_SIZE_REM: f32 = 0.75;
    const ARROW_SIZE_PX: f32 = 12.0;
    const ARROW_HALF_PX: f32 = ARROW_SIZE_PX / 2.0;

    let mut style = format!(
        "position: absolute; width: {ARROW_SIZE_REM}rem; height: {ARROW_SIZE_REM}rem; rotate: 45deg; background-color: {background}; pointer-events: none; display: block; left: auto; right: auto; top: auto; bottom: auto;"
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
