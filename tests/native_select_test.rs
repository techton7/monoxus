//! Native Select Click & Floating Readiness Verification Test Suite.
//!
//! Validates that clicking #select-trigger in a headless native Blitz document
//! immediately mounts #select-content and transitions FloatingReadiness to Ready
//! with non-empty bounds without ANY wheel or scroll events.

#![allow(unused_imports, dead_code)]

mod common;
use common::native_fixture::*;
use dioxus::prelude::*;

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
