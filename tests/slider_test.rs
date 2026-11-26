use dioxus::prelude::*;
use monoxus::foundation::{
    compose::Slottable,
    shared::{Direction, Orientation, ScopeHandle},
};
use monoxus::slider::*;

#[test]
fn slider_parts_and_constants_contract() {
    assert_eq!(
        SLIDER_PARTS,
        [
            SliderPart::Root,
            SliderPart::Track,
            SliderPart::Range,
            SliderPart::Thumb,
            SliderPart::Tick,
            SliderPart::TickLabel,
            SliderPart::ThumbLabel,
        ]
    );
    assert_eq!(
        SliderThumbOrderPolicy::default(),
        SliderThumbOrderPolicy::Swap
    );
    assert_eq!(
        SliderThumbAlignment::default(),
        SliderThumbAlignment::Contain
    );
    assert_eq!(SliderLabelPosition::default(), SliderLabelPosition::Top);
}

#[test]
fn slider_value_enum_and_conversions() {
    let single = SliderValue::Single(50.0);
    assert!(single.is_single());
    assert!(!single.is_multiple());
    assert_eq!(single.len(), 1);
    assert_eq!(single.to_vec(), vec![50.0]);

    let multiple = SliderValue::Multiple(vec![20.0, 40.0, 80.0]);
    assert!(!multiple.is_single());
    assert!(multiple.is_multiple());
    assert_eq!(multiple.len(), 3);
    assert_eq!(multiple.to_vec(), vec![20.0, 40.0, 80.0]);

    let from_f64: SliderValue = 42.0.into();
    assert_eq!(from_f64, SliderValue::Single(42.0));

    let from_vec: SliderValue = vec![10.0, 90.0].into();
    assert_eq!(from_vec, SliderValue::Multiple(vec![10.0, 90.0]));

    let from_slice: SliderValue = [5.0, 25.0].as_slice().into();
    assert_eq!(from_slice, SliderValue::Multiple(vec![5.0, 25.0]));

    let from_array: SliderValue = [1.0, 2.0, 3.0].into();
    assert_eq!(from_array, SliderValue::Multiple(vec![1.0, 2.0, 3.0]));
}

#[test]
fn slider_definition_and_builder_contract() {
    let scope = ScopeHandle::root("slider-def-test");
    let def = SliderDefinition::new(scope.clone())
        .with_min(10.0)
        .with_max(90.0)
        .with_step(2.5)
        .with_steps(Some(vec![10.0, 30.0, 50.0, 90.0]))
        .with_min_steps_between_thumbs(2)
        .with_order_policy(SliderThumbOrderPolicy::Preserve)
        .with_thumb_alignment(SliderThumbAlignment::Exact)
        .with_track_padding(Some(4.0))
        .with_orientation(Orientation::Vertical)
        .with_direction(Direction::Rtl)
        .with_inverted(true)
        .with_disabled(true);

    assert_eq!(def.min, 10.0);
    assert_eq!(def.max, 90.0);
    assert_eq!(def.step, 2.5);
    assert_eq!(def.steps, Some(vec![10.0, 30.0, 50.0, 90.0]));
    assert_eq!(def.min_steps_between_thumbs, 2);
    assert_eq!(def.order_policy, SliderThumbOrderPolicy::Preserve);
    assert_eq!(def.thumb_alignment, SliderThumbAlignment::Exact);
    assert_eq!(def.track_padding, Some(4.0));
    assert_eq!(def.orientation, Orientation::Vertical);
    assert_eq!(def.direction, Direction::Rtl);
    assert!(def.inverted);
    assert!(def.disabled);

    // Normalization handles non-finite values cleanly
    let invalid_def = SliderDefinition::new(scope)
        .with_min(f64::NAN)
        .with_max(f64::INFINITY)
        .with_step(-1.0)
        .normalize();
    assert_eq!(invalid_def.min, 0.0);
    assert_eq!(invalid_def.max, 100.0);
    assert_eq!(invalid_def.step, 1.0);
}

#[test]
fn slider_single_thumb_clamping_and_stepping() {
    // Clamping to bounds
    assert_eq!(value_to_percent(150.0, 0.0, 100.0), 100.0);
    assert_eq!(value_to_percent(-20.0, 0.0, 100.0), 0.0);

    // Decimal count handles scientific notation
    assert_eq!(get_decimal_count(1e-5), 5);
    assert_eq!(get_decimal_count(1.23e-4), 6);
    assert_eq!(get_decimal_count(0.01), 2);
    assert_eq!(get_decimal_count(10.0), 0);

    // Rounding without float drift
    assert_eq!(round_to_precision(0.1 + 0.2, 1), 0.3);
    assert_eq!(round_to_precision(-0.0, 1), 0.0);

    // Off-grid step recovery (Radix specification)
    // From off-grid value 27 with step 5:
    // ArrowUp (+1) snaps to 30
    assert_eq!(get_next_step_value(27.0, 0.0, 100.0, 5.0, 1, 1), 30.0);
    // ArrowDown (-1) snaps to 25
    assert_eq!(get_next_step_value(27.0, 0.0, 100.0, 5.0, -1, 1), 25.0);
    // PageUp (+10) from 27 snaps to 75
    assert_eq!(get_next_step_value(27.0, 0.0, 100.0, 5.0, 1, 10), 75.0);

    // Fractional step recovery with small values
    assert_eq!(get_next_step_value(0.005, 0.0, 1.0, 0.01, 1, 1), 0.01);
    assert_eq!(get_next_step_value(0.005, 0.0, 1.0, 0.01, -1, 1), 0.0);
}

#[test]
fn slider_multi_thumb_swap_vs_preserve_contract() {
    let initial = vec![30.0, 70.0];
    let step = 1.0;
    let min_steps = 5; // 5 units min distance

    // Preserve mode blocks thumb crossover
    let (preserved_vals, active_idx) = update_thumb_value(
        &initial,
        0,
        75.0,
        0.0,
        100.0,
        step,
        None,
        min_steps,
        SliderThumbOrderPolicy::Preserve,
    );
    // Thumb 0 cannot exceed 70.0 - 5.0 = 65.0
    assert_eq!(preserved_vals, vec![65.0, 70.0]);
    assert_eq!(active_idx, 0);

    // Swap mode permits crossover and reorders thumbs
    let (swapped_vals, active_idx) = update_thumb_value(
        &initial,
        0,
        85.0,
        0.0,
        100.0,
        step,
        None,
        0,
        SliderThumbOrderPolicy::Swap,
    );
    // Thumb 0 moved past thumb 1 (70.0) to 85.0; array is re-sorted
    assert_eq!(swapped_vals, vec![70.0, 85.0]);
    // Active index is now 1 (tracking the moved thumb)
    assert_eq!(active_idx, 1);
}

#[test]
fn slider_custom_discrete_steps_contract() {
    let steps = vec![0.0, 15.0, 30.0, 60.0, 100.0];

    // Snapping off-grid value to nearest custom step
    assert_eq!(snap_to_steps(20.0, &steps), 15.0);
    assert_eq!(snap_to_steps(25.0, &steps), 30.0);

    // Stepping along custom steps
    assert_eq!(get_next_custom_step_value(30.0, &steps, 1, 1), 60.0);
    assert_eq!(get_next_custom_step_value(30.0, &steps, -1, 1), 15.0);

    // Off-grid step up from 40 snaps to 60
    assert_eq!(get_next_custom_step_value(40.0, &steps, 1, 1), 60.0);
    // Off-grid step down from 40 snaps to 30
    assert_eq!(get_next_custom_step_value(40.0, &steps, -1, 1), 30.0);
}

#[test]
fn slider_contain_vs_exact_alignment_contract() {
    // Exact mode: unadjusted percent
    assert_eq!(
        get_contained_percent(0.0, SliderThumbAlignment::Exact, Some(5.0), None, None),
        0.0
    );
    assert_eq!(
        get_contained_percent(100.0, SliderThumbAlignment::Exact, Some(5.0), None, None),
        100.0
    );

    // Contain mode with padding = 6%:
    // 0% maps to 6%
    assert_eq!(
        get_contained_percent(0.0, SliderThumbAlignment::Contain, Some(6.0), None, None),
        6.0
    );
    // 100% maps to 94%
    assert_eq!(
        get_contained_percent(100.0, SliderThumbAlignment::Contain, Some(6.0), None, None),
        94.0
    );
    // 50% maps to 50%
    assert_eq!(
        get_contained_percent(50.0, SliderThumbAlignment::Contain, Some(6.0), None, None),
        50.0
    );
}

#[test]
fn slider_full_keyboard_matrix_contract() {
    // Horizontal LTR normal
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

    // Horizontal RTL normal (reverses horizontal step directions)
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

    // Vertical normal (ArrowUp is positive, ArrowDown is negative)
    assert_eq!(
        resolve_keyboard_step_direction("ArrowUp", Orientation::Vertical, Direction::Ltr, false),
        Some((1, 1))
    );
    assert_eq!(
        resolve_keyboard_step_direction("ArrowDown", Orientation::Vertical, Direction::Ltr, false),
        Some((-1, 1))
    );

    // Vertical inverted (ArrowDown is positive, ArrowUp is negative)
    assert_eq!(
        resolve_keyboard_step_direction("ArrowUp", Orientation::Vertical, Direction::Ltr, true),
        Some((-1, 1))
    );
    assert_eq!(
        resolve_keyboard_step_direction("ArrowDown", Orientation::Vertical, Direction::Ltr, true),
        Some((1, 1))
    );

    // PageUp / PageDown 10x multiplier
    assert_eq!(
        resolve_keyboard_step_direction("PageUp", Orientation::Horizontal, Direction::Ltr, false),
        Some((1, 10))
    );
    assert_eq!(
        resolve_keyboard_step_direction("PageDown", Orientation::Horizontal, Direction::Ltr, false),
        Some((-1, 10))
    );

    // Non-navigation keys return None
    assert_eq!(
        resolve_keyboard_step_direction("Enter", Orientation::Horizontal, Direction::Ltr, false),
        None
    );
    assert_eq!(
        resolve_keyboard_step_direction("Space", Orientation::Horizontal, Direction::Ltr, false),
        None
    );
}

#[test]
fn slider_wai_aria_and_data_attributes_contract() {
    // Root attributes
    let root = SliderRootAttributes::new(
        "slider-1".to_string(),
        Orientation::Horizontal,
        Direction::Ltr,
        false,
    );
    assert_eq!(root.data_orientation(), "horizontal");
    assert_eq!(root.data_disabled(), None);
    assert_eq!(root.aria_disabled(), None);

    let disabled_root = SliderRootAttributes::new(
        "slider-2".to_string(),
        Orientation::Vertical,
        Direction::Rtl,
        true,
    );
    assert_eq!(disabled_root.data_orientation(), "vertical");
    assert_eq!(disabled_root.data_disabled(), Some(""));
    assert_eq!(disabled_root.aria_disabled(), Some("true"));

    // Track attributes
    let track = SliderTrackAttributes::new("track-1".to_string(), Orientation::Horizontal, false);
    assert_eq!(track.data_orientation(), "horizontal");
    assert_eq!(track.data_disabled(), None);

    // Range attributes
    let range = SliderRangeAttributes::new(
        "range-1".to_string(),
        Orientation::Horizontal,
        Direction::Ltr,
        false,
        false,
        20.0,
        80.0,
    );
    assert!(range.style().contains("left: 20%"));
    assert!(range.style().contains("width: 60%"));

    // Thumb attributes with auto-label for 2 thumbs
    let thumb_min = SliderThumbAttributes::new(
        "thumb-0".to_string(),
        20.0,
        0.0,
        100.0,
        Orientation::Horizontal,
        Direction::Ltr,
        false,
        false,
        false,
        20.0,
        resolve_thumb_aria_label(None, 0, 2),
        None,
        None,
    );
    assert_eq!(thumb_min.role(), "slider");
    assert_eq!(thumb_min.tabindex(), 0);
    assert_eq!(thumb_min.aria_valuenow, 20.0);
    assert_eq!(thumb_min.aria_valuemin, 0.0);
    assert_eq!(thumb_min.aria_valuemax, 100.0);
    assert_eq!(thumb_min.aria_label.as_deref(), Some("Minimum"));

    let thumb_max = SliderThumbAttributes::new(
        "thumb-1".to_string(),
        80.0,
        0.0,
        100.0,
        Orientation::Horizontal,
        Direction::Ltr,
        false,
        false,
        true,
        80.0,
        resolve_thumb_aria_label(None, 1, 2),
        None,
        None,
    );
    assert_eq!(thumb_max.aria_label.as_deref(), Some("Maximum"));
    assert_eq!(thumb_max.data_active(), Some(""));

    // Tick attributes
    let tick = SliderTickAttributes::new(
        "tick-50".to_string(),
        50.0,
        Orientation::Horizontal,
        Direction::Ltr,
        false,
        false,
        true,
        false,
        50.0,
    );
    assert_eq!(tick.data_bounded(), Some(""));
    assert_eq!(tick.data_selected(), None);
    assert_eq!(tick.data_value(), "50");

    // TickLabel attributes
    let tick_label = SliderTickLabelAttributes::new(
        "tick-label-50".to_string(),
        50.0,
        SliderLabelPosition::Bottom,
        Orientation::Horizontal,
        Direction::Ltr,
        false,
        false,
        true,
        false,
        50.0,
    );
    assert_eq!(tick_label.data_position(), "bottom");
    assert_eq!(tick_label.data_bounded(), Some(""));

    // ThumbLabel attributes
    let thumb_label = SliderThumbLabelAttributes::new(
        "thumb-label-0".to_string(),
        20.0,
        SliderLabelPosition::Top,
        Orientation::Horizontal,
        Direction::Ltr,
        false,
        false,
        true,
        20.0,
    );
    assert_eq!(thumb_label.data_position(), "top");
    assert_eq!(thumb_label.data_active(), Some(""));
}

#[test]
fn slider_as_child_projection_contract() {
    let root_attrs = SliderRootAttributes::new(
        "proj-root".to_string(),
        Orientation::Horizontal,
        Direction::Ltr,
        false,
    );
    let (target, attrs, content) =
        project_slider_root_as_child(&root_attrs, "custom-slider", Slottable::new("inner"));
    assert_eq!(target, "custom-slider");
    assert_eq!(attrs.id(), "proj-root");
    assert_eq!(content, "inner");

    let track_attrs =
        SliderTrackAttributes::new("proj-track".to_string(), Orientation::Horizontal, false);
    let (target, attrs, content) = project_slider_track_as_child(
        &track_attrs,
        "custom-track",
        Slottable::new("track-content"),
    );
    assert_eq!(target, "custom-track");
    assert_eq!(attrs.id(), "proj-track");
    assert_eq!(content, "track-content");

    let range_attrs = SliderRangeAttributes::new(
        "proj-range".to_string(),
        Orientation::Horizontal,
        Direction::Ltr,
        false,
        false,
        0.0,
        50.0,
    );
    let (target, attrs, _) =
        project_slider_range_as_child(&range_attrs, "custom-range", Slottable::new(()));
    assert_eq!(target, "custom-range");
    assert_eq!(attrs.id(), "proj-range");

    let thumb_attrs = SliderThumbAttributes::new(
        "proj-thumb".to_string(),
        50.0,
        0.0,
        100.0,
        Orientation::Horizontal,
        Direction::Ltr,
        false,
        false,
        false,
        50.0,
        None,
        None,
        None,
    );
    let (target, attrs, _) =
        project_slider_thumb_as_child(&thumb_attrs, "custom-thumb", Slottable::new(()));
    assert_eq!(target, "custom-thumb");
    assert_eq!(attrs.id(), "proj-thumb");

    let tick_attrs = SliderTickAttributes::new(
        "proj-tick".to_string(),
        50.0,
        Orientation::Horizontal,
        Direction::Ltr,
        false,
        false,
        true,
        false,
        50.0,
    );
    let (target, attrs, _) =
        project_slider_tick_as_child(&tick_attrs, "custom-tick", Slottable::new(()));
    assert_eq!(target, "custom-tick");
    assert_eq!(attrs.id(), "proj-tick");

    let tick_label_attrs = SliderTickLabelAttributes::new(
        "proj-tick-label".to_string(),
        50.0,
        SliderLabelPosition::Bottom,
        Orientation::Horizontal,
        Direction::Ltr,
        false,
        false,
        true,
        false,
        50.0,
    );
    let (target, attrs, _) = project_slider_tick_label_as_child(
        &tick_label_attrs,
        "custom-tick-label",
        Slottable::new(()),
    );
    assert_eq!(target, "custom-tick-label");
    assert_eq!(attrs.id(), "proj-tick-label");

    let thumb_label_attrs = SliderThumbLabelAttributes::new(
        "proj-thumb-label".to_string(),
        50.0,
        SliderLabelPosition::Top,
        Orientation::Horizontal,
        Direction::Ltr,
        false,
        false,
        false,
        50.0,
    );
    let (target, attrs, _) = project_slider_thumb_label_as_child(
        &thumb_label_attrs,
        "custom-thumb-label",
        Slottable::new(()),
    );
    assert_eq!(target, "custom-thumb-label");
    assert_eq!(attrs.id(), "proj-thumb-label");
}

#[component]
fn SingleThumbSliderScenario() -> Element {
    rsx! {
        SliderRoot {
            id: "single-slider",
            default_value: SliderValue::Single(30.0),
            min: 0.0,
            max: 100.0,
            step: 1.0,
            name: "volume".to_string(),
            SliderTrack {
                SliderRange {}
            }
            SliderThumb {
                index: 0,
                aria_label: "Volume".to_string(),
            }
        }
    }
}

#[component]
fn RangeSliderWithTicksScenario() -> Element {
    rsx! {
        SliderRoot {
            id: "range-slider",
            default_value: SliderValue::Multiple(vec![25.0, 75.0]),
            min: 0.0,
            max: 100.0,
            step: 5.0,
            order_policy: SliderThumbOrderPolicy::Preserve,
            min_steps_between_thumbs: 2,
            SliderTrack {
                SliderRange {}
            }
            SliderTick { value: 0.0 }
            SliderTick { value: 25.0 }
            SliderTick { value: 50.0 }
            SliderTick { value: 75.0 }
            SliderTick { value: 100.0 }
            SliderTickLabel { value: 0.0, "0%" }
            SliderTickLabel { value: 50.0, "50%" }
            SliderTickLabel { value: 100.0, "100%" }
            SliderThumb { index: 0 }
            SliderThumb { index: 1 }
            SliderThumbLabel { index: 0 }
            SliderThumbLabel { index: 1 }
        }
    }
}

#[test]
fn slider_virtual_dom_scenarios_render_cleanly() {
    let mut single_dom = VirtualDom::new(SingleThumbSliderScenario);
    single_dom.rebuild_in_place();

    let mut range_dom = VirtualDom::new(RangeSliderWithTicksScenario);
    range_dom.rebuild_in_place();
}

#[test]
fn slider_rtl_and_inverted_positioning_contract() {
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
        "RTL thumb must position with right: 20%, got {}",
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
        "Inverted vertical thumb must position with top: 20%, got {}",
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
fn slider_contain_range_and_tick_sync_contract() {
    let mut dom = VirtualDom::new(|| rsx! {});
    dom.rebuild_in_place();
    dom.in_scope(ScopeId::APP, || {
        let scope = ScopeHandle::root("contain-sync-contract");
        let rels = SliderRelationships::new(scope.clone());
        let def = SliderDefinition::new(scope)
            .with_min(0.0)
            .with_max(100.0)
            .with_thumb_alignment(SliderThumbAlignment::Contain);

        let mut rt = SliderRuntime {
            def,
            rels: rels.clone(),
            values: Signal::new(vec![50.0]),
            values_ctl: None,
            is_single: true,
            active_thumb: Signal::new(None),
            is_dragging: Signal::new(false),
            drag_start_values: Signal::new(None),
            focused_thumb: Signal::new(None),
            on_value_change: CopyValue::new(None),
            on_value_commit: CopyValue::new(None),
            track_size: Signal::new(Some(300.0)),
            thumb_sizes: Signal::new(vec![Some(30.0)]),
            track_id: Signal::new(rels.track_id().to_string()),
            thumb_ids: Signal::new(vec![None]),
            thumb_drag_claimed: Signal::new(false),
        };

        // Track 300px, thumb 30px => padding = 5%.
        // Single-thumb at 25.0 -> 5 + 0.25 * 90 = 27.5%
        rt.values.set(vec![25.0]);
        assert_eq!(
            rt.range_attrs().style(),
            "position: absolute; left: 0%; width: 27.5%;"
        );
        assert!(rt.tick_attrs(0.0).style().contains("left: 5%;"));
        assert!(
            rt.tick_label_attrs(100.0, None)
                .style()
                .contains("left: 95%;")
        );

        // Multi-thumb at [0.0, 100.0] keeps range aligned between contained thumb centers [5%, 95%].
        rt.is_single = false;
        rt.thumb_sizes.set(vec![Some(30.0), Some(30.0)]);
        rt.values.set(vec![0.0, 100.0]);
        assert_eq!(
            rt.range_attrs().style(),
            "position: absolute; left: 5%; width: 90%;"
        );
    });
}

#[cfg(feature = "native")]
#[test]
fn slider_sibling_thumb_dom_structure_contract() {
    use crate::common::native_fixture::{NATIVE_TEST_LOCK, mount_native_scenario};

    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    // Single-thumb scenario: Thumb is sibling of Track under Root, Range is inside Track.
    {
        let harness = mount_native_scenario(SingleThumbSliderScenario);
        let root_id = harness
            .query("#single-slider")
            .expect("SliderRoot must mount");
        let track_id = harness
            .query("[data-monoxus-slider-track]")
            .expect("SliderTrack must mount");
        let range_id = harness
            .query("[data-monoxus-slider-range]")
            .expect("SliderRange must mount");
        let thumb_id = harness
            .query("[data-monoxus-slider-thumb]")
            .expect("SliderThumb must mount");

        let base = harness.base();
        let root_children = &base.get_node(root_id).expect("root node").children;
        let track_children = &base.get_node(track_id).expect("track node").children;

        assert!(
            root_children.contains(&track_id),
            "SliderTrack must be a direct child of SliderRoot"
        );
        assert!(
            root_children.contains(&thumb_id),
            "SliderThumb must be a direct sibling of SliderTrack under SliderRoot"
        );
        assert!(
            !track_children.contains(&thumb_id),
            "SliderThumb must not be nested inside SliderTrack"
        );
        assert!(
            track_children.contains(&range_id),
            "SliderRange must remain inside SliderTrack"
        );
    }

    // Multi-thumb + ticks scenario: all thumbs, ticks, and labels are direct siblings of Track.
    {
        let harness = mount_native_scenario(RangeSliderWithTicksScenario);
        let root_id = harness
            .query("#range-slider")
            .expect("Range SliderRoot must mount");
        let track_id = harness
            .query("[data-monoxus-slider-track]")
            .expect("SliderTrack must mount");
        let range_id = harness
            .query("[data-monoxus-slider-range]")
            .expect("SliderRange must mount");
        let thumb_ids = harness.query_all("[data-monoxus-slider-thumb]");
        let tick_ids = harness.query_all("[data-monoxus-slider-tick]");
        let tick_label_ids = harness.query_all("[data-monoxus-slider-tick-label]");
        let thumb_label_ids = harness.query_all("[data-monoxus-slider-thumb-label]");

        assert_eq!(thumb_ids.len(), 2);
        assert_eq!(tick_ids.len(), 5);
        assert_eq!(tick_label_ids.len(), 3);
        assert_eq!(thumb_label_ids.len(), 2);

        let base = harness.base();
        let root_children = &base.get_node(root_id).expect("root node").children;
        let track_children = &base.get_node(track_id).expect("track node").children;

        assert!(track_children.contains(&range_id));
        for id in thumb_ids
            .into_iter()
            .chain(tick_ids)
            .chain(tick_label_ids)
            .chain(thumb_label_ids)
        {
            assert!(
                root_children.contains(&id),
                "Slider part {id} must be a direct child of SliderRoot"
            );
            assert!(
                !track_children.contains(&id),
                "Slider part {id} must not be nested inside SliderTrack"
            );
        }
    }
}

#[cfg(feature = "native")]
#[component]
fn ContainHorizontalSliderScenario() -> Element {
    rsx! {
        div {
            style: "width: 400px; padding: 20px;",
            SliderRoot {
                id: "contain-slider",
                default_value: SliderValue::Single(0.0),
                min: 0.0,
                max: 100.0,
                step: 1.0,
                style: "width: 300px; height: 24px; display: flex; align-items: center;",
                SliderTrack {
                    id: "contain-slider-track",
                    style: "width: 300px; height: 8px; position: relative;",
                    SliderRange {
                        id: "contain-slider-range",
                    }
                }
                SliderThumb {
                    id: "contain-slider-thumb",
                    index: 0,
                    aria_label: "Volume".to_string(),
                    style: "width: 20px; height: 20px; display: block;",
                }
            }
        }
    }
}

#[cfg(feature = "native")]
#[test]
fn slider_contain_post_mount_size_observation_contract() {
    use crate::common::native_fixture::{
        NATIVE_TEST_LOCK, focus_node, mount_native_scenario, send_key_and_pump,
    };
    use keyboard_types::{Code, Key, NamedKey};

    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(ContainHorizontalSliderScenario);
    harness.pump();

    let native_doc = oxidase::dom::Document::from_base(harness.doc.inner.clone());
    oxidase::dom::Document::set_current(Some(native_doc.clone()));
    native_doc.notify_layout_observers();
    harness.pump();

    let style_at_min = harness
        .attr("#contain-slider-thumb", "style")
        .expect("Contain slider thumb must have a style attribute at value=0");
    assert!(
        style_at_min.contains("left: 3.333"),
        "Contain thumb at value=0 must resolve inset left: 3.333...%, got: {style_at_min}"
    );
    assert!(
        !style_at_min.contains("left: 0%;"),
        "Contain thumb at value=0 must not remain at left: 0%, got: {style_at_min}"
    );

    focus_node(&mut harness, "#contain-slider-thumb");
    send_key_and_pump(&mut harness, Key::Named(NamedKey::End), Code::End);

    let style_at_max = harness
        .attr("#contain-slider-thumb", "style")
        .expect("Contain slider thumb must have a style attribute at value=100");
    assert!(
        style_at_max.contains("left: 96.666"),
        "Contain thumb at value=100 must resolve inset left: 96.666...%, got: {style_at_max}"
    );
    assert!(
        !style_at_max.contains("left: 100%;"),
        "Contain thumb at value=100 must not remain at left: 100%, got: {style_at_max}"
    );
}

#[test]
fn slider_controlled_rejection_keeps_aria_valuenow_and_nan_safety() {
    use std::cell::RefCell;
    use std::rc::Rc;

    #[component]
    fn ControlledProbe(
        proposed: Rc<RefCell<Vec<SliderValue>>>,
        observed_after_key: Rc<RefCell<Option<(Vec<f64>, f64)>>>,
    ) -> Element {
        let mut rt = use_slider_context().0;
        let obs = observed_after_key.clone();
        use_hook(move || {
            rt.handle_keydown(0, "ArrowRight", false);
            let vals = rt.current_values();
            let valuenow = rt.thumb_attrs(0, None, None, None).aria_valuenow;
            *obs.borrow_mut() = Some((vals, valuenow));
        });
        let _ = proposed;
        rsx! {}
    }

    #[component]
    fn ControlledRejectionApp(
        proposed: Rc<RefCell<Vec<SliderValue>>>,
        observed_after_key: Rc<RefCell<Option<(Vec<f64>, f64)>>>,
    ) -> Element {
        let proposed_cb = proposed.clone();
        rsx! {
            SliderRoot {
                id: "fm-col-1-controlled-slider",
                value: SliderValue::Single(25.0),
                min: 0.0,
                max: 100.0,
                step: 1.0,
                on_value_change: move |next: SliderValue| {
                    // Intentionally reject update (parent does not mutate `value`).
                    proposed_cb.borrow_mut().push(next);
                },
                SliderTrack {
                    SliderRange {}
                }
                SliderThumb { index: 0 }
                ControlledProbe {
                    proposed,
                    observed_after_key,
                }
            }
        }
    }

    let proposed = Rc::new(RefCell::new(Vec::new()));
    let observed = Rc::new(RefCell::new(None));
    let mut dom = VirtualDom::new_with_props(
        ControlledRejectionApp,
        ControlledRejectionAppProps {
            proposed: proposed.clone(),
            observed_after_key: observed.clone(),
        },
    );
    dom.rebuild_in_place();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);

    assert_eq!(*proposed.borrow(), vec![SliderValue::Single(26.0)]);
    let (vals, valuenow) = observed.borrow().clone().expect("probe ran");
    assert_eq!(vals, vec![25.0], "controlled rejection must keep internal values at 25.0");
    assert_eq!(valuenow, 25.0, "controlled rejection must keep aria-valuenow at 25 with zero 1-frame flash");

    // f64::NAN safety: must sanitize to `min` and re-render cleanly without infinite loop.
    #[component]
    fn NanProbe(out: Rc<RefCell<Vec<f64>>>) -> Element {
        let rt = use_slider_context().0;
        *out.borrow_mut() = rt.current_values();
        rsx! {}
    }

    #[component]
    fn NanSliderApp(out: Rc<RefCell<Vec<f64>>>) -> Element {
        rsx! {
            SliderRoot {
                id: "fm-col-1-nan-slider",
                value: SliderValue::Single(f64::NAN),
                min: 10.0,
                max: 100.0,
                SliderTrack { SliderRange {} }
                SliderThumb { index: 0 }
                NanProbe { out }
            }
        }
    }

    let nan_out = Rc::new(RefCell::new(Vec::new()));
    let mut nan_dom = VirtualDom::new_with_props(
        NanSliderApp,
        NanSliderAppProps {
            out: nan_out.clone(),
        },
    );
    nan_dom.rebuild_in_place();
    nan_dom.render_immediate(&mut dioxus::core::NoOpMutations);
    assert_eq!(*nan_out.borrow(), vec![10.0]);
}

#[cfg(feature = "native")]
#[test]
fn slider_native_controlled_rejection_preserves_aria_valuenow() {
    use crate::common::native_fixture::{
        NATIVE_TEST_LOCK, focus_node, mount_native_scenario, send_key_and_pump,
    };
    use keyboard_types::{Code, Key, NamedKey};

    #[component]
    fn NativeControlledRejectionScenario() -> Element {
        let mut last_proposed = use_signal(String::new);
        rsx! {
            div {
                div { id: "ctrl-proposed", "data-val": "{last_proposed}" }
                SliderRoot {
                    id: "ctrl-slider",
                    value: SliderValue::Single(25.0),
                    min: 0.0,
                    max: 100.0,
                    step: 1.0,
                    on_value_change: move |next: SliderValue| {
                        let first = next.to_vec().first().copied().unwrap_or_default();
                        last_proposed.set(format!("{first}"));
                    },
                    SliderTrack { SliderRange {} }
                    SliderThumb {
                        id: "ctrl-slider-thumb",
                        index: 0,
                        aria_label: "Controlled".to_string(),
                    }
                }
            }
        }
    }

    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(NativeControlledRejectionScenario);
    assert_eq!(
        harness.attr("#ctrl-slider-thumb", "aria-valuenow").as_deref(),
        Some("25")
    );

    focus_node(&mut harness, "#ctrl-slider-thumb");
    send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowRight), Code::ArrowRight);

    assert_eq!(
        harness.attr("#ctrl-proposed", "data-val").as_deref(),
        Some("26"),
        "on_value_change must emit proposed value 26"
    );
    assert_eq!(
        harness.attr("#ctrl-slider-thumb", "aria-valuenow").as_deref(),
        Some("25"),
        "rejected controlled slider must retain aria-valuenow=25"
    );
}
