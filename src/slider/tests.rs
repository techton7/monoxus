use super::*;
use crate::foundation::shared::ScopeHandle;

#[test]
fn test_runtime_range_and_tick_contain_alignment() {
    use dioxus::prelude::*;

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
            def: Signal::new(def.clone()),
            rels: rels.clone(),
            values: Signal::new(vec![25.0]),
            values_ctl: None,
            is_single: Signal::new(true),
            active_thumb: Signal::new(None),
            is_dragging: Signal::new(false),
            drag_start_values: Signal::new(None),
            focused_thumb: Signal::new(None),
            on_value_change: CopyValue::new(None),
            on_value_commit: CopyValue::new(None),
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
        rt.is_single.set(false);
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
        rt.def.write().thumb_alignment = SliderThumbAlignment::Exact;
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
