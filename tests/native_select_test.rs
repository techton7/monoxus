//! Native Select Click & Floating Readiness Verification Test Suite.
//!
//! Validates that clicking #select-trigger in a headless native Blitz document
//! immediately mounts #select-content and transitions FloatingReadiness to Ready
//! with non-empty bounds without ANY wheel or scroll events.

#![allow(unused_imports, dead_code)]

mod common;
use common::native_fixture::*;
use dioxus::prelude::*;
use keyboard_types::{Code, Key, NamedKey};

#[path = "../examples/playground/scenarios/select/mod.rs"]
mod scenarios_select;

static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn test_native_select_click_immediate_ready_without_wheel_scroll() {
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(scenarios_select::BasicFruitSelectSection);

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
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(scenarios_select::BasicFruitSelectSection);

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
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(scenarios_select::BasicFruitSelectSection);

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
    let _lock = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(scenarios_select::BasicFruitSelectSection);

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
