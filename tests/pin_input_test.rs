use monoxus::foundation::compose::project_attrs_as_child;
use dioxus::prelude::*;
use monoxus::foundation::{compose::Slottable, shared::ScopeHandle};
use monoxus::pin_input::*;

#[test]
fn test_pin_input_parts_relationships_and_builtin_patterns() {
    assert_eq!(
        PIN_INPUT_PARTS
            .iter()
            .map(PinInputPart::as_str)
            .collect::<Vec<_>>(),
        ["root", "input", "cell"]
    );

    let scope = ScopeHandle::root("pin-input").child("contract");
    let rels = PinInputRelationships::new(scope.clone());
    assert_eq!(rels.root_id(), scope.token());
    assert_eq!(rels.input_id(), scope.qualify("input"));
    assert_eq!(rels.hidden_input_id(), scope.qualify("hidden-input"));
    assert_eq!(rels.cell_id(0), scope.qualify("cell-0"));

    assert!(PinInputPattern::Digits.matches_str("123456"));
    assert!(!PinInputPattern::Digits.matches_str("12a4"));
    assert!(PinInputPattern::Alpha.matches_str("abcDEF"));
    assert!(!PinInputPattern::Alpha.matches_str("abc1"));
    assert!(PinInputPattern::Alphanumeric.matches_str("a1B2"));
    assert!(!PinInputPattern::Alphanumeric.matches_str("a1-2"));
    assert!(PinInputPattern::Any.matches_str("a1-!$"));

    assert_eq!(
        sanitize_pin_value("1a2$34567", 6, &PinInputPattern::Digits),
        "123456"
    );
    assert_eq!(
        filter_pin_chars("1a2b3C!", 6, &PinInputPattern::Alpha),
        "abC"
    );
}

#[test]
fn test_pin_input_custom_pattern_and_paste_transformer_contract() {
    let mut dom = VirtualDom::new(|| rsx! {});
    dom.rebuild_in_place();
    dom.in_scope(ScopeId::APP, || {
        let hex_only = PinInputPattern::Custom(Callback::new(|s: String| {
            s.chars().all(|c| c.is_ascii_hexdigit())
        }));
        assert!(hex_only.matches_str("deadBEEF"));
        assert!(!hex_only.matches_str("xyz"));
        assert_eq!(sanitize_pin_value("1aZ9f!", 6, &hex_only), "1a9f");

        let strip_hyphens = Callback::new(|s: String| s.replace('-', ""));
        let pasted = apply_pin_splice_with_callbacks(
            "",
            Some(0),
            Some(0),
            "123-456",
            6,
            Some(&strip_hyphens),
            &PinInputPattern::Digits,
        );
        assert_eq!(pasted.value, "123456");
        assert_eq!(pasted.selection_start, 5);
        assert_eq!(pasted.selection_end, 6);
        assert!(pasted.changed);

        let mid_replace = apply_pin_splice_with_callbacks(
            "1234",
            Some(1),
            Some(3),
            "9",
            6,
            None,
            &PinInputPattern::Digits,
        );
        assert_eq!(mid_replace.value, "194");
        assert_eq!(mid_replace.selection_start, 2);
        assert_eq!(mid_replace.selection_end, 2);
    });
}

#[test]
fn test_pin_input_cells_selection_and_on_complete_edge_contract() {
    let focused_empty = compute_pin_cells("", 6, PinInputSelection::collapsed(0), true, false);
    assert_eq!(focused_empty.len(), 6);
    assert!(focused_empty[0].is_active);
    assert!(focused_empty[0].has_fake_caret);
    assert!(!focused_empty[1].is_active);

    let partial = compute_pin_cells("12", 6, PinInputSelection::collapsed(2), true, false);
    assert_eq!(partial[0].char, Some('1'));
    assert_eq!(partial[1].char, Some('2'));
    assert!(partial[2].is_active);
    assert!(partial[2].has_fake_caret);

    let full = compute_pin_cells(
        "123456",
        6,
        PinInputSelection::range(5, 6, PinInputSelectionDirection::Backward),
        true,
        false,
    );
    assert!(full[5].is_active);
    assert!(!full[5].has_fake_caret);
    assert_eq!(full[5].display_char(true, '•'), Some('•'));
    assert_eq!(full[5].display_char(false, '•'), Some('6'));

    assert!(!should_fire_on_complete("1234", "12345", 6));
    assert!(should_fire_on_complete("12345", "123456", 6));
    assert!(!should_fire_on_complete("123456", "123456", 6));
    assert!(!should_fire_on_complete("123456", "123459", 6));
}

#[test]
fn test_pin_input_attributes_and_as_child_projection_contract() {
    let scope = ScopeHandle::root("pin-input").child("attr-contract");
    let def = PinInputDefinition::new(scope)
        .with_max_length(6)
        .with_builtin_pattern(PinInputBuiltinPattern::Digits)
        .with_mask(true)
        .with_disabled(false);

    let root = def.root_attributes(None);
    assert_eq!(root.aria_disabled(), None);
    assert_eq!(root.data_disabled(), None);
    assert_eq!(root.data_monoxus_pin_input_root(), "");

    let active_cell = def.cell_attributes(0, None, true);
    assert_eq!(active_cell.data_active(), Some(""));
    assert_eq!(active_cell.data_inactive(), None);
    assert_eq!(active_cell.data_disabled(), None);

    let inactive_cell = def.cell_attributes(1, None, false);
    assert_eq!(inactive_cell.data_active(), None);
    assert_eq!(inactive_cell.data_inactive(), Some(""));
    assert_eq!(inactive_cell.data_disabled(), None);

    let disabled_def = def.with_disabled(true);
    let disabled_root = disabled_def.root_attributes(None);
    assert_eq!(disabled_root.aria_disabled(), Some("true"));
    assert_eq!(disabled_root.data_disabled(), Some(""));

    let disabled_cell = disabled_def.cell_attributes(0, None, true);
    assert_eq!(disabled_cell.data_active(), None);
    assert_eq!(disabled_cell.data_inactive(), Some(""));
    assert_eq!(disabled_cell.data_disabled(), Some(""));

    let (_, proj_root, _) =
        project_attrs_as_child(&root, "custom-root", Slottable::new("inner"));
    assert_eq!(proj_root, root);
    let (_, proj_cell, _) =
        project_attrs_as_child(&active_cell, "custom-cell", Slottable::new("0"));
    assert_eq!(proj_cell, active_cell);
}

#[component]
fn PinInputBasicVdomApp() -> Element {
    rsx! {
        PinInputRoot {
            id: "vdom-pin-root",
            input_id: "vdom-pin-input",
            default_value: "12".to_string(),
            max_length: 4,
            pattern: PinInputPattern::Digits,
            mask: true,
            PinInputCell { id: "vdom-cell-0", index: 0 }
            PinInputCell { id: "vdom-cell-1", index: 1 }
            PinInputCell { id: "vdom-cell-2", index: 2 }
            PinInputCell { id: "vdom-cell-3", index: 3 }
        }
    }
}

#[component]
fn PinInputRenderChildrenVdomApp() -> Element {
    rsx! {
        PinInputRoot {
            id: "vdom-render-root",
            max_length: 6,
            render_children: move |snap: PinInputSnapshot| rsx! {
                for cell in snap.cells.iter() {
                    PinInputCell {
                        key: "{cell.index}",
                        index: cell.index,
                    }
                }
            }
        }
    }
}

#[component]
fn PinInputAsChildVdomApp() -> Element {
    rsx! {
        PinInputRoot {
            id: "vdom-as-child-root",
            max_length: 2,
            as_child: true,
            render_child: move |rc: PinInputRootRenderChildProps| rsx! {
                section {
                    id: "{rc.attrs.id}",
                    style: "{rc.attrs.style}",
                    "data-monoxus-pin-input-root": "{rc.attrs.data_monoxus_pin_input_root}",
                    {rc.children}
                }
            },
            PinInputCell {
                index: 0,
                as_child: true,
                render_child: move |cell_rc: PinInputCellRenderChildProps| rsx! {
                    span {
                        id: "{cell_rc.attrs.id}",
                        "data-active": cell_rc.attrs.data_active,
                        "data-inactive": cell_rc.attrs.data_inactive,
                        "{cell_rc.display_char.unwrap_or(' ')}"
                    }
                }
            }
            PinInputCell { index: 1 }
        }
    }
}

#[test]
fn test_pin_input_vdom_scenarios_render_cleanly() {
    let mut basic_dom = VirtualDom::new(PinInputBasicVdomApp);
    basic_dom.rebuild_in_place();

    let mut render_dom = VirtualDom::new(PinInputRenderChildrenVdomApp);
    render_dom.rebuild_in_place();

    let mut as_child_dom = VirtualDom::new(PinInputAsChildVdomApp);
    as_child_dom.rebuild_in_place();
}
