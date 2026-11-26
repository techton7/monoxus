use crate::foundation::compose::project_attrs_as_child;
use crate::{
    foundation::{
        compose::Slottable,
        roving_focus::{RovingCandidate, RovingModifiers, resolve_next_stop},
        shared::{Direction, Orientation, ScopeHandle},
        state::DataState,
    },
    separator::SeparatorOrientation,
    toggle_group::{ToggleGroup, ToggleGroupMode},
};

use super::*;

#[test]
fn toolbar_exposes_all_six_parts_and_root_aria_contract() {
    assert_eq!(TOOLBAR_PARTS.len(), 6);
    assert!(TOOLBAR_PARTS.contains(&ToolbarPart::Separator));
    assert!(TOOLBAR_PARTS.contains(&ToolbarPart::ToggleGroup));

    let scope = ScopeHandle::root("tests").child("toolbar");
    let toolbar = Toolbar::new(scope);
    let root = toolbar.root_attributes();

    assert_eq!(root.role(), "toolbar");
    assert_eq!(root.aria_orientation(), "horizontal");
    assert_eq!(root.dir(), "ltr");
    assert_eq!(root.data_orientation(), "horizontal");
    assert_eq!(root.data_toolbar_root(), "");
    assert!(toolbar.loop_focus());
}

#[test]
fn toolbar_separator_inverts_toolbar_orientation() {
    let scope = ScopeHandle::root("tests").child("sep");

    let horizontal = Toolbar::new(scope.clone()).with_orientation(Orientation::Horizontal);
    assert_eq!(
        horizontal.inverted_separator_orientation(),
        SeparatorOrientation::Vertical
    );
    let sep_v = horizontal.separator_attributes(Some("sep-v".into()), false);
    assert_eq!(sep_v.role(), "separator");
    assert_eq!(sep_v.aria_orientation(), Some("vertical"));
    assert_eq!(sep_v.data_orientation(), "vertical");
    assert_eq!(sep_v.data_separator_root(), "");
    assert_eq!(sep_v.data_toolbar_separator(), "");

    let vertical = Toolbar::new(scope).with_orientation(Orientation::Vertical);
    assert_eq!(
        vertical.inverted_separator_orientation(),
        SeparatorOrientation::Horizontal
    );
    let sep_h = vertical.separator_attributes(Some("sep-h".into()), false);
    assert_eq!(sep_h.role(), "separator");
    assert_eq!(sep_h.aria_orientation(), None);
    assert_eq!(sep_h.data_orientation(), "horizontal");

    let decorative = vertical.separator_attributes(None, true);
    assert_eq!(decorative.role(), "none");
    assert_eq!(decorative.aria_orientation(), None);
}

#[test]
fn toolbar_button_and_link_attributes_and_data_disabled_omission() {
    let scope = ScopeHandle::root("tests").child("items");
    let toolbar = Toolbar::new(scope);

    let btn_enabled = toolbar.button_attributes("btn-1".into(), false, 0);
    assert_eq!(btn_enabled.button_type(), "button");
    assert_eq!(btn_enabled.tabindex(), 0);
    assert!(!btn_enabled.is_disabled());
    assert_eq!(btn_enabled.data_disabled(), None);
    assert_eq!(btn_enabled.data_toolbar_button(), "");
    assert_eq!(btn_enabled.data_toolbar_item(), "");

    let btn_disabled = toolbar.button_attributes("btn-2".into(), true, 0);
    assert_eq!(btn_disabled.tabindex(), -1);
    assert!(btn_disabled.is_disabled());
    assert_eq!(btn_disabled.data_disabled(), Some(""));

    let link = toolbar.link_attributes("lnk-1".into(), Some("https://example.com".into()), -1);
    assert_eq!(link.href(), Some("https://example.com"));
    assert_eq!(link.tabindex(), -1);
    assert_eq!(link.data_toolbar_link(), "");
    assert_eq!(link.data_toolbar_item(), "");

    assert!(Toolbar::should_activate_link_on_key(" ", true));
    assert!(!Toolbar::should_activate_link_on_key(" ", false));
    assert!(!Toolbar::should_activate_link_on_key("Enter", true));
}

#[test]
fn toolbar_toggle_group_single_and_multiple_attribute_projection() {
    let scope = ScopeHandle::root("tests").child("tg");
    let toolbar = Toolbar::new(scope.clone()).with_direction(Direction::Rtl);

    let single_group = ToggleGroup::new(scope.child("single"), ToggleGroupMode::Single)
        .with_orientation(toolbar.toggle_group_orientation())
        .with_direction(toolbar.dir())
        .with_roving_focus(false)
        .with_single_value("center");
    let single_attrs = toolbar.toggle_group_attributes(&single_group, None);
    assert_eq!(single_attrs.role(), "radiogroup");
    assert_eq!(single_attrs.dir(), "rtl");
    assert_eq!(single_attrs.data_toolbar_group(), "");
    assert_eq!(single_attrs.data_disabled(), None);

    let item_on = toolbar.toggle_item_attributes(&single_group, "center", None, false, 0);
    assert_eq!(item_on.role(), Some("radio"));
    assert_eq!(item_on.aria_checked(), Some("true"));
    assert_eq!(item_on.aria_pressed(), None);
    assert_eq!(item_on.data_state(), &DataState::On);
    assert_eq!(item_on.data_toolbar_group_item(), "");
    assert_eq!(item_on.data_toolbar_item(), "");

    let multi_group = ToggleGroup::new(scope.child("multi"), ToggleGroupMode::Multiple)
        .with_orientation(toolbar.toggle_group_orientation())
        .with_direction(toolbar.dir())
        .with_roving_focus(false)
        .with_disabled(true);
    let multi_attrs = toolbar.toggle_group_attributes(&multi_group, None);
    assert_eq!(multi_attrs.role(), "group");
    assert_eq!(multi_attrs.data_disabled(), Some(""));

    let item_off = toolbar.toggle_item_attributes(&multi_group, "bold", None, false, 0);
    assert_eq!(item_off.role(), None);
    assert_eq!(item_off.aria_checked(), None);
    assert_eq!(item_off.aria_pressed(), Some("false"));
    assert_eq!(item_off.data_state(), &DataState::Off);
    assert_eq!(item_off.tabindex(), -1);
    assert_eq!(item_off.data_disabled(), Some(""));
}

#[test]
fn toolbar_roving_config_supports_rtl_mirroring_and_loop_clamping() {
    let scope = ScopeHandle::root("tests").child("roving");
    let candidates = vec![
        RovingCandidate::new("btn", "btn", true, false),
        RovingCandidate::new("disabled", "disabled", false, false),
        RovingCandidate::new("link", "link", true, false),
    ];

    let rtl = Toolbar::new(scope.clone())
        .with_direction(Direction::Rtl)
        .with_loop_focus(false);
    let rtl_cfg = rtl.roving_config();

    // RTL horizontal: ArrowLeft moves to next focusable item, skipping disabled.
    assert_eq!(
        resolve_next_stop(
            &rtl_cfg,
            &candidates,
            Some("btn"),
            "ArrowLeft",
            RovingModifiers::none()
        ),
        Some("link".to_string())
    );
    // loop_focus=false clamps at boundary.
    assert_eq!(
        resolve_next_stop(
            &rtl_cfg,
            &candidates,
            Some("link"),
            "ArrowLeft",
            RovingModifiers::none()
        ),
        None
    );

    let ltr_loop = Toolbar::new(scope)
        .with_direction(Direction::Ltr)
        .with_loop_focus(true);
    assert_eq!(
        resolve_next_stop(
            &ltr_loop.roving_config(),
            &candidates,
            Some("link"),
            "ArrowRight",
            RovingModifiers::none()
        ),
        Some("btn".to_string())
    );
}

#[test]
fn toolbar_as_child_projections_preserve_attributes() {
    let scope = ScopeHandle::root("tests").child("proj");
    let toolbar = Toolbar::new(scope.clone());

    let root = toolbar.root_attributes();
    let (_, proj_root, _) = project_attrs_as_child(&root, "div", Slottable::new(()));
    assert_eq!(proj_root.role(), "toolbar");

    let btn = toolbar.button_attributes("b1".into(), false, 0);
    let (_, proj_btn, _) = project_attrs_as_child(&btn, "button", Slottable::new(()));
    assert_eq!(proj_btn.data_toolbar_button(), "");

    let lnk = toolbar.link_attributes("l1".into(), None, -1);
    let (_, proj_lnk, _) = project_attrs_as_child(&lnk, "a", Slottable::new(()));
    assert_eq!(proj_lnk.data_toolbar_link(), "");

    let sep = toolbar.separator_attributes(None, false);
    let (_, proj_sep, _) = project_attrs_as_child(&sep, "hr", Slottable::new(()));
    assert_eq!(proj_sep.data_toolbar_separator(), "");

    let tg = ToggleGroup::new(scope.child("tg"), ToggleGroupMode::Multiple);
    let tg_attrs = toolbar.toggle_group_attributes(&tg, None);
    let (_, proj_tg, _) =
        project_attrs_as_child(&tg_attrs, "div", Slottable::new(()));
    assert_eq!(proj_tg.data_toolbar_group(), "");

    let ti_attrs = toolbar.toggle_item_attributes(&tg, "b", None, false, 0);
    let (_, proj_ti, _) =
        project_attrs_as_child(&ti_attrs, "button", Slottable::new(()));
    assert_eq!(proj_ti.data_toolbar_item(), "");
}
