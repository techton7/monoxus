//! Black-box contract and integration tests for disclosure dimensions and lifecycle.
//!
//! Validates:
//! - Dimension formatting helper (`format_disclosure_dimension`)
//! - CSS variable publication (`--radix-*` and `--monoxus-*`) for Collapsible and Accordion
//! - User `style` merging and semicolon hygiene (`merge_disclosure_styles`)
//! - `PresenceController` exit lifecycle on disclosure content
//! - `force_mount` and `hidden_until_found` contracts
//! - Strict boolean attribute omission for `data-disabled`

use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use monoxus::{
    accordion::{
        AccordionContent, AccordionHeader, AccordionItem, AccordionMode, AccordionRoot,
        AccordionTrigger, format_accordion_css_vars,
    },
    collapsible::{
        CollapsibleContent, CollapsibleRoot, CollapsibleTrigger, ElementSize,
        format_collapsible_css_vars, format_disclosure_dimension, merge_disclosure_styles,
    },
    foundation::overlay::{PresenceController, PresenceState},
};

// ===========================================================================
// 1. Dimension Formatting Helper Tests
// ===========================================================================

#[test]
fn test_format_disclosure_dimension() {
    assert_eq!(format_disclosure_dimension(120.0), "120px");
    assert_eq!(format_disclosure_dimension(120.5), "120.5px");
    assert_eq!(format_disclosure_dimension(120.25), "120.25px");
    assert_eq!(format_disclosure_dimension(120.755), "120.76px");
    assert_eq!(format_disclosure_dimension(0.0), "0px");
    assert_eq!(format_disclosure_dimension(-15.0), "0px");
    assert_eq!(format_disclosure_dimension(f64::NAN), "0px");
    assert_eq!(format_disclosure_dimension(f64::INFINITY), "0px");
    assert_eq!(format_disclosure_dimension(f64::NEG_INFINITY), "0px");
}

// ===========================================================================
// 2. CSS Custom Property Formatting Tests
// ===========================================================================

#[test]
fn test_format_collapsible_css_vars() {
    let size = ElementSize {
        width: 350.0,
        height: 180.5,
    };
    let vars = format_collapsible_css_vars(size);

    assert!(vars.contains("--radix-collapsible-content-height: 180.5px"));
    assert!(vars.contains("--radix-collapsible-content-width: 350px"));
    assert!(vars.contains("--monoxus-collapsible-content-height: 180.5px"));
    assert!(vars.contains("--monoxus-collapsible-content-width: 350px"));
}

#[test]
fn test_format_accordion_css_vars() {
    let size = ElementSize {
        width: 240.0,
        height: 90.0,
    };
    let vars = format_accordion_css_vars(size);

    assert!(vars.contains("--radix-accordion-content-height: 90px"));
    assert!(vars.contains("--radix-accordion-content-width: 240px"));
    assert!(vars.contains("--monoxus-accordion-content-height: 90px"));
    assert!(vars.contains("--monoxus-accordion-content-width: 240px"));
}

// ===========================================================================
// 3. Style Merging & Semicolon Hygiene Tests
// ===========================================================================

#[test]
fn test_merge_disclosure_styles_combinations() {
    // 1. User style with trailing semicolon + guard + vars
    let merged1 = merge_disclosure_styles(
        Some("background-color: red;"),
        Some("animation-name: none; transition-duration: 0s;"),
        Some("--radix-collapsible-content-height: 120px;"),
    )
    .expect("must produce merged style");
    assert!(!merged1.contains(";;"));
    assert!(merged1.contains("background-color: red"));
    assert!(merged1.contains("animation-name: none"));
    assert!(merged1.contains("--radix-collapsible-content-height: 120px"));

    // 2. User style without trailing semicolon + vars
    let merged2 = merge_disclosure_styles(
        Some("color: blue"),
        None,
        Some("--radix-collapsible-content-width: 200px"),
    )
    .expect("must produce merged style");
    assert_eq!(
        merged2,
        "color: blue; --radix-collapsible-content-width: 200px"
    );

    // 3. Only guard style
    let merged3 = merge_disclosure_styles(
        None,
        Some("animation-name: none; transition-duration: 0s"),
        None,
    )
    .expect("must produce guard style");
    assert_eq!(merged3, "animation-name: none; transition-duration: 0s");

    // 4. All None
    assert_eq!(merge_disclosure_styles(None, None, None), None);

    // 5. Empty or whitespace-only inputs
    assert_eq!(merge_disclosure_styles(Some("   "), None, None), None);
    assert_eq!(merge_disclosure_styles(Some(""), Some("  "), None), None);
}

// ===========================================================================
// 4. PresenceController Exit Lifecycle Contract Tests
// ===========================================================================

#[test]
fn test_presence_controller_disclosure_exit_lifecycle() {
    let mut controller = PresenceController::new(false).with_retained_mount(true);

    // 1. Initial closed state
    assert_eq!(controller.state(), PresenceState::Unmounted);
    assert!(!controller.should_render());
    assert_eq!(controller.active_close_cycle(), None);

    // 2. Open disclosure -> Mounted
    let up_open = controller.sync(true);
    assert_eq!(up_open.state(), PresenceState::Mounted);
    assert!(up_open.should_render());
    assert_eq!(up_open.active_close_cycle(), None);
    assert_eq!(up_open.started_close_cycle(), None);

    // 3. Close disclosure -> Suspended (retained mount during exit animation)
    let up_close1 = controller.sync(false);
    assert_eq!(up_close1.state(), PresenceState::Suspended);
    assert!(up_close1.should_render());
    let first_cycle = up_close1
        .started_close_cycle()
        .expect("must assign first close cycle");
    assert_eq!(controller.active_close_cycle(), Some(first_cycle));

    // 4. Reopen and close again to invalidate first cycle
    let reopen = controller.sync(true);
    assert_eq!(reopen.invalidated_close_cycle(), Some(first_cycle));
    assert_eq!(controller.state(), PresenceState::Mounted);
    assert!(controller.should_render());

    let up_close2 = controller.sync(false);
    let second_cycle = up_close2
        .started_close_cycle()
        .expect("must assign second close cycle");
    assert_ne!(first_cycle, second_cycle);

    // Stale/invalidated first cycle does not unmount
    assert!(!controller.complete_close_cycle(first_cycle));
    assert_eq!(controller.state(), PresenceState::Suspended);
    assert!(controller.should_render());

    // 5. Active second cycle completes -> Unmounted
    assert!(controller.complete_close_cycle(second_cycle));
    assert_eq!(controller.state(), PresenceState::Unmounted);
    assert!(!controller.should_render());
    assert_eq!(controller.active_close_cycle(), None);
}

// ===========================================================================
// 5. Strict Boolean Attribute Omission Tests
// ===========================================================================

#[test]
fn test_data_disabled_strict_omission_contract() {
    let scope = monoxus::foundation::shared::ScopeHandle::root("disclosure-disabled-test");
    let enabled_col = monoxus::collapsible::Collapsible::new(scope.clone(), true);
    assert_eq!(enabled_col.root().data_disabled(), None);
    assert_eq!(enabled_col.trigger().data_disabled(), None);
    assert_eq!(enabled_col.content().data_disabled(), None);

    let disabled_col = enabled_col.with_disabled(true);
    assert_eq!(disabled_col.root().data_disabled(), Some(""));
    assert_eq!(disabled_col.trigger().data_disabled(), Some(""));
    assert_eq!(disabled_col.content().data_disabled(), Some(""));
}

// ===========================================================================
// 6. Component Rendering & VirtualDom Contract Tests
// ===========================================================================

#[component]
fn CollapsibleDimensionsSample(
    default_open: bool,
    force_mount: bool,
    hidden_until_found: bool,
    custom_style: Option<String>,
) -> Element {
    rsx! {
        CollapsibleRoot {
            id: "dim-root",
            default_open: default_open,
            CollapsibleTrigger {
                id: "dim-trigger",
                "Toggle"
            }
            CollapsibleContent {
                id: "dim-content",
                force_mount: force_mount,
                hidden_until_found: hidden_until_found,
                style: custom_style,
                div {
                    id: "dim-inner",
                    "Panel Content"
                }
            }
        }
    }
}

#[test]
fn test_collapsible_dimensions_component_mounts_cleanly_open() {
    let mut dom = VirtualDom::new_with_props(
        CollapsibleDimensionsSample,
        CollapsibleDimensionsSampleProps {
            default_open: true,
            force_mount: false,
            hidden_until_found: false,
            custom_style: Some("background: #f0f0f0;".to_string()),
        },
    );
    dom.rebuild_in_place();
}

#[test]
fn test_collapsible_dimensions_component_mounts_cleanly_closed() {
    let mut dom = VirtualDom::new_with_props(
        CollapsibleDimensionsSample,
        CollapsibleDimensionsSampleProps {
            default_open: false,
            force_mount: false,
            hidden_until_found: false,
            custom_style: None,
        },
    );
    dom.rebuild_in_place();
}

#[test]
fn test_collapsible_hidden_until_found_prop() {
    let mut dom = VirtualDom::new_with_props(
        CollapsibleDimensionsSample,
        CollapsibleDimensionsSampleProps {
            default_open: false,
            force_mount: true,
            hidden_until_found: true,
            custom_style: None,
        },
    );
    dom.rebuild_in_place();
}

#[component]
fn AccordionDimensionsSample() -> Element {
    rsx! {
        AccordionRoot {
            mode: AccordionMode::Single { collapsible: true },
            default_value: Some("item-1".to_string()),
            AccordionItem {
                value: "item-1".to_string(),
                AccordionHeader {
                    AccordionTrigger {
                        id: "acc-trigger-1",
                        "Header 1"
                    }
                }
                AccordionContent {
                    class: "acc-content".to_string(),
                    style: "padding: 12px;".to_string(),
                    div {
                        "Accordion Panel Body"
                    }
                }
            }
        }
    }
}

#[test]
fn test_accordion_dimensions_component_mounts_cleanly() {
    let mut dom = VirtualDom::new(AccordionDimensionsSample);
    dom.rebuild_in_place();
}

#[component]
fn CollapsibleUnmountProbe(mounted: Rc<RefCell<bool>>) -> Element {
    *mounted.borrow_mut() = true;
    rsx! { div { "Mounted" } }
}

#[component]
fn CollapsibleLifecycleHarness(
    is_open: bool,
    force_mount: bool,
    child_mounted: Rc<RefCell<bool>>,
) -> Element {
    rsx! {
        CollapsibleRoot {
            default_open: is_open,
            CollapsibleTrigger { "Toggle" }
            CollapsibleContent {
                force_mount: force_mount,
                CollapsibleUnmountProbe {
                    mounted: child_mounted,
                }
            }
        }
    }
}

#[test]
fn test_collapsible_closed_without_force_mount_does_not_mount_children() {
    let mounted = Rc::new(RefCell::new(false));
    let mut dom = VirtualDom::new_with_props(
        CollapsibleLifecycleHarness,
        CollapsibleLifecycleHarnessProps {
            is_open: false,
            force_mount: false,
            child_mounted: mounted.clone(),
        },
    );
    dom.rebuild_in_place();
    assert!(!*mounted.borrow());
}

#[test]
fn test_collapsible_closed_with_force_mount_mounts_children() {
    let mounted = Rc::new(RefCell::new(false));
    let mut dom = VirtualDom::new_with_props(
        CollapsibleLifecycleHarness,
        CollapsibleLifecycleHarnessProps {
            is_open: false,
            force_mount: true,
            child_mounted: mounted.clone(),
        },
    );
    dom.rebuild_in_place();
    assert!(*mounted.borrow());
}

#[cfg(feature = "native")]
const SW2B_COLLAPSIBLE_CSS: &str = r#"
@keyframes sw2bSlideDown {
    from {
        height: 0;
    }
    to {
        height: var(--radix-collapsible-content-height);
    }
}

@keyframes sw2bSlideUp {
    from {
        height: var(--radix-collapsible-content-height);
    }
    to {
        height: 0;
    }
}

.sw2b-collapsible-animated {
    overflow: hidden;
}

.sw2b-collapsible-animated[data-state="open"] {
    animation: sw2bSlideDown 250ms ease-out;
}

.sw2b-collapsible-animated[data-state="closed"] {
    animation: sw2bSlideUp 350ms ease-out forwards;
}
"#;

#[cfg(feature = "native")]
#[component]
fn Sw2bCollapsibleNativeScenario() -> Element {
    rsx! {
        style { "{SW2B_COLLAPSIBLE_CSS}" }
        div {
            style: "width: 320px;",
            CollapsibleRoot {
                id: "sw2b-collapsible-root".to_string(),
                default_open: false,
                CollapsibleTrigger {
                    id: "sw2b-collapsible-trigger".to_string(),
                    style: "width: 120px; height: 32px; display: block;",
                    "Toggle"
                }
                CollapsibleContent {
                    id: "sw2b-collapsible-content".to_string(),
                    class: "sw2b-collapsible-animated".to_string(),
                    style: "width: 320px; box-sizing: border-box;".to_string(),
                    div {
                        style: "height: 120px; width: 320px; padding: 14px; box-sizing: border-box;",
                        "Dynamic dimensions & presence exit animation."
                    }
                }
            }
        }
    }
}

#[cfg(feature = "native")]
#[test]
fn test_native_collapsible_sw2b_dimensions_and_animation_lifecycle() {
    use crate::common::native_fixture::{
        NATIVE_TEST_LOCK, advance_frame_and_pump, click_and_pump, mount_native_scenario,
    };

    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(Sw2bCollapsibleNativeScenario);
    assert!(
        harness.query("#sw2b-collapsible-content").is_none(),
        "CollapsibleContent must start unmounted when default_open=false"
    );

    click_and_pump(&mut harness, "#sw2b-collapsible-trigger");

    let content_node_id = harness.node("#sw2b-collapsible-content");
    let open_style = harness
        .attr("#sw2b-collapsible-content", "style")
        .expect("CollapsibleContent must publish inline style after open");
    assert!(
        open_style.contains("--radix-collapsible-content-height: 120px"),
        "expected --radix-collapsible-content-height: 120px in style, got: {open_style}"
    );
    assert!(
        harness.base().node_has_active_animations(content_node_id),
        "CollapsibleContent must have active open animation after opening"
    );

    advance_frame_and_pump(&mut harness, 0.04);
    let h_open_05 = harness.layout_rect("#sw2b-collapsible-content").height;
    assert!(
        h_open_05 > 0.0 && h_open_05 < 120.0,
        "expected intermediate open height 0 < h < 120 at t=0.05s, got {h_open_05}"
    );

    advance_frame_and_pump(&mut harness, 0.07);
    let h_open_12 = harness.layout_rect("#sw2b-collapsible-content").height;
    assert!(
        h_open_12 > 0.0 && h_open_12 < 120.0,
        "expected intermediate open height 0 < h < 120 at t=0.12s, got {h_open_12}"
    );

    advance_frame_and_pump(&mut harness, 0.18);
    let h_open_30 = harness.layout_rect("#sw2b-collapsible-content").height;
    assert!(
        (h_open_30 - 120.0).abs() < 0.01,
        "expected settled open height 120.0 at t=0.30s, got {h_open_30}"
    );

    click_and_pump(&mut harness, "#sw2b-collapsible-trigger");
    let closing_node_id = harness
        .query("#sw2b-collapsible-content")
        .expect("CollapsibleContent must remain mounted during 350ms exit animation");
    assert_eq!(
        harness
            .attr("#sw2b-collapsible-content", "data-state")
            .as_deref(),
        Some("closed")
    );
    assert!(
        harness.base().node_has_active_animations(closing_node_id),
        "CollapsibleContent must have active exit animation after closing"
    );

    advance_frame_and_pump(&mut harness, 0.09);
    let h_close_10 = harness.layout_rect("#sw2b-collapsible-content").height;
    assert!(
        h_close_10 > 0.0 && h_close_10 < 120.0,
        "expected intermediate close height 0 < h < 120 at t=0.10s, got {h_close_10}"
    );

    advance_frame_and_pump(&mut harness, 0.10);
    let h_close_20 = harness.layout_rect("#sw2b-collapsible-content").height;
    assert!(
        h_close_20 > 0.0 && h_close_20 < 120.0,
        "expected intermediate close height 0 < h < 120 at t=0.20s, got {h_close_20}"
    );

    advance_frame_and_pump(&mut harness, 0.20);
    assert!(
        harness.query("#sw2b-collapsible-content").is_none(),
        "CollapsibleContent must cleanly unmount after advancing past 350ms exit animation"
    );
}
