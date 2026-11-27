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
            child: move |props: Vec<Attribute>| rsx! {
                section {
                    ..props,
                    PinInputCell {
                        index: 0,
                        child: move |cell_props: Vec<Attribute>| rsx! {
                            span {
                                ..cell_props,
                                "0"
                            }
                        }
                    }
                    PinInputCell { index: 1 }
                }
            },
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

#[test]
fn test_pin_input_cell_drop_unregisters_live_index_and_id() {
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Clone)]
    struct RuntimeHolder(Rc<RefCell<Option<PinInputRuntime>>>);
    impl PartialEq for RuntimeHolder {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.0, &other.0)
        }
    }

    #[component]
    fn RuntimeCapturer(holder: RuntimeHolder) -> Element {
        let ctx = use_pin_input_context();
        *holder.0.borrow_mut() = Some(ctx.runtime);
        rsx! {}
    }

    #[component]
    fn ReindexHost(
        runtime_holder: RuntimeHolder,
        controls: Rc<RefCell<Option<(Signal<usize>, Signal<String>, Signal<bool>)>>>,
    ) -> Element {
        let idx = use_signal(|| 0usize);
        let id = use_signal(|| "cell-initial".to_string());
        let mounted = use_signal(|| true);
        *controls.borrow_mut() = Some((idx, id, mounted));

        rsx! {
            PinInputRoot {
                id: "m12-drop-root",
                max_length: 4,
                RuntimeCapturer { holder: runtime_holder }
                if mounted() {
                    PinInputCell {
                        id: id(),
                        index: idx(),
                    }
                }
            }
        }
    }

    let rt_holder = RuntimeHolder(Rc::new(RefCell::new(None)));
    let controls = Rc::new(RefCell::new(None));
    let mut dom = VirtualDom::new_with_props(
        ReindexHost,
        ReindexHostProps {
            runtime_holder: rt_holder.clone(),
            controls: controls.clone(),
        },
    );
    dom.rebuild_in_place();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);

    let rt = rt_holder.0.borrow().clone().expect("PinInputRuntime captured");
    let (mut idx, mut id, mut mounted) = controls.borrow().expect("controls captured");
    assert_eq!(
        rt.cell_ids.borrow().get(&0).map(String::as_str),
        Some("cell-initial")
    );

    // Mutate cell index & id post-mount
    dom.in_scope(ScopeId::APP, || {
        idx.set(2);
        id.set("cell-moved".to_string());
    });
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    assert!(rt.cell_ids.borrow().get(&0).is_none());
    assert_eq!(
        rt.cell_ids.borrow().get(&2).map(String::as_str),
        Some("cell-moved")
    );

    // Unmount cell -> use_drop must unregister live (2, "cell-moved"), leaving map empty
    dom.in_scope(ScopeId::APP, || {
        mounted.set(false);
    });
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    assert!(
        rt.cell_ids.borrow().is_empty(),
        "FM-W2-FRM: PinInputCell drop must unregister live (index, id), remaining: {:?}",
        rt.cell_ids.borrow()
    );
}

#[cfg(feature = "native")]
mod native_cell_context_tests {
    use super::*;
    use crate::common::native_fixture::*;
    use keyboard_types::{Code, Key};

    #[component]
    fn PinCellContextProbe(probe_id: String) -> Element {
        let cell_ctx = use_pin_input_cell_context();
        let cell = cell_ctx.cell();
        let display = cell_ctx
            .display_char()
            .map(|c| c.to_string())
            .unwrap_or_default();
        let disabled = cell_ctx.is_disabled();
        rsx! {
            span {
                id: "{probe_id}",
                "data-index": "{cell.index}",
                "data-active": "{cell.is_active}",
                "data-caret": "{cell.has_fake_caret}",
                "data-char": "{cell.char.map(|c| c.to_string()).unwrap_or_default()}",
                "data-display": "{display}",
                "data-disabled": "{disabled}",
            }
        }
    }

    #[component]
    fn PinCellContextScenario() -> Element {
        let mut disabled = use_signal(|| false);
        rsx! {
            div {
                style: "padding: 16px; width: 320px;",
                button {
                    id: "m12-toggle-disabled",
                    r#type: "button",
                    onclick: move |_| disabled.toggle(),
                    "Toggle Disabled"
                }
                PinInputRoot {
                    id: "m12-pin-root",
                    input_id: "m12-pin-input",
                    max_length: 4,
                    pattern: PinInputPattern::Digits,
                    disabled: disabled(),
                    style: "display: flex; gap: 8px; width: 220px; height: 40px;",
                    for i in 0..4 {
                        PinInputCell {
                            key: "{i}",
                            id: format!("m12-cell-{i}"),
                            index: i,
                            style: "width: 44px; height: 40px;",
                            PinCellContextProbe { probe_id: format!("m12-probe-{i}") }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_native_pin_input_cell_context_updates_reactively_on_focus_and_input() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(PinCellContextScenario);

        // Initial unfocused state
        assert_eq!(
            harness.attr("#m12-probe-0", "data-active").as_deref(),
            Some("false")
        );
        assert_eq!(
            harness.attr("#m12-probe-0", "data-caret").as_deref(),
            Some("false")
        );
        assert_eq!(
            harness.attr("#m12-probe-0", "data-char").as_deref(),
            Some("")
        );

        // Focus input -> cell 0 becomes active with fake caret via reactive PinInputCellContext
        focus_node(&mut harness, "#m12-pin-input");
        assert_eq!(
            harness.attr("#m12-probe-0", "data-active").as_deref(),
            Some("true"),
            "FM-W2-FRM: use_pin_input_cell_context() must reactively update is_active on focus"
        );
        assert_eq!(
            harness.attr("#m12-probe-0", "data-caret").as_deref(),
            Some("true")
        );
        assert_eq!(
            harness.attr("#m12-probe-1", "data-active").as_deref(),
            Some("false")
        );

        // Type '5' -> cell 0 gets char '5' and becomes inactive, cell 1 becomes active with caret
        send_key_and_pump(&mut harness, Key::Character("5".into()), Code::Digit5);
        assert_eq!(
            harness.attr("#m12-probe-0", "data-char").as_deref(),
            Some("5"),
            "FM-W2-FRM: use_pin_input_cell_context() must reactively update char on input"
        );
        assert_eq!(
            harness.attr("#m12-probe-0", "data-display").as_deref(),
            Some("5")
        );
        assert_eq!(
            harness.attr("#m12-probe-0", "data-active").as_deref(),
            Some("false")
        );
        assert_eq!(
            harness.attr("#m12-probe-1", "data-active").as_deref(),
            Some("true")
        );
        assert_eq!(
            harness.attr("#m12-probe-1", "data-caret").as_deref(),
            Some("true")
        );

        // Post-mount disabled toggle propagates reactively to cell context
        click_and_pump(&mut harness, "#m12-toggle-disabled");
        assert_eq!(
            harness.attr("#m12-probe-0", "data-disabled").as_deref(),
            Some("true")
        );
        assert_eq!(
            harness.attr("#m12-probe-1", "data-disabled").as_deref(),
            Some("true")
        );
        assert_eq!(
            harness.attr("#m12-probe-1", "data-active").as_deref(),
            Some("false")
        );
    }
}
