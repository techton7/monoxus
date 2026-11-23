use crate::foundation::{compose::Slottable, shared::ScopeHandle};

use super::*;

#[test]
fn clamping_guards_bounds_and_non_finite_values() {
    assert_eq!(clamp_rating_value(3.5, 0.0, 5), 3.5);
    assert_eq!(clamp_rating_value(-2.0, 0.0, 5), 0.0);
    assert_eq!(clamp_rating_value(9.0, 0.0, 5), 5.0);
    assert_eq!(clamp_rating_value(0.0, 1.0, 5), 1.0);
    assert_eq!(clamp_rating_value(f64::NAN, 1.0, 5), 1.0);
    assert_eq!(clamp_rating_value(f64::INFINITY, 0.0, 5), 0.0);
    assert_eq!(clamp_rating_value(-0.0, 0.0, 5), 0.0);
    assert_eq!(clamp_rating_value(3.0, 5.0, 2), 5.0);
}

#[test]
fn item_state_derivation_covers_whole_half_and_hover_values() {
    let whole = derive_items(3.0, 5, false);
    assert_eq!(
        whole.iter().map(|i| i.state).collect::<Vec<_>>(),
        vec![
            RatingGroupItemState::Active,
            RatingGroupItemState::Active,
            RatingGroupItemState::Active,
            RatingGroupItemState::Inactive,
            RatingGroupItemState::Inactive,
        ]
    );
    assert_eq!(whole[0].value, 1.0);
    assert_eq!(whole[4].value, 5.0);

    let half = derive_items(2.5, 5, true);
    assert_eq!(
        half.iter().map(|i| i.state).collect::<Vec<_>>(),
        vec![
            RatingGroupItemState::Active,
            RatingGroupItemState::Active,
            RatingGroupItemState::Partial,
            RatingGroupItemState::Inactive,
            RatingGroupItemState::Inactive,
        ]
    );

    // When allow_half is false, 2.5 does not produce Partial
    assert_eq!(
        derive_item_state(2.5, 2, false),
        RatingGroupItemState::Inactive
    );
    assert_eq!(
        derive_item_state(0.5, 0, true),
        RatingGroupItemState::Partial
    );
    assert_eq!(
        derive_item_state(0.0, 0, true),
        RatingGroupItemState::Inactive
    );
}

#[test]
fn pointer_fraction_math_handles_ltr_rtl_vertical_and_zero_dims() {
    // Horizontal LTR: left quarter -> 2.5, right quarter -> 3.0
    assert_eq!(
        calculate_rating_from_pointer(
            2,
            true,
            Orientation::Horizontal,
            Direction::Ltr,
            108.0,
            20.0,
            100.0,
            0.0,
            40.0,
            40.0,
        ),
        2.5
    );
    assert_eq!(
        calculate_rating_from_pointer(
            2,
            true,
            Orientation::Horizontal,
            Direction::Ltr,
            132.0,
            20.0,
            100.0,
            0.0,
            40.0,
            40.0,
        ),
        3.0
    );

    // Horizontal RTL: right half -> 2.5 (1 - >0.5 < 0.5), left half -> 3.0
    assert_eq!(
        calculate_rating_from_pointer(
            2,
            true,
            Orientation::Horizontal,
            Direction::Rtl,
            132.0,
            20.0,
            100.0,
            0.0,
            40.0,
            40.0,
        ),
        2.5
    );
    assert_eq!(
        calculate_rating_from_pointer(
            2,
            true,
            Orientation::Horizontal,
            Direction::Rtl,
            108.0,
            20.0,
            100.0,
            0.0,
            40.0,
            40.0,
        ),
        3.0
    );

    // Vertical invariant under RTL (Decision D-4): top half -> 2.5, bottom half -> 3.0
    for dir in [Direction::Ltr, Direction::Rtl] {
        assert_eq!(
            calculate_rating_from_pointer(
                2,
                true,
                Orientation::Vertical,
                dir,
                20.0,
                108.0,
                0.0,
                100.0,
                40.0,
                40.0,
            ),
            2.5
        );
        assert_eq!(
            calculate_rating_from_pointer(
                2,
                true,
                Orientation::Vertical,
                dir,
                20.0,
                132.0,
                0.0,
                100.0,
                40.0,
                40.0,
            ),
            3.0
        );
    }

    // Zero-dimension rect guard does not divide by zero / produce NaN
    let safe = calculate_rating_from_pointer(
        1,
        true,
        Orientation::Horizontal,
        Direction::Ltr,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
    );
    assert_eq!(safe, 1.5);
}

#[test]
fn first_item_click_to_clear_matches_bits_contract() {
    assert_eq!(resolve_item_click_value(0, 1.0, 0.0, 5, 1.0), 0.0);
    assert_eq!(resolve_item_click_value(0, 0.5, 0.0, 5, 0.5), 0.0);
    // Clicking right half when 0.5 sets 1.0 instead of clearing
    assert_eq!(resolve_item_click_value(0, 0.5, 0.0, 5, 1.0), 1.0);
    // When min > 0, clicking item 0 does not clear below min
    assert_eq!(resolve_item_click_value(0, 1.0, 1.0, 5, 1.0), 1.0);
    // Non-first item does not clear
    assert_eq!(resolve_item_click_value(2, 3.0, 0.0, 5, 3.0), 3.0);
}

#[test]
fn keyboard_stepping_and_decimal_sequence_state_machine() {
    let scope = ScopeHandle::root("rating-test");
    let def = RatingGroupDefinition::new(scope.clone()).with_allow_half(true);

    let right = reduce_rating_group_key(
        &def,
        2.0,
        RatingGroupKeySequence::Idle,
        RatingGroupKey::ArrowRight,
    );
    assert_eq!(right.next_value, Some(2.5));
    assert!(right.clear_hover);
    assert!(right.prevent_default);

    let page_up = reduce_rating_group_key(
        &def,
        2.5,
        RatingGroupKeySequence::Idle,
        RatingGroupKey::PageUp,
    );
    assert_eq!(page_up.next_value, Some(3.5));

    let end = reduce_rating_group_key(&def, 1.0, RatingGroupKeySequence::Idle, RatingGroupKey::End);
    assert_eq!(end.next_value, Some(5.0));

    let home =
        reduce_rating_group_key(&def, 5.0, RatingGroupKeySequence::Idle, RatingGroupKey::Home);
    assert_eq!(home.next_value, Some(0.0));

    // RTL flips ArrowLeft / ArrowRight but leaves ArrowUp / ArrowDown intact
    let rtl_def = def.clone().with_direction(Direction::Rtl);
    let rtl_left = reduce_rating_group_key(
        &rtl_def,
        2.0,
        RatingGroupKeySequence::Idle,
        RatingGroupKey::ArrowLeft,
    );
    assert_eq!(rtl_left.next_value, Some(2.5));
    let rtl_right = reduce_rating_group_key(
        &rtl_def,
        2.0,
        RatingGroupKeySequence::Idle,
        RatingGroupKey::ArrowRight,
    );
    assert_eq!(rtl_right.next_value, Some(1.5));
    let rtl_up = reduce_rating_group_key(
        &rtl_def,
        2.0,
        RatingGroupKeySequence::Idle,
        RatingGroupKey::ArrowUp,
    );
    assert_eq!(rtl_up.next_value, Some(2.5));

    // Decimal typing: '2' -> '.' -> '5' => 2.5
    let d2 = reduce_rating_group_key(
        &def,
        0.0,
        RatingGroupKeySequence::Idle,
        RatingGroupKey::Digit(2),
    );
    assert_eq!(d2.next_value, Some(2.0));
    assert_eq!(d2.next_sequence, RatingGroupKeySequence::Digit(2));
    assert!(d2.start_decimal_timer);

    let dot = reduce_rating_group_key(&def, 2.0, d2.next_sequence, RatingGroupKey::DecimalPoint);
    assert_eq!(dot.next_value, None);
    assert_eq!(dot.next_sequence, RatingGroupKeySequence::DigitDot(2));
    assert!(dot.prevent_default);

    let d5 = reduce_rating_group_key(&def, 2.0, dot.next_sequence, RatingGroupKey::Digit(5));
    assert_eq!(d5.next_value, Some(2.5));
    assert_eq!(d5.next_sequence, RatingGroupKeySequence::Idle);
    assert!(d5.clear_decimal_timer);

    // Space / Enter / unknown keys pass through without prevent_default
    let other = reduce_rating_group_key(
        &def,
        2.5,
        RatingGroupKeySequence::Digit(2),
        RatingGroupKey::Other,
    );
    assert_eq!(other.next_value, None);
    assert!(!other.prevent_default);
    assert_eq!(other.next_sequence, RatingGroupKeySequence::Idle);

    // Disabled and read_only block mutations
    let ro_def = def.clone().with_read_only(true);
    let ro = reduce_rating_group_key(
        &ro_def,
        2.0,
        RatingGroupKeySequence::Idle,
        RatingGroupKey::ArrowRight,
    );
    assert_eq!(ro.next_value, None);
    assert!(!ro.prevent_default);
}

#[test]
fn root_and_item_attributes_respect_omission_and_projection() {
    let rels = RatingGroupRelationships::new(ScopeHandle::root("rg"));
    let root_enabled = RatingGroupRootAttributes::new(
        rels.root_id().to_string(),
        2.5,
        0.0,
        5,
        Orientation::Horizontal,
        Direction::Ltr,
        false,
        false,
        true,
        None,
        None,
        None,
    );
    assert_eq!(root_enabled.role, "slider");
    assert_eq!(root_enabled.tabindex, 0);
    assert_eq!(root_enabled.aria_valuenow, 2.5);
    assert_eq!(root_enabled.aria_valuemin, 0.0);
    assert_eq!(root_enabled.aria_valuemax, 5);
    assert_eq!(root_enabled.aria_valuetext, "2.5 out of 5");
    assert_eq!(root_enabled.aria_required, "true");
    assert_eq!(root_enabled.aria_disabled, None);
    assert_eq!(root_enabled.aria_label.as_deref(), Some("Rating"));
    assert_eq!(root_enabled.data_disabled, None);
    assert_eq!(root_enabled.data_readonly, None);

    let root_disabled = RatingGroupRootAttributes::new(
        rels.root_id().to_string(),
        0.0,
        0.0,
        5,
        Orientation::Vertical,
        Direction::Rtl,
        true,
        true,
        false,
        Some("Score".to_string()),
        None,
        Some("Zero".to_string()),
    );
    assert_eq!(root_disabled.tabindex, -1);
    assert_eq!(root_disabled.aria_disabled, Some("true"));
    assert_eq!(root_disabled.data_disabled, Some(""));
    assert_eq!(root_disabled.data_readonly, Some(""));
    assert_eq!(root_disabled.data_orientation, "vertical");
    assert_eq!(root_disabled.dir, "rtl");

    let item = RatingGroupItemAttributes::new(
        rels.item_id(1),
        1,
        Orientation::Horizontal,
        false,
        true,
        RatingGroupItemState::Partial,
    );
    assert_eq!(item.role, "presentation");
    assert_eq!(item.data_value, 2);
    assert_eq!(item.data_disabled, None);
    assert_eq!(item.data_readonly, Some(""));
    assert_eq!(item.data_state.as_str(), "partial");

    let (target, proj_root, content) =
        project_rating_group_root_as_child(&root_enabled, "custom-rg", Slottable::new("stars"));
    assert_eq!(target, "custom-rg");
    assert_eq!(proj_root, root_enabled);
    assert_eq!(content, "stars");
}
