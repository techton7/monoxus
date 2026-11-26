use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use monoxus::foundation::shared::{Direction, Orientation};
use monoxus::toolbar::{
    Toolbar, ToolbarButton, ToolbarLink, ToolbarRoot, ToolbarRootContext, ToolbarSeparator,
    ToolbarToggleGroup, ToolbarToggleGroupMode, ToolbarToggleItem,
};

#[derive(Clone, Default, PartialEq)]
struct ToolbarProbeResults {
    role: Option<String>,
    aria_orientation: Option<String>,
    dir: Option<String>,
    sep_role: Option<String>,
    sep_aria_orientation: Option<String>,
    sep_data_orientation: Option<String>,
    roving_candidate_count: usize,
}

#[component]
fn ToolbarProbe(results: Rc<RefCell<ToolbarProbeResults>>) -> Element {
    let ctx = use_context::<ToolbarRootContext>();
    let state = ctx.runtime.state();
    let root = state.root_attributes();
    let sep = state.separator_attributes(None, false);
    let candidates = ctx.runtime.roving.candidates();

    *results.borrow_mut() = ToolbarProbeResults {
        role: Some(root.role().to_string()),
        aria_orientation: Some(root.aria_orientation().to_string()),
        dir: Some(root.dir().to_string()),
        sep_role: Some(sep.role().to_string()),
        sep_aria_orientation: sep.aria_orientation().map(str::to_string),
        sep_data_orientation: Some(sep.data_orientation().to_string()),
        roving_candidate_count: candidates.len(),
    };
    rsx! {}
}

#[component]
fn ToolbarCompoundSample(
    orientation: Orientation,
    dir: Direction,
    probe_out: Rc<RefCell<ToolbarProbeResults>>,
) -> Element {
    rsx! {
        Toolbar {
            id: "tb-compound",
            orientation: orientation,
            dir: dir,
            ToolbarButton {
                id: "tb-btn-cut",
                value: "cut".to_string(),
                "Cut"
            }
            ToolbarSeparator { id: "tb-sep-1" }
            ToolbarToggleGroup {
                id: "tb-tg-align",
                r#type: ToolbarToggleGroupMode::Single,
                default_value: "center".to_string(),
                ToolbarToggleItem { id: "tb-align-left", value: "left".to_string(), "Left" }
                ToolbarToggleItem { id: "tb-align-center", value: "center".to_string(), "Center" }
            }
            ToolbarSeparator { id: "tb-sep-2", decorative: true }
            ToolbarToggleGroup {
                id: "tb-tg-marks",
                r#type: ToolbarToggleGroupMode::Multiple,
                default_values: vec!["bold".to_string()],
                ToolbarToggleItem { id: "tb-mark-bold", value: "bold".to_string(), "Bold" }
                ToolbarToggleItem { id: "tb-mark-italic", value: "italic".to_string(), "Italic" }
            }
            ToolbarLink {
                id: "tb-link-docs",
                href: "https://example.com",
                value: "docs".to_string(),
                "Docs"
            }
            ToolbarProbe { results: probe_out }
        }
    }
}

#[test]
fn test_toolbar_compound_parts_orientation_inversion_and_separator_exclusion() {
    let probe_h = Rc::new(RefCell::new(ToolbarProbeResults::default()));
    let mut dom_h = VirtualDom::new_with_props(
        ToolbarCompoundSample,
        ToolbarCompoundSampleProps {
            orientation: Orientation::Horizontal,
            dir: Direction::Ltr,
            probe_out: probe_h.clone(),
        },
    );
    dom_h.rebuild_in_place();

    let h = probe_h.borrow();
    assert_eq!(h.role.as_deref(), Some("toolbar"));
    assert_eq!(h.aria_orientation.as_deref(), Some("horizontal"));
    assert_eq!(h.dir.as_deref(), Some("ltr"));
    assert_eq!(h.sep_role.as_deref(), Some("separator"));
    assert_eq!(h.sep_aria_orientation.as_deref(), Some("vertical"));
    assert_eq!(h.sep_data_orientation.as_deref(), Some("vertical"));
    // 1 button + 2 single items + 2 multiple items + 1 link = 6 (2 separators excluded)
    assert_eq!(h.roving_candidate_count, 6);

    let probe_v = Rc::new(RefCell::new(ToolbarProbeResults::default()));
    let mut dom_v = VirtualDom::new_with_props(
        ToolbarCompoundSample,
        ToolbarCompoundSampleProps {
            orientation: Orientation::Vertical,
            dir: Direction::Rtl,
            probe_out: probe_v.clone(),
        },
    );
    dom_v.rebuild_in_place();

    let v = probe_v.borrow();
    assert_eq!(v.aria_orientation.as_deref(), Some("vertical"));
    assert_eq!(v.dir.as_deref(), Some("rtl"));
    assert_eq!(v.sep_data_orientation.as_deref(), Some("horizontal"));
    assert_eq!(v.sep_aria_orientation, None);
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use crate::common::native_fixture::*;
    use keyboard_types::{Code, Key, NamedKey};

    #[component]
    fn NativeToolbarFullScenario(
        orientation: Orientation,
        dir: Direction,
        loop_focus: bool,
        link_clicks: Rc<RefCell<usize>>,
    ) -> Element {
        let on_link_click = link_clicks.clone();
        rsx! {
            div {
                style: "padding: 20px;",
                ToolbarRoot {
                    id: "native-toolbar",
                    orientation: orientation,
                    dir: dir,
                    loop_focus: loop_focus,
                    ToolbarButton {
                        id: "ntb-btn-1",
                        value: "btn-1".to_string(),
                        "Undo"
                    }
                    ToolbarSeparator { id: "ntb-sep-1" }
                    ToolbarButton {
                        id: "ntb-btn-disabled",
                        value: "btn-disabled".to_string(),
                        disabled: true,
                        "Disabled"
                    }
                    ToolbarToggleGroup {
                        id: "ntb-tg-single",
                        r#type: ToolbarToggleGroupMode::Single,
                        default_value: "left".to_string(),
                        ToolbarToggleItem { id: "ntb-single-left", value: "left".to_string(), "Left" }
                        ToolbarToggleItem { id: "ntb-single-right", value: "right".to_string(), "Right" }
                    }
                    ToolbarSeparator { id: "ntb-sep-2" }
                    ToolbarToggleGroup {
                        id: "ntb-tg-multi",
                        r#type: ToolbarToggleGroupMode::Multiple,
                        default_values: vec!["bold".to_string()],
                        ToolbarToggleItem { id: "ntb-multi-bold", value: "bold".to_string(), "B" }
                        ToolbarToggleItem { id: "ntb-multi-italic", value: "italic".to_string(), "I" }
                    }
                    ToolbarLink {
                        id: "ntb-link",
                        href: "https://example.com",
                        value: "link".to_string(),
                        on_click: move |_| *on_link_click.borrow_mut() += 1,
                        "Help"
                    }
                }
            }
        }
    }

    #[test]
    fn native_toolbar_unified_roving_focus_skips_separators_and_disabled() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let clicks = Rc::new(RefCell::new(0usize));
        let clicks_prop = clicks.clone();
        let mut harness = mount_native_scenario_with_props(
            Orientation::Horizontal,
            Direction::Ltr,
            true,
            clicks_prop,
        );

        assert_eq!(harness.attr("#native-toolbar", "role").as_deref(), Some("toolbar"));
        assert_eq!(
            harness.attr("#ntb-sep-1", "data-orientation").as_deref(),
            Some("vertical")
        );
        assert_eq!(
            harness.attr("#ntb-btn-disabled", "data-disabled").as_deref(),
            Some("")
        );
        assert_eq!(harness.attr("#ntb-btn-1", "data-disabled"), None);

        // Focus first button and navigate across groups skipping separator & disabled button
        focus_node(&mut harness, "#ntb-btn-1");
        assert_native_focus_id(&harness, "ntb-btn-1");
        assert_eq!(harness.attr("#ntb-btn-1", "tabindex").as_deref(), Some("0"));

        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowRight), Code::ArrowRight);
        assert_native_focus_id(&harness, "ntb-single-left");
        assert_eq!(harness.attr("#ntb-single-left", "tabindex").as_deref(), Some("0"));
        assert_eq!(harness.attr("#ntb-btn-1", "tabindex").as_deref(), Some("-1"));

        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowRight), Code::ArrowRight);
        assert_native_focus_id(&harness, "ntb-single-right");

        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowRight), Code::ArrowRight);
        assert_native_focus_id(&harness, "ntb-multi-bold");

        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowRight), Code::ArrowRight);
        assert_native_focus_id(&harness, "ntb-multi-italic");

        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowRight), Code::ArrowRight);
        assert_native_focus_id(&harness, "ntb-link");

        // Wrap around at end when loop_focus = true
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowRight), Code::ArrowRight);
        assert_native_focus_id(&harness, "ntb-btn-1");

        // End / Home keys
        send_key_and_pump(&mut harness, Key::Named(NamedKey::End), Code::End);
        assert_native_focus_id(&harness, "ntb-link");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Home), Code::Home);
        assert_native_focus_id(&harness, "ntb-btn-1");
    }

    #[test]
    fn native_toolbar_toggle_groups_and_link_space_activation_and_nested_guard() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let clicks = Rc::new(RefCell::new(0usize));

        #[component]
        fn ToolbarInteractiveScenario(clicks_out: Rc<RefCell<usize>>) -> Element {
            let c1 = clicks_out.clone();
            rsx! {
                div {
                    ToolbarRoot {
                        id: "tb-int",
                        ToolbarToggleGroup {
                            id: "tg-single",
                            r#type: ToolbarToggleGroupMode::Single,
                            default_value: "a".to_string(),
                            ToolbarToggleItem { id: "tg-s-a", value: "a".to_string(), "A" }
                            ToolbarToggleItem { id: "tg-s-b", value: "b".to_string(), "B" }
                        }
                        ToolbarToggleGroup {
                            id: "tg-multi",
                            r#type: ToolbarToggleGroupMode::Multiple,
                            default_values: vec!["x".to_string()],
                            ToolbarToggleItem { id: "tg-m-x", value: "x".to_string(), "X" }
                            ToolbarToggleItem { id: "tg-m-y", value: "y".to_string(), "Y" }
                        }
                        ToolbarLink {
                            id: "tb-int-link",
                            href: "https://example.com",
                            on_click: move |_| *c1.borrow_mut() += 1,
                            "Go"
                            input { id: "tb-link-nested-input", r#type: "text" }
                        }
                    }
                }
            }
        }

        let mut harness = {
            let opts = blitz_test_harness::HarnessOptions::default();
            let mut doc = dioxus_native_dom::DioxusDocument::new(
                VirtualDom::new_with_props(
                    ToolbarInteractiveScenario,
                    ToolbarInteractiveScenarioProps {
                        clicks_out: clicks.clone(),
                    },
                ),
                blitz_dom::DocumentConfig {
                    viewport: Some(blitz_traits::shell::Viewport::new(
                        opts.width,
                        opts.height,
                        opts.scale,
                        opts.color_scheme,
                    )),
                    ..Default::default()
                },
            );
            let native_doc = oxidase::dom::Document::from_base(doc.inner.clone());
            native_doc.attach_engine_bridges();
            oxidase::dom::Document::set_current(Some(native_doc.clone()));
            doc.vdom.in_scope(dioxus::core::ScopeId::ROOT, move || {
                dioxus::prelude::provide_context(native_doc);
            });
            doc.initial_build();
            let mut h = blitz_test_harness::Harness::wrap(doc);
            drain_polls_and_resolve(&mut h);
            h
        };

        // Single toggle mutual exclusion
        assert_eq!(harness.attr("#tg-s-a", "aria-checked").as_deref(), Some("true"));
        assert_eq!(harness.attr("#tg-s-b", "aria-checked").as_deref(), Some("false"));
        click_and_pump(&mut harness, "#tg-s-b");
        assert_eq!(harness.attr("#tg-s-a", "aria-checked").as_deref(), Some("false"));
        assert_eq!(harness.attr("#tg-s-b", "aria-checked").as_deref(), Some("true"));

        // Multiple toggle independent selection via Enter/Space
        focus_node(&mut harness, "#tg-m-y");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
        assert_eq!(harness.attr("#tg-m-x", "aria-pressed").as_deref(), Some("true"));
        assert_eq!(harness.attr("#tg-m-y", "aria-pressed").as_deref(), Some("true"));

        // #3232 guard: Space on nested input inside ToolbarLink must NOT trigger ToolbarLink click
        focus_node(&mut harness, "#tb-link-nested-input");
        send_key_and_pump(
            &mut harness,
            Key::Character(" ".to_string()),
            Code::Space,
        );
        assert_eq!(*clicks.borrow(), 0);

        // Space directly on ToolbarLink triggers click
        focus_node(&mut harness, "#tb-int-link");
        send_key_and_pump(
            &mut harness,
            Key::Character(" ".to_string()),
            Code::Space,
        );
        assert_eq!(*clicks.borrow(), 1);
    }

    #[test]
    fn native_toolbar_rtl_vertical_and_nested_isolation() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        #[component]
        fn NestedAndRtlScenario() -> Element {
            rsx! {
                div {
                    ToolbarRoot {
                        id: "outer-tb",
                        dir: Direction::Rtl,
                        loop_focus: false,
                        ToolbarButton { id: "outer-1", value: "o1".to_string(), "O1" }
                        div {
                            ToolbarRoot {
                                id: "inner-tb",
                                orientation: Orientation::Vertical,
                                ToolbarButton { id: "inner-1", value: "i1".to_string(), "I1" }
                                ToolbarSeparator { id: "inner-sep" }
                                ToolbarButton { id: "inner-2", value: "i2".to_string(), "I2" }
                            }
                        }
                        ToolbarButton { id: "outer-2", value: "o2".to_string(), "O2" }
                    }
                }
            }
        }

        let mut harness = mount_native_scenario(NestedAndRtlScenario);

        // RTL outer toolbar: ArrowLeft moves o1 -> o2, skipping inner toolbar completely
        focus_node(&mut harness, "#outer-1");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowLeft), Code::ArrowLeft);
        assert_native_focus_id(&harness, "outer-2");

        // loop_focus=false clamps at o2
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowLeft), Code::ArrowLeft);
        assert_native_focus_id(&harness, "outer-2");

        // Vertical inner toolbar: separator flips to horizontal, ArrowDown moves i1 -> i2
        assert_eq!(
            harness.attr("#inner-sep", "data-orientation").as_deref(),
            Some("horizontal")
        );
        focus_node(&mut harness, "#inner-1");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
        assert_native_focus_id(&harness, "inner-2");
    }

    #[test]
    fn native_toolbar_mixed_single_tab_stop_invariant() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let clicks = Rc::new(RefCell::new(0usize));
        let mut harness = mount_native_scenario_with_props(
            Orientation::Horizontal,
            Direction::Ltr,
            true,
            clicks,
        );

        let all_items = [
            "#ntb-btn-1",
            "#ntb-btn-disabled",
            "#ntb-single-left",
            "#ntb-single-right",
            "#ntb-multi-bold",
            "#ntb-multi-italic",
            "#ntb-link",
        ];

        let count_tabindex_zero =
            |h: &blitz_test_harness::Harness<dioxus_native_dom::DioxusDocument>| {
                all_items
                    .iter()
                    .filter(|sel| h.attr(sel, "tabindex").as_deref() == Some("0"))
                    .count()
            };

        // FM-COL-4: mixed Toolbar (Button + ToggleGroup/ToggleItem + Link) must maintain exactly one tabindex="0".
        assert_eq!(
            count_tabindex_zero(&harness),
            1,
            "initial mixed toolbar must expose exactly one tabindex=0"
        );

        focus_node(&mut harness, "#ntb-btn-1");
        for _ in 0..6 {
            send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowRight), Code::ArrowRight);
            assert_eq!(
                count_tabindex_zero(&harness),
                1,
                "mixed toolbar must keep exactly one tabindex=0 after navigation"
            );
        }
    }

    fn mount_native_scenario_with_props(
        orientation: Orientation,
        dir: Direction,
        loop_focus: bool,
        link_clicks: Rc<RefCell<usize>>,
    ) -> blitz_test_harness::Harness<dioxus_native_dom::DioxusDocument> {
        let opts = blitz_test_harness::HarnessOptions::default();
        let mut doc = dioxus_native_dom::DioxusDocument::new(
            VirtualDom::new_with_props(
                NativeToolbarFullScenario,
                NativeToolbarFullScenarioProps {
                    orientation,
                    dir,
                    loop_focus,
                    link_clicks,
                },
            ),
            blitz_dom::DocumentConfig {
                viewport: Some(blitz_traits::shell::Viewport::new(
                    opts.width,
                    opts.height,
                    opts.scale,
                    opts.color_scheme,
                )),
                ..Default::default()
            },
        );
        let native_doc = oxidase::dom::Document::from_base(doc.inner.clone());
        native_doc.attach_engine_bridges();
        oxidase::dom::Document::set_current(Some(native_doc.clone()));
        doc.vdom.in_scope(dioxus::core::ScopeId::ROOT, move || {
            dioxus::prelude::provide_context(native_doc);
        });
        doc.initial_build();
        let mut harness = blitz_test_harness::Harness::wrap(doc);
        drain_polls_and_resolve(&mut harness);
        harness
    }
}
