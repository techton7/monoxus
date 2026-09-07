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

#[test]
fn select_content_reference_hidden_contract() {
    let scope = ScopeHandle::root("select-test").child("ref-hidden");
    let select = Select::new(scope.clone()).with_open(true);

    let mut attrs = select.content_attributes(None);
    assert!(!attrs.is_reference_hidden());
    assert_eq!(attrs.data_reference_hidden_str(), None);

    attrs.reference_hidden = true;
    assert!(attrs.is_reference_hidden());
    assert_eq!(attrs.data_reference_hidden_str(), Some("true"));
}

#[test]
fn select_pointer_down_outside_event_prevent_default_contract() {
    use monoxus::select::PointerDownOutsideEvent;

    let evt = PointerDownOutsideEvent::new();
    assert!(evt.default_action_enabled());

    let evt_clone = evt.clone();
    evt_clone.prevent_default();

    // prevent_default on clone mutates shared cell
    assert!(!evt.default_action_enabled());
    assert!(!evt_clone.default_action_enabled());
}

#[test]
fn select_content_phase35_props_and_attributes_contract() {
    let scope = ScopeHandle::root("select-test").child("phase35");
    let select = Select::new(scope.clone()).with_open(true);

    let attrs = select.content_attributes(None);
    // baseline assertions
    assert_eq!(attrs.data_state_str(), "open");
    assert_eq!(attrs.data_side_str(), "bottom");
    assert_eq!(attrs.data_align_str(), "start");
}

#[test]
fn select_custom_anchor_and_prevent_overflow_text_selection_contract() {
    use monoxus::foundation::overlay::{FloatingLayer, PlacementAlign, PlacementSide, Rect, Size};

    // 1. Custom anchor dynamic floating placement:
    // If standard trigger is constrained at bottom, placement flips to Top.
    // But if custom anchor is at a spacious location (e.g. y=100), placement stays at Bottom!
    let layer = FloatingLayer::new(PlacementSide::Bottom)
        .with_align(PlacementAlign::Start)
        .with_side_offset(4.0);

    let default_trigger_anchor = Rect::new(100.0, 700.0, 200.0, 40.0);
    let custom_target_anchor = Rect::new(100.0, 100.0, 200.0, 40.0);
    let content = Size::new(200.0, 150.0);
    let viewport = Size::new(1024.0, 768.0);

    let default_pos = layer.position_with_available_size(default_trigger_anchor, content, viewport);
    assert_eq!(default_pos.side(), PlacementSide::Top);

    // Custom anchor at spacious coordinate stays at preferred bottom
    let custom_pos = layer.position_with_available_size(custom_target_anchor, content, viewport);
    assert_eq!(custom_pos.side(), PlacementSide::Bottom);
}

#[test]
fn select_content_force_mount_state_contract() {
    let scope = ScopeHandle::root("select-test").child("force-mount");
    let select = Select::new(scope.clone()).with_open(false);

    let attrs = select.content_attributes(None);
    // Closed state
    assert_eq!(attrs.data_state_str(), "closed");
}


