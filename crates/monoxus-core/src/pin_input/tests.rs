use crate::foundation::{compose::Slottable, shared::ScopeHandle};

use super::{
    attrs::{
        PinInputCellAttributes, PinInputRootAttributes, project_pin_input_cell_as_child,
        project_pin_input_root_as_child,
    },
    relationships::PinInputRelationships,
    state::{
        PinEditSegment, PinInputDefinition, PinShiftNavKey, apply_pin_splice, char_index_to_utf16,
        collapse_pin_selection_to_single_cell, compute_pin_cells, extend_pin_selection_with_shift,
        extract_pin_edit, filter_pin_chars_with, format_pin_cell_display, initial_focus_selection,
        normalize_pin_selection, project_pin_cell_char, replace_pin_value_from_paste_with,
        resolve_pin_cell_hit, resolve_pin_drag_selection, resolve_pin_pointer_selection,
        sanitize_pin_value_with, should_fire_on_complete, should_prevent_keydown_with,
        step_pin_single_cell, target_selection_for_cell_click, utf16_to_char_index,
    },
    types::{
        PIN_INPUT_PARTS, PinInputAutoComplete, PinInputBuiltinPattern, PinInputCellData,
        PinInputMode, PinInputPart, PinInputSelection, PinInputSelectionDirection,
        PinInputTextAlign,
    },
};

#[test]
fn parts_and_relationships_are_deterministic() {
    assert_eq!(
        PIN_INPUT_PARTS
            .iter()
            .map(PinInputPart::as_str)
            .collect::<Vec<_>>(),
        ["root", "input", "cell"]
    );
    let scope = ScopeHandle::root("pin-input").child("otp");
    let rels = PinInputRelationships::new(scope.clone());
    assert_eq!(rels.root_id(), scope.token());
    assert_eq!(rels.input_id(), scope.qualify("input"));
    assert_eq!(rels.hidden_input_id(), scope.qualify("hidden-input"));
    assert_eq!(rels.cell_id(0), scope.qualify("cell-0"));
    assert_eq!(rels.cell_id(5), scope.qualify("cell-5"));
}

#[test]
fn builtin_patterns_and_filtering() {
    assert!(PinInputBuiltinPattern::Any.matches_str("1a$!"));
    assert!(PinInputBuiltinPattern::Digits.matches_str("0123456789"));
    assert!(!PinInputBuiltinPattern::Digits.matches_str("12a"));
    assert!(PinInputBuiltinPattern::Alpha.matches_str("abcXYZ"));
    assert!(!PinInputBuiltinPattern::Alpha.matches_str("ab1"));
    assert!(PinInputBuiltinPattern::Alphanumeric.matches_str("a1B2"));
    assert!(!PinInputBuiltinPattern::Alphanumeric.matches_str("a1-2"));

    assert_eq!(
        filter_pin_chars_with("1a2$3b4567", 6, |s| PinInputBuiltinPattern::Digits.matches_str(s)),
        "123456"
    );
    assert_eq!(
        filter_pin_chars_with("1a2b3C!", 6, |s| PinInputBuiltinPattern::Alpha.matches_str(s)),
        "abC"
    );
    assert_eq!(
        filter_pin_chars_with("1a-2$Z", 4, |s| s.chars().all(|c| c.is_ascii_hexdigit())),
        "1a2"
    );
    assert_eq!(
        sanitize_pin_value_with("12a34b567", 6, |s| PinInputBuiltinPattern::Digits.matches_str(s)),
        "123456"
    );
}

#[test]
fn keydown_filtering_ignores_shortcuts_composing_and_named_keys() {
    let digits = |s: &str| PinInputBuiltinPattern::Digits.matches_str(s);
    assert!(!should_prevent_keydown_with("1", false, false, digits));
    assert!(should_prevent_keydown_with("a", false, false, digits));
    assert!(should_prevent_keydown_with(" ", false, false, digits));
    assert!(!should_prevent_keydown_with("v", true, false, digits));
    assert!(!should_prevent_keydown_with("a", false, true, digits));
    assert!(!should_prevent_keydown_with("Backspace", false, false, digits));
    assert!(!should_prevent_keydown_with("ArrowLeft", false, false, digits));
}

#[test]
fn splice_handles_collapsed_range_paste_transformer_and_truncation() {
    let strip_dashes = |s: &str| s.replace('-', "");
    let digits = |s: &str| PinInputBuiltinPattern::Digits.matches_str(s);

    // Paste into empty input with hyphen transformer -> full fill selects last cell
    let res = apply_pin_splice("", Some(0), Some(0), "123-456", 6, strip_dashes, digits);
    assert_eq!(res.value, "123456");
    assert_eq!(res.selection_start, 5);
    assert_eq!(res.selection_end, 6);
    assert!(res.changed);

    // Non-collapsed range replacement ("23" -> "9") in "1234" -> "194"
    let mid = apply_pin_splice("1234", Some(1), Some(3), "9", 6, |s| s.to_string(), digits);
    assert_eq!(mid.value, "194");
    assert_eq!(mid.selection_start, 2);
    assert_eq!(mid.selection_end, 2);

    // Overflow paste truncates to max_length
    let overflow = apply_pin_splice("12", Some(2), Some(2), "345678", 6, |s| s.to_string(), digits);
    assert_eq!(overflow.value, "123456");
    assert_eq!(overflow.selection_start, 5);
    assert_eq!(overflow.selection_end, 6);

    // Completely invalid paste preserves existing selection and value
    let invalid = apply_pin_splice("1234", Some(1), Some(3), "abc", 6, |s| s.to_string(), digits);
    assert_eq!(invalid.value, "1234");
    assert!(!invalid.changed);
}

#[test]
fn edit_segment_extraction_recovers_pasted_middle_for_native_oninput() {
    assert_eq!(
        extract_pin_edit("", "123-456"),
        PinEditSegment {
            prefix_chars: 0,
            prev_end_chars: 0,
            inserted: "123-456".to_string(),
        }
    );
    assert_eq!(
        extract_pin_edit("12", "1234-56"),
        PinEditSegment {
            prefix_chars: 2,
            prev_end_chars: 2,
            inserted: "34-56".to_string(),
        }
    );
    assert_eq!(
        extract_pin_edit("1234", "198-74"),
        PinEditSegment {
            prefix_chars: 1,
            prev_end_chars: 3,
            inserted: "98-7".to_string(),
        }
    );
}

#[test]
fn selection_normalization_and_cell_projection_matrix() {
    // 1. Empty focus -> cell 0 active + has_fake_caret
    let empty_sel = normalize_pin_selection("", Some(0), Some(0), PinInputSelection::empty(), 6);
    let empty_cells = compute_pin_cells("", 6, empty_sel, true, false);
    assert_eq!(
        empty_cells[0],
        PinInputCellData {
            index: 0,
            char: None,
            is_active: true,
            has_fake_caret: true,
        }
    );
    assert!(!empty_cells[1].is_active);
    assert!(!empty_cells[1].has_fake_caret);

    // 2. Partial fill "12" -> cell 2 active + has_fake_caret
    let partial_sel = normalize_pin_selection("12", Some(2), Some(2), PinInputSelection::collapsed(1), 6);
    let partial_cells = compute_pin_cells("12", 6, partial_sel, true, false);
    assert_eq!(partial_cells[0].char, Some('1'));
    assert!(!partial_cells[0].is_active);
    assert_eq!(partial_cells[1].char, Some('2'));
    assert!(!partial_cells[1].is_active);
    assert_eq!(
        partial_cells[2],
        PinInputCellData {
            index: 2,
            char: None,
            is_active: true,
            has_fake_caret: true,
        }
    );

    // 3. Full fill "123456" -> cell 5 active, has_fake_caret: false
    let full_sel = normalize_pin_selection(
        "123456",
        Some(6),
        Some(6),
        PinInputSelection::collapsed(5),
        6,
    );
    assert_eq!(full_sel.start, Some(5));
    assert_eq!(full_sel.end, Some(6));
    let full_cells = compute_pin_cells("123456", 6, full_sel, true, false);
    assert!(!full_cells[4].is_active);
    assert_eq!(
        full_cells[5],
        PinInputCellData {
            index: 5,
            char: Some('6'),
            is_active: true,
            has_fake_caret: false,
        }
    );

    // Overwriting full last cell keeps [5, 6) active; Backspace drops to collapsed (5, 5) insert mode;
    // mid-string replacement on full input ("129456" at caret 3) advances to [3, 4).
    let overwrite_full_sel = normalize_pin_selection("123457", Some(6), Some(6), full_sel, 6);
    assert_eq!(
        (
            overwrite_full_sel.start,
            overwrite_full_sel.end,
            overwrite_full_sel.direction
        ),
        (Some(5), Some(6), PinInputSelectionDirection::Backward)
    );
    let backspace_full_sel =
        normalize_pin_selection("12345", Some(5), Some(5), PinInputSelection::empty(), 6);
    assert_eq!(backspace_full_sel, PinInputSelection::collapsed(5));
    let backspace_cells = compute_pin_cells("12345", 6, backspace_full_sel, true, false);
    assert!(backspace_cells[5].is_active);
    assert!(backspace_cells[5].has_fake_caret);

    let mid_full_advance =
        normalize_pin_selection("129456", Some(3), Some(3), PinInputSelection::empty(), 6);
    assert_eq!(
        (
            mid_full_advance.start,
            mid_full_advance.end,
            mid_full_advance.direction
        ),
        (Some(3), Some(4), PinInputSelectionDirection::Forward)
    );

    // 4. Mid-string ArrowLeft navigation inside "1234"
    let step1 = normalize_pin_selection("1234", Some(3), Some(3), PinInputSelection::collapsed(4), 6);
    assert_eq!((step1.start, step1.end), (Some(3), Some(4)));
    assert_eq!(step1.direction, PinInputSelectionDirection::Backward);
    let step2 = normalize_pin_selection("1234", Some(3), Some(3), step1, 6);
    assert_eq!((step2.start, step2.end), (Some(2), Some(3)));
    assert_eq!(step2.direction, PinInputSelectionDirection::Backward);
    let mid_cells = compute_pin_cells("1234", 6, step2, true, false);
    assert!(mid_cells[2].is_active);
    assert!(!mid_cells[2].has_fake_caret);
    assert!(!mid_cells[3].is_active);

    // 5. Range selection 1..4 -> cells 1, 2, 3 active
    let range_sel = normalize_pin_selection("1234", Some(1), Some(4), step2, 6);
    let range_cells = compute_pin_cells("1234", 6, range_sel, true, false);
    assert!(!range_cells[0].is_active);
    assert!(range_cells[1].is_active);
    assert!(range_cells[2].is_active);
    assert!(range_cells[3].is_active);
    assert!(!range_cells[4].is_active);

    // 6. Blurred -> all inactive
    let blurred_cells = compute_pin_cells("12", 6, partial_sel, false, false);
    assert!(blurred_cells.iter().all(|c| !c.is_active && !c.has_fake_caret));

    // 7. Disabled -> all inactive
    let disabled_cells = compute_pin_cells("12", 6, partial_sel, true, true);
    assert!(disabled_cells.iter().all(|c| !c.is_active && !c.has_fake_caret));

    // 8. Focus selection helper
    let init_full = initial_focus_selection("123456", 6);
    assert_eq!((init_full.start, init_full.end), (Some(5), Some(6)));
    let init_partial = initial_focus_selection("12", 6);
    assert_eq!((init_partial.start, init_partial.end), (Some(2), Some(2)));
}

#[test]
fn on_complete_edge_detection_and_mask_projection() {
    assert!(!should_fire_on_complete("", "12345", 6));
    assert!(should_fire_on_complete("12345", "123456", 6));
    assert!(!should_fire_on_complete("123456", "123456", 6));
    assert!(!should_fire_on_complete("123456", "123459", 6));

    assert_eq!(project_pin_cell_char(Some('4'), false, '•'), Some('4'));
    assert_eq!(project_pin_cell_char(Some('4'), true, '•'), Some('•'));
    assert_eq!(project_pin_cell_char(None, true, '•'), None);
    assert_eq!(format_pin_cell_display(Some('9'), true, '*'), "*");
    assert_eq!(format_pin_cell_display(None, true, '*'), "");
}

#[test]
fn utf16_offset_helpers_round_trip() {
    let s = "a😀b";
    assert_eq!(char_index_to_utf16(s, 0), 0);
    assert_eq!(char_index_to_utf16(s, 1), 1);
    assert_eq!(char_index_to_utf16(s, 2), 3);
    assert_eq!(char_index_to_utf16(s, 3), 4);

    assert_eq!(utf16_to_char_index(s, 0), 0);
    assert_eq!(utf16_to_char_index(s, 1), 1);
    assert_eq!(utf16_to_char_index(s, 3), 2);
    assert_eq!(utf16_to_char_index(s, 4), 3);
}

#[test]
fn attributes_and_as_child_projection_follow_omission_contract() {
    let scope = ScopeHandle::root("pin-input").child("attrs");
    let def = PinInputDefinition::new(scope)
        .with_max_length(4)
        .with_builtin_pattern(PinInputBuiltinPattern::Digits)
        .with_text_align(PinInputTextAlign::Center)
        .with_input_mode(PinInputMode::Tel)
        .with_autocomplete(PinInputAutoComplete::Off)
        .with_mask(true)
        .with_disabled(false);

    let root = def.root_attributes(None);
    assert_eq!(root.aria_disabled(), None);
    assert_eq!(root.data_disabled(), None);
    assert_eq!(root.data_monoxus_pin_input_root(), "");
    assert!(root.style().contains("position: relative;"));

    let input = def.input_attributes(
        None,
        "12",
        PinInputSelection::collapsed(2),
        def.builtin_pattern().html_pattern(),
    );
    assert_eq!(input.input_type(), "password");
    assert_eq!(input.autocomplete(), "off");
    assert_eq!(input.inputmode(), "tel");
    assert_eq!(input.pattern(), Some("[0-9]*"));
    assert_eq!(input.maxlength(), 4);
    assert_eq!(input.value(), "12");
    assert!(input.style().contains("opacity: 0;"));
    assert!(input.style().contains("text-align: center;"));

    let active_cell = def.cell_attributes(2, None, true);
    assert_eq!(active_cell.data_active(), Some(""));
    assert_eq!(active_cell.data_inactive(), None);
    assert_eq!(active_cell.data_disabled(), None);

    let inactive_cell = def.cell_attributes(0, None, false);
    assert_eq!(inactive_cell.data_active(), None);
    assert_eq!(inactive_cell.data_inactive(), Some(""));
    assert_eq!(inactive_cell.data_disabled(), None);

    let disabled_def = def.with_disabled(true);
    let disabled_root = disabled_def.root_attributes(None);
    assert_eq!(disabled_root.aria_disabled(), Some("true"));
    assert_eq!(disabled_root.data_disabled(), Some(""));

    let disabled_cell = disabled_def.cell_attributes(0, None, true);
    assert_eq!(disabled_cell.data_active(), None);
    assert_eq!(disabled_cell.data_inactive(), Some(""));
    assert_eq!(disabled_cell.data_disabled(), Some(""));

    let (_, proj_root, _) =
        project_pin_input_root_as_child(&root, "slot-root", Slottable::new("body"));
    assert_eq!(proj_root, root);
    let (_, proj_cell, _) =
        project_pin_input_cell_as_child(&active_cell, "slot-cell", Slottable::new("cell"));
    assert_eq!(proj_cell, active_cell);
    let _: PinInputRootAttributes = proj_root;
    let _: PinInputCellAttributes = proj_cell;
}

#[test]
fn pointer_hit_resolution_and_target_selection_matrix() {
    // 6 cells of width 44, height 52, gap 8 starting at x=10, y=20.
    let cell_rects: Vec<(usize, (f64, f64, f64, f64))> = (0..6)
        .map(|i| {
            let left = 10.0 + (i as f64) * 52.0;
            (i, (left, 20.0, left + 44.0, 72.0))
        })
        .collect();

    assert_eq!(resolve_pin_cell_hit(&[], 30.0, 45.0), None);

    // Exact cell hits
    assert_eq!(resolve_pin_cell_hit(&cell_rects, 10.0, 20.0), Some(0));
    assert_eq!(resolve_pin_cell_hit(&cell_rects, 32.0, 46.0), Some(0));
    assert_eq!(resolve_pin_cell_hit(&cell_rects, 84.0, 46.0), Some(1));
    assert_eq!(resolve_pin_cell_hit(&cell_rects, 292.0, 46.0), Some(5));

    // Gap hits: cell 0 right=54.0, cell 1 left=62.0 (midpoint 58.0)
    assert_eq!(resolve_pin_cell_hit(&cell_rects, 55.0, 46.0), Some(0));
    assert_eq!(resolve_pin_cell_hit(&cell_rects, 61.0, 46.0), Some(1));

    // Outside horizontal bounds or slightly above/below row
    assert_eq!(resolve_pin_cell_hit(&cell_rects, 2.0, 46.0), Some(0));
    assert_eq!(resolve_pin_cell_hit(&cell_rects, 340.0, 46.0), Some(5));
    assert_eq!(resolve_pin_cell_hit(&cell_rects, 136.0, 14.0), Some(2));

    // Target selection: empty value -> always collapsed(0)
    for idx in 0..6 {
        assert_eq!(
            target_selection_for_cell_click(idx, 0, 6),
            PinInputSelection::collapsed(0)
        );
    }

    // Partial fill "1234" (len 4, max 6):
    // Clicking filled cells 0, 1, 3 selects [idx, idx + 1) Forward
    assert_eq!(
        target_selection_for_cell_click(0, 4, 6),
        PinInputSelection::range(0, 1, PinInputSelectionDirection::Forward)
    );
    assert_eq!(
        target_selection_for_cell_click(1, 4, 6),
        PinInputSelection::range(1, 2, PinInputSelectionDirection::Forward)
    );
    assert_eq!(
        target_selection_for_cell_click(3, 4, 6),
        PinInputSelection::range(3, 4, PinInputSelectionDirection::Forward)
    );
    // Clicking empty cell 4 or beyond (5) clamps to insert caret at 4
    assert_eq!(
        target_selection_for_cell_click(4, 4, 6),
        PinInputSelection::collapsed(4)
    );
    assert_eq!(
        target_selection_for_cell_click(5, 4, 6),
        PinInputSelection::collapsed(4)
    );

    // Full fill (len 6, max 6): clicking any cell 0..=5 selects [idx, idx + 1) Forward
    for idx in 0..6 {
        assert_eq!(
            target_selection_for_cell_click(idx, 6, 6),
            PinInputSelection::range(idx, idx + 1, PinInputSelectionDirection::Forward)
        );
    }

    // Composite pointer selection helper
    assert_eq!(
        resolve_pin_pointer_selection(&cell_rects, 84.0, 46.0, "1234", 6),
        Some(PinInputSelection::range(
            1,
            2,
            PinInputSelectionDirection::Forward
        ))
    );
    assert_eq!(
        resolve_pin_pointer_selection(&cell_rects, 292.0, 46.0, "12", 6),
        Some(PinInputSelection::collapsed(2))
    );
    assert_eq!(
        resolve_pin_pointer_selection(&cell_rects, 32.0, 46.0, "", 6),
        Some(PinInputSelection::collapsed(0))
    );
}

#[test]
fn pointer_drag_selection_covers_ltr_rtl_same_cell_and_empty_cells() {
    // Empty value always collapses to 0
    assert_eq!(
        resolve_pin_drag_selection(0, 3, "", 6),
        PinInputSelection::collapsed(0)
    );
    assert_eq!(
        resolve_pin_drag_selection(2, 2, "", 6),
        PinInputSelection::collapsed(0)
    );

    // Same cell delegates to single-cell click target
    assert_eq!(
        resolve_pin_drag_selection(1, 1, "123456", 6),
        PinInputSelection::range(1, 2, PinInputSelectionDirection::Forward)
    );
    assert_eq!(
        resolve_pin_drag_selection(5, 5, "1234", 6),
        PinInputSelection::collapsed(4)
    );

    // Dragging exclusively across empty trailing cells collapses to val_len
    assert_eq!(
        resolve_pin_drag_selection(4, 5, "1234", 6),
        PinInputSelection::collapsed(4)
    );
    assert_eq!(
        resolve_pin_drag_selection(5, 4, "1234", 6),
        PinInputSelection::collapsed(4)
    );

    // L -> R drag across filled cells
    assert_eq!(
        resolve_pin_drag_selection(1, 3, "123456", 6),
        PinInputSelection::range(1, 4, PinInputSelectionDirection::Forward)
    );
    // R -> L drag across filled cells
    assert_eq!(
        resolve_pin_drag_selection(4, 2, "123456", 6),
        PinInputSelection::range(2, 5, PinInputSelectionDirection::Backward)
    );

    // Dragging between filled cell and empty cell clamps hi to val_len
    assert_eq!(
        resolve_pin_drag_selection(2, 5, "1234", 6),
        PinInputSelection::range(2, 4, PinInputSelectionDirection::Forward)
    );
    assert_eq!(
        resolve_pin_drag_selection(5, 2, "1234", 6),
        PinInputSelection::range(2, 4, PinInputSelectionDirection::Backward)
    );
}

#[test]
fn shift_navigation_selection_extension_and_pivot_crossing() {
    // Empty value stays at collapsed(0)
    assert_eq!(
        extend_pin_selection_with_shift(
            "",
            6,
            PinInputSelection::collapsed(0),
            None,
            PinShiftNavKey::ArrowLeft
        ),
        (PinInputSelection::collapsed(0), None)
    );

    // 1. Filled cell 3 ([3, 4)) on "123456": Shift+ArrowLeft -> [2,4) -> [1,4) -> [0,4),
    // then Shift+ArrowRight shrinks back and crosses pivot to [3, 5).
    let s0 = PinInputSelection::range(3, 4, PinInputSelectionDirection::Forward);
    let (s1, a1) =
        extend_pin_selection_with_shift("123456", 6, s0, None, PinShiftNavKey::ArrowLeft);
    assert_eq!(
        s1,
        PinInputSelection::range(2, 4, PinInputSelectionDirection::Backward)
    );
    assert_eq!(a1, Some((4, 2)));

    let (s2, a2) =
        extend_pin_selection_with_shift("123456", 6, s1, a1, PinShiftNavKey::ArrowLeft);
    assert_eq!(
        s2,
        PinInputSelection::range(1, 4, PinInputSelectionDirection::Backward)
    );
    assert_eq!(a2, Some((4, 1)));

    let (s3, a3) =
        extend_pin_selection_with_shift("123456", 6, s2, a2, PinShiftNavKey::ArrowLeft);
    assert_eq!(
        s3,
        PinInputSelection::range(0, 4, PinInputSelectionDirection::Backward)
    );
    assert_eq!(a3, Some((4, 0)));

    // Clamps at start
    let (s3_clamp, a3_clamp) =
        extend_pin_selection_with_shift("123456", 6, s3, a3, PinShiftNavKey::ArrowLeft);
    assert_eq!(s3_clamp, s3);
    assert_eq!(a3_clamp, Some((4, 0)));

    // Shrink back via Shift+ArrowRight: [0,4) -> [1,4) -> [2,4) -> [3,4) -> cross pivot [3,5)
    let (r1, ra1) =
        extend_pin_selection_with_shift("123456", 6, s3, a3, PinShiftNavKey::ArrowRight);
    assert_eq!(
        r1,
        PinInputSelection::range(1, 4, PinInputSelectionDirection::Backward)
    );
    assert_eq!(ra1, Some((4, 1)));

    let (r2, ra2) =
        extend_pin_selection_with_shift("123456", 6, r1, ra1, PinShiftNavKey::ArrowRight);
    assert_eq!(
        r2,
        PinInputSelection::range(2, 4, PinInputSelectionDirection::Backward)
    );
    assert_eq!(ra2, Some((4, 2)));

    let (r3, ra3) =
        extend_pin_selection_with_shift("123456", 6, r2, ra2, PinShiftNavKey::ArrowRight);
    assert_eq!(
        r3,
        PinInputSelection::range(3, 4, PinInputSelectionDirection::Backward)
    );
    assert_eq!(ra3, Some((4, 3)));

    let (r4, ra4) =
        extend_pin_selection_with_shift("123456", 6, r3, ra3, PinShiftNavKey::ArrowRight);
    assert_eq!(
        r4,
        PinInputSelection::range(3, 5, PinInputSelectionDirection::Forward)
    );
    assert_eq!(ra4, Some((3, 5)));

    // Cross pivot back left: [3,5) -> [3,4) -> [2,4)
    let (b1, ba1) =
        extend_pin_selection_with_shift("123456", 6, r4, ra4, PinShiftNavKey::ArrowLeft);
    assert_eq!(
        b1,
        PinInputSelection::range(3, 4, PinInputSelectionDirection::Forward)
    );
    assert_eq!(ba1, Some((3, 4)));

    let (b2, ba2) =
        extend_pin_selection_with_shift("123456", 6, b1, ba1, PinShiftNavKey::ArrowLeft);
    assert_eq!(
        b2,
        PinInputSelection::range(2, 4, PinInputSelectionDirection::Backward)
    );
    assert_eq!(ba2, Some((4, 2)));

    // 2. Empty insert caret (4, 4) on "1234" (max_length 6):
    // Shift+ArrowRight / End stay at (4, 4)
    assert_eq!(
        extend_pin_selection_with_shift(
            "1234",
            6,
            PinInputSelection::collapsed(4),
            None,
            PinShiftNavKey::ArrowRight
        ),
        (PinInputSelection::collapsed(4), None)
    );
    assert_eq!(
        extend_pin_selection_with_shift(
            "1234",
            6,
            PinInputSelection::collapsed(4),
            None,
            PinShiftNavKey::End
        ),
        (PinInputSelection::collapsed(4), None)
    );

    // Shift+ArrowLeft -> [3, 4) -> [2, 4), then Shift+ArrowRight -> [3, 4) -> (4, 4)
    let (e1, ea1) = extend_pin_selection_with_shift(
        "1234",
        6,
        PinInputSelection::collapsed(4),
        None,
        PinShiftNavKey::ArrowLeft,
    );
    assert_eq!(
        e1,
        PinInputSelection::range(3, 4, PinInputSelectionDirection::Backward)
    );
    assert_eq!(ea1, Some((4, 3)));

    let (e2, ea2) =
        extend_pin_selection_with_shift("1234", 6, e1, ea1, PinShiftNavKey::ArrowLeft);
    assert_eq!(
        e2,
        PinInputSelection::range(2, 4, PinInputSelectionDirection::Backward)
    );
    assert_eq!(ea2, Some((4, 2)));

    let (e3, ea3) =
        extend_pin_selection_with_shift("1234", 6, e2, ea2, PinShiftNavKey::ArrowRight);
    assert_eq!(
        e3,
        PinInputSelection::range(3, 4, PinInputSelectionDirection::Backward)
    );
    assert_eq!(ea3, Some((4, 3)));

    let (e4, ea4) =
        extend_pin_selection_with_shift("1234", 6, e3, ea3, PinShiftNavKey::ArrowRight);
    assert_eq!(e4, PinInputSelection::collapsed(4));
    assert_eq!(ea4, None);

    // 3. Shift+Home and Shift+End from empty caret and filled cell
    let (eh, eha) = extend_pin_selection_with_shift(
        "1234",
        6,
        PinInputSelection::collapsed(4),
        None,
        PinShiftNavKey::Home,
    );
    assert_eq!(
        eh,
        PinInputSelection::range(0, 4, PinInputSelectionDirection::Backward)
    );
    assert_eq!(eha, Some((4, 0)));

    // Shift+End from [0, 4) anchored at 4 returns to collapsed (4, 4)
    assert_eq!(
        extend_pin_selection_with_shift("1234", 6, eh, eha, PinShiftNavKey::End),
        (PinInputSelection::collapsed(4), None)
    );

    let cell2 = PinInputSelection::range(2, 3, PinInputSelectionDirection::Forward);
    let (fh, fha) =
        extend_pin_selection_with_shift("123456", 6, cell2, None, PinShiftNavKey::Home);
    assert_eq!(
        fh,
        PinInputSelection::range(0, 3, PinInputSelectionDirection::Backward)
    );
    assert_eq!(fha, Some((3, 0)));

    let (fe, fea) =
        extend_pin_selection_with_shift("123456", 6, cell2, None, PinShiftNavKey::End);
    assert_eq!(
        fe,
        PinInputSelection::range(2, 6, PinInputSelectionDirection::Forward)
    );
    assert_eq!(fea, Some((2, 6)));

    // 4. Continuing from a drag-selected range without shift_anchor
    let drag_fwd = PinInputSelection::range(1, 4, PinInputSelectionDirection::Forward);
    assert_eq!(
        extend_pin_selection_with_shift("123456", 6, drag_fwd, None, PinShiftNavKey::ArrowRight),
        (
            PinInputSelection::range(1, 5, PinInputSelectionDirection::Forward),
            Some((1, 5))
        )
    );
    let drag_bwd = PinInputSelection::range(1, 4, PinInputSelectionDirection::Backward);
    assert_eq!(
        extend_pin_selection_with_shift("123456", 6, drag_bwd, None, PinShiftNavKey::ArrowLeft),
        (
            PinInputSelection::range(0, 4, PinInputSelectionDirection::Backward),
            Some((4, 0))
        )
    );
}

#[test]
fn single_cell_collapse_step_and_flush_paste_helpers() {
    // 1. collapse_pin_selection_to_single_cell
    assert_eq!(
        collapse_pin_selection_to_single_cell("1234", PinInputSelection::empty(), 6),
        PinInputSelection::empty()
    );
    assert_eq!(
        collapse_pin_selection_to_single_cell("", PinInputSelection::range(0, 3, PinInputSelectionDirection::Forward), 6),
        PinInputSelection::collapsed(0)
    );
    assert_eq!(
        collapse_pin_selection_to_single_cell("12", PinInputSelection::collapsed(2), 6),
        PinInputSelection::collapsed(2)
    );
    assert_eq!(
        collapse_pin_selection_to_single_cell("123456", PinInputSelection::collapsed(6), 6),
        PinInputSelection::range(5, 6, PinInputSelectionDirection::Backward)
    );
    let single = PinInputSelection::range(2, 3, PinInputSelectionDirection::Backward);
    assert_eq!(
        collapse_pin_selection_to_single_cell("123456", single, 6),
        single
    );
    assert_eq!(
        collapse_pin_selection_to_single_cell(
            "123456",
            PinInputSelection::range(1, 4, PinInputSelectionDirection::Forward),
            6
        ),
        PinInputSelection::range(1, 2, PinInputSelectionDirection::Forward)
    );
    assert_eq!(
        collapse_pin_selection_to_single_cell(
            "123456",
            PinInputSelection::range(2, 5, PinInputSelectionDirection::Backward),
            6
        ),
        PinInputSelection::range(2, 3, PinInputSelectionDirection::Forward)
    );

    // 2. step_pin_single_cell
    assert_eq!(
        step_pin_single_cell("", 6, PinInputSelection::collapsed(0), PinShiftNavKey::ArrowRight),
        PinInputSelection::collapsed(0)
    );
    // Partial "1234" (max 6)
    let c3 = PinInputSelection::range(3, 4, PinInputSelectionDirection::Forward);
    assert_eq!(
        step_pin_single_cell("1234", 6, c3, PinShiftNavKey::ArrowRight),
        PinInputSelection::collapsed(4)
    );
    assert_eq!(
        step_pin_single_cell("1234", 6, PinInputSelection::collapsed(4), PinShiftNavKey::ArrowRight),
        PinInputSelection::collapsed(4)
    );
    assert_eq!(
        step_pin_single_cell("1234", 6, PinInputSelection::collapsed(4), PinShiftNavKey::ArrowLeft),
        PinInputSelection::range(3, 4, PinInputSelectionDirection::Backward)
    );
    assert_eq!(
        step_pin_single_cell(
            "1234",
            6,
            PinInputSelection::range(0, 1, PinInputSelectionDirection::Backward),
            PinShiftNavKey::ArrowLeft
        ),
        PinInputSelection::range(0, 1, PinInputSelectionDirection::Forward)
    );
    assert_eq!(
        step_pin_single_cell("1234", 6, c3, PinShiftNavKey::Home),
        PinInputSelection::range(0, 1, PinInputSelectionDirection::Forward)
    );
    assert_eq!(
        step_pin_single_cell("1234", 6, PinInputSelection::range(0, 1, PinInputSelectionDirection::Forward), PinShiftNavKey::End),
        PinInputSelection::collapsed(4)
    );
    // Full "123456" (max 6)
    assert_eq!(
        step_pin_single_cell(
            "123456",
            6,
            PinInputSelection::range(4, 5, PinInputSelectionDirection::Forward),
            PinShiftNavKey::ArrowRight
        ),
        PinInputSelection::range(5, 6, PinInputSelectionDirection::Forward)
    );
    assert_eq!(
        step_pin_single_cell(
            "123456",
            6,
            PinInputSelection::range(5, 6, PinInputSelectionDirection::Forward),
            PinShiftNavKey::ArrowRight
        ),
        PinInputSelection::range(5, 6, PinInputSelectionDirection::Forward)
    );
    assert_eq!(
        step_pin_single_cell(
            "123456",
            6,
            PinInputSelection::range(0, 1, PinInputSelectionDirection::Forward),
            PinShiftNavKey::End
        ),
        PinInputSelection::range(5, 6, PinInputSelectionDirection::Backward)
    );

    // 3. replace_pin_value_from_paste_with
    let digits = |s: &str| PinInputBuiltinPattern::Digits.matches_str(s);
    assert_eq!(
        replace_pin_value_from_paste_with("", 6, None::<fn(&str) -> String>, digits),
        None
    );
    assert_eq!(
        replace_pin_value_from_paste_with("abc", 6, None::<fn(&str) -> String>, digits),
        None
    );
    assert_eq!(
        replace_pin_value_from_paste_with("89", 6, None::<fn(&str) -> String>, digits),
        Some(("89".to_string(), 2))
    );
    assert_eq!(
        replace_pin_value_from_paste_with(
            "654-321",
            6,
            Some(|s: &str| s.replace('-', "")),
            digits
        ),
        Some(("654321".to_string(), 6))
    );
    assert_eq!(
        replace_pin_value_from_paste_with("123456789", 6, None::<fn(&str) -> String>, digits),
        Some(("123456".to_string(), 6))
    );
}
