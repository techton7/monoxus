use crate::foundation::shared::{Direction, Orientation, ScopeHandle};

use super::{
    attrs::{
        ScrollAreaContentAttributes, ScrollAreaCornerAttributes, ScrollAreaRootAttributes,
        ScrollAreaScrollbarAttributes, ScrollAreaThumbAttributes, ScrollAreaViewportAttributes,
    },
    relationships::ScrollAreaRelationships,
    state::{
        ScrollArea, clamp, get_scroll_position_from_pointer, get_thumb_offset_from_scroll,
        get_thumb_size, has_corner, has_thumb, is_overflowing, is_scrollbar_visible,
        is_scrolling_within_scrollbar_bounds, linear_scale, thumb_ratio, transition_scroll_state,
    },
    types::{
        DEFAULT_SCROLL_HIDE_DELAY_MS, SCROLL_AREA_MIN_THUMB_SIZE, SCROLL_AREA_PARTS,
        ScrollAreaPart, ScrollAreaSizes, ScrollAreaStateEvent, ScrollAreaType,
        ScrollAreaVisibilityState, ScrollbarPadding,
    },
};

#[test]
fn test_scroll_area_parts_enum_and_constants() {
    assert_eq!(SCROLL_AREA_PARTS.len(), 6);
    assert_eq!(ScrollAreaPart::Root.as_str(), "root");
    assert_eq!(ScrollAreaPart::Viewport.as_str(), "viewport");
    assert_eq!(ScrollAreaPart::Content.as_str(), "content");
    assert_eq!(ScrollAreaPart::Scrollbar.as_str(), "scrollbar");
    assert_eq!(ScrollAreaPart::Thumb.as_str(), "thumb");
    assert_eq!(ScrollAreaPart::Corner.as_str(), "corner");

    assert_eq!(SCROLL_AREA_MIN_THUMB_SIZE, 18.0);
    assert_eq!(DEFAULT_SCROLL_HIDE_DELAY_MS, 600);

    assert_eq!(ScrollAreaType::default(), ScrollAreaType::Hover);
    assert_eq!(ScrollAreaType::Hover.as_str(), "hover");
    assert_eq!(ScrollAreaType::Scroll.as_str(), "scroll");
    assert_eq!(ScrollAreaType::Auto.as_str(), "auto");
    assert_eq!(ScrollAreaType::Always.as_str(), "always");

    assert_eq!(
        ScrollAreaVisibilityState::default(),
        ScrollAreaVisibilityState::Hidden
    );
    assert_eq!(ScrollAreaVisibilityState::Hidden.as_str(), "hidden");
    assert_eq!(ScrollAreaVisibilityState::Showing.as_str(), "showing");
    assert_eq!(ScrollAreaVisibilityState::Scrolling.as_str(), "scrolling");
    assert_eq!(
        ScrollAreaVisibilityState::Interacting.as_str(),
        "interacting"
    );
    assert_eq!(ScrollAreaVisibilityState::Idle.as_str(), "idle");
    assert_eq!(ScrollAreaVisibilityState::Visible.as_str(), "visible");

    assert!(!ScrollAreaVisibilityState::Hidden.is_visible());
    assert!(ScrollAreaVisibilityState::Scrolling.is_visible());
    assert!(ScrollAreaVisibilityState::Interacting.is_visible());
    assert!(ScrollAreaVisibilityState::Idle.is_visible());
    assert!(ScrollAreaVisibilityState::Visible.is_visible());
}

#[test]
fn test_thumb_ratio_and_has_thumb() {
    assert_eq!(thumb_ratio(0.0, 100.0), 0.0);
    assert_eq!(thumb_ratio(100.0, 0.0), 0.0);
    assert_eq!(thumb_ratio(-50.0, 100.0), 0.0);
    assert_eq!(thumb_ratio(100.0, -50.0), 0.0);

    let sizes_overflow = ScrollAreaSizes::new(1000.0, 200.0, 200.0, ScrollbarPadding::default());
    assert!(has_thumb(sizes_overflow));
    assert!(is_overflowing(sizes_overflow));

    let sizes_no_overflow = ScrollAreaSizes::new(200.0, 200.0, 200.0, ScrollbarPadding::default());
    assert!(!has_thumb(sizes_no_overflow));
    assert!(!is_overflowing(sizes_no_overflow));

    let sizes_content_smaller =
        ScrollAreaSizes::new(100.0, 200.0, 200.0, ScrollbarPadding::default());
    assert!(!has_thumb(sizes_content_smaller));
    assert!(!is_overflowing(sizes_content_smaller));
}

#[test]
fn test_thumb_size_minimum_18px_clamp() {
    // Huge content (100,000px) with small viewport (100px) and track (100px)
    // ratio = 100 / 100,000 = 0.001
    // raw_size = 100 * 0.001 = 0.1px -> clamped to 18.0px
    let huge_content_sizes =
        ScrollAreaSizes::new(100_000.0, 100.0, 100.0, ScrollbarPadding::default());
    let thumb_size = get_thumb_size(huge_content_sizes);
    assert_eq!(thumb_size, 18.0);

    // Normal content: 400px content, 200px viewport, 200px track
    // ratio = 200 / 400 = 0.5
    // raw_size = 200 * 0.5 = 100.0px > 18.0px
    let normal_sizes = ScrollAreaSizes::new(400.0, 200.0, 200.0, ScrollbarPadding::default());
    assert_eq!(get_thumb_size(normal_sizes), 100.0);

    // With track padding: 200px track, padding 10px start + 10px end = 180px usable
    // raw_size = 180 * 0.5 = 90.0px
    let padded_sizes = ScrollAreaSizes::new(400.0, 200.0, 200.0, ScrollbarPadding::new(10.0, 10.0));
    assert_eq!(get_thumb_size(padded_sizes), 90.0);
}

#[test]
fn test_linear_scale_and_clamp() {
    assert_eq!(linear_scale((0.0, 100.0), (0.0, 50.0), 50.0), 25.0);
    assert_eq!(linear_scale((0.0, 100.0), (0.0, 50.0), 0.0), 0.0);
    assert_eq!(linear_scale((0.0, 100.0), (0.0, 50.0), 100.0), 50.0);

    // Zero input diff
    assert_eq!(linear_scale((50.0, 50.0), (0.0, 100.0), 50.0), 0.0);
    // Zero output diff
    assert_eq!(linear_scale((0.0, 100.0), (50.0, 50.0), 25.0), 50.0);

    assert_eq!(clamp(50.0, 0.0, 100.0), 50.0);
    assert_eq!(clamp(-10.0, 0.0, 100.0), 0.0);
    assert_eq!(clamp(150.0, 0.0, 100.0), 100.0);
    assert_eq!(clamp(50.0, 100.0, 0.0), 50.0); // Handles reversed bounds
}

#[test]
fn test_thumb_offset_from_scroll_ltr_and_rtl() {
    // Content 1000px, Viewport 200px, Track 200px, padding 0
    // thumb_size = 200 * (200/1000) = 40px
    // max_scroll = 800px
    // max_thumb = 200 - 40 = 160px
    let sizes = ScrollAreaSizes::new(1000.0, 200.0, 200.0, ScrollbarPadding::default());

    // LTR
    assert_eq!(
        get_thumb_offset_from_scroll(0.0, sizes, Direction::Ltr),
        0.0
    );
    assert_eq!(
        get_thumb_offset_from_scroll(400.0, sizes, Direction::Ltr),
        80.0
    );
    assert_eq!(
        get_thumb_offset_from_scroll(800.0, sizes, Direction::Ltr),
        160.0
    );
    // Momentum clamp beyond bounds in LTR
    assert_eq!(
        get_thumb_offset_from_scroll(1000.0, sizes, Direction::Ltr),
        160.0
    );
    assert_eq!(
        get_thumb_offset_from_scroll(-50.0, sizes, Direction::Ltr),
        0.0
    );

    // RTL (W3C signed negative coordinates: 0 is origin, -800 is fully scrolled)
    assert_eq!(
        get_thumb_offset_from_scroll(0.0, sizes, Direction::Rtl),
        0.0
    );
    assert_eq!(
        get_thumb_offset_from_scroll(-400.0, sizes, Direction::Rtl),
        -80.0
    );
    assert_eq!(
        get_thumb_offset_from_scroll(-800.0, sizes, Direction::Rtl),
        -160.0
    );
    // Momentum clamp beyond bounds in RTL
    assert_eq!(
        get_thumb_offset_from_scroll(-1000.0, sizes, Direction::Rtl),
        -160.0
    );
    assert_eq!(
        get_thumb_offset_from_scroll(50.0, sizes, Direction::Rtl),
        0.0
    );
}

#[test]
fn test_scroll_position_from_pointer_ltr_and_rtl() {
    // Content 1000px, Viewport 200px, Track 200px, padding 0
    // thumb_size = 40px
    // min_pointer = 20px (offset = thumb_size/2 = 20px)
    // max_pointer = 200 - 20 = 180px
    // max_scroll = 800px
    let sizes = ScrollAreaSizes::new(1000.0, 200.0, 200.0, ScrollbarPadding::default());

    // LTR
    assert_eq!(
        get_scroll_position_from_pointer(20.0, 0.0, sizes, Direction::Ltr),
        0.0
    );
    assert_eq!(
        get_scroll_position_from_pointer(100.0, 0.0, sizes, Direction::Ltr),
        400.0
    );
    assert_eq!(
        get_scroll_position_from_pointer(180.0, 0.0, sizes, Direction::Ltr),
        800.0
    );

    // RTL
    assert_eq!(
        get_scroll_position_from_pointer(20.0, 0.0, sizes, Direction::Rtl),
        -800.0
    );
    assert_eq!(
        get_scroll_position_from_pointer(100.0, 0.0, sizes, Direction::Rtl),
        -400.0
    );
    assert_eq!(
        get_scroll_position_from_pointer(180.0, 0.0, sizes, Direction::Rtl),
        0.0
    );
}

#[test]
fn test_is_scrolling_within_scrollbar_bounds() {
    assert!(is_scrolling_within_scrollbar_bounds(50.0, 100.0));
    assert!(!is_scrolling_within_scrollbar_bounds(0.0, 100.0));
    assert!(!is_scrolling_within_scrollbar_bounds(100.0, 100.0));
    assert!(!is_scrolling_within_scrollbar_bounds(-10.0, 100.0));
    assert!(!is_scrolling_within_scrollbar_bounds(120.0, 100.0));
}

#[test]
fn test_scroll_state_machine_transitions() {
    let mut state = ScrollAreaVisibilityState::Hidden;

    // Hidden + Scroll -> Scrolling
    state = transition_scroll_state(state, ScrollAreaStateEvent::Scroll);
    assert_eq!(state, ScrollAreaVisibilityState::Scrolling);

    // Scrolling + PointerEnter -> Interacting
    state = transition_scroll_state(state, ScrollAreaStateEvent::PointerEnter);
    assert_eq!(state, ScrollAreaVisibilityState::Interacting);

    // Interacting + Scroll -> Interacting
    state = transition_scroll_state(state, ScrollAreaStateEvent::Scroll);
    assert_eq!(state, ScrollAreaVisibilityState::Interacting);

    // Interacting + PointerLeave -> Idle
    state = transition_scroll_state(state, ScrollAreaStateEvent::PointerLeave);
    assert_eq!(state, ScrollAreaVisibilityState::Idle);

    // Idle + Scroll -> Scrolling
    state = transition_scroll_state(state, ScrollAreaStateEvent::Scroll);
    assert_eq!(state, ScrollAreaVisibilityState::Scrolling);

    // Scrolling + ScrollEnd -> Idle
    state = transition_scroll_state(state, ScrollAreaStateEvent::ScrollEnd);
    assert_eq!(state, ScrollAreaVisibilityState::Idle);

    // Idle + Hide -> Hidden
    state = transition_scroll_state(state, ScrollAreaStateEvent::Hide);
    assert_eq!(state, ScrollAreaVisibilityState::Hidden);
}

#[test]
fn test_visibility_across_all_scroll_area_types() {
    // Hover mode: only visible when hover_visible is true AND overflowing is true
    assert!(!is_scrollbar_visible(
        ScrollAreaType::Hover,
        ScrollAreaVisibilityState::Hidden,
        false,
        true
    ));
    assert!(!is_scrollbar_visible(
        ScrollAreaType::Hover,
        ScrollAreaVisibilityState::Hidden,
        true,
        false
    ));
    assert!(is_scrollbar_visible(
        ScrollAreaType::Hover,
        ScrollAreaVisibilityState::Hidden,
        true,
        true
    ));

    // Scroll mode: visible when scroll_state != Hidden
    assert!(!is_scrollbar_visible(
        ScrollAreaType::Scroll,
        ScrollAreaVisibilityState::Hidden,
        false,
        true
    ));
    assert!(is_scrollbar_visible(
        ScrollAreaType::Scroll,
        ScrollAreaVisibilityState::Scrolling,
        false,
        true
    ));
    assert!(is_scrollbar_visible(
        ScrollAreaType::Scroll,
        ScrollAreaVisibilityState::Interacting,
        false,
        true
    ));
    assert!(is_scrollbar_visible(
        ScrollAreaType::Scroll,
        ScrollAreaVisibilityState::Idle,
        false,
        true
    ));

    // Auto mode: visible when overflowing
    assert!(!is_scrollbar_visible(
        ScrollAreaType::Auto,
        ScrollAreaVisibilityState::Hidden,
        false,
        false
    ));
    assert!(is_scrollbar_visible(
        ScrollAreaType::Auto,
        ScrollAreaVisibilityState::Hidden,
        false,
        true
    ));

    // Always mode: always visible
    assert!(is_scrollbar_visible(
        ScrollAreaType::Always,
        ScrollAreaVisibilityState::Hidden,
        false,
        false
    ));
}

#[test]
fn test_corner_visibility_conditions() {
    // Both visible in Auto mode
    assert!(has_corner(ScrollAreaType::Auto, true, true));
    // One invisible
    assert!(!has_corner(ScrollAreaType::Auto, true, false));
    assert!(!has_corner(ScrollAreaType::Auto, false, true));
    assert!(!has_corner(ScrollAreaType::Auto, false, false));

    // Scroll mode never renders corner (Radix UI spec)
    assert!(!has_corner(ScrollAreaType::Scroll, true, true));

    // Always mode renders corner when both visible
    assert!(has_corner(ScrollAreaType::Always, true, true));
}

#[test]
fn test_relationships_deterministic_ids() {
    let scope = ScopeHandle::root("scroll-area").child("test-instance");
    let rels = ScrollAreaRelationships::new(scope.clone());

    assert_eq!(rels.scope(), &scope);
    assert_eq!(rels.root_id(), scope.token());
    assert_eq!(rels.viewport_id(), scope.qualify("viewport"));
    assert_eq!(rels.content_id(), scope.qualify("content"));
    assert_eq!(rels.scrollbar_x_id(), scope.qualify("scrollbar-x"));
    assert_eq!(rels.scrollbar_y_id(), scope.qualify("scrollbar-y"));
    assert_eq!(rels.thumb_x_id(), scope.qualify("thumb-x"));
    assert_eq!(rels.thumb_y_id(), scope.qualify("thumb-y"));
    assert_eq!(rels.corner_id(), scope.qualify("corner"));

    assert_eq!(
        rels.scrollbar_id(Orientation::Horizontal),
        scope.qualify("scrollbar-x")
    );
    assert_eq!(
        rels.scrollbar_id(Orientation::Vertical),
        scope.qualify("scrollbar-y")
    );
    assert_eq!(
        rels.thumb_id(Orientation::Horizontal),
        scope.qualify("thumb-x")
    );
    assert_eq!(
        rels.thumb_id(Orientation::Vertical),
        scope.qualify("thumb-y")
    );
}

#[test]
fn test_attributes_contracts_and_strict_disabled_omission() {
    let root = ScrollAreaRootAttributes::new("root-id")
        .with_dir(Some(Direction::Ltr))
        .with_disabled(false);
    assert_eq!(root.id(), "root-id");
    assert_eq!(root.dir_str(), Some("ltr"));
    assert_eq!(root.data_disabled(), None);
    assert_eq!(root.data_monoxus_scroll_area_root(), "");

    let root_disabled = ScrollAreaRootAttributes::new("root-id").with_disabled(true);
    assert_eq!(root_disabled.data_disabled(), Some(""));

    let viewport = ScrollAreaViewportAttributes::new("vp-id");
    assert_eq!(viewport.id(), "vp-id");
    assert_eq!(viewport.tabindex(), 0);
    assert_eq!(viewport.data_disabled(), None);
    assert_eq!(viewport.data_monoxus_scroll_area_viewport(), "");

    let content = ScrollAreaContentAttributes::new("content-id");
    assert_eq!(content.id(), "content-id");
    assert_eq!(content.role(), "presentation");
    assert_eq!(content.data_monoxus_scroll_area_content(), "");

    let scrollbar = ScrollAreaScrollbarAttributes::new("sb-id", Orientation::Vertical, "vp-id")
        .with_visible(true)
        .with_disabled(false);
    assert_eq!(scrollbar.role(), "scrollbar");
    assert_eq!(scrollbar.aria_orientation(), "vertical");
    assert_eq!(scrollbar.aria_controls(), "vp-id");
    assert_eq!(scrollbar.data_orientation(), "vertical");
    assert_eq!(scrollbar.data_state(), "visible");
    assert_eq!(scrollbar.data_disabled(), None);
    assert_eq!(scrollbar.data_monoxus_scroll_area_scrollbar(), "");

    let thumb = ScrollAreaThumbAttributes::new("th-id").with_visible(false);
    assert_eq!(thumb.id(), "th-id");
    assert_eq!(thumb.data_state(), "hidden");
    assert_eq!(thumb.data_disabled(), None);
    assert_eq!(thumb.data_monoxus_scroll_area_thumb(), "");

    let corner = ScrollAreaCornerAttributes::new("cn-id");
    assert_eq!(corner.id(), "cn-id");
    assert_eq!(corner.data_state(), "visible");
    assert_eq!(corner.data_monoxus_scroll_area_corner(), "");
}

#[test]
fn test_scroll_area_pure_state_builder() {
    let scope = ScopeHandle::root("scroll-area").child("demo");
    let rels = ScrollAreaRelationships::new(scope);

    let sizes_x = ScrollAreaSizes::new(800.0, 200.0, 200.0, ScrollbarPadding::default());
    let sizes_y = ScrollAreaSizes::new(1200.0, 300.0, 300.0, ScrollbarPadding::default());

    let area = ScrollArea::new(rels.clone(), ScrollAreaType::Hover)
        .with_dir(Direction::Rtl)
        .with_scroll_hide_delay(800)
        .with_sizes(Orientation::Horizontal, sizes_x)
        .with_sizes(Orientation::Vertical, sizes_y)
        .with_scrollbar_enabled(Orientation::Horizontal, true)
        .with_scrollbar_enabled(Orientation::Vertical, true)
        .with_hover_visible(true);

    assert_eq!(area.dir(), Direction::Rtl);
    assert_eq!(area.scroll_hide_delay(), 800);
    assert!(area.is_scrollbar_enabled(Orientation::Horizontal));
    assert!(area.is_scrollbar_enabled(Orientation::Vertical));
    assert!(area.is_overflowing(Orientation::Horizontal));
    assert!(area.is_overflowing(Orientation::Vertical));
    assert!(area.is_scrollbar_visible(Orientation::Horizontal));
    assert!(area.is_scrollbar_visible(Orientation::Vertical));
    assert!(area.has_corner());
    assert_eq!(area.thumb_size(Orientation::Horizontal), 50.0);
    assert_eq!(area.thumb_size(Orientation::Vertical), 75.0);

    let area_both = ScrollArea::new(rels, ScrollAreaType::Hover)
        .with_sizes(Orientation::Both, sizes_x)
        .with_scrollbar_enabled(Orientation::Both, true);
    assert!(area_both.is_scrollbar_enabled(Orientation::Both));
    assert_eq!(area_both.sizes(Orientation::Both), sizes_x);
}
