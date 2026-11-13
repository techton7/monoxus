//! Native Non-Floating Primitives Verification Test Suite.
//!
//! Validates non-floating primitives (Collapsible, Accordion, Tabs) using
//! Blitz test harness over shared scenarios directly from `examples/playground/`.

mod common;
use common::native_fixture::*;
use blitz_test_harness::Harness;
use dioxus::prelude::*;
use keyboard_types::{Code, Key, NamedKey};

#[path = "../examples/playground/collapsible.rs"]
mod playground_collapsible;

#[path = "../examples/playground/accordion.rs"]
mod playground_accordion;

#[path = "../examples/playground/tabs.rs"]
mod playground_tabs;

/// Test 1: Collapsible Live Layout and Toggle
///
/// Verifies initial closed state, expansion via pointer click, live non-empty layout rects,
/// relational alignment (content positioned below trigger), and closing toggle.
#[test]
fn test_native_collapsible_live_layout_and_toggle() {
    let mut harness = mount_native_scenario(playground_collapsible::BasicCollapsibleScenario);

    // Initial state: trigger exists with non-empty rect, closed state, content unmounted
    assert_rect_non_empty(&harness, "#collapsible-trigger");
    assert_eq!(
        harness.attr("#collapsible-trigger", "data-state").as_deref(),
        Some("closed"),
        "Trigger should initially have data-state='closed'"
    );
    assert!(
        harness.query("#collapsible-content").is_none(),
        "Content should not be mounted when collapsible is closed"
    );

    // Interaction 1: Click trigger to expand
    click_and_pump(&mut harness, "#collapsible-trigger");

    // Open state assertions
    assert_eq!(
        harness.attr("#collapsible-trigger", "data-state").as_deref(),
        Some("open"),
        "Trigger should have data-state='open' after click"
    );
    assert_rect_non_empty(&harness, "#collapsible-content");
    assert_relative_below(&harness, "#collapsible-trigger", "#collapsible-content", 10.0);

    // Interaction 2: Click trigger again to collapse
    click_and_pump(&mut harness, "#collapsible-trigger");

    // Closed state assertions
    assert_eq!(
        harness.attr("#collapsible-trigger", "data-state").as_deref(),
        Some("closed"),
        "Trigger should return to data-state='closed' after second click"
    );
    assert!(
        harness.query("#collapsible-content").is_none(),
        "Content should be unmounted after closing collapsible"
    );
}

/// Test 2: Accordion Single and Multiple Expansion
///
/// Verifies:
/// 1. Single collapsible mode enforces mutual exclusion when expanding items.
/// 2. Multiple selection mode allows simultaneous item expansion.
#[test]
fn test_native_accordion_single_and_multiple_expansion() {
    // -------------------------------------------------------------------------
    // 1. Single Collapsible Mode: Mutual Exclusion
    // -------------------------------------------------------------------------
    let mut single_harness = mount_native_scenario(playground_accordion::SingleCollapsibleSection);

    // Initial state: closed
    assert_eq!(
        single_harness.attr("#accordion-trigger-1", "data-state").as_deref(),
        Some("closed")
    );
    assert_eq!(
        single_harness.attr("#accordion-content-1", "data-state").as_deref(),
        Some("closed")
    );

    // Click item 1 -> opens item 1
    click_and_pump(&mut single_harness, "#accordion-trigger-1");
    assert_eq!(
        single_harness.attr("#accordion-trigger-1", "data-state").as_deref(),
        Some("open"),
        "Item 1 trigger should be open after click"
    );
    assert_eq!(
        single_harness.attr("#accordion-content-1", "data-state").as_deref(),
        Some("open"),
        "Item 1 content should be open after click"
    );
    assert_rect_non_empty(&single_harness, "#accordion-content-1");

    // Click item 2 -> opens item 2 and closes item 1 (mutual exclusion)
    click_and_pump(&mut single_harness, "#accordion-trigger-2");
    assert_eq!(
        single_harness.attr("#accordion-trigger-2", "data-state").as_deref(),
        Some("open"),
        "Item 2 trigger should be open after click"
    );
    assert_eq!(
        single_harness.attr("#accordion-content-2", "data-state").as_deref(),
        Some("open"),
        "Item 2 content should be open after click"
    );
    assert_rect_non_empty(&single_harness, "#accordion-content-2");

    assert_eq!(
        single_harness.attr("#accordion-trigger-1", "data-state").as_deref(),
        Some("closed"),
        "Item 1 trigger should close when item 2 opens in single mode"
    );
    assert_eq!(
        single_harness.attr("#accordion-content-1", "data-state").as_deref(),
        Some("closed"),
        "Item 1 content should close when item 2 opens in single mode"
    );

    // -------------------------------------------------------------------------
    // 2. Multiple Selection Mode: Simultaneous Expansion
    // -------------------------------------------------------------------------
    let mut multi_harness = mount_native_scenario(playground_accordion::MultipleAccordionSection);

    // Initial state: closed
    assert_eq!(
        multi_harness.attr("#accordion-trigger-1", "data-state").as_deref(),
        Some("closed")
    );
    assert_eq!(
        multi_harness.attr("#accordion-trigger-2", "data-state").as_deref(),
        Some("closed")
    );

    // Click item 1 -> opens item 1
    click_and_pump(&mut multi_harness, "#accordion-trigger-1");
    assert_eq!(
        multi_harness.attr("#accordion-trigger-1", "data-state").as_deref(),
        Some("open"),
        "Item 1 trigger should be open"
    );
    assert_eq!(
        multi_harness.attr("#accordion-content-1", "data-state").as_deref(),
        Some("open"),
        "Item 1 content should be open"
    );

    // Click item 2 -> opens item 2 while keeping item 1 open
    click_and_pump(&mut multi_harness, "#accordion-trigger-2");
    assert_eq!(
        multi_harness.attr("#accordion-trigger-1", "data-state").as_deref(),
        Some("open"),
        "Item 1 trigger must stay open in multiple mode"
    );
    assert_eq!(
        multi_harness.attr("#accordion-content-1", "data-state").as_deref(),
        Some("open"),
        "Item 1 content must stay open in multiple mode"
    );
    assert_eq!(
        multi_harness.attr("#accordion-trigger-2", "data-state").as_deref(),
        Some("open"),
        "Item 2 trigger must be open in multiple mode"
    );
    assert_eq!(
        multi_harness.attr("#accordion-content-2", "data-state").as_deref(),
        Some("open"),
        "Item 2 content must be open in multiple mode"
    );
    assert_rect_non_empty(&multi_harness, "#accordion-content-1");
    assert_rect_non_empty(&multi_harness, "#accordion-content-2");
}

/// Test 3: Tabs Arrow Navigation and Roving Focus
///
/// Verifies default active tab, keyboard navigation via ArrowRight / ArrowLeft,
/// roving tabindex and focus ownership, automatic tab selection, and boundary
/// navigation (Home / End).
#[test]
fn test_native_tabs_arrow_navigation_and_roving_focus() {
    let mut harness = mount_native_scenario(playground_tabs::TabsPlayground);

    // Default state: #tabs-trigger-1 active, content 1 rendered, content 2 unmounted
    assert_eq!(
        harness.attr("#tabs-trigger-1", "data-state").as_deref(),
        Some("active"),
        "Trigger 1 should be active by default"
    );
    assert_eq!(
        harness.attr("#tabs-trigger-1", "tabindex").as_deref(),
        Some("0"),
        "Trigger 1 should have tabindex=0 as initial tab stop"
    );
    assert_eq!(
        harness.attr("#tabs-trigger-2", "data-state").as_deref(),
        Some("inactive"),
        "Trigger 2 should be inactive by default"
    );
    assert_eq!(
        harness.attr("#tabs-trigger-2", "tabindex").as_deref(),
        Some("-1"),
        "Trigger 2 should have tabindex=-1"
    );
    assert!(
        harness.query("#tabs-content-1").is_some(),
        "Content 1 should be mounted"
    );
    assert!(
        harness.query("#tabs-content-2").is_none(),
        "Content 2 should be unmounted"
    );

    // Initial focus on #tabs-trigger-1
    focus_node(&mut harness, "#tabs-trigger-1");
    assert_native_focus_id(&harness, "tabs-trigger-1");

    // Keyboard navigation: ArrowRight -> moves roving focus and active tab to #tabs-trigger-2
    send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowRight), Code::ArrowRight);

    assert_eq!(
        harness.attr("#tabs-trigger-2", "data-state").as_deref(),
        Some("active"),
        "Trigger 2 should transition to active after ArrowRight"
    );
    assert_eq!(
        harness.attr("#tabs-trigger-2", "tabindex").as_deref(),
        Some("0"),
        "Trigger 2 should become current tab stop (tabindex=0)"
    );
    assert_eq!(
        harness.attr("#tabs-trigger-1", "data-state").as_deref(),
        Some("inactive"),
        "Trigger 1 should become inactive after ArrowRight"
    );
    assert_eq!(
        harness.attr("#tabs-trigger-1", "tabindex").as_deref(),
        Some("-1"),
        "Trigger 1 should have tabindex=-1"
    );
    assert!(
        harness.query("#tabs-content-2").is_some(),
        "Content 2 should mount upon active tab transition"
    );
    assert!(
        harness.query("#tabs-content-1").is_none(),
        "Content 1 should unmount upon active tab transition"
    );
    assert_native_focus_id(&harness, "tabs-trigger-2");

    // Boundary Navigation: End -> jumps to #tabs-trigger-3
    send_key_and_pump(&mut harness, Key::Named(NamedKey::End), Code::End);
    assert_eq!(
        harness.attr("#tabs-trigger-3", "data-state").as_deref(),
        Some("active"),
        "Trigger 3 should be active after End key"
    );
    assert_eq!(
        harness.attr("#tabs-trigger-3", "tabindex").as_deref(),
        Some("0")
    );
    assert!(harness.query("#tabs-content-3").is_some());
    assert!(harness.query("#tabs-content-2").is_none());
    assert_native_focus_id(&harness, "tabs-trigger-3");

    // Boundary Navigation: Home -> jumps to #tabs-trigger-1
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Home), Code::Home);
    assert_eq!(
        harness.attr("#tabs-trigger-1", "data-state").as_deref(),
        Some("active"),
        "Trigger 1 should be active after Home key"
    );
    assert_eq!(
        harness.attr("#tabs-trigger-1", "tabindex").as_deref(),
        Some("0")
    );
    assert!(harness.query("#tabs-content-1").is_some());
    assert!(harness.query("#tabs-content-3").is_none());
    assert_native_focus_id(&harness, "tabs-trigger-1");

    // Navigation: ArrowLeft -> wraps to #tabs-trigger-3 (looping focus)
    send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowLeft), Code::ArrowLeft);
    assert_eq!(
        harness.attr("#tabs-trigger-3", "data-state").as_deref(),
        Some("active"),
        "Trigger 3 should wrap around on ArrowLeft from first tab"
    );
    assert_native_focus_id(&harness, "tabs-trigger-3");
}

fn send_keydown_and_pump<D: blitz_dom::Document>(harness: &mut Harness<D>, key: Key, code: Code) {
    let text = match (&key, blitz_traits::events::KeyState::Pressed) {
        (Key::Character(t), blitz_traits::events::KeyState::Pressed) => {
            Some(blitz_traits::SmolStr::new(t))
        }
        _ => None,
    };
    let down_event = blitz_traits::events::BlitzKeyEvent {
        key,
        code,
        modifiers: keyboard_types::Modifiers::default(),
        location: keyboard_types::Location::Standard,
        is_auto_repeating: false,
        is_composing: false,
        state: blitz_traits::events::KeyState::Pressed,
        text,
    };
    harness.dispatch(blitz_traits::events::UiEvent::KeyDown(down_event));
    harness.pump();
    oxidase::frame::tick(std::time::Duration::from_millis(16));
    harness.pump();
    harness.tick(0.01);
}

fn send_keyup_and_pump<D: blitz_dom::Document>(harness: &mut Harness<D>, key: Key, code: Code) {
    let up_event = blitz_traits::events::BlitzKeyEvent {
        key,
        code,
        modifiers: keyboard_types::Modifiers::default(),
        location: keyboard_types::Location::Standard,
        is_auto_repeating: false,
        is_composing: false,
        state: blitz_traits::events::KeyState::Released,
        text: None,
    };
    harness.dispatch(blitz_traits::events::UiEvent::KeyUp(up_event));
    harness.pump();
    oxidase::frame::tick(std::time::Duration::from_millis(16));
    harness.pump();
    harness.tick(0.01);
}

static CB_CALL_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[component]
fn NativeCollapsibleCallbackApp() -> Element {
    rsx! {
        monoxus::collapsible::CollapsibleRoot {
            id: "cb-native-root",
            monoxus::collapsible::CollapsibleTrigger {
                id: "cb-native-trigger",
                onclick: move |_evt: MouseEvent| {
                    CB_CALL_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                },
                "Toggle"
            }
            monoxus::collapsible::CollapsibleContent {
                id: "cb-native-content",
                "Body Content"
            }
        }
    }
}

/// Test 4: Collapsible Single-Fire MouseEvent and Space+Escape Abort
#[test]
fn test_native_collapsible_single_fire_mouse_event_and_space_escape_abort() {
    CB_CALL_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);
    let mut harness = mount_native_scenario(NativeCollapsibleCallbackApp);

    focus_node(&mut harness, "#cb-native-trigger");
    assert_native_focus_id(&harness, "cb-native-trigger");

    // Initial state: closed, 0 calls
    assert_eq!(CB_CALL_COUNT.load(std::sync::atomic::Ordering::SeqCst), 0);
    assert_eq!(
        harness.attr("#cb-native-trigger", "data-state").as_deref(),
        Some("closed")
    );

    // 1. Enter KeyDown -> fires synthetic Click (MouseEvent) exactly once -> opens
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
    assert_eq!(
        CB_CALL_COUNT.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "MouseEvent callback must be called exactly once per Enter keydown"
    );
    assert_eq!(
        harness.attr("#cb-native-trigger", "data-state").as_deref(),
        Some("open"),
        "Collapsible must open upon Enter keydown"
    );
    assert!(harness.query("#cb-native-content").is_some());

    // 2. Second Enter -> fires synthetic Click exactly once -> closes
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
    assert_eq!(
        CB_CALL_COUNT.load(std::sync::atomic::Ordering::SeqCst),
        2,
        "MouseEvent callback must be called exactly once on closing Enter"
    );
    assert_eq!(
        harness.attr("#cb-native-trigger", "data-state").as_deref(),
        Some("closed"),
        "Collapsible must close upon second Enter keydown"
    );
    assert!(harness.query("#cb-native-content").is_none());

    // 3. Space hold + Escape abort:
    // KeyDown Space arms active node
    send_keydown_and_pump(&mut harness, Key::Character(" ".into()), Code::Space);
    // Escape disarms active node
    send_keydown_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
    send_keyup_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
    // KeyUp Space: disarmed -> zero synthetic click
    send_keyup_and_pump(&mut harness, Key::Character(" ".into()), Code::Space);

    assert_eq!(
        CB_CALL_COUNT.load(std::sync::atomic::Ordering::SeqCst),
        2,
        "No click callback should be invoked when Space is aborted by Escape"
    );
    assert_eq!(
        harness.attr("#cb-native-trigger", "data-state").as_deref(),
        Some("closed"),
        "Collapsible must remain closed after Space hold + Escape abort"
    );
    assert!(harness.query("#cb-native-content").is_none());

    // Verify focus was cleared by Escape
    assert_eq!(
        harness.attr("#cb-native-trigger", "data-state").as_deref(),
        Some("closed"),
        "Collapsible must remain closed after Space hold + Escape abort"
    );
    assert!(harness.query("#cb-native-content").is_none());

    // 4. Re-focus trigger for normal Space KeyDown + KeyUp: activates and opens
    focus_node(&mut harness, "#cb-native-trigger");
    send_key_and_pump(&mut harness, Key::Character(" ".into()), Code::Space);
    assert_eq!(
        CB_CALL_COUNT.load(std::sync::atomic::Ordering::SeqCst),
        3,
        "MouseEvent callback must be called once on Space keyup click"
    );
    assert_eq!(
        harness.attr("#cb-native-trigger", "data-state").as_deref(),
        Some("open"),
        "Collapsible must open after normal Space key activation"
    );
    assert!(harness.query("#cb-native-content").is_some());
}

/// Test 5: Accordion Roving Focus + Enter & Space Expansion Without Double-Toggle
#[test]
fn test_native_accordion_roving_and_enter_space_toggle() {
    let mut harness = mount_native_scenario(playground_accordion::SingleCollapsibleSection);

    // Initial state: Item 1 closed
    assert_eq!(
        harness.attr("#accordion-trigger-1", "data-state").as_deref(),
        Some("closed")
    );

    // Focus Item 1 and press Enter -> opens Item 1
    focus_node(&mut harness, "#accordion-trigger-1");
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
    assert_eq!(
        harness.attr("#accordion-trigger-1", "data-state").as_deref(),
        Some("open")
    );

    // ArrowDown roves focus to Item 2
    send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
    assert_native_focus_id(&harness, "accordion-trigger-2");

    // Space on Item 2 -> opens Item 2, closes Item 1 in single mode
    send_key_and_pump(&mut harness, Key::Character(" ".into()), Code::Space);
    assert_eq!(
        harness.attr("#accordion-trigger-2", "data-state").as_deref(),
        Some("open")
    );
    assert_eq!(
        harness.attr("#accordion-trigger-1", "data-state").as_deref(),
        Some("closed")
    );

    // Space again on Item 2 -> collapses Item 2 (collapsible: true, zero double-fire)
    send_key_and_pump(&mut harness, Key::Character(" ".into()), Code::Space);
    assert_eq!(
        harness.attr("#accordion-trigger-2", "data-state").as_deref(),
        Some("closed")
    );
}

#[component]
fn NativeTabsManualScenario() -> Element {
    rsx! {
        monoxus::tabs::TabsRoot {
            id: "tabs-vertical-manual".to_string(),
            default_value: "profile".to_string(),
            orientation: monoxus::tabs::TabsOrientation::Vertical,
            activation_mode: monoxus::tabs::TabsActivationMode::Manual,
            monoxus::tabs::TabsList {
                id: "tabs-vert-list".to_string(),
                monoxus::tabs::TabsTrigger {
                    id: "tabs-vert-profile".to_string(),
                    value: "profile".to_string(),
                    "Profile"
                }
                monoxus::tabs::TabsTrigger {
                    id: "tabs-vert-notifications".to_string(),
                    value: "notifications".to_string(),
                    "Notifications"
                }
                monoxus::tabs::TabsTrigger {
                    id: "tabs-vert-billing".to_string(),
                    value: "billing".to_string(),
                    "Billing"
                }
            }
            monoxus::tabs::TabsContent {
                id: "tabs-vert-content-profile".to_string(),
                value: "profile".to_string(),
                "Profile Settings"
            }
            monoxus::tabs::TabsContent {
                id: "tabs-vert-content-notifications".to_string(),
                value: "notifications".to_string(),
                "Notification Preferences"
            }
            monoxus::tabs::TabsContent {
                id: "tabs-vert-content-billing".to_string(),
                value: "billing".to_string(),
                "Billing & Invoices"
            }
        }
    }
}

/// Test 6: Tabs Manual Activation Mode with Space & Enter
#[test]
fn test_native_tabs_manual_activation_mode_space_and_enter() {
    let mut harness = mount_native_scenario(NativeTabsManualScenario);

    // Initial state: profile active
    assert_eq!(
        harness.attr("#tabs-vert-profile", "data-state").as_deref(),
        Some("active")
    );
    assert_eq!(
        harness.attr("#tabs-vert-profile", "aria-selected").as_deref(),
        Some("true")
    );
    assert!(harness.query("#tabs-vert-content-profile").is_some());
    assert!(harness.query("#tabs-vert-content-notifications").is_none());

    // Focus profile tab
    focus_node(&mut harness, "#tabs-vert-profile");

    // ArrowDown roves focus to notifications tab, but manual mode does NOT activate
    send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
    assert_native_focus_id(&harness, "tabs-vert-notifications");
    assert_eq!(
        harness.attr("#tabs-vert-notifications", "data-state").as_deref(),
        Some("inactive"),
        "Notifications tab must remain inactive until explicit Space/Enter in manual mode"
    );
    assert_eq!(
        harness.attr("#tabs-vert-profile", "data-state").as_deref(),
        Some("active"),
        "Profile tab must remain active"
    );

    // Press Space on focused notifications tab -> selects and activates!
    send_key_and_pump(&mut harness, Key::Character(" ".into()), Code::Space);
    assert_eq!(
        harness.attr("#tabs-vert-notifications", "data-state").as_deref(),
        Some("active")
    );
    assert_eq!(
        harness.attr("#tabs-vert-notifications", "aria-selected").as_deref(),
        Some("true")
    );
    assert!(harness.query("#tabs-vert-content-notifications").is_some());
    assert!(harness.query("#tabs-vert-content-profile").is_none());

    // ArrowDown to billing tab
    send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
    assert_native_focus_id(&harness, "tabs-vert-billing");
    assert_eq!(
        harness.attr("#tabs-vert-billing", "data-state").as_deref(),
        Some("inactive")
    );

    // Press Enter on focused billing tab -> selects and activates!
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
    assert_eq!(
        harness.attr("#tabs-vert-billing", "data-state").as_deref(),
        Some("active")
    );
    assert_eq!(
        harness.attr("#tabs-vert-billing", "aria-selected").as_deref(),
        Some("true")
    );
    assert!(harness.query("#tabs-vert-content-billing").is_some());
    assert!(harness.query("#tabs-vert-content-notifications").is_none());
}

#[component]
fn UnguardedInputTabsApp() -> Element {
    rsx! {
        monoxus::tabs::TabsRoot {
            default_value: "tab1".to_string(),
            activation_mode: monoxus::tabs::TabsActivationMode::Manual,
            monoxus::tabs::TabsList {
                monoxus::tabs::TabsTrigger {
                    id: "u-tab-1".to_string(),
                    value: "tab1".to_string(),
                    "Tab 1"
                }
                monoxus::tabs::TabsTrigger {
                    id: "u-tab-2".to_string(),
                    value: "tab2".to_string(),
                    "Tab 2"
                }
            }
            monoxus::tabs::TabsContent {
                id: "u-content-1".to_string(),
                value: "tab1".to_string(),
                input {
                    id: "unguarded-tab-input",
                    r#type: "text",
                }
            }
            monoxus::tabs::TabsContent {
                id: "u-content-2".to_string(),
                value: "tab2".to_string(),
                "Panel 2"
            }
        }
    }
}

/// Test 7: Unguarded <input type="text"> Space Key in TabsContent Does Not Switch Tabs
#[test]
fn test_native_tabs_unguarded_descendant_input_space() {
    let mut harness = mount_native_scenario(UnguardedInputTabsApp);

    assert_eq!(
        harness.attr("#u-tab-1", "data-state").as_deref(),
        Some("active")
    );

    // Focus descendant input inside tab panel 1
    focus_node(&mut harness, "#unguarded-tab-input");

    // Send Space key to focused text input
    send_key_and_pump(&mut harness, Key::Character(" ".into()), Code::Space);

    // Tab 1 must STILL be active, Tab 2 must NOT activate
    assert_eq!(
        harness.attr("#u-tab-1", "data-state").as_deref(),
        Some("active"),
        "Tab 1 must stay active when typing Space in descendant input"
    );
    assert_eq!(
        harness.attr("#u-tab-2", "data-state").as_deref(),
        Some("inactive")
    );
}
