use monoxus::foundation::overlay::{FloatingLayer, PlacementAlign, PlacementSide, Rect, Size};

#[test]
fn floating_layers_flip_and_publish_runtime_arrow_contracts() {
    let flipped = FloatingLayer::new(PlacementSide::Top)
        .with_align(PlacementAlign::Start)
        .with_side_offset(8.0)
        .position_with_available_size(
            Rect::new(24.0, 8.0, 120.0, 38.0),
            Size::new(184.0, 72.0),
            Size::new(240.0, 140.0),
        );
    let constrained = FloatingLayer::new(PlacementSide::Bottom)
        .with_align(PlacementAlign::Center)
        .with_side_offset(8.0)
        .position_with_available_size(
            Rect::new(0.0, 30.0, 16.0, 20.0),
            Size::new(100.0, 40.0),
            Size::new(120.0, 120.0),
        );

    assert_eq!(flipped.side(), PlacementSide::Bottom);
    assert_eq!(flipped.align(), PlacementAlign::Start);
    assert_eq!(flipped.geometry().x(), 24.0);
    assert_eq!(flipped.geometry().y(), 54.0);
    assert_eq!(flipped.arrow().x(), Some(60.0));
    assert_eq!(flipped.arrow().y(), None);
    assert!(!flipped.arrow().hidden());
    assert!(!flipped.reference_hidden());

    assert_eq!(constrained.side(), PlacementSide::Bottom);
    assert!(constrained.arrow().hidden());
    assert_eq!(constrained.arrow().x(), None);
    assert_eq!(constrained.arrow().y(), None);
    assert!(!constrained.reference_hidden());
}

#[test]
fn floating_layers_can_flag_detached_references_for_hidden_popovers() {
    let hidden = FloatingLayer::new(PlacementSide::Bottom)
        .with_hide_when_detached(true)
        .position_with_available_size(
            Rect::new(24.0, -80.0, 120.0, 38.0),
            Size::new(184.0, 72.0),
            Size::new(240.0, 140.0),
        );
    let still_visible = FloatingLayer::new(PlacementSide::Bottom)
        .with_hide_when_detached(true)
        .position_with_available_size(
            Rect::new(24.0, -8.0, 120.0, 38.0),
            Size::new(184.0, 72.0),
            Size::new(240.0, 140.0),
        );

    assert!(hidden.reference_hidden());
    assert!(hidden.arrow().hidden());
    assert_eq!(hidden.arrow().x(), None);
    assert_eq!(hidden.arrow().y(), None);
    assert!(!still_visible.reference_hidden());
}

#[test]
fn floating_layers_keep_attached_popovers_following_anchors_offscreen() {
    let placement = FloatingLayer::new(PlacementSide::Bottom)
        .with_align(PlacementAlign::Start)
        .with_side_offset(12.0)
        .position_with_available_size(
            Rect::new(24.0, -198.0, 120.0, 35.0),
            Size::new(184.0, 209.0),
            Size::new(1280.0, 720.0),
        );

    assert_eq!(placement.side(), PlacementSide::Bottom);
    assert!(placement.geometry().y() < 0.0);
    assert_eq!(placement.geometry().y(), -151.0);
    assert!(!placement.reference_hidden());
}
