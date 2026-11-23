use std::cell::Cell;
use std::rc::Rc;

use dioxus::prelude::*;
use monoxus::foundation::{
    compose::Slottable,
    shared::{Direction, Orientation, ScopeHandle},
};
#[allow(unused_imports)]
use monoxus::rating_group::{
    RATING_GROUP_KEY_SEQUENCE_TIMEOUT_MS, RATING_GROUP_PARTS, RatingGroup,
    RatingGroupAriaValuetext, RatingGroupContext, RatingGroupDefinition, RatingGroupItem,
    RatingGroupItemAttributes, RatingGroupItemContext, RatingGroupItemData, RatingGroupItemProps,
    RatingGroupItemRenderProps, RatingGroupItemState, RatingGroupKey, RatingGroupKeySequence,
    RatingGroupPart, RatingGroupRelationships, RatingGroupRoot, RatingGroupRootAttributes,
    RatingGroupRootProps, RatingGroupRootRenderProps, RatingGroupRuntime,
    calculate_rating_from_pointer, clamp_rating_value, derive_item_state, derive_items,
    format_default_aria_valuetext, format_rating_value, project_rating_group_item_as_child,
    project_rating_group_root_as_child, reduce_rating_group_key, resolve_item_click_value,
    use_rating_group_context, use_rating_group_item_context,
};

#[cfg(feature = "native")]
use crate::common;

#[component]
fn RatingGroupIndicatorProbe() -> Element {
    let item_ctx = use_rating_group_item_context();
    rsx! {
        span {
            "data-probe-state": item_ctx.state().as_str(),
            "{item_ctx.value()}"
        }
    }
}

#[component]
fn UncontrolledRatingScenario() -> Element {
    rsx! {
        RatingGroupRoot {
            id: "rg-uncontrolled",
            default_value: 2.5,
            max: 5,
            allow_half: true,
            name: "rating".to_string(),
            aria_valuetext: RatingGroupAriaValuetext::Formatter(Callback::new(|(val, max)| {
                format!("{val} stars out of {max}")
            })),
            for idx in 0..5 {
                RatingGroupItem {
                    key: "{idx}",
                    index: idx,
                    RatingGroupIndicatorProbe {}
                }
            }
        }
    }
}

#[component]
fn RenderItemsScenario() -> Element {
    rsx! {
        RatingGroup {
            id: "rg-render-items",
            value: 3.0,
            max: 4,
            render_items: move |items: Vec<RatingGroupItemData>| {
                rsx! {
                    for item in items {
                        RatingGroupItem {
                            key: "{item.index}",
                            index: item.index,
                            "{item.state.as_str()}"
                        }
                    }
                }
            },
        }
    }
}

#[test]
fn rating_group_vdom_renders_uncontrolled_and_render_items() {
    let mut dom = VirtualDom::new(UncontrolledRatingScenario);
    dom.rebuild_in_place();

    let mut render_items_dom = VirtualDom::new(RenderItemsScenario);
    render_items_dom.rebuild_in_place();
}

#[test]
fn rating_group_public_contracts_and_math() {
    assert_eq!(
        RATING_GROUP_PARTS,
        [RatingGroupPart::Root, RatingGroupPart::Item]
    );
    assert_eq!(RATING_GROUP_KEY_SEQUENCE_TIMEOUT_MS, 1000);
    assert_eq!(RatingGroupItemState::default(), RatingGroupItemState::Inactive);
    assert_eq!(format_rating_value(-0.0), "0");
    assert_eq!(format_rating_value(2.5), "2.5");
    assert_eq!(format_default_aria_valuetext(3.0, 5), "3 out of 5");

    assert_eq!(clamp_rating_value(-5.0, 0.0, 5), 0.0);
    assert_eq!(clamp_rating_value(8.0, 0.0, 5), 5.0);
    assert_eq!(derive_item_state(2.5, 2, true), RatingGroupItemState::Partial);
    assert_eq!(derive_items(1.0, 3, false).len(), 3);

    // RTL horizontal pointer flips, RTL vertical does not (D-4)
    assert_eq!(
        calculate_rating_from_pointer(
            0,
            true,
            Orientation::Horizontal,
            Direction::Rtl,
            85.0,
            10.0,
            0.0,
            0.0,
            100.0,
            40.0,
        ),
        0.5
    );
    assert_eq!(
        calculate_rating_from_pointer(
            0,
            true,
            Orientation::Vertical,
            Direction::Rtl,
            10.0,
            10.0,
            0.0,
            0.0,
            40.0,
            100.0,
        ),
        0.5
    );

    // First-item click-to-clear
    assert_eq!(resolve_item_click_value(0, 1.0, 0.0, 5, 1.0), 0.0);
    assert_eq!(resolve_item_click_value(0, 0.5, 0.0, 5, 0.5), 0.0);

    // Keyboard & decimal typing
    let scope = ScopeHandle::root("rg-contract");
    let def = RatingGroupDefinition::new(scope)
        .with_allow_half(true)
        .with_direction(Direction::Rtl);
    let outcome = reduce_rating_group_key(
        &def,
        2.0,
        RatingGroupKeySequence::Idle,
        RatingGroupKey::ArrowLeft,
    );
    assert_eq!(outcome.next_value, Some(2.5));

    // as_child projection helpers
    let rels = RatingGroupRelationships::new(ScopeHandle::root("rg-proj"));
    let root_attrs = RatingGroupRootAttributes::new(
        rels.root_id().to_string(),
        2.0,
        0.0,
        5,
        Orientation::Horizontal,
        Direction::Ltr,
        false,
        false,
        false,
        None,
        None,
        None,
    );
    let item_attrs = RatingGroupItemAttributes::new(
        rels.item_id(0),
        0,
        Orientation::Horizontal,
        false,
        false,
        RatingGroupItemState::Active,
    );
    let (rt, ra, rc) =
        project_rating_group_root_as_child(&root_attrs, "root-slot", Slottable::new("items"));
    assert_eq!(rt, "root-slot");
    assert_eq!(ra.role, "slider");
    assert_eq!(rc, "items");

    let (it, ia, ic) =
        project_rating_group_item_as_child(&item_attrs, "item-slot", Slottable::new("star"));
    assert_eq!(it, "item-slot");
    assert_eq!(ia.role, "presentation");
    assert_eq!(ic, "star");
}

#[test]
fn rating_group_runtime_hover_preview_touch_and_dedup_contract() {
    let mut dom = VirtualDom::new(|| rsx! {});
    dom.rebuild_in_place();

    let changes = Rc::new(Cell::new(0_u32));
    let last_val = Rc::new(Cell::new(0.0_f64));

    dom.in_scope(ScopeId::APP, || {
        let scope = ScopeHandle::root("rg-runtime-test");
        let rels = RatingGroupRelationships::new(scope.clone());
        let def = RatingGroupDefinition::new(scope)
            .with_allow_half(true)
            .with_hover_preview(true);

        let changes_cb = Rc::clone(&changes);
        let last_val_cb = Rc::clone(&last_val);
        let on_change = Callback::new(move |v: f64| {
            changes_cb.set(changes_cb.get() + 1);
            last_val_cb.set(v);
        });

        let mut rt = monoxus::rating_group::use_rating_group_runtime(
            def,
            rels,
            "rg-dom".to_string(),
            1.0,
            false,
            Some(on_change),
            None,
            None,
            None,
        );

        // Hover preview overrides item_state without mutating committed value or firing on_value_change
        rt.handle_item_pointer_move(3, false, "mouse", 0.0, 0.0, "missing");
        assert_eq!(rt.current_value(), 1.0);
        assert_eq!(rt.display_value(), 4.0);
        assert_eq!(rt.item_state(3), RatingGroupItemState::Active);
        assert_eq!(changes.get(), 0);

        // Touch pointer move is ignored
        rt.clear_hover_value();
        assert_eq!(rt.display_value(), 1.0);
        rt.handle_item_pointer_move(4, false, "touch", 0.0, 0.0, "missing");
        assert_eq!(rt.display_value(), 1.0);
        assert_eq!(rt.item_state(4), RatingGroupItemState::Inactive);

        // Setting same value deduplicates on_value_change
        rt.set_value(1.0);
        assert_eq!(changes.get(), 0);

        // Decimal key sequence: '2' -> '.' -> '5'
        assert!(rt.handle_keydown(RatingGroupKey::Digit(2)));
        assert_eq!(rt.current_value(), 2.0);
        assert_eq!(changes.get(), 1);
        assert!(rt.handle_keydown(RatingGroupKey::DecimalPoint));
        assert!(rt.handle_keydown(RatingGroupKey::Digit(5)));
        assert_eq!(rt.current_value(), 2.5);
        assert_eq!(last_val.get(), 2.5);
        assert_eq!(changes.get(), 2);
    });
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use common::native_fixture::*;
    use keyboard_types::{Code, Key, NamedKey};
    use monoxus::button::{Button, ButtonType};
    use monoxus::foundation::form::collect_form_data;

    #[component]
    fn NativeRatingInteractiveApp() -> Element {
        let mut last_change = use_signal(|| String::from("none"));
        let mut max_items = use_signal(|| 5_usize);

        rsx! {
            div {
                id: "rg-probe",
                "data-last-change": "{last_change}",
                style: "width: 600px; height: 400px;",
                form {
                    id: "rg-form",
                    RatingGroupRoot {
                        id: "rg-root",
                        default_value: 2.0,
                        max: max_items(),
                        allow_half: true,
                        required: true,
                        name: "score".to_string(),
                        style: "display: flex; flex-direction: row;",
                        on_value_change: move |val: f64| {
                            last_change.set(format_rating_value(val));
                        },
                        for idx in 0..max_items() {
                            RatingGroupItem {
                                key: "{idx}",
                                id: format!("rg-item-{idx}"),
                                index: idx,
                                style: "width: 40px; height: 40px;",
                                "*"
                            }
                        }
                    }
                    Button {
                        id: "rg-reset-btn".to_string(),
                        r#type: ButtonType::Reset,
                        style: "width: 80px; height: 30px;".to_string(),
                        "Reset"
                    }
                }
                button {
                    id: "rg-shrink-btn",
                    r#type: "button",
                    style: "width: 80px; height: 30px;".to_string(),
                    onclick: move |_| max_items.set(3),
                    "Shrink"
                }
            }
        }
    }

    #[component]
    fn NativeRatingRtlVerticalApp() -> Element {
        rsx! {
            div {
                style: "width: 400px; height: 400px;",
                RatingGroupRoot {
                    id: "rg-rtl",
                    default_value: 1.0,
                    allow_half: true,
                    dir: Direction::Rtl,
                    style: "display: flex; flex-direction: row;",
                    for idx in 0..5 {
                        RatingGroupItem {
                            key: "{idx}",
                            id: format!("rtl-item-{idx}"),
                            index: idx,
                            style: "width: 40px; height: 40px;",
                            "*"
                        }
                    }
                }
                RatingGroupRoot {
                    id: "rg-vert",
                    default_value: 0.0,
                    allow_half: true,
                    orientation: Orientation::Vertical,
                    dir: Direction::Rtl,
                    style: "display: flex; flex-direction: column;",
                    for idx in 0..5 {
                        RatingGroupItem {
                            key: "{idx}",
                            id: format!("vert-item-{idx}"),
                            index: idx,
                            style: "width: 40px; height: 40px;",
                            "*"
                        }
                    }
                }
                RatingGroupRoot {
                    id: "rg-readonly",
                    default_value: 3.0,
                    read_only: true,
                    style: "display: flex; flex-direction: row;",
                    for idx in 0..3 {
                        RatingGroupItem {
                            key: "{idx}",
                            id: format!("ro-item-{idx}"),
                            index: idx,
                            style: "width: 40px; height: 40px;",
                            "*"
                        }
                    }
                }
                RatingGroupRoot {
                    id: "rg-disabled",
                    default_value: 2.0,
                    disabled: true,
                    style: "display: flex; flex-direction: row;",
                    for idx in 0..3 {
                        RatingGroupItem {
                            key: "{idx}",
                            id: format!("dis-item-{idx}"),
                            index: idx,
                            style: "width: 40px; height: 40px;",
                            "*"
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn native_rating_group_aria_pointer_keyboard_and_form_reset() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeRatingInteractiveApp);

        let root_id = harness.query("#rg-root").expect("RatingGroupRoot mounted");
        {
            let base = harness.base();
            let root = base.get_node(root_id).unwrap();
            assert_eq!(root.attr("role".into()), Some("slider"));
            assert_eq!(root.attr("tabindex".into()), Some("0"));
            assert_eq!(root.attr("aria-valuenow".into()), Some("2"));
            assert_eq!(root.attr("aria-valuemin".into()), Some("0"));
            assert_eq!(root.attr("aria-valuemax".into()), Some("5"));
            assert_eq!(root.attr("aria-valuetext".into()), Some("2 out of 5"));
            assert_eq!(root.attr("aria-orientation".into()), Some("horizontal"));
            assert_eq!(root.attr("aria-required".into()), Some("true"));
            assert_eq!(root.attr("aria-label".into()), Some("Rating"));
            assert_eq!(root.attr("aria-disabled".into()), None);
            assert_eq!(root.attr("data-disabled".into()), None);
            assert_eq!(root.attr("data-readonly".into()), None);
            assert_eq!(root.attr("data-rating-group-root".into()), Some(""));

            let item0 = base.get_node(harness.node("#rg-item-0")).unwrap();
            assert_eq!(item0.attr("role".into()), Some("presentation"));
            assert_eq!(item0.attr("tabindex".into()), None);
            assert_eq!(item0.attr("aria-label".into()), None);
            assert_eq!(item0.attr("data-state".into()), Some("active"));

            let item2 = base.get_node(harness.node("#rg-item-2")).unwrap();
            assert_eq!(item2.attr("data-state".into()), Some("inactive"));
        }

        // Click left quarter of item-2 -> 2.5, item-2 becomes partial, root receives focus
        let item2_rect = harness.layout_rect("#rg-item-2");
        click_at_and_pump(
            &mut harness,
            item2_rect.x + item2_rect.width * 0.2,
            item2_rect.y + item2_rect.height * 0.5,
        );
        assert_native_focus_id(&harness, "#rg-root");
        {
            let base = harness.base();
            let root = base.get_node(root_id).unwrap();
            assert_eq!(root.attr("aria-valuenow".into()), Some("2.5"));
            assert_eq!(root.attr("aria-valuetext".into()), Some("2.5 out of 5"));
            assert_eq!(
                base.get_node(harness.node("#rg-item-2"))
                    .unwrap()
                    .attr("data-state".into()),
                Some("partial")
            );
        }

        // Form submission reflects updated score "2.5"
        let form_data = collect_form_data("rg-form").expect("collect_form_data");
        assert_eq!(form_data, vec![("score".to_string(), "2.5".to_string())]);

        // Keyboard stepping: PageUp (+1 -> 3.5), End (5), Home (0)
        send_key_and_pump(&mut harness, Key::Named(NamedKey::PageUp), Code::PageUp);
        assert_eq!(
            harness
                .base()
                .get_node(root_id)
                .unwrap()
                .attr("aria-valuenow".into()),
            Some("3.5")
        );
        send_key_and_pump(&mut harness, Key::Named(NamedKey::End), Code::End);
        assert_eq!(
            harness
                .base()
                .get_node(root_id)
                .unwrap()
                .attr("aria-valuenow".into()),
            Some("5")
        );
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Home), Code::Home);
        assert_eq!(
            harness
                .base()
                .get_node(root_id)
                .unwrap()
                .attr("aria-valuenow".into()),
            Some("0")
        );

        // Decimal sequence: '3' -> '.' -> '5' => 3.5
        send_key_and_pump(&mut harness, Key::Character("3".into()), Code::Digit3);
        send_key_and_pump(&mut harness, Key::Character(".".into()), Code::Period);
        send_key_and_pump(&mut harness, Key::Character("5".into()), Code::Digit5);
        assert_eq!(
            harness
                .base()
                .get_node(root_id)
                .unwrap()
                .attr("aria-valuenow".into()),
            Some("3.5")
        );

        // Native form reset restores initial default_value (2)
        click_and_pump(&mut harness, "#rg-reset-btn");
        assert_eq!(
            harness
                .base()
                .get_node(root_id)
                .unwrap()
                .attr("aria-valuenow".into()),
            Some("2")
        );
        let reset_form_data = collect_form_data("rg-form").expect("collect_form_data after reset");
        assert_eq!(
            reset_form_data,
            vec![("score".to_string(), "2".to_string())]
        );

        // Click-to-clear on item 0: click right half -> 1.0, click right half again -> 0.0
        let item0_rect = harness.layout_rect("#rg-item-0");
        click_at_and_pump(
            &mut harness,
            item0_rect.x + item0_rect.width * 0.8,
            item0_rect.y + item0_rect.height * 0.5,
        );
        assert_eq!(
            harness
                .base()
                .get_node(root_id)
                .unwrap()
                .attr("aria-valuenow".into()),
            Some("1")
        );
        click_at_and_pump(
            &mut harness,
            item0_rect.x + item0_rect.width * 0.8,
            item0_rect.y + item0_rect.height * 0.5,
        );
        assert_eq!(
            harness
                .base()
                .get_node(root_id)
                .unwrap()
                .attr("aria-valuenow".into()),
            Some("0")
        );

        // Dynamic max resize clamps value cleanly
        send_key_and_pump(&mut harness, Key::Named(NamedKey::End), Code::End);
        assert_eq!(
            harness
                .base()
                .get_node(root_id)
                .unwrap()
                .attr("aria-valuenow".into()),
            Some("5")
        );
        click_and_pump(&mut harness, "#rg-shrink-btn");
        assert_eq!(
            harness
                .base()
                .get_node(root_id)
                .unwrap()
                .attr("aria-valuemax".into()),
            Some("3")
        );
        assert_eq!(
            harness
                .base()
                .get_node(root_id)
                .unwrap()
                .attr("aria-valuenow".into()),
            Some("3")
        );
    }

    #[test]
    fn native_rating_group_rtl_vertical_readonly_and_disabled() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeRatingRtlVerticalApp);

        // RTL keyboard: ArrowLeft increments (+0.5), ArrowRight decrements (-0.5)
        focus_node(&mut harness, "#rg-rtl");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowLeft), Code::ArrowLeft);
        assert_eq!(
            harness
                .base()
                .get_node(harness.node("#rg-rtl"))
                .unwrap()
                .attr("aria-valuenow".into()),
            Some("1.5")
        );
        send_key_and_pump(
            &mut harness,
            Key::Named(NamedKey::ArrowRight),
            Code::ArrowRight,
        );
        assert_eq!(
            harness
                .base()
                .get_node(harness.node("#rg-rtl"))
                .unwrap()
                .attr("aria-valuenow".into()),
            Some("1")
        );

        // Vertical + RTL pointer click (D-4): top half of vert-item-1 -> 1.5
        let vert_rect = harness.layout_rect("#vert-item-1");
        click_at_and_pump(
            &mut harness,
            vert_rect.x + vert_rect.width * 0.5,
            vert_rect.y + vert_rect.height * 0.2,
        );
        assert_eq!(
            harness
                .base()
                .get_node(harness.node("#rg-vert"))
                .unwrap()
                .attr("aria-valuenow".into()),
            Some("1.5")
        );

        // Readonly keeps tabindex="0" and data-readonly="" while blocking click & key mutations
        let ro_id = harness.node("#rg-readonly");
        {
            let base = harness.base();
            let ro = base.get_node(ro_id).unwrap();
            assert_eq!(ro.attr("tabindex".into()), Some("0"));
            assert_eq!(ro.attr("data-readonly".into()), Some(""));
            assert_eq!(ro.attr("aria-disabled".into()), None);
            assert_eq!(ro.attr("data-disabled".into()), None);
        }
        click_and_pump(&mut harness, "#ro-item-0");
        focus_node(&mut harness, "#rg-readonly");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowLeft), Code::ArrowLeft);
        assert_eq!(
            harness
                .base()
                .get_node(ro_id)
                .unwrap()
                .attr("aria-valuenow".into()),
            Some("3")
        );

        // Disabled sets tabindex="-1", aria-disabled="true", data-disabled="" and blocks clicks
        let dis_id = harness.node("#rg-disabled");
        {
            let base = harness.base();
            let dis = base.get_node(dis_id).unwrap();
            assert_eq!(dis.attr("tabindex".into()), Some("-1"));
            assert_eq!(dis.attr("aria-disabled".into()), Some("true"));
            assert_eq!(dis.attr("data-disabled".into()), Some(""));
            assert_eq!(dis.attr("data-readonly".into()), None);
        }
        click_and_pump(&mut harness, "#dis-item-2");
        assert_eq!(
            harness
                .base()
                .get_node(dis_id)
                .unwrap()
                .attr("aria-valuenow".into()),
            Some("2")
        );
    }
}
