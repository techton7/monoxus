use super::*;
use crate::foundation::shared::ScopeHandle;

#[test]
fn test_decimal_count() {
    assert_eq!(get_decimal_count(1.0), 0);
    assert_eq!(get_decimal_count(10.0), 0);
    assert_eq!(get_decimal_count(0.5), 1);
    assert_eq!(get_decimal_count(0.01), 2);
    assert_eq!(get_decimal_count(0.005), 3);
    assert_eq!(get_decimal_count(0.0001), 4);
    assert_eq!(get_decimal_count(1e-5), 5);
    assert_eq!(get_decimal_count(1.23e-4), 6);
    assert_eq!(get_decimal_count(1e-7), 7);
    assert_eq!(get_decimal_count(f64::NAN), 0);
    assert_eq!(get_decimal_count(f64::INFINITY), 0);
}

#[test]
fn test_round_to_precision() {
    assert_eq!(round_to_precision(0.1 + 0.2, 1), 0.3);
    assert_eq!(round_to_precision(1.23456, 2), 1.23);
    assert_eq!(round_to_precision(1.23556, 2), 1.24);
    assert_eq!(round_to_precision(-0.0, 2), 0.0);
    assert_eq!(round_to_precision(42.0, 0), 42.0);
}

#[test]
fn test_validate_and_normalize_bounds() {
    let (min, max, step) = validate_and_normalize_bounds(0.0, 100.0, 1.0);
    assert_eq!(min, 0.0);
    assert_eq!(max, 100.0);
    assert_eq!(step, 1.0);

    let (min, max, step) = validate_and_normalize_bounds(f64::NAN, f64::NAN, -5.0);
    assert_eq!(min, 0.0);
    assert_eq!(max, 100.0);
    assert_eq!(step, 1.0);

    let (min, max, step) = validate_and_normalize_bounds(50.0, 10.0, 0.0);
    assert_eq!(min, 50.0);
    assert_eq!(max, 150.0);
    assert_eq!(step, 1.0);
}

#[test]
fn test_normalize_custom_steps() {
    let raw = vec![50.0, 10.0, 50.0, -5.0, 120.0, 25.0, f64::NAN];
    let normalized = normalize_custom_steps(&raw, 0.0, 100.0);
    assert_eq!(normalized, vec![10.0, 25.0, 50.0]);
}

#[test]
fn test_snap_to_steps() {
    let steps = vec![0.0, 25.0, 50.0, 75.0, 100.0];
    assert_eq!(snap_to_steps(22.0, &steps), 25.0);
    assert_eq!(snap_to_steps(12.0, &steps), 0.0);
    assert_eq!(snap_to_steps(80.0, &steps), 75.0);
    assert_eq!(snap_to_steps(90.0, &steps), 100.0);
}

#[test]
fn test_off_grid_step_recovery() {
    // Starting from off-grid value 23 with step 10:
    // Direction +1 (ArrowUp) should snap to next grid point 30 without skipping
    let next_up = get_next_step_value(23.0, 0.0, 100.0, 10.0, 1, 1);
    assert_eq!(next_up, 30.0);

    // Direction -1 (ArrowDown) should snap to previous grid point 20 without skipping
    let next_down = get_next_step_value(23.0, 0.0, 100.0, 10.0, -1, 1);
    assert_eq!(next_down, 20.0);

    // Starting from aligned value 20 with step 10:
    let next_aligned_up = get_next_step_value(20.0, 0.0, 100.0, 10.0, 1, 1);
    assert_eq!(next_aligned_up, 30.0);

    let next_aligned_down = get_next_step_value(20.0, 0.0, 100.0, 10.0, -1, 1);
    assert_eq!(next_aligned_down, 10.0);

    // Multiplier 10 (PageUp) from 23:
    let page_up = get_next_step_value(23.0, 0.0, 100.0, 10.0, 1, 10);
    assert_eq!(page_up, 100.0); // Clamped to max

    // Fractional step precision with scientific notation:
    let frac_step = get_next_step_value(0.005, 0.0, 1.0, 0.01, 1, 1);
    assert_eq!(frac_step, 0.01);
}

#[test]
fn test_get_next_custom_step_value() {
    let steps = vec![0.0, 10.0, 25.0, 50.0, 100.0];

    // Off-grid step up from 15 -> 25
    assert_eq!(get_next_custom_step_value(15.0, &steps, 1, 1), 25.0);
    // Off-grid step down from 15 -> 10
    assert_eq!(get_next_custom_step_value(15.0, &steps, -1, 1), 10.0);

    // Aligned step up from 25 -> 50
    assert_eq!(get_next_custom_step_value(25.0, &steps, 1, 1), 50.0);
    // Aligned step down from 25 -> 10
    assert_eq!(get_next_custom_step_value(25.0, &steps, -1, 1), 10.0);

    // Multiplier stepping
    assert_eq!(get_next_custom_step_value(0.0, &steps, 1, 3), 50.0);
}

#[test]
fn test_value_to_percent_and_percent_to_value() {
    assert_eq!(value_to_percent(50.0, 0.0, 100.0), 50.0);
    assert_eq!(value_to_percent(0.0, 0.0, 100.0), 0.0);
    assert_eq!(value_to_percent(100.0, 0.0, 100.0), 100.0);
    assert_eq!(value_to_percent(-10.0, 0.0, 100.0), 0.0);
    assert_eq!(value_to_percent(150.0, 0.0, 100.0), 100.0);

    // Negative bounds
    assert_eq!(value_to_percent(0.0, -50.0, 50.0), 50.0);
    assert_eq!(value_to_percent(-25.0, -50.0, 50.0), 25.0);

    assert_eq!(percent_to_value(50.0, 0.0, 100.0), 50.0);
    assert_eq!(percent_to_value(25.0, -50.0, 50.0), -25.0);
}

#[test]
fn test_get_contained_percent() {
    // Exact mode is identity
    assert_eq!(
        get_contained_percent(0.0, SliderThumbAlignment::Exact, Some(5.0), None, None),
        0.0
    );
    assert_eq!(
        get_contained_percent(50.0, SliderThumbAlignment::Exact, Some(5.0), None, None),
        50.0
    );

    // Contain mode with explicit track_padding = 5%
    // 0% maps to 5%
    assert_eq!(
        get_contained_percent(0.0, SliderThumbAlignment::Contain, Some(5.0), None, None),
        5.0
    );
    // 100% maps to 95%
    assert_eq!(
        get_contained_percent(100.0, SliderThumbAlignment::Contain, Some(5.0), None, None),
        95.0
    );
    // 50% maps to 50%
    assert_eq!(
        get_contained_percent(50.0, SliderThumbAlignment::Contain, Some(5.0), None, None),
        50.0
    );

    // Contain mode with measured sizes: track = 200px, thumb = 20px
    // padding = (20 / 2 / 200) * 100 = 5%
    assert_eq!(
        get_contained_percent(
            0.0,
            SliderThumbAlignment::Contain,
            None,
            Some(200.0),
            Some(20.0)
        ),
        5.0
    );
}

#[test]
fn test_thumb_order_policy_preserve() {
    let values = vec![20.0, 80.0];
    let min_steps = 2; // min_distance = 2 * 5.0 = 10.0
    let step = 5.0;

    // Moving thumb 0 up toward thumb 1 is clamped to 80 - 10 = 70
    let (new_vals, active_idx) = update_thumb_value(
        &values,
        0,
        75.0,
        0.0,
        100.0,
        step,
        None,
        min_steps,
        SliderThumbOrderPolicy::Preserve,
    );
    assert_eq!(new_vals, vec![70.0, 80.0]);
    assert_eq!(active_idx, 0);

    // Moving thumb 1 down toward thumb 0 is clamped to 20 + 10 = 30
    let (new_vals, active_idx) = update_thumb_value(
        &values,
        1,
        25.0,
        0.0,
        100.0,
        step,
        None,
        min_steps,
        SliderThumbOrderPolicy::Preserve,
    );
    assert_eq!(new_vals, vec![20.0, 30.0]);
    assert_eq!(active_idx, 1);
}

#[test]
fn test_thumb_order_policy_swap() {
    let values = vec![20.0, 80.0];

    // Moving thumb 0 past thumb 1 to 90 swaps the array order
    let (new_vals, active_idx) = update_thumb_value(
        &values,
        0,
        90.0,
        0.0,
        100.0,
        1.0,
        None,
        0,
        SliderThumbOrderPolicy::Swap,
    );
    assert_eq!(new_vals, vec![80.0, 90.0]);
    // Thumb 0 is now at index 1
    assert_eq!(active_idx, 1);

    // Moving thumb 1 below thumb 0 to 10 swaps the array order
    let (new_vals, active_idx) = update_thumb_value(
        &values,
        1,
        10.0,
        0.0,
        100.0,
        1.0,
        None,
        0,
        SliderThumbOrderPolicy::Swap,
    );
    assert_eq!(new_vals, vec![10.0, 20.0]);
    // Thumb 1 is now at index 0
    assert_eq!(active_idx, 0);
}

#[test]
fn test_keyboard_step_matrix() {
    // Horizontal LTR !inverted
    assert_eq!(
        resolve_keyboard_step_direction(
            "ArrowRight",
            Orientation::Horizontal,
            Direction::Ltr,
            false
        ),
        Some((1, 1))
    );
    assert_eq!(
        resolve_keyboard_step_direction(
            "ArrowLeft",
            Orientation::Horizontal,
            Direction::Ltr,
            false
        ),
        Some((-1, 1))
    );

    // Horizontal RTL !inverted (inverts Left / Right)
    assert_eq!(
        resolve_keyboard_step_direction(
            "ArrowRight",
            Orientation::Horizontal,
            Direction::Rtl,
            false
        ),
        Some((-1, 1))
    );
    assert_eq!(
        resolve_keyboard_step_direction(
            "ArrowLeft",
            Orientation::Horizontal,
            Direction::Rtl,
            false
        ),
        Some((1, 1))
    );

    // Vertical !inverted (ArrowUp is +1, ArrowDown is -1)
    assert_eq!(
        resolve_keyboard_step_direction("ArrowUp", Orientation::Vertical, Direction::Ltr, false),
        Some((1, 1))
    );
    assert_eq!(
        resolve_keyboard_step_direction("ArrowDown", Orientation::Vertical, Direction::Ltr, false),
        Some((-1, 1))
    );

    // Vertical inverted (ArrowDown is +1, ArrowUp is -1)
    assert_eq!(
        resolve_keyboard_step_direction("ArrowUp", Orientation::Vertical, Direction::Ltr, true),
        Some((-1, 1))
    );
    assert_eq!(
        resolve_keyboard_step_direction("ArrowDown", Orientation::Vertical, Direction::Ltr, true),
        Some((1, 1))
    );

    // PageUp / PageDown have multiplier 10
    assert_eq!(
        resolve_keyboard_step_direction("PageUp", Orientation::Horizontal, Direction::Ltr, false),
        Some((1, 10))
    );
    assert_eq!(
        resolve_keyboard_step_direction("PageDown", Orientation::Horizontal, Direction::Ltr, false),
        Some((-1, 10))
    );
}

#[test]
fn test_slider_relationships() {
    let scope = ScopeHandle::root("test-slider");
    let rels = SliderRelationships::new(scope);

    assert_eq!(rels.root_id(), "11:test-slider");
    assert_eq!(rels.track_id(), "11:test-slider|5:track");
    assert_eq!(rels.range_id(), "11:test-slider|5:range");
    assert_eq!(rels.thumb_id(0), "11:test-slider|7:thumb-0");
    assert_eq!(rels.thumb_id(1), "11:test-slider|7:thumb-1");
    assert_eq!(rels.input_id(0), "11:test-slider|7:input-0");
    assert_eq!(rels.tick_id(5), "11:test-slider|6:tick-5");
    assert_eq!(rels.tick_label_id(5), "11:test-slider|12:tick-label-5");
    assert_eq!(rels.thumb_label_id(0), "11:test-slider|13:thumb-label-0");
}

#[test]
fn test_attrs_publishers() {
    let root_attrs = SliderRootAttributes::new(
        "root-1".to_string(),
        Orientation::Horizontal,
        Direction::Ltr,
        false,
    );
    assert_eq!(root_attrs.data_orientation(), "horizontal");
    assert_eq!(root_attrs.data_disabled(), None);
    assert_eq!(root_attrs.aria_disabled(), None);

    let disabled_root_attrs = SliderRootAttributes::new(
        "root-2".to_string(),
        Orientation::Vertical,
        Direction::Rtl,
        true,
    );
    assert_eq!(disabled_root_attrs.data_orientation(), "vertical");
    assert_eq!(disabled_root_attrs.data_disabled(), Some(""));
    assert_eq!(disabled_root_attrs.aria_disabled(), Some("true"));

    let thumb_attrs = SliderThumbAttributes::new(
        "thumb-0".to_string(),
        42.0,
        0.0,
        100.0,
        Orientation::Horizontal,
        Direction::Ltr,
        false,
        false,
        true,
        42.0,
        Some("Volume".to_string()),
        None,
        Some("42 percent".to_string()),
    );
    assert_eq!(thumb_attrs.role(), "slider");
    assert_eq!(thumb_attrs.tabindex(), 0);
    assert_eq!(thumb_attrs.aria_valuenow, 42.0);
    assert_eq!(thumb_attrs.aria_valuemin, 0.0);
    assert_eq!(thumb_attrs.aria_valuemax, 100.0);
    assert_eq!(thumb_attrs.aria_label.as_deref(), Some("Volume"));
    assert_eq!(thumb_attrs.aria_valuetext.as_deref(), Some("42 percent"));
    assert_eq!(thumb_attrs.data_active(), Some(""));
    assert_eq!(thumb_attrs.data_value(), "42");

    let tick_attrs = SliderTickAttributes::new(
        "tick-1".to_string(),
        50.0,
        Orientation::Horizontal,
        Direction::Ltr,
        false,
        false,
        true,
        false,
        50.0,
    );
    assert_eq!(tick_attrs.data_bounded(), Some(""));
    assert_eq!(tick_attrs.data_selected(), None);
}

#[test]
fn test_rtl_and_inverted_attributes_positioning() {
    let thumb_rtl = SliderThumbAttributes::new(
        "thumb-rtl".to_string(),
        20.0,
        0.0,
        100.0,
        Orientation::Horizontal,
        Direction::Rtl,
        false,
        false,
        false,
        20.0,
        None,
        None,
        None,
    );
    assert!(
        thumb_rtl.style().contains("right: 20%;"),
        "RTL thumb must contain 'right: 20%;', got {}",
        thumb_rtl.style()
    );

    let range_rtl = SliderRangeAttributes::new(
        "range-rtl".to_string(),
        Orientation::Horizontal,
        Direction::Rtl,
        false,
        false,
        0.0,
        20.0,
    );
    assert_eq!(
        range_rtl.style(),
        "position: absolute; right: 0%; width: 20%;"
    );

    let thumb_v_inv = SliderThumbAttributes::new(
        "thumb-v-inv".to_string(),
        20.0,
        0.0,
        100.0,
        Orientation::Vertical,
        Direction::Ltr,
        true,
        false,
        false,
        20.0,
        None,
        None,
        None,
    );
    assert!(
        thumb_v_inv.style().contains("top: 20%;"),
        "Inverted vertical thumb must contain 'top: 20%;', got {}",
        thumb_v_inv.style()
    );

    let range_v_inv = SliderRangeAttributes::new(
        "range-v-inv".to_string(),
        Orientation::Vertical,
        Direction::Ltr,
        true,
        false,
        0.0,
        20.0,
    );
    assert_eq!(
        range_v_inv.style(),
        "position: absolute; top: 0%; height: 20%;"
    );
}

#[test]
fn test_runtime_range_and_tick_contain_alignment() {
    use dioxus::prelude::{ScopeId, Signal, VirtualDom, rsx};

    let mut dom = VirtualDom::new(|| rsx! {});
    dom.rebuild_in_place();
    dom.in_scope(ScopeId::APP, || {
        let scope = ScopeHandle::root("runtime-contain-test");
        let rels = SliderRelationships::new(scope.clone());
        let def = SliderDefinition::new(scope)
            .with_min(0.0)
            .with_max(100.0)
            .with_thumb_alignment(SliderThumbAlignment::Contain);

        let mut rt = SliderRuntime {
            def: def.clone(),
            rels: rels.clone(),
            values: Signal::new(vec![25.0]),
            is_single: true,
            active_thumb: Signal::new(None),
            is_dragging: Signal::new(false),
            drag_start_values: Signal::new(None),
            focused_thumb: Signal::new(None),
            on_value_change: None,
            on_value_commit: None,
            track_size: Signal::new(Some(200.0)),
            thumb_sizes: Signal::new(vec![Some(20.0)]),
            track_id: Signal::new(rels.track_id().to_string()),
            thumb_ids: Signal::new(vec![None]),
            thumb_drag_claimed: Signal::new(false),
        };

        // Track 200px, thumb 20px => padding = 5%.
        // At val = 25.0 (intermediate), contained pct = 5 + 0.25 * 90 = 27.5%.
        let thumb_mid = rt.thumb_attrs(0, None, None, None);
        assert!(
            thumb_mid.style().contains("left: 27.5%;"),
            "Thumb at 25 should align at 27.5%, got {}",
            thumb_mid.style()
        );
        assert_eq!(
            rt.range_attrs().style(),
            "position: absolute; left: 0%; width: 27.5%;"
        );

        // At min, single-thumb range is empty [0, 0]; at max, range fills [0, 100].
        rt.values.set(vec![0.0]);
        assert_eq!(
            rt.range_attrs().style(),
            "position: absolute; left: 0%; width: 0%;"
        );
        rt.values.set(vec![100.0]);
        assert_eq!(
            rt.range_attrs().style(),
            "position: absolute; left: 0%; width: 100%;"
        );

        // Ticks and tick labels use thumb 0's size in Contain mode (5%..95%).
        let tick_0 = rt.tick_attrs(0.0);
        let tick_100 = rt.tick_attrs(100.0);
        assert!(
            tick_0.style().contains("left: 5%;"),
            "Tick 0 should align at 5%, got {}",
            tick_0.style()
        );
        assert!(
            tick_100.style().contains("left: 95%;"),
            "Tick 100 should align at 95%, got {}",
            tick_100.style()
        );
        let tick_label_0 = rt.tick_label_attrs(0.0, None);
        let tick_label_100 = rt.tick_label_attrs(100.0, None);
        assert!(
            tick_label_0.style().contains("left: 5%;"),
            "TickLabel 0 should align at 5%, got {}",
            tick_label_0.style()
        );
        assert!(
            tick_label_100.style().contains("left: 95%;"),
            "TickLabel 100 should align at 95%, got {}",
            tick_label_100.style()
        );

        // Multi-thumb range aligns both endpoints to their contained thumb centers.
        rt.is_single = false;
        rt.thumb_sizes.set(vec![Some(20.0), Some(20.0)]);
        rt.values.set(vec![0.0, 100.0]);
        assert_eq!(
            rt.range_attrs().style(),
            "position: absolute; left: 5%; width: 90%;"
        );
        rt.values.set(vec![20.0, 80.0]);
        assert_eq!(
            rt.range_attrs().style(),
            "position: absolute; left: 23%; width: 54%;"
        );

        // Exact mode leaves range and ticks uncontained.
        rt.def.thumb_alignment = SliderThumbAlignment::Exact;
        assert_eq!(
            rt.range_attrs().style(),
            "position: absolute; left: 20%; width: 60%;"
        );
        assert!(rt.tick_attrs(0.0).style().contains("left: 0%;"));
        assert!(
            rt.tick_label_attrs(100.0, None)
                .style()
                .contains("left: 100%;")
        );
    });
}
