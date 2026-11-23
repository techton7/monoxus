//! Native PinInput Verification Test Suite.
//!
//! Drives a real `blitz_dom::BaseDocument` + `EventDriver` with focus, pointer clicks,
//! keystrokes, `UiEvent::Paste`, arrow/backspace editing, masking, and `<form>` reset
//! to verify dual-host parity prior to Phase-2 GUI execution.

use crate::common;
use blitz_traits::SmolStr;
use blitz_traits::events::{
    BlitzKeyEvent, BlitzPointerEvent, BlitzPointerId, KeyState, MouseEventButton,
    MouseEventButtons, PointerCoords, PointerDetails, UiEvent,
};
use common::native_fixture::*;
use dioxus::prelude::*;
use keyboard_types::{Code, Key, Location, Modifiers, NamedKey};
use monoxus::button::{Button, ButtonType};
use monoxus::foundation::form::collect_form_data;
use monoxus::pin_input::*;

fn send_arrow_left_and_pump<D: blitz_dom::Document>(harness: &mut blitz_test_harness::Harness<D>) {
    #[cfg(target_os = "macos")]
    {
        harness.dispatch(UiEvent::AppleStandardKeybinding(SmolStr::new("moveLeft:")));
        let up_event = BlitzKeyEvent {
            key: Key::Named(NamedKey::ArrowLeft),
            code: Code::ArrowLeft,
            modifiers: Modifiers::default(),
            location: Location::Standard,
            is_auto_repeating: false,
            is_composing: false,
            state: KeyState::Released,
            text: None,
        };
        harness.dispatch(UiEvent::KeyUp(up_event));
        drain_polls_and_resolve(harness);
        oxidase::frame::tick(std::time::Duration::from_millis(16));
        drain_polls_and_resolve(harness);
        harness.tick(0.01);
    }
    #[cfg(not(target_os = "macos"))]
    {
        send_key_and_pump(harness, Key::Named(NamedKey::ArrowLeft), Code::ArrowLeft);
    }
}

fn send_backspace_and_pump<D: blitz_dom::Document>(harness: &mut blitz_test_harness::Harness<D>) {
    #[cfg(target_os = "macos")]
    {
        harness.dispatch(UiEvent::AppleStandardKeybinding(SmolStr::new(
            "deleteBackward:",
        )));
        let up_event = BlitzKeyEvent {
            key: Key::Named(NamedKey::Backspace),
            code: Code::Backspace,
            modifiers: Modifiers::default(),
            location: Location::Standard,
            is_auto_repeating: false,
            is_composing: false,
            state: KeyState::Released,
            text: None,
        };
        harness.dispatch(UiEvent::KeyUp(up_event));
        drain_polls_and_resolve(harness);
        oxidase::frame::tick(std::time::Duration::from_millis(16));
        drain_polls_and_resolve(harness);
        harness.tick(0.01);
    }
    #[cfg(not(target_os = "macos"))]
    {
        send_key_and_pump(harness, Key::Named(NamedKey::Backspace), Code::Backspace);
    }
}

fn send_char_and_pump<D: blitz_dom::Document>(
    harness: &mut blitz_test_harness::Harness<D>,
    ch: &str,
) {
    send_key_and_pump(
        harness,
        Key::Character(ch.to_string()),
        Code::Unidentified,
    );
}

fn send_paste_and_pump<D: blitz_dom::Document>(
    harness: &mut blitz_test_harness::Harness<D>,
    text: &str,
) {
    harness.dispatch(UiEvent::Paste(text.to_string()));
    drain_polls_and_resolve(harness);
    oxidase::frame::tick(std::time::Duration::from_millis(16));
    drain_polls_and_resolve(harness);
    harness.tick(0.01);
}

#[component]
fn NativePinInputTypingApp() -> Element {
    let mut val = use_signal(String::new);
    let mut complete_count = use_signal(|| 0usize);
    let mut completed_val = use_signal(String::new);

    rsx! {
        div {
            style: "padding: 16px; width: 400px; height: 120px;",
            div {
                id: "pin-probe",
                "data-value": "{val}",
                "data-complete-count": "{complete_count}",
                "data-completed-val": "{completed_val}",
            }
            PinInputRoot {
                id: "pin-root",
                input_id: "pin-input",
                max_length: 6,
                pattern: PinInputPattern::Digits,
                style: "display: flex; gap: 8px; width: 280px; height: 40px;",
                on_value_change: move |next: String| val.set(next),
                on_complete: move |full: String| {
                    complete_count += 1;
                    completed_val.set(full);
                },
                render_children: move |snap: PinInputSnapshot| rsx! {
                    for cell in snap.cells.iter() {
                        PinInputCell {
                            key: "{cell.index}",
                            id: "pin-cell-{cell.index}",
                            style: "width: 36px; height: 36px;",
                            "data-char": cell.char.map(|c| c.to_string()).unwrap_or_default(),
                            "data-fake-caret": if cell.has_fake_caret { "true" } else { "false" },
                            index: cell.index,
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test_native_pin_input_focus_typing_pattern_and_on_complete() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(NativePinInputTypingApp);

    // Click root area -> hits overlaid <input> and sets focus + cell 0 active with fake caret.
    click_and_pump(&mut harness, "#pin-root");
    assert_native_focus_id(&harness, "pin-input");
    assert_eq!(harness.attr("#pin-cell-0", "data-active").as_deref(), Some(""));
    assert_eq!(harness.attr("#pin-cell-0", "data-inactive"), None);
    assert_eq!(
        harness.attr("#pin-cell-0", "data-fake-caret").as_deref(),
        Some("true")
    );
    assert_eq!(
        harness.attr("#pin-cell-1", "data-inactive").as_deref(),
        Some("")
    );

    // Invalid non-digit characters are rejected while Digits pattern is active.
    send_char_and_pump(&mut harness, "a");
    send_char_and_pump(&mut harness, "$");
    assert_eq!(harness.attr("#pin-probe", "data-value").as_deref(), Some(""));
    assert_eq!(harness.attr("#pin-cell-0", "data-active").as_deref(), Some(""));

    // Type 6 digits sequentially -> advances active cell and fires on_complete once.
    for (idx, digit) in ["1", "2", "3", "4", "5", "6"].iter().enumerate() {
        send_char_and_pump(&mut harness, digit);
        if idx < 5 {
            let active_sel = format!("#pin-cell-{}", idx + 1);
            assert_eq!(harness.attr(&active_sel, "data-active").as_deref(), Some(""));
            assert_eq!(
                harness.attr(&active_sel, "data-fake-caret").as_deref(),
                Some("true")
            );
        }
    }

    assert_eq!(
        harness.attr("#pin-probe", "data-value").as_deref(),
        Some("123456")
    );
    assert_eq!(
        harness.attr("#pin-probe", "data-complete-count").as_deref(),
        Some("1")
    );
    assert_eq!(
        harness.attr("#pin-probe", "data-completed-val").as_deref(),
        Some("123456")
    );
    assert_eq!(harness.attr("#pin-cell-5", "data-active").as_deref(), Some(""));
    assert_eq!(
        harness.attr("#pin-cell-5", "data-fake-caret").as_deref(),
        Some("false")
    );

    // Typing a 7th digit when full overwrites the last cell ("123456" -> "123457")
    // without re-firing edge-triggered on_complete.
    send_char_and_pump(&mut harness, "7");
    assert_eq!(
        harness.attr("#pin-probe", "data-value").as_deref(),
        Some("123457")
    );
    assert_eq!(
        harness.attr("#pin-probe", "data-complete-count").as_deref(),
        Some("1")
    );
    assert_eq!(
        harness.attr("#pin-probe", "data-completed-val").as_deref(),
        Some("123456")
    );
    assert_eq!(harness.attr("#pin-cell-5", "data-active").as_deref(), Some(""));
    assert_eq!(
        harness.attr("#pin-cell-5", "data-fake-caret").as_deref(),
        Some("false")
    );

    // Clicking the filled last cell while already focused preserves [5, 6) selection
    // and allows immediate consecutive overwrites ("8" -> "9").
    click_and_pump(&mut harness, "#pin-cell-5");
    assert_eq!(
        harness
            .attr("#pin-input", "data-pin-input-input-mss")
            .as_deref(),
        Some("5")
    );
    assert_eq!(
        harness
            .attr("#pin-input", "data-pin-input-input-mse")
            .as_deref(),
        Some("6")
    );
    send_char_and_pump(&mut harness, "8");
    assert_eq!(
        harness.attr("#pin-probe", "data-value").as_deref(),
        Some("123458")
    );
    send_char_and_pump(&mut harness, "9");
    assert_eq!(
        harness.attr("#pin-probe", "data-value").as_deref(),
        Some("123459")
    );
    assert_eq!(
        harness.attr("#pin-probe", "data-complete-count").as_deref(),
        Some("1")
    );

    // ArrowLeft from full input steps to cell 4 ('5') and allows replacement.
    send_arrow_left_and_pump(&mut harness);
    assert_eq!(harness.attr("#pin-cell-4", "data-active").as_deref(), Some(""));
    send_char_and_pump(&mut harness, "0");
    assert_eq!(
        harness.attr("#pin-probe", "data-value").as_deref(),
        Some("123409")
    );
    assert_eq!(harness.attr("#pin-cell-5", "data-active").as_deref(), Some(""));

    // Blur -> all cells become inactive.
    harness.base_mut().clear_focus();
    drain_polls_and_resolve(&mut harness);
    assert_eq!(
        harness.attr("#pin-cell-5", "data-inactive").as_deref(),
        Some("")
    );
    assert_eq!(harness.attr("#pin-cell-5", "data-active"), None);

    // Clicking the filled last cell from unfocused restores focus, selects [5, 6),
    // supports immediate overwrite, and Backspace returns cell 5 to insert mode with fake caret.
    click_and_pump(&mut harness, "#pin-cell-5");
    assert_native_focus_id(&harness, "pin-input");
    assert_eq!(
        harness
            .attr("#pin-input", "data-pin-input-input-mss")
            .as_deref(),
        Some("5")
    );
    assert_eq!(
        harness
            .attr("#pin-input", "data-pin-input-input-mse")
            .as_deref(),
        Some("6")
    );
    assert_eq!(harness.attr("#pin-cell-5", "data-active").as_deref(), Some(""));
    send_char_and_pump(&mut harness, "8");
    assert_eq!(
        harness.attr("#pin-probe", "data-value").as_deref(),
        Some("123408")
    );
    send_backspace_and_pump(&mut harness);
    assert_eq!(
        harness.attr("#pin-probe", "data-value").as_deref(),
        Some("12340")
    );
    assert_eq!(
        harness
            .attr("#pin-input", "data-pin-input-input-mss")
            .as_deref(),
        Some("5")
    );
    assert_eq!(
        harness
            .attr("#pin-input", "data-pin-input-input-mse")
            .as_deref(),
        Some("5")
    );
    assert_eq!(harness.attr("#pin-cell-5", "data-active").as_deref(), Some(""));
    assert_eq!(
        harness.attr("#pin-cell-5", "data-fake-caret").as_deref(),
        Some("true")
    );

    // Paste overflow still clamps strictly to max_length=6 ("123456789" -> "123456").
    let mut paste_harness = mount_native_scenario(NativePinInputTypingApp);
    click_and_pump(&mut paste_harness, "#pin-root");
    send_paste_and_pump(&mut paste_harness, "123456789");
    assert_eq!(
        paste_harness.attr("#pin-probe", "data-value").as_deref(),
        Some("123456")
    );
    assert_eq!(
        paste_harness
            .attr("#pin-probe", "data-complete-count")
            .as_deref(),
        Some("1")
    );
}

#[test]
fn test_native_pin_input_mid_string_navigation_replacement_and_backspace() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(NativePinInputTypingApp);

    focus_node(&mut harness, "#pin-input");
    for d in ["1", "2", "3", "4"] {
        send_char_and_pump(&mut harness, d);
    }
    assert_eq!(
        harness.attr("#pin-probe", "data-value").as_deref(),
        Some("1234")
    );
    assert_eq!(harness.attr("#pin-cell-4", "data-active").as_deref(), Some(""));

    // Move left twice -> cell 3 ('4'), then cell 2 ('3').
    send_arrow_left_and_pump(&mut harness);
    assert_eq!(harness.attr("#pin-cell-3", "data-active").as_deref(), Some(""));
    send_arrow_left_and_pump(&mut harness);
    assert_eq!(harness.attr("#pin-cell-2", "data-active").as_deref(), Some(""));

    // Replace cell 2 with '9' -> value becomes "1294", active moves to cell 3.
    send_char_and_pump(&mut harness, "9");
    assert_eq!(
        harness.attr("#pin-probe", "data-value").as_deref(),
        Some("1294")
    );
    assert_eq!(harness.attr("#pin-cell-3", "data-active").as_deref(), Some(""));

    // Backspace from end of "123" deletes last digit cleanly.
    let mut bs_harness = mount_native_scenario(NativePinInputTypingApp);
    focus_node(&mut bs_harness, "#pin-input");
    for d in ["1", "2", "3"] {
        send_char_and_pump(&mut bs_harness, d);
    }
    send_backspace_and_pump(&mut bs_harness);
    assert_eq!(
        bs_harness.attr("#pin-probe", "data-value").as_deref(),
        Some("12")
    );
    assert_eq!(
        bs_harness.attr("#pin-cell-2", "data-active").as_deref(),
        Some("")
    );
    assert_eq!(
        bs_harness.attr("#pin-cell-2", "data-fake-caret").as_deref(),
        Some("true")
    );
}

#[component]
fn NativePinInputPasteMaskAndFormApp() -> Element {
    let mut val = use_signal(|| String::from("12"));
    let mut completed = use_signal(String::new);

    rsx! {
        div {
            style: "padding: 16px; width: 500px; height: 220px;",
            div {
                id: "form-probe",
                "data-val": "{val}",
                "data-completed": "{completed}",
            }
            form {
                id: "otp-form",
                style: "display: flex; flex-direction: column; gap: 12px;",
                PinInputRoot {
                    id: "otp-pin-root",
                    input_id: "otp-pin-input",
                    name: "otp".to_string(),
                    default_value: "12".to_string(),
                    max_length: 6,
                    mask: true,
                    mask_char: '*',
                    pattern: PinInputPattern::Digits,
                    paste_transformer: Callback::new(|s: String| s.replace('-', "")),
                    style: "display: flex; gap: 6px; width: 260px; height: 40px;",
                    on_value_change: move |next: String| val.set(next),
                    on_complete: move |full: String| completed.set(full),
                    PinInputCell { id: "otp-cell-0", index: 0, style: "width: 36px; height: 36px;" }
                    PinInputCell { id: "otp-cell-1", index: 1, style: "width: 36px; height: 36px;" }
                    PinInputCell { id: "otp-cell-2", index: 2, style: "width: 36px; height: 36px;" }
                    PinInputCell { id: "otp-cell-3", index: 3, style: "width: 36px; height: 36px;" }
                    PinInputCell { id: "otp-cell-4", index: 4, style: "width: 36px; height: 36px;" }
                    PinInputCell { id: "otp-cell-5", index: 5, style: "width: 36px; height: 36px;" }
                }
                Button {
                    id: "otp-reset-btn".to_string(),
                    r#type: ButtonType::Reset,
                    style: "width: 100px; height: 32px;".to_string(),
                    "Reset"
                }
            }
        }
    }
}

#[test]
fn test_native_pin_input_paste_mask_and_form_reset() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(NativePinInputPasteMaskAndFormApp);
    advance_frame_and_pump(&mut harness, 0.016);

    // Mask renders '*' inside populated cells while FormData holds raw "12".
    assert_eq!(harness.text_content("#otp-cell-0"), "*");
    assert_eq!(harness.text_content("#otp-cell-1"), "*");
    assert_eq!(harness.text_content("#otp-cell-2"), "");
    assert_eq!(
        collect_form_data("otp-form").expect("initial otp-form data"),
        vec![("otp".to_string(), "12".to_string())]
    );

    // Focus input and paste "1234-56" (flush-from-0 replaces "12" -> "123456").
    focus_node(&mut harness, "#otp-pin-input");
    send_paste_and_pump(&mut harness, "1234-56");

    assert_eq!(
        harness.attr("#form-probe", "data-val").as_deref(),
        Some("123456")
    );
    assert_eq!(
        harness.attr("#form-probe", "data-completed").as_deref(),
        Some("123456")
    );
    assert_eq!(harness.text_content("#otp-cell-5"), "*");
    assert_eq!(
        collect_form_data("otp-form").expect("pasted otp-form data"),
        vec![("otp".to_string(), "123456".to_string())]
    );

    // Trigger native <form> reset -> restores default_value "12" in state, cells, and FormData.
    click_and_pump(&mut harness, "#otp-reset-btn");
    assert_eq!(
        harness.attr("#form-probe", "data-val").as_deref(),
        Some("12")
    );
    assert_eq!(harness.text_content("#otp-cell-0"), "*");
    assert_eq!(harness.text_content("#otp-cell-1"), "*");
    assert_eq!(harness.text_content("#otp-cell-2"), "");
    assert_eq!(
        collect_form_data("otp-form").expect("post-reset otp-form data"),
        vec![("otp".to_string(), "12".to_string())]
    );
}

#[component]
fn NativePinInputPointerApp() -> Element {
    let mut val = use_signal(|| String::from("1234"));
    let mut partial_val = use_signal(|| String::from("12"));

    rsx! {
        div {
            style: "padding: 16px; width: 400px; height: 220px; display: flex; flex-direction: column; gap: 16px;",
            div {
                id: "ptr-probe",
                "data-value": "{val}",
                "data-partial-value": "{partial_val}",
            }
            PinInputRoot {
                id: "ptr-pin-root",
                input_id: "ptr-pin-input",
                default_value: "1234".to_string(),
                max_length: 6,
                pattern: PinInputPattern::Digits,
                style: "display: flex; gap: 8px; width: 280px; height: 40px;",
                on_value_change: move |next: String| val.set(next),
                render_children: move |snap: PinInputSnapshot| rsx! {
                    for cell in snap.cells.iter() {
                        PinInputCell {
                            key: "{cell.index}",
                            id: "ptr-cell-{cell.index}",
                            style: "width: 36px; height: 36px;",
                            "data-char": cell.char.map(|c| c.to_string()).unwrap_or_default(),
                            "data-fake-caret": if cell.has_fake_caret { "true" } else { "false" },
                            index: cell.index,
                        }
                    }
                }
            }
            PinInputRoot {
                id: "ptr2-pin-root",
                input_id: "ptr2-pin-input",
                default_value: "12".to_string(),
                max_length: 6,
                pattern: PinInputPattern::Digits,
                style: "display: flex; gap: 8px; width: 280px; height: 40px;",
                on_value_change: move |next: String| partial_val.set(next),
                render_children: move |snap: PinInputSnapshot| rsx! {
                    for cell in snap.cells.iter() {
                        PinInputCell {
                            key: "{cell.index}",
                            id: "ptr2-cell-{cell.index}",
                            style: "width: 36px; height: 36px;",
                            "data-char": cell.char.map(|c| c.to_string()).unwrap_or_default(),
                            "data-fake-caret": if cell.has_fake_caret { "true" } else { "false" },
                            index: cell.index,
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test_native_pin_input_pointer_cell_targeting() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(NativePinInputPointerApp);

    // Clicking cell 0 on an unfocused PinInput ("1234") focuses and selects (0, 1) (not overwritten by handle_focus).
    click_and_pump(&mut harness, "#ptr-cell-0");
    assert_native_focus_id(&harness, "ptr-pin-input");
    assert_eq!(
        harness
            .attr("#ptr-pin-input", "data-pin-input-input-mss")
            .as_deref(),
        Some("0")
    );
    assert_eq!(
        harness
            .attr("#ptr-pin-input", "data-pin-input-input-mse")
            .as_deref(),
        Some("1")
    );
    assert_eq!(harness.attr("#ptr-cell-0", "data-active").as_deref(), Some(""));
    assert_eq!(
        harness.attr("#ptr-cell-3", "data-inactive").as_deref(),
        Some("")
    );

    // Blur and click cell 1 on unfocused PinInput ("1234") -> selects (1, 2), typing "9" replaces index 1 -> "1934".
    harness.base_mut().clear_focus();
    drain_polls_and_resolve(&mut harness);
    click_and_pump(&mut harness, "#ptr-cell-1");
    assert_native_focus_id(&harness, "ptr-pin-input");
    assert_eq!(
        harness
            .attr("#ptr-pin-input", "data-pin-input-input-mss")
            .as_deref(),
        Some("1")
    );
    assert_eq!(
        harness
            .attr("#ptr-pin-input", "data-pin-input-input-mse")
            .as_deref(),
        Some("2")
    );
    assert_eq!(harness.attr("#ptr-cell-1", "data-active").as_deref(), Some(""));

    send_char_and_pump(&mut harness, "9");
    assert_eq!(
        harness.attr("#ptr-probe", "data-value").as_deref(),
        Some("1934")
    );

    // Clicking an already-focused PinInput on cell 0 selects (0, 1) and allows immediate replacement.
    click_and_pump(&mut harness, "#ptr-cell-0");
    assert_eq!(
        harness
            .attr("#ptr-pin-input", "data-pin-input-input-mss")
            .as_deref(),
        Some("0")
    );
    assert_eq!(
        harness
            .attr("#ptr-pin-input", "data-pin-input-input-mse")
            .as_deref(),
        Some("1")
    );
    assert_eq!(harness.attr("#ptr-cell-0", "data-active").as_deref(), Some(""));

    // Clicking empty cell 5 when value is "12" clamps to cell 2 (2, 2) with fake caret.
    click_and_pump(&mut harness, "#ptr2-cell-5");
    assert_native_focus_id(&harness, "ptr2-pin-input");
    assert_eq!(
        harness
            .attr("#ptr2-pin-input", "data-pin-input-input-mss")
            .as_deref(),
        Some("2")
    );
    assert_eq!(
        harness
            .attr("#ptr2-pin-input", "data-pin-input-input-mse")
            .as_deref(),
        Some("2")
    );
    assert_eq!(harness.attr("#ptr2-cell-2", "data-active").as_deref(), Some(""));
    assert_eq!(
        harness.attr("#ptr2-cell-2", "data-fake-caret").as_deref(),
        Some("true")
    );
    assert_eq!(
        harness.attr("#ptr2-cell-5", "data-inactive").as_deref(),
        Some("")
    );
}

fn make_mouse_pointer_event(
    x: f32,
    y: f32,
    button: MouseEventButton,
    buttons: MouseEventButtons,
) -> BlitzPointerEvent {
    BlitzPointerEvent {
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
        button,
        buttons,
        mods: Default::default(),
        details: PointerDetails::default(),
        element: Default::default(),
        active_pointers: Default::default(),
    }
}

fn pump_pointer_step<D: blitz_dom::Document>(harness: &mut blitz_test_harness::Harness<D>) {
    drain_polls_and_resolve(harness);
    oxidase::frame::tick(std::time::Duration::from_millis(16));
    drain_polls_and_resolve(harness);
    harness.tick(0.01);
}

fn drag_across_points<D: blitz_dom::Document>(
    harness: &mut blitz_test_harness::Harness<D>,
    points: &[(f32, f32)],
) {
    assert!(!points.is_empty());
    let (sx, sy) = points[0];
    harness.dispatch(UiEvent::PointerDown(make_mouse_pointer_event(
        sx,
        sy,
        MouseEventButton::Main,
        MouseEventButtons::Primary,
    )));
    pump_pointer_step(harness);
    for &(mx, my) in &points[1..] {
        harness.dispatch(UiEvent::PointerMove(make_mouse_pointer_event(
            mx,
            my,
            MouseEventButton::Main,
            MouseEventButtons::Primary,
        )));
        pump_pointer_step(harness);
    }
    let (ex, ey) = *points.last().unwrap();
    harness.dispatch(UiEvent::PointerUp(make_mouse_pointer_event(
        ex,
        ey,
        MouseEventButton::Main,
        MouseEventButtons::empty(),
    )));
    pump_pointer_step(harness);
}

fn assert_active_cells<D: blitz_dom::Document>(
    harness: &blitz_test_harness::Harness<D>,
    prefix: &str,
    total_cells: usize,
    expected_active: &[usize],
) {
    for idx in 0..total_cells {
        let sel = format!("#{prefix}-cell-{idx}");
        if expected_active.contains(&idx) {
            assert_eq!(
                harness.attr(&sel, "data-active").as_deref(),
                Some(""),
                "expected {sel} to be active"
            );
            assert_eq!(
                harness.attr(&sel, "data-inactive"),
                None,
                "expected {sel} not to have data-inactive"
            );
        } else {
            assert_eq!(
                harness.attr(&sel, "data-active"),
                None,
                "expected {sel} not to have data-active"
            );
            assert_eq!(
                harness.attr(&sel, "data-inactive").as_deref(),
                Some(""),
                "expected {sel} to be inactive"
            );
        }
    }
}

fn assert_input_selection<D: blitz_dom::Document>(
    harness: &blitz_test_harness::Harness<D>,
    input_sel: &str,
    expected_start: &str,
    expected_end: &str,
) {
    assert_eq!(
        harness
            .attr(input_sel, "data-pin-input-input-mss")
            .as_deref(),
        Some(expected_start),
        "unexpected mss for {input_sel}"
    );
    assert_eq!(
        harness
            .attr(input_sel, "data-pin-input-input-mse")
            .as_deref(),
        Some(expected_end),
        "unexpected mse for {input_sel}"
    );
}

#[component]
fn NativePinInputMultiSelectApp() -> Element {
    let mut full_val = use_signal(|| String::from("123456"));
    let mut rtl_val = use_signal(|| String::from("123456"));
    let mut partial_val = use_signal(|| String::from("1234"));

    rsx! {
        div {
            style: "padding: 16px; width: 400px; height: 260px; display: flex; flex-direction: column; gap: 16px;",
            div {
                id: "ms-probe",
                "data-full-value": "{full_val}",
                "data-rtl-value": "{rtl_val}",
                "data-partial-value": "{partial_val}",
            }
            PinInputRoot {
                id: "ms-pin-root",
                input_id: "ms-pin-input",
                default_value: "123456".to_string(),
                max_length: 6,
                pattern: PinInputPattern::Digits,
                style: "display: flex; gap: 8px; width: 280px; height: 40px;",
                on_value_change: move |next: String| full_val.set(next),
                render_children: move |snap: PinInputSnapshot| rsx! {
                    for cell in snap.cells.iter() {
                        PinInputCell {
                            key: "{cell.index}",
                            id: "ms-cell-{cell.index}",
                            style: "width: 36px; height: 36px;",
                            "data-char": cell.char.map(|c| c.to_string()).unwrap_or_default(),
                            "data-fake-caret": if cell.has_fake_caret { "true" } else { "false" },
                            index: cell.index,
                        }
                    }
                }
            }
            PinInputRoot {
                id: "rtl-pin-root",
                input_id: "rtl-pin-input",
                default_value: "123456".to_string(),
                max_length: 6,
                pattern: PinInputPattern::Digits,
                paste_transformer: Callback::new(|s: String| s.replace('-', "")),
                style: "display: flex; gap: 8px; width: 280px; height: 40px;",
                on_value_change: move |next: String| rtl_val.set(next),
                render_children: move |snap: PinInputSnapshot| rsx! {
                    for cell in snap.cells.iter() {
                        PinInputCell {
                            key: "{cell.index}",
                            id: "rtl-cell-{cell.index}",
                            style: "width: 36px; height: 36px;",
                            "data-char": cell.char.map(|c| c.to_string()).unwrap_or_default(),
                            "data-fake-caret": if cell.has_fake_caret { "true" } else { "false" },
                            index: cell.index,
                        }
                    }
                }
            }
            PinInputRoot {
                id: "sa2-pin-root",
                input_id: "sa2-pin-input",
                default_value: "1234".to_string(),
                max_length: 6,
                pattern: PinInputPattern::Digits,
                style: "display: flex; gap: 8px; width: 280px; height: 40px;",
                on_value_change: move |next: String| partial_val.set(next),
                render_children: move |snap: PinInputSnapshot| rsx! {
                    for cell in snap.cells.iter() {
                        PinInputCell {
                            key: "{cell.index}",
                            id: "sa2-cell-{cell.index}",
                            style: "width: 36px; height: 36px;",
                            "data-char": cell.char.map(|c| c.to_string()).unwrap_or_default(),
                            "data-fake-caret": if cell.has_fake_caret { "true" } else { "false" },
                            index: cell.index,
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test_native_pin_input_single_cell_drag_select_all_and_flush_paste() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(NativePinInputMultiSelectApp);

    // 1. Single click on cell 1 of "123456" WITH micro PointerMove (+4px inside cell 1)
    // stays single cell [1, 2).
    let (c1x, c1y) = harness.center_of("#ms-cell-1");
    drag_across_points(&mut harness, &[(c1x, c1y), (c1x + 4.0, c1y)]);
    assert_native_focus_id(&harness, "ms-pin-input");
    assert_input_selection(&harness, "#ms-pin-input", "1", "2");
    assert_active_cells(&harness, "ms", 6, &[1]);

    // 2. Dragging mouse from cell 1 to cell 3 does NOT multi-select — stays single cell [1, 2).
    let (c2x, c2y) = harness.center_of("#ms-cell-2");
    let (c3x, c3y) = harness.center_of("#ms-cell-3");
    drag_across_points(&mut harness, &[(c1x, c1y), (c2x, c2y), (c3x, c3y)]);
    assert_input_selection(&harness, "#ms-pin-input", "1", "2");
    assert_active_cells(&harness, "ms", 6, &[1]);

    // 3. Cmd/Ctrl+A does NOT multi-select — stays single cell [1, 2).
    send_key_and_pump_with(
        &mut harness,
        Key::Character("a".into()),
        Code::KeyA,
        Modifiers::META,
    );
    assert_input_selection(&harness, "#ms-pin-input", "1", "2");
    assert_active_cells(&harness, "ms", 6, &[1]);

    send_key_and_pump_with(
        &mut harness,
        Key::Character("a".into()),
        Code::KeyA,
        Modifiers::CONTROL,
    );
    assert_input_selection(&harness, "#ms-pin-input", "1", "2");
    assert_active_cells(&harness, "ms", 6, &[1]);

    // 5. Cmd/Ctrl+Backspace clears "123456" -> "" with cell 0 active + fake caret (0, 0).
    send_key_and_pump_with(
        &mut harness,
        Key::Named(NamedKey::Backspace),
        Code::Backspace,
        Modifiers::META,
    );
    assert_eq!(
        harness.attr("#ms-probe", "data-full-value").as_deref(),
        Some("")
    );
    assert_input_selection(&harness, "#ms-pin-input", "0", "0");
    assert_active_cells(&harness, "ms", 6, &[0]);
    assert_eq!(
        harness.attr("#ms-cell-0", "data-fake-caret").as_deref(),
        Some("true")
    );

    // 6a. Flush-from-0 paste: from cell 2 on "123456", pasting invalid "abc" leaves "123456" unchanged.
    click_and_pump(&mut harness, "#rtl-cell-2");
    assert_native_focus_id(&harness, "rtl-pin-input");
    assert_input_selection(&harness, "#rtl-pin-input", "2", "3");
    send_paste_and_pump(&mut harness, "abc");
    assert_eq!(
        harness.attr("#ms-probe", "data-rtl-value").as_deref(),
        Some("123456")
    );
    assert_input_selection(&harness, "#rtl-pin-input", "2", "3");
    assert_active_cells(&harness, "rtl", 6, &[2]);

    // 6b. Flush-from-0 paste: from cell 5 on "123456", pasting "654-321" (with hyphen transformer)
    // replaces entire value -> "654321" with cell 5 active [5, 6).
    click_and_pump(&mut harness, "#rtl-cell-5");
    assert_input_selection(&harness, "#rtl-pin-input", "5", "6");
    send_paste_and_pump(&mut harness, "654-321");
    assert_eq!(
        harness.attr("#ms-probe", "data-rtl-value").as_deref(),
        Some("654321")
    );
    assert_input_selection(&harness, "#rtl-pin-input", "5", "6");
    assert_active_cells(&harness, "rtl", 6, &[5]);

    // 6c. Flush-from-0 paste: from cell 2 on "654321", pasting "89" replaces entire value -> "89"
    // with insert caret at cell 2 (2, 2).
    click_and_pump(&mut harness, "#rtl-cell-2");
    assert_input_selection(&harness, "#rtl-pin-input", "2", "3");
    send_paste_and_pump(&mut harness, "89");
    assert_eq!(
        harness.attr("#ms-probe", "data-rtl-value").as_deref(),
        Some("89")
    );
    assert_input_selection(&harness, "#rtl-pin-input", "2", "2");
    assert_active_cells(&harness, "rtl", 6, &[2]);
    assert_eq!(
        harness.attr("#rtl-cell-2", "data-fake-caret").as_deref(),
        Some("true")
    );
}

#[test]
fn test_native_pin_input_shift_arrow_filled_and_empty_cells() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(NativePinInputMultiSelectApp);

    // 4a. Shift+ArrowLeft / Shift+ArrowRight moves single cell (no multi-cell range) on filled cells.
    click_and_pump(&mut harness, "#ms-cell-3");
    assert_native_focus_id(&harness, "ms-pin-input");
    assert_input_selection(&harness, "#ms-pin-input", "3", "4");
    assert_active_cells(&harness, "ms", 6, &[3]);

    send_key_and_pump_with(
        &mut harness,
        Key::Named(NamedKey::ArrowLeft),
        Code::ArrowLeft,
        Modifiers::SHIFT,
    );
    assert_input_selection(&harness, "#ms-pin-input", "2", "3");
    assert_active_cells(&harness, "ms", 6, &[2]);

    send_key_and_pump_with(
        &mut harness,
        Key::Named(NamedKey::ArrowLeft),
        Code::ArrowLeft,
        Modifiers::SHIFT,
    );
    assert_input_selection(&harness, "#ms-pin-input", "1", "2");
    assert_active_cells(&harness, "ms", 6, &[1]);

    send_key_and_pump_with(
        &mut harness,
        Key::Named(NamedKey::ArrowRight),
        Code::ArrowRight,
        Modifiers::SHIFT,
    );
    assert_input_selection(&harness, "#ms-pin-input", "2", "3");
    assert_active_cells(&harness, "ms", 6, &[2]);

    send_key_and_pump_with(
        &mut harness,
        Key::Named(NamedKey::ArrowRight),
        Code::ArrowRight,
        Modifiers::SHIFT,
    );
    assert_input_selection(&harness, "#ms-pin-input", "3", "4");
    assert_active_cells(&harness, "ms", 6, &[3]);

    // 4b. Shift+ArrowLeft / Shift+ArrowRight moves single cell on empty insert caret ("1234" -> (4, 4)).
    click_and_pump(&mut harness, "#sa2-cell-4");
    assert_native_focus_id(&harness, "sa2-pin-input");
    assert_input_selection(&harness, "#sa2-pin-input", "4", "4");
    assert_active_cells(&harness, "sa2", 6, &[4]);
    assert_eq!(
        harness.attr("#sa2-cell-4", "data-fake-caret").as_deref(),
        Some("true")
    );

    send_key_and_pump_with(
        &mut harness,
        Key::Named(NamedKey::ArrowLeft),
        Code::ArrowLeft,
        Modifiers::SHIFT,
    );
    assert_input_selection(&harness, "#sa2-pin-input", "3", "4");
    assert_active_cells(&harness, "sa2", 6, &[3]);

    send_key_and_pump_with(
        &mut harness,
        Key::Named(NamedKey::ArrowLeft),
        Code::ArrowLeft,
        Modifiers::SHIFT,
    );
    assert_input_selection(&harness, "#sa2-pin-input", "2", "3");
    assert_active_cells(&harness, "sa2", 6, &[2]);

    send_key_and_pump_with(
        &mut harness,
        Key::Named(NamedKey::ArrowRight),
        Code::ArrowRight,
        Modifiers::SHIFT,
    );
    assert_input_selection(&harness, "#sa2-pin-input", "3", "4");
    assert_active_cells(&harness, "sa2", 6, &[3]);

    send_key_and_pump_with(
        &mut harness,
        Key::Named(NamedKey::ArrowRight),
        Code::ArrowRight,
        Modifiers::SHIFT,
    );
    assert_input_selection(&harness, "#sa2-pin-input", "4", "4");
    assert_active_cells(&harness, "sa2", 6, &[4]);
    assert_eq!(
        harness.attr("#sa2-cell-4", "data-fake-caret").as_deref(),
        Some("true")
    );
}
