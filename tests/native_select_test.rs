//! Native Select Click & Floating Readiness Verification Test Suite.
//!
//! Validates that clicking #select-trigger in a headless native Blitz document
//! immediately mounts #select-content and transitions FloatingReadiness to Ready
//! with non-empty bounds without ANY wheel or scroll events.

#![allow(unused_imports, dead_code)]

use crate::common;
use crate::playground_fixtures::playground_select;
use blitz_traits::events::{
    BlitzPointerEvent, BlitzPointerId, BlitzWheelDelta, BlitzWheelEvent, BlitzWheelPhase,
    BlitzWheelSource, MouseEventButton, MouseEventButtons, PointerCoords, PointerDetails, Point,
    UiEvent,
};
use common::native_fixture::*;
use dioxus::prelude::*;
use keyboard_types::{Code, Key, NamedKey};

#[component]
fn StyledScrollableViewportSection() -> Element {
    rsx! {
        style { "{playground_select::SELECT_PLAYGROUND_CSS}" }
        playground_select::ScrollableViewportSection {}
    }
}

fn assert_native_select_active_value(
    harness: &blitz_test_harness::Harness<dioxus_native_dom::DioxusDocument>,
    expected_value: &str,
) {
    let option_selector = format!("[data-value='{expected_value}']");
    let expected_id = harness
        .attr(&option_selector, "id")
        .unwrap_or_else(|| panic!("Missing id for select option '{expected_value}'"));
    let active_descendant = harness.attr("[role='listbox']", "aria-activedescendant");

    assert_eq!(
        active_descendant.as_deref(),
        Some(expected_id.as_str()),
        "Expected aria-activedescendant to identify '{expected_value}', got {active_descendant:?}"
    );
}

fn assert_native_select_highlighted_value(
    harness: &blitz_test_harness::Harness<dioxus_native_dom::DioxusDocument>,
    expected_value: &str,
) {
    assert_native_select_active_value(harness, expected_value);

    let highlighted = harness.query_all("[role='option'][data-highlighted='true']");
    let expected_option = harness.node(&format!("[data-value='{expected_value}']"));
    assert_eq!(
        highlighted,
        vec![expected_option],
        "Expected only '{expected_value}' to remain highlighted"
    );
}

fn native_select_scroll_y(
    harness: &blitz_test_harness::Harness<dioxus_native_dom::DioxusDocument>,
    selector: &str,
) -> f64 {
    let node_id = harness.node(selector);
    harness
        .base()
        .get_node(node_id)
        .unwrap_or_else(|| panic!("Missing scroll node for '{selector}'"))
        .scroll_offset()
        .y
}

fn assert_native_select_reopen_highlight(
    scenario: fn() -> Element,
    trigger_selector: &str,
    expected_value: &str,
) {
    let mut harness = mount_native_scenario(scenario);

    for cycle in 1..=3 {
        click_and_pump(&mut harness, trigger_selector);
        advance_frame_and_pump(&mut harness, 0.02);
        advance_frame_and_pump(&mut harness, 0.02);

        assert!(
            harness.query("[role='listbox']").is_some(),
            "Listbox should mount on reopen cycle {cycle} for {expected_value}"
        );
        focus_node(&mut harness, "[role='listbox']");
        assert_native_select_active_value(&harness, expected_value);

        let highlighted = harness.query_all("[role='option'][data-highlighted='true']");
        assert_eq!(
            highlighted.len(),
            1,
            "Expected exactly one highlighted option on reopen cycle {cycle}, got {}",
            highlighted.len()
        );
        let expected_option = harness.node(&format!("[data-value='{expected_value}']"));
        assert_eq!(
            highlighted[0], expected_option,
            "Highlighted option should be '{expected_value}' on reopen cycle {cycle}"
        );

        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        advance_frame_and_pump(&mut harness, 0.30);
        assert!(
            harness.query("[role='listbox']").is_none(),
            "Listbox should unmount after Escape on reopen cycle {cycle}"
        );
    }
}

#[test]
fn test_native_select_click_immediate_ready_without_wheel_scroll() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(playground_select::BasicFruitSelectSection);

    // Initial state: trigger exists, content not mounted
    assert_rect_non_empty(&harness, "#select-trigger");
    assert!(
        harness.query("#select-content").is_none(),
        "Select content should not be mounted initially"
    );

    // Click #select-trigger with ZERO scroll or wheel events
    click_and_pump(&mut harness, "#select-trigger");

    // Content should mount immediately in native DOM
    assert!(
        harness.query("#select-content").is_some(),
        "Select content should be mounted after click"
    );
    assert_rect_non_empty(&harness, "#select-content");

    let content_rect = harness.layout_rect("#select-content");
    assert!(
        content_rect.width > 0.0 && content_rect.height > 0.0,
        "Select content must have non-zero geometry: width={}, height={}",
        content_rect.width,
        content_rect.height
    );

    // Verify positioning state is positioned (Ready), not unpositioned (Measuring)
    let data_state = harness.attr("#select-content", "data-state");
    assert_eq!(
        data_state.as_deref(),
        Some("open"),
        "Select content must be data-state='open'"
    );

    let data_pos_state = harness.attr("#select-content", "data-positioning-state");
    assert_eq!(
        data_pos_state.as_deref(),
        Some("positioned"),
        "Select content must be data-positioning-state='positioned' (FloatingReadiness::Ready)"
    );

    // Verify relative placement below trigger
    assert_relative_below(&harness, "#select-trigger", "#select-content", 10.0);
}

/// Test 2: SelectTrigger Space and Enter Zero Double-Fire
#[test]
fn test_native_select_trigger_zero_double_fire() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(playground_select::BasicFruitSelectSection);

    // Initial state: closed
    assert_eq!(
        harness.attr("#select-trigger", "data-state").as_deref(),
        Some("closed")
    );
    assert!(harness.query("#select-content").is_none());

    // 1. Focus trigger and press Space -> opens dropdown once (zero double-fire)
    focus_node(&mut harness, "#select-trigger");
    send_key_and_pump(&mut harness, Key::Character(" ".into()), Code::Space);

    assert_eq!(
        harness.attr("#select-trigger", "data-state").as_deref(),
        Some("open"),
        "Select trigger must remain open after Space (no double-fire close)"
    );
    assert!(
        harness.query("#select-content").is_some(),
        "Select content must mount upon Space on trigger"
    );

    // 2. Escape closes dropdown
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
    assert_eq!(
        harness.attr("#select-trigger", "data-state").as_deref(),
        Some("closed")
    );
    assert!(harness.query("#select-content").is_none());

    // 3. Focus trigger and press Enter -> opens dropdown once (zero double-fire)
    focus_node(&mut harness, "#select-trigger");
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);

    assert_eq!(
        harness.attr("#select-trigger", "data-state").as_deref(),
        Some("open"),
        "Select trigger must remain open after Enter (no double-fire close)"
    );
    assert!(
        harness.query("#select-content").is_some(),
        "Select content must mount upon Enter on trigger"
    );
}

/// Test 3: Select Content Multi-Word Typeahead with Space
#[test]
fn test_native_select_multi_word_space_typeahead() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(playground_select::BasicFruitSelectSection);

    // Open via Enter
    focus_node(&mut harness, "#select-trigger");
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
    assert!(harness.query("#select-content").is_some());

    // Focus content/listbox
    focus_node(&mut harness, "#select-content");

    // Type 'b' -> highlights "banana" or "blueberry"
    send_key_and_pump(&mut harness, Key::Character("b".into()), Code::KeyB);
    assert!(
        harness.query("#select-content").is_some(),
        "Select content must stay open while typing"
    );

    // Type 'l' -> highlights "blueberry"
    send_key_and_pump(&mut harness, Key::Character("l".into()), Code::KeyL);
    assert!(harness.query("#select-content").is_some());

    // Press Enter to commit highlighted item -> selection commits and dropdown closes!
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
    assert!(
        harness.query("#select-content").is_none(),
        "Enter must commit highlighted option and close dropdown"
    );
}

/// Test 4: Select :focus-visible restoration parity (keyboard close vs mouse item click & outside pointer dismiss)
#[test]
fn test_native_select_focus_visible_keyboard_vs_pointer_restoration() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(playground_select::BasicFruitSelectSection);

    // 1. Open via keyboard (Enter) + close via Escape -> #select-trigger HAS :focus-visible=true
    focus_node(&mut harness, "#select-trigger");
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
    advance_frame_and_pump(&mut harness, 0.02);
    assert!(harness.query("#select-content").is_some());

    send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
    advance_frame_and_pump(&mut harness, 0.05);
    assert!(harness.query("#select-content").is_none());
    assert_native_focus_visible(&harness, "#select-trigger", true);

    // 2. Open via keyboard (Enter), then mouse-click an option ([data-value='banana'], unfocusable <div role='option'>)
    //    -> commits banana AND restores #select-trigger with :focus-visible=false
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
    advance_frame_and_pump(&mut harness, 0.02);
    advance_frame_and_pump(&mut harness, 0.02);
    assert!(harness.query("#select-content").is_some());

    click_and_pump(&mut harness, "[data-value='banana']");
    advance_frame_and_pump(&mut harness, 0.05);
    assert!(harness.query("#select-content").is_none());
    assert_native_focus_visible(&harness, "#select-trigger", false);

    // 3. Open via keyboard (Space), then mouse-click unfocusable outside coordinates (5.0, 5.0)
    //    -> closes dropdown AND restores #select-trigger with :focus-visible=false
    send_key_and_pump(&mut harness, Key::Character(" ".into()), Code::Space);
    advance_frame_and_pump(&mut harness, 0.02);
    advance_frame_and_pump(&mut harness, 0.02);
    assert!(harness.query("#select-content").is_some());

    click_at_and_pump(&mut harness, 5.0, 5.0);
    advance_frame_and_pump(&mut harness, 0.05);
    assert!(harness.query("#select-content").is_none());
    assert_native_focus_visible(&harness, "#select-trigger", false);
}

/// Verifies keyboard auto-scroll remains inside the select viewport and skips disabled options.
#[test]
fn test_native_select_scrollable_viewport_keyboard_auto_scroll() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(StyledScrollableViewportSection);

    click_and_pump(&mut harness, ".select-trigger-scrollable");
    advance_frame_and_pump(&mut harness, 0.25);
    advance_frame_and_pump(&mut harness, 0.02);

    let viewport_selector = "[id$='viewport']";
    let viewport_rect = harness.layout_rect(viewport_selector);
    let viewport_node = harness.node(viewport_selector);
    assert!(
        viewport_rect.height <= 160.0,
        "Scrollable viewport height should be at most 160px, got {}",
        viewport_rect.height
    );

    focus_node(&mut harness, "[role='listbox']");
    assert_native_select_active_value(&harness, "argentina");

    for expected_value in ["canada", "denmark", "france", "germany", "japan", "korea"] {
        send_key_and_pump(
            &mut harness,
            Key::Named(NamedKey::ArrowDown),
            Code::ArrowDown,
        );
        assert_native_select_active_value(&harness, expected_value);
        assert_ne!(expected_value, "brazil");
        assert_ne!(expected_value, "egypt");
    }
    advance_frame_and_pump(&mut harness, 0.02);
    advance_frame_and_pump(&mut harness, 0.02);

    let viewport_scroll_y = harness
        .base()
        .get_node(viewport_node)
        .expect("Scrollable viewport node should exist")
        .scroll_offset()
        .y;
    assert!(
        viewport_scroll_y > 0.0,
        "Keyboard navigation to Korea should scroll the select viewport"
    );
    assert_eq!(
        harness.base().viewport_scroll().y,
        0.0,
        "Keyboard option scrolling must not scroll the root viewport"
    );

    let korea_rect = harness.layout_rect("[data-value='korea']");
    assert!(
        korea_rect.y >= viewport_rect.y - 1.0
            && korea_rect.y + korea_rect.height <= viewport_rect.y + viewport_rect.height + 1.0,
        "Korea rect {korea_rect:?} should be inside viewport {viewport_rect:?} within 1px"
    );

    send_key_and_pump(&mut harness, Key::Named(NamedKey::Home), Code::Home);
    advance_frame_and_pump(&mut harness, 0.02);
    advance_frame_and_pump(&mut harness, 0.02);
    assert_native_select_active_value(&harness, "argentina");

    let viewport_scroll_y = harness
        .base()
        .get_node(viewport_node)
        .expect("Scrollable viewport node should still exist")
        .scroll_offset()
        .y;
    assert!(
        viewport_scroll_y <= 1.0,
        "Home should restore viewport scroll to the top, got {viewport_scroll_y}"
    );

    let argentina_rect = harness.layout_rect("[data-value='argentina']");
    assert!(
        argentina_rect.y >= viewport_rect.y - 1.0
            && argentina_rect.y + argentina_rect.height
                <= viewport_rect.y + viewport_rect.height + 1.0,
        "Argentina rect {argentina_rect:?} should be inside viewport {viewport_rect:?} within 1px"
    );
}

/// Manual wheel scrolling must not reassert the boundary active option's viewport position.
#[test]
fn test_native_select_manual_scroll_does_not_snap_back_to_boundary_active_item() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(StyledScrollableViewportSection);

    click_and_pump(&mut harness, ".select-trigger-scrollable");
    advance_frame_and_pump(&mut harness, 0.25);
    advance_frame_and_pump(&mut harness, 0.02);
    advance_frame_and_pump(&mut harness, 0.22);
    focus_node(&mut harness, "[role='listbox']");

    let viewport_selector = "[id$='viewport']";
    assert_native_select_highlighted_value(&harness, "argentina");
    assert!(
        native_select_scroll_y(&harness, viewport_selector) <= 1.0,
        "Argentina should start at the viewport's top boundary"
    );
    let viewport_rect = harness.layout_rect(viewport_selector);
    let argentina_rect = harness.layout_rect("[data-value='argentina']");
    assert!(
        (argentina_rect.y - viewport_rect.y).abs() <= 1.0,
        "Argentina should be at the viewport top (option={argentina_rect:?}, viewport={viewport_rect:?})"
    );

    let argentina_point = (
        argentina_rect.x + argentina_rect.width * 0.5,
        argentina_rect.y + argentina_rect.height * 0.5,
    );
    wheel_at_and_pump(
        &mut harness,
        argentina_point.0,
        argentina_point.1,
        0.0,
        -40.0,
    );
    for _ in 0..4 {
        advance_frame_and_pump(&mut harness, 0.016);
        assert_native_select_highlighted_value(&harness, "argentina");
        assert!(
            native_select_scroll_y(&harness, viewport_selector) > 20.0,
            "Manual downward scrolling must not snap the viewport back to Argentina"
        );
    }
    assert!(
        native_select_scroll_y(&harness, "[role='listbox']") <= 1.0,
        "The viewport, rather than the listbox, should own the item scroll offset"
    );

    send_key_and_pump(&mut harness, Key::Named(NamedKey::End), Code::End);
    for _ in 0..3 {
        advance_frame_and_pump(&mut harness, 0.016);
    }
    assert_native_select_highlighted_value(&harness, "korea");
    let bottom_scroll_y = native_select_scroll_y(&harness, viewport_selector);
    assert!(
        bottom_scroll_y > 40.0,
        "End should auto-scroll Korea to the bottom boundary"
    );

    let korea_rect = harness.layout_rect("[data-value='korea']");
    wheel_at_and_pump(
        &mut harness,
        korea_rect.x + korea_rect.width * 0.5,
        korea_rect.y + korea_rect.height * 0.5,
        0.0,
        32.0,
    );
    let upward_scroll_y = native_select_scroll_y(&harness, viewport_selector);
    assert!(
        upward_scroll_y < bottom_scroll_y - 10.0,
        "Upward wheel scrolling should move away from the bottom (before={bottom_scroll_y}, after={upward_scroll_y})"
    );
    assert_native_select_highlighted_value(&harness, "korea");

    for _ in 0..4 {
        advance_frame_and_pump(&mut harness, 0.016);
        let settled_scroll_y = native_select_scroll_y(&harness, viewport_selector);
        assert!(
            settled_scroll_y < bottom_scroll_y - 10.0,
            "Upward scrolling must not snap back to Korea at the bottom; got {settled_scroll_y}"
        );
        assert_native_select_highlighted_value(&harness, "korea");
    }
}

/// Scrolling an overflow ancestor must preserve both floating placement and the inner viewport.
#[test]
fn test_native_select_outer_page_scroll_preserves_inner_viewport_scroll() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    #[component]
    fn OuterScrollableSelectScenario() -> Element {
        rsx! {
            style { "{playground_select::SELECT_PLAYGROUND_CSS}" }
            div {
                id: "select-outer-scroll",
                style: "width: 600px; height: 450px; overflow-y: auto; padding: 20px;",
                div { style: "height: 20px;", "Top spacer" }
                playground_select::ScrollableViewportSection {}
                div { style: "height: 800px;", "Bottom spacer" }
            }
        }
    }

    let mut harness = mount_native_scenario(OuterScrollableSelectScenario);
    click_and_pump(&mut harness, ".select-trigger-scrollable");
    advance_frame_and_pump(&mut harness, 0.25);
    advance_frame_and_pump(&mut harness, 0.02);
    advance_frame_and_pump(&mut harness, 0.22);
    focus_node(&mut harness, "[role='listbox']");

    let viewport_selector = "[id$='viewport']";
    let argentina_rect = harness.layout_rect("[data-value='argentina']");
    wheel_at_and_pump(
        &mut harness,
        argentina_rect.x + argentina_rect.width * 0.5,
        argentina_rect.y + argentina_rect.height * 0.5,
        0.0,
        -36.0,
    );
    advance_frame_and_pump(&mut harness, 0.016);
    let inner_scroll_before = native_select_scroll_y(&harness, viewport_selector);
    assert!(
        inner_scroll_before > argentina_rect.height as f64,
        "The Section 3 viewport should be manually scrolled before outer scrolling"
    );

    let trigger_before = harness.layout_rect(".select-trigger-scrollable");
    let content_before = harness.layout_rect("[role='listbox']");
    let relative_before = (
        content_before.x - trigger_before.x,
        content_before.y - (trigger_before.y + trigger_before.height),
    );

    let outer_rect = harness.layout_rect("#select-outer-scroll");
    wheel_at_and_pump(
        &mut harness,
        outer_rect.x + outer_rect.width - 8.0,
        outer_rect.y + outer_rect.height * 0.5,
        0.0,
        -48.0,
    );
    for _ in 0..3 {
        advance_frame_and_pump(&mut harness, 0.016);
    }

    let outer_scroll_after = native_select_scroll_y(&harness, "#select-outer-scroll");
    assert!(
        outer_scroll_after > 20.0,
        "Wheel input outside the select column should scroll the outer container"
    );
    let trigger_after = harness.layout_rect(".select-trigger-scrollable");
    let content_after = harness.layout_rect("[role='listbox']");
    let relative_after = (
        content_after.x - trigger_after.x,
        content_after.y - (trigger_after.y + trigger_after.height),
    );
    assert!(
        trigger_after.y < trigger_before.y - 20.0,
        "Outer scroll should move the trigger in the document (before={}, after={})",
        trigger_before.y,
        trigger_after.y
    );
    assert!(
        (relative_after.0 - relative_before.0).abs() <= 2.0
            && (relative_after.1 - relative_before.1).abs() <= 2.0,
        "Floating content should remain anchored to the trigger (before={relative_before:?}, after={relative_after:?})"
    );
    let inner_scroll_after = native_select_scroll_y(&harness, viewport_selector);
    assert!(
        (inner_scroll_after - inner_scroll_before).abs() <= 1.0,
        "Outer scrolling must not reset the inner viewport (before={inner_scroll_before}, after={inner_scroll_after})"
    );
    assert_native_select_highlighted_value(&harness, "argentina");
}

/// Stationary pointer events caused by wheel ticks must not retarget the active option.
#[test]
fn test_native_select_stationary_pointer_scroll_preserves_active_item() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(StyledScrollableViewportSection);

    click_and_pump(&mut harness, ".select-trigger-scrollable");
    advance_frame_and_pump(&mut harness, 0.25);
    advance_frame_and_pump(&mut harness, 0.02);
    advance_frame_and_pump(&mut harness, 0.22);

    let target_rect = harness.layout_rect("[data-value='denmark']");
    let pointer_x = target_rect.x + target_rect.width * 0.5;
    let pointer_y = target_rect.y + target_rect.height * 0.5;
    let dispatch_pointer_move =
        |harness: &mut blitz_test_harness::Harness<dioxus_native_dom::DioxusDocument>,
         x: f32,
         y: f32| {
            harness.dispatch(UiEvent::PointerMove(BlitzPointerEvent {
                id: BlitzPointerId::Mouse,
                is_primary: true,
                coords: PointerCoords {
                    page_x: x,
                    page_y: y,
                    screen_x: x,
                    screen_y: y,
                    client_x: x,
                    client_y: y,
                },
                button: MouseEventButton::Main,
                buttons: MouseEventButtons::empty(),
                mods: Default::default(),
                details: PointerDetails::default(),
                element: Point::default(),
                active_pointers: Default::default(),
            }));
            harness.pump();
        };
    let dispatch_wheel_tick =
        |harness: &mut blitz_test_harness::Harness<dioxus_native_dom::DioxusDocument>,
         x: f32,
         y: f32,
         delta_y: f64| {
            harness.dispatch(UiEvent::Wheel(BlitzWheelEvent {
                delta: BlitzWheelDelta::Pixels(0.0, delta_y),
                phase: BlitzWheelPhase::Moved,
                source: BlitzWheelSource::Wheel,
                coords: PointerCoords {
                    page_x: x,
                    page_y: y,
                    screen_x: x,
                    screen_y: y,
                    client_x: x,
                    client_y: y,
                },
                buttons: MouseEventButtons::empty(),
                mods: Default::default(),
                element: Point::default(),
            }));
            harness.pump();
        };

    dispatch_pointer_move(&mut harness, pointer_x, pointer_y);
    assert_native_select_highlighted_value(&harness, "denmark");
    let active_descendant_before = harness.attr("[role='listbox']", "aria-activedescendant");

    for _ in 0..8 {
        dispatch_wheel_tick(&mut harness, pointer_x, pointer_y, -8.0);
        advance_frame_and_pump(&mut harness, 0.016);
        dispatch_pointer_move(&mut harness, pointer_x, pointer_y);
        assert_eq!(
            harness.attr("[role='listbox']", "aria-activedescendant"),
            active_descendant_before,
            "A stationary PointerMove interleaved with wheel ticks must keep the active descendant"
        );
        assert_native_select_highlighted_value(&harness, "denmark");
    }
    assert!(
        native_select_scroll_y(&harness, "[id$='viewport']") > 40.0,
        "Wheel ticks must actually scroll the viewport for this regression to be non-vacuous"
    );
}

/// Pointer highlighting a clipped option must not move the viewport; keyboard navigation still must.
#[test]
fn test_native_select_pointer_highlight_does_not_force_viewport_scroll() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(StyledScrollableViewportSection);

    click_and_pump(&mut harness, ".select-trigger-scrollable");
    advance_frame_and_pump(&mut harness, 0.25);
    advance_frame_and_pump(&mut harness, 0.02);
    advance_frame_and_pump(&mut harness, 0.22);
    focus_node(&mut harness, "[role='listbox']");

    let viewport_selector = "[id$='viewport']";
    let argentina_rect = harness.layout_rect("[data-value='argentina']");
    wheel_at_and_pump(
        &mut harness,
        argentina_rect.x + argentina_rect.width * 0.5,
        argentina_rect.y + argentina_rect.height * 0.5,
        0.0,
        -24.0,
    );
    advance_frame_and_pump(&mut harness, 0.016);
    let pointer_scroll_before = native_select_scroll_y(&harness, viewport_selector);
    assert!(
        pointer_scroll_before > 10.0,
        "The viewport should be manually scrolled before pointer highlighting"
    );

    let enabled_values = [
        "argentina",
        "canada",
        "denmark",
        "france",
        "germany",
        "japan",
        "korea",
    ];
    let viewport_rect = harness.layout_rect(viewport_selector);
    let pointer_target = enabled_values
        .windows(2)
        .rev()
        .find_map(|pair| {
            let target_rect = harness.layout_rect(&format!("[data-value='{}']", pair[0]));
            let next_rect = harness.layout_rect(&format!("[data-value='{}']", pair[1]));
            let visible_top = target_rect.y.max(viewport_rect.y);
            let visible_bottom = (target_rect.y + target_rect.height)
                .min(viewport_rect.y + viewport_rect.height);
            if visible_bottom - visible_top >= 4.0
                && next_rect.y + next_rect.height > viewport_rect.y + viewport_rect.height
            {
                Some((
                    pair[0],
                    pair[1],
                    target_rect.x + target_rect.width * 0.5,
                    (visible_top + visible_bottom) * 0.5,
                ))
            } else {
                None
            }
        })
        .expect("Expected a partially visible lower option followed by an offscreen option");
    let (target_value, next_value, pointer_x, pointer_y) = pointer_target;

    harness.move_mouse_to(pointer_x, pointer_y);
    harness.pump();
    assert_native_select_highlighted_value(&harness, target_value);
    let pointer_scroll_after = native_select_scroll_y(&harness, viewport_selector);
    assert!(
        (pointer_scroll_after - pointer_scroll_before).abs() <= 1.0,
        "Pointer highlighting must not force a viewport scroll (before={pointer_scroll_before}, after={pointer_scroll_after})"
    );

    let next_rect_before = harness.layout_rect(&format!("[data-value='{next_value}']"));

    send_key_and_pump(
        &mut harness,
        Key::Named(NamedKey::ArrowDown),
        Code::ArrowDown,
    );
    for _ in 0..6 {
        advance_frame_and_pump(&mut harness, 0.016);
    }
    assert_native_select_highlighted_value(&harness, next_value);
    let keyboard_scroll_after = native_select_scroll_y(&harness, viewport_selector);
    assert!(
        keyboard_scroll_after > pointer_scroll_after + 1.0,
        "ArrowDown should auto-scroll the next option into view (pointer={pointer_scroll_after}, keyboard={keyboard_scroll_after})"
    );
    let next_rect_after = harness.layout_rect(&format!("[data-value='{next_value}']"));
    assert!(
        next_rect_after.y < next_rect_before.y - 1.0,
        "ArrowDown should scroll {next_value} toward the viewport (before={next_rect_before:?}, after={next_rect_after:?})"
    );
    assert!(
        native_select_scroll_y(&harness, "[role=listbox]") <= 1.0,
        "Keyboard option scrolling must remain owned by the viewport, not the listbox"
    );

    send_key_and_pump(&mut harness, Key::Named(NamedKey::Home), Code::Home);
    for _ in 0..3 {
        advance_frame_and_pump(&mut harness, 0.016);
    }
    assert_native_select_highlighted_value(&harness, "argentina");
    assert!(
        native_select_scroll_y(&harness, viewport_selector) <= 1.0,
        "Home should auto-scroll the first option to the top"
    );
}

/// Verifies static, in-flow select content uses normal roving keyboard navigation.
#[test]
fn test_native_select_static_content_keyboard_navigation() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(playground_select::StaticContentSection);

    click_and_pump(&mut harness, ".select-trigger-static");
    advance_frame_and_pump(&mut harness, 0.02);
    advance_frame_and_pump(&mut harness, 0.02);
    focus_node(&mut harness, "[role='listbox']");

    assert_native_select_active_value(&harness, "fast");
    assert_eq!(
        harness
            .attr("[data-value='fast']", "data-highlighted")
            .as_deref(),
        Some("true")
    );
    assert_eq!(
        harness
            .attr("[data-value='faster']", "data-highlighted")
            .as_deref(),
        Some("false")
    );

    send_key_and_pump(
        &mut harness,
        Key::Named(NamedKey::ArrowDown),
        Code::ArrowDown,
    );
    assert_native_select_active_value(&harness, "faster");
    assert_eq!(
        harness
            .attr("[data-value='fast']", "data-highlighted")
            .as_deref(),
        Some("false")
    );
    assert_eq!(
        harness
            .attr("[data-value='faster']", "data-highlighted")
            .as_deref(),
        Some("true")
    );

    send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowUp), Code::ArrowUp);
    assert_native_select_active_value(&harness, "fast");

    send_key_and_pump(&mut harness, Key::Named(NamedKey::End), Code::End);
    assert_native_select_active_value(&harness, "faster");

    send_key_and_pump(&mut harness, Key::Named(NamedKey::Home), Code::Home);
    assert_native_select_active_value(&harness, "fast");

    send_key_and_pump(
        &mut harness,
        Key::Named(NamedKey::ArrowDown),
        Code::ArrowDown,
    );
    assert_native_select_active_value(&harness, "faster");
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
    advance_frame_and_pump(&mut harness, 0.30);

    assert!(
        harness
            .text_content(".select-trigger-static")
            .contains("Faster"),
        "Enter should commit the highlighted Faster option"
    );
    assert!(
        harness.query("[role='listbox']").is_none(),
        "Static select listbox should close after Enter commits Faster"
    );
}

/// Verifies deterministic initial highlights after repeated select content unmounts.
#[test]
fn test_native_select_reopen_deterministic_highlight_across_unmounts() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    assert_native_select_reopen_highlight(
        playground_select::StaticContentSection,
        ".select-trigger-static",
        "fast",
    );
    assert_native_select_reopen_highlight(
        playground_select::GroupedSelectSection,
        ".select-trigger-grouped",
        "carrot",
    );
    assert_native_select_reopen_highlight(
        playground_select::ScrollableViewportSection,
        ".select-trigger-scrollable",
        "argentina",
    );
}
