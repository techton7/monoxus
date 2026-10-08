//! Headless Native Test Fixture for Dioxus / Blitz Native testing.
//!
//! Provides thin, ergonomic harness orchestration and assertions around
//! `blitz-test-harness` for testing headless Dioxus components without window or GPU overhead.

use blitz_dom::Document;
use blitz_test_harness::Harness;
use blitz_traits::events::{BlitzKeyEvent, KeyState, UiEvent};
use blitz_traits::SmolStr;
use dioxus::prelude::Element;
use dioxus_native_dom::DioxusDocument;
use keyboard_types::{Code, Key, Location, Modifiers};

/// Mounts a Dioxus scenario component headlessly into a Blitz document harness.
///
/// Constructs `Harness::from_component(app)` and performs initial layout/style pump.
pub fn mount_native_scenario(app: fn() -> Element) -> Harness<DioxusDocument> {
    let mut harness = Harness::from_component(app);
    let native_doc = oxidase::dom::Document::from_base(harness.doc.inner.clone());
    oxidase::dom::Document::set_current(Some(native_doc));
    harness.pump();
    harness
}

/// Interaction Helper: Clicks the node matching `selector`, then runs `pump()` and `tick(0.01)`.
pub fn click_and_pump<D: Document>(harness: &mut Harness<D>, selector: &str) {
    harness.click(selector);
    harness.pump();
    oxidase::frame::tick(std::time::Duration::from_millis(16));
    harness.pump();
    harness.tick(0.01);
}

/// Interaction Helper: Sends a keydown followed by keyup for the given `key` and `code`, then runs `pump()`.
pub fn send_key_and_pump<D: Document>(harness: &mut Harness<D>, key: Key, code: Code) {
    send_key_and_pump_with(harness, key, code, Modifiers::default());
}

/// Interaction Helper: Sends keydown and keyup with explicit `modifiers`, then runs `pump()` and `tick(0.01)`.
pub fn send_key_and_pump_with<D: Document>(
    harness: &mut Harness<D>,
    key: Key,
    code: Code,
    modifiers: Modifiers,
) {
    let text = match (&key, KeyState::Pressed) {
        (Key::Character(t), KeyState::Pressed) => Some(SmolStr::new(t)),
        _ => None,
    };

    let down_event = BlitzKeyEvent {
        key: key.clone(),
        code,
        modifiers,
        location: Location::Standard,
        is_auto_repeating: false,
        is_composing: false,
        state: KeyState::Pressed,
        text,
    };

    let up_event = BlitzKeyEvent {
        key,
        code,
        modifiers,
        location: Location::Standard,
        is_auto_repeating: false,
        is_composing: false,
        state: KeyState::Released,
        text: None,
    };

    harness.dispatch(UiEvent::KeyDown(down_event));
    harness.dispatch(UiEvent::KeyUp(up_event));
    harness.pump();
    oxidase::frame::tick(std::time::Duration::from_millis(16));
    harness.pump();
    harness.tick(0.01);
}

/// Sets native focus to the element matching `selector` and pumps the harness.
pub fn focus_node<D: Document>(harness: &mut Harness<D>, selector: &str) {
    let node_id = harness.node(selector);
    harness.base_mut().set_focus_to(node_id);
    harness.pump();
}

/// Focus Assertion Helper: Verifies that `harness.base().active_focus_node_id()`
/// (or `harness.focused()`) matches the DOM node for `expected_id`.
pub fn assert_native_focus_id<D: Document>(harness: &Harness<D>, expected_id: &str) {
    let raw_id = expected_id.trim_start_matches('#');
    let sel = format!("#{raw_id}");

    let expected_node_id = harness
        .query(&sel)
        .unwrap_or_else(|| panic!("No node found in document matching selector '{sel}'"));

    let active_node_id = harness
        .base()
        .active_focus_node_id()
        .or_else(|| harness.focused())
        .unwrap_or_else(|| {
            panic!("Expected active focused node for '{expected_id}', but no node was focused")
        });

    assert_eq!(
        active_node_id, expected_node_id,
        "Active focus node {active_node_id:?} does not match expected node {expected_node_id:?} for '#{raw_id}'"
    );

    let doc = harness.base();
    let node = doc
        .get_node(active_node_id)
        .expect("Active focus node not found in document");
    let node_id_attr = node.attr(blitz_dom::local_name!("id"));
    assert_eq!(
        node_id_attr,
        Some(raw_id),
        "Active focused element has id attribute {node_id_attr:?}, expected '{raw_id}'"
    );
}

/// Relational Geometry Helper: Asserts that `content_sel` is vertically positioned below
/// `trigger_sel` within the given tolerance.
///
/// Verifies: `content_rect.y >= (trigger_rect.y + trigger_rect.height) - tolerance`.
pub fn assert_relative_below<D: Document>(
    harness: &Harness<D>,
    trigger_sel: &str,
    content_sel: &str,
    tolerance: f32,
) {
    let trigger_rect = harness.layout_rect(trigger_sel);
    let content_rect = harness.layout_rect(content_sel);
    let trigger_bottom = trigger_rect.y + trigger_rect.height;

    assert!(
        content_rect.y >= trigger_bottom - tolerance,
        "Expected '{content_sel}' (y={}) to be below '{trigger_sel}' (bottom={trigger_bottom}) with tolerance {tolerance}",
        content_rect.y
    );
}

/// Relational Geometry Helper: Asserts that `content_sel` is horizontally aligned with
/// `trigger_sel` within the given tolerance (checking left-edge or center-point alignment).
#[allow(dead_code)]
pub fn assert_relative_aligned<D: Document>(
    harness: &Harness<D>,
    trigger_sel: &str,
    content_sel: &str,
    tolerance: f32,
) {
    let trigger_rect = harness.layout_rect(trigger_sel);
    let content_rect = harness.layout_rect(content_sel);

    let left_diff = (content_rect.x - trigger_rect.x).abs();
    let center_diff = (content_rect.center().0 - trigger_rect.center().0).abs();

    assert!(
        left_diff <= tolerance || center_diff <= tolerance,
        "Expected '{content_sel}' to align horizontally with '{trigger_sel}' within tolerance {tolerance}. Left diff: {left_diff}, Center diff: {center_diff}",
    );
}

/// Relational Geometry Helper: Asserts that the element matching `selector` has non-empty width and height.
pub fn assert_rect_non_empty<D: Document>(harness: &Harness<D>, selector: &str) {
    let rect = harness.layout_rect(selector);
    assert!(
        rect.width > 0.0 && rect.height > 0.0,
        "Expected non-empty rect for '{selector}', but got width={}, height={}",
        rect.width,
        rect.height
    );
}
