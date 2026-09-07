use monoxus::{
    foundation::{
        overlay::{FloatingLayer, PlacementAlign, PlacementSide, PortalHost, Rect, Size},
        shared::ScopeHandle,
    },
    select::Select,
};

#[test]
fn select_floating_layer_flips_to_top_when_space_constrained() {
    let layer = FloatingLayer::new(PlacementSide::Bottom)
        .with_align(PlacementAlign::Start)
        .with_side_offset(4.0);

    let anchor = Rect::new(100.0, 700.0, 200.0, 40.0);
    let content = Size::new(200.0, 150.0);
    let viewport = Size::new(1024.0, 768.0);

    let pos = layer.position_with_available_size(anchor, content, viewport);
    assert_eq!(pos.side(), PlacementSide::Top);
}

#[test]
fn select_floating_layer_stays_bottom_when_space_sufficient() {
    let layer = FloatingLayer::new(PlacementSide::Bottom)
        .with_align(PlacementAlign::Start)
        .with_side_offset(4.0);

    let anchor = Rect::new(100.0, 100.0, 200.0, 40.0);
    let content = Size::new(200.0, 150.0);
    let viewport = Size::new(1024.0, 768.0);

    let pos = layer.position_with_available_size(anchor, content, viewport);
    assert_eq!(pos.side(), PlacementSide::Bottom);
}

#[test]
fn select_portal_and_content_attributes_publish_side_and_host() {
    let scope = ScopeHandle::root("select-test").child("portal-content");
    let portal_host = PortalHost::named("overlay-container");

    let select = Select::new(scope.clone())
        .with_portal_host(portal_host.clone())
        .with_open(true);

    let portal_attrs = select.portal_attributes();
    assert_eq!(portal_attrs.host(), &portal_host);

    let content_attrs = select.content_attributes_with_side(None, PlacementSide::Top);
    assert_eq!(content_attrs.data_side_str(), "top");
    assert_eq!(content_attrs.data_align_str(), "start");
}

#[test]
fn select_dynamic_scroll_reposition_and_flip_contract() {
    let layer = FloatingLayer::new(PlacementSide::Bottom)
        .with_align(PlacementAlign::Start)
        .with_side_offset(4.0);

    let content = Size::new(220.0, 180.0);
    let viewport = Size::new(1024.0, 768.0);

    let initial_anchor = Rect::new(100.0, 650.0, 220.0, 38.0);
    let initial_pos = layer.position_with_available_size(initial_anchor, content, viewport);
    assert_eq!(initial_pos.side(), PlacementSide::Top);

    let scrolled_anchor = Rect::new(100.0, 450.0, 220.0, 38.0);
    let scrolled_pos = layer.position_with_available_size(scrolled_anchor, content, viewport);
    assert_eq!(scrolled_pos.side(), PlacementSide::Bottom);
}
