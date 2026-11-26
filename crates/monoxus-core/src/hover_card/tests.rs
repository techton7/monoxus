use crate::foundation::compose::project_attrs_as_child;
use crate::foundation::{
    compose::Slottable,
    overlay::{FloatingLayer, GeometryVars, PlacementAlign, PlacementSide, Point2D, Rect2D},
    shared::ScopeHandle,
    state::DataState,
};

use super::*;

#[test]
fn hover_card_exposes_all_six_parts_and_radix_defaults() {
    assert_eq!(HOVER_CARD_PARTS.len(), 6);
    assert!(HOVER_CARD_PARTS.contains(&HoverCardPart::ContentStatic));

    let scope = ScopeHandle::root("tests").child("hover-card");
    let card = HoverCard::new(scope, false);

    assert_eq!(card.open_delay_ms(), 700);
    assert_eq!(card.close_delay_ms(), 300);
    assert!(card.ignore_non_keyboard_focus());
    assert!(!card.is_disabled());
    assert!(!card.is_open());
    assert_eq!(card.lifecycle().floating().side(), PlacementSide::Bottom);
    assert_eq!(card.lifecycle().floating().align(), PlacementAlign::Center);
    assert_eq!(card.lifecycle().floating().side_offset(), 4.0);
    assert!(!card.lifecycle().dismiss_layer().is_modal());
}

#[test]
fn hover_card_trigger_preserves_anchor_semantics_and_data_disabled_omission() {
    let scope = ScopeHandle::root("tests").child("trigger");
    let card = HoverCard::new(scope, true);

    let enabled = card.trigger(
        Some("https://example.com".to_string()),
        Some("_blank".to_string()),
        Some("noreferrer".to_string()),
        false,
    );
    assert_eq!(enabled.href(), Some("https://example.com"));
    assert_eq!(enabled.target(), Some("_blank"));
    assert_eq!(enabled.rel(), Some("noreferrer"));
    assert_eq!(enabled.aria_disabled(), None);
    assert_eq!(enabled.tabindex(), None);
    assert_eq!(enabled.data_disabled(), None);
    assert_eq!(enabled.data_state(), &DataState::Open);
    assert_eq!(enabled.data_monoxus_hover_card_trigger(), "");

    let disabled = card.trigger(
        Some("https://example.com".to_string()),
        Some("_blank".to_string()),
        None,
        true,
    );
    assert_eq!(disabled.href(), None);
    assert_eq!(disabled.target(), None);
    assert_eq!(disabled.aria_disabled(), Some("true"));
    assert_eq!(disabled.tabindex(), Some(-1));
    assert_eq!(disabled.data_disabled(), Some(""));
    assert_eq!(disabled.data_monoxus_hover_card_trigger(), "");
}

#[test]
fn hover_card_content_css_variables_emit_monoxus_and_radix_aliases() {
    let geometry = GeometryVars::new(
        HOVER_CARD_GEOMETRY_NAMESPACE,
        12.0,
        48.0,
        32.0,
        0.0,
        320.0,
        240.0,
        80.0,
        24.0,
        256.0,
        120.0,
    );

    let vars = HoverCard::content_css_variables(&geometry);
    assert!(vars.contains(&(
        "--monoxus-hover-card-floating-x".to_string(),
        "12px".to_string(),
    )));
    assert!(vars.contains(&(
        "--radix-hover-card-content-transform-origin".to_string(),
        "32px 0px".to_string(),
    )));
    assert!(vars.contains(&(
        "--radix-hover-card-content-available-width".to_string(),
        "320px".to_string(),
    )));
    assert!(vars.contains(&(
        "--radix-hover-card-trigger-width".to_string(),
        "80px".to_string(),
    )));
    assert!(vars.contains(&(
        "--radix-hover-card-trigger-height".to_string(),
        "24px".to_string(),
    )));
}

#[test]
fn hover_card_interaction_state_guards_selection_focus_and_tokens() {
    let mut state = HoverCardInteractionState::default();

    assert!(HoverCardInteractionState::is_touch_pointer("touch"));
    assert!(!HoverCardInteractionState::is_touch_pointer("mouse"));
    assert!(!HoverCardInteractionState::is_touch_pointer("pen"));

    // Request tokens cancel prior pending transitions deterministically.
    let open_tok = state.schedule_open_token();
    state.cancel_pending_open();
    assert_ne!(state.open_request_token, open_tok);

    let close_tok = state.schedule_close_token();
    state.cancel_pending_close();
    assert_ne!(state.close_request_token, close_tok);

    // DEC-HC-1 / D-2: focus-opened hover card ignores pointer leave until blur/Escape/close.
    state.opened_via_focus = true;
    assert!(state.should_ignore_pointer_leave());
    assert!(!state.can_commit_scheduled_close());
    state.opened_via_focus = false;
    assert!(!state.should_ignore_pointer_leave());
    assert!(state.can_commit_scheduled_close());

    // HC-7: active text selection or pointer-down inside content suppresses close.
    state.begin_content_selection();
    assert!(state.contain_selection);
    assert!(state.should_suppress_close());
    assert!(!state.can_commit_scheduled_close());

    state.end_content_pointer_down(true);
    assert!(!state.contain_selection);
    assert!(!state.is_pointer_down_on_content);
    assert!(state.has_selection);
    assert!(state.should_suppress_close());
    assert!(!state.can_commit_scheduled_close());

    state.reset_selection_and_hover();
    assert!(!state.should_suppress_close());
    assert!(state.can_commit_scheduled_close());
}

#[test]
fn hover_card_grace_transit_detects_corridor_and_exits() {
    let trigger = Rect2D::new(100.0, 50.0, 50.0, 20.0);
    let content = Rect2D::new(80.0, 100.0, 90.0, 40.0);
    let exit_point = Point2D::new(125.0, 70.0);

    let transit = HoverCardGraceTransit::new(exit_point, trigger, content, Some(5.0));
    assert_eq!(
        transit.inspect_point(Point2D::new(125.0, 70.0)),
        HoverCardGraceStatus::InTransit
    );
    assert_eq!(
        transit.inspect_point(Point2D::new(125.0, 85.0)),
        HoverCardGraceStatus::InTransit
    );
    assert_eq!(
        transit.inspect_point(Point2D::new(125.0, 115.0)),
        HoverCardGraceStatus::ReachedTarget
    );
    assert_eq!(
        transit.inspect_point(Point2D::new(10.0, 85.0)),
        HoverCardGraceStatus::ExitedCorridor
    );

    // Single-step pointer jump far away clamps the exit anchor to trigger edge and exits immediately.
    let jump_away = Point2D::new(2.0, 2.0);
    let jump_transit = HoverCardGraceTransit::new(jump_away, trigger, content, Some(5.0));
    assert_eq!(
        jump_transit.inspect_point(jump_away),
        HoverCardGraceStatus::ExitedCorridor
    );
}

#[test]
fn hover_card_as_child_projection_and_content_static_attributes() {
    let scope = ScopeHandle::root("tests").child("as-child");
    let card = HoverCard::new(scope, true).with_floating(
        FloatingLayer::new(PlacementSide::Top)
            .with_align(PlacementAlign::Start)
            .with_side_offset(6.0),
    );

    let trigger_attrs = card.trigger(None, None, None, false);
    let (_, proj_trigger, _) =
        project_attrs_as_child(&trigger_attrs, "a", Slottable::new(()));
    assert_eq!(proj_trigger.data_state(), &DataState::Open);

    let content_attrs = card.content();
    assert_eq!(content_attrs.tabindex(), -1);
    assert_eq!(content_attrs.data_side(), "top");
    assert_eq!(content_attrs.data_align(), "start");
    let (_, proj_content, _) =
        project_attrs_as_child(&content_attrs, "div", Slottable::new(()));
    assert_eq!(proj_content.data_monoxus_hover_card_content(), "");

    let static_attrs = card.content_static();
    let (_, proj_static, _) =
        project_attrs_as_child(&static_attrs, "div", Slottable::new(()));
    assert_eq!(proj_static.tabindex(), -1);

    let arrow_attrs = card.arrow();
    let (_, proj_arrow, _) =
        project_attrs_as_child(&arrow_attrs, "svg", Slottable::new(()));
    assert_eq!(proj_arrow.data_side(), "top");
    assert_eq!(proj_arrow.data_monoxus_hover_card_arrow(), "");
}
