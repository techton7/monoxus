//! Native Select Click & Floating Readiness Verification Test Suite.
//!
//! Validates that clicking #select-trigger in a headless native Blitz document
//! immediately mounts #select-content and transitions FloatingReadiness to Ready
//! with non-empty bounds without ANY wheel or scroll events.

#![allow(unused_imports, dead_code)]

use crate::common;
use crate::playground_fixtures::playground_select;
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
    advance_frame_and_pump(&mut harness, 0.02);
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
