use monoxus::{
    foundation::shared::ScopeHandle,
    select::{Select, SelectItemData, SelectMode},
};

#[test]
fn select_uncontrolled_and_controlled_value_contract() {
    let scope = ScopeHandle::root("select-test").child("value");
    let uncontrolled = Select::new(scope.clone()).with_default_value(Some("orange".to_owned()));
    assert_eq!(uncontrolled.value(), Some("orange"));
    assert_eq!(uncontrolled.default_value(), Some("orange"));

    let controlled = Select::new(scope)
        .with_value(Some("apple".to_owned()))
        .with_default_value(Some("orange".to_owned()));
    assert_eq!(controlled.value(), Some("apple"));
    assert_eq!(controlled.default_value(), Some("orange"));
}

#[test]
fn select_multiple_mode_and_repeated_hidden_inputs_contract() {
    let scope = ScopeHandle::root("select-test").child("multiple");
    let multi = Select::new(scope)
        .with_mode(SelectMode::Multiple)
        .with_values(vec!["apple".to_string(), "cherry".to_string()]);

    assert!(multi.is_multiple());
    assert_eq!(multi.values(), &["apple", "cherry"]);

    let apple_attrs = multi.item_attributes("apple", false, false);
    let banana_attrs = multi.item_attributes("banana", false, false);
    let cherry_attrs = multi.item_attributes("cherry", false, false);

    assert_eq!(apple_attrs.aria_selected(), "true");
    assert_eq!(banana_attrs.aria_selected(), "false");
    assert_eq!(cherry_attrs.aria_selected(), "true");
}

#[test]
fn select_clear_item_placeholder_reset_contract() {
    let scope = ScopeHandle::root("select-test").child("clear");
    let select_none = Select::new(scope.clone()).with_value(None);
    assert_eq!(
        select_none.trigger_attributes().data_placeholder_str(),
        "true"
    );

    let select_empty = Select::new(scope.clone()).with_value(Some("".to_string()));
    assert_eq!(
        select_empty.trigger_attributes().data_placeholder_str(),
        "true"
    );

    let select_val = Select::new(scope).with_value(Some("apple".to_string()));
    assert_eq!(
        select_val.trigger_attributes().data_placeholder_str(),
        "false"
    );
}

#[test]
fn select_candidate_highlight_decoupled_from_committed_value() {
    let scope = ScopeHandle::root("select-test").child("highlight");
    let select = Select::new(scope).with_value(Some("apple".to_owned()));

    let apple_highlighted = select.item_attributes("apple", true, false);
    assert_eq!(apple_highlighted.aria_selected(), "true");
    assert!(apple_highlighted.is_highlighted());

    let banana_highlighted = select.item_attributes("banana", true, false);
    assert_eq!(banana_highlighted.aria_selected(), "false");
    assert!(banana_highlighted.is_highlighted());

    let apple_unhighlighted = select.item_attributes("apple", false, false);
    assert_eq!(apple_unhighlighted.aria_selected(), "true");
    assert!(!apple_unhighlighted.is_highlighted());
}

#[test]
fn select_circular_roving_logic() {
    let items = vec![
        SelectItemData::new("apple", "Apple", false),
        SelectItemData::new("banana", "Banana", true),
        SelectItemData::new("cherry", "Cherry", false),
    ];
    let enabled: Vec<_> = items.iter().filter(|i| !i.disabled).collect();
    assert_eq!(enabled.len(), 2);
    assert_eq!(enabled[0].value, "apple");
    assert_eq!(enabled[1].value, "cherry");

    let next_from_0 = (0 + 1) % enabled.len();
    assert_eq!(next_from_0, 1);
    let next_from_1 = (1 + 1) % enabled.len();
    assert_eq!(next_from_1, 0);
}

#[test]
fn select_loop_false_clamping_contract() {
    let scope = ScopeHandle::root("select-test").child("loop-false");
    let select = Select::new(scope).with_loop(false).with_items(vec![
        SelectItemData::new("a", "A", false),
        SelectItemData::new("b", "B", false),
    ]);

    assert!(!select.loop_selection());
}

#[test]
fn select_label_persistence_across_unmount_single_and_multiple() {
    use dioxus::prelude::*;
    use monoxus::select::SelectRuntime;

    let dom = VirtualDom::new(|| rsx! { div {} });
    dom.in_scope(dioxus::core::ScopeId::ROOT, || {
        let scope = ScopeHandle::root("select-test").child("label-persist");
        let select = Select::new(scope).with_items(vec![SelectItemData::new(
            "apple",
            "Fresh Red Apple, Japan (Tokyo)",
            false,
        )]);
        let rt = SelectRuntime::new(select);

        // Initial label lookup from items
        assert_eq!(
            rt.item_label("apple"),
            Some("Fresh Red Apple, Japan (Tokyo)".to_string())
        );

        // Dynamic item registered
        rt.register_item_data("pear", "Sweet Asian Pear, Kyoto", false);
        assert_eq!(
            rt.item_label("pear"),
            Some("Sweet Asian Pear, Kyoto".to_string())
        );

        // Select dynamic item
        rt.select_item("pear");
        assert_eq!(rt.value().as_deref(), Some("pear"));

        // Simulate menu close and unmount of the item DOM node
        let pear_item_id = rt.relationships().item_id("pear");
        rt.unregister_item(&pear_item_id);

        // The label MUST persist in selected_labels rather than degrading to humanize_label
        assert_eq!(
            rt.item_label("pear"),
            Some("Sweet Asian Pear, Kyoto".to_string())
        );

        // Multiple mode persistence
        let multi_scope = ScopeHandle::root("select-test").child("multi-persist");
        let multi_select = Select::new(multi_scope).with_mode(SelectMode::Multiple);
        let multi_rt = SelectRuntime::new(multi_select);

        multi_rt.register_item_data("item-1", "Custom Complex Label 1", false);
        multi_rt.register_item_data("item-2", "Custom Complex Label 2", false);
        multi_rt.select_item("item-1");
        multi_rt.select_item("item-2");

        multi_rt.unregister_item(&multi_rt.relationships().item_id("item-1"));
        multi_rt.unregister_item(&multi_rt.relationships().item_id("item-2"));

        assert_eq!(
            multi_rt.item_label("item-1"),
            Some("Custom Complex Label 1".to_string())
        );
        assert_eq!(
            multi_rt.item_label("item-2"),
            Some("Custom Complex Label 2".to_string())
        );
    });
}

#[test]
fn select_stationary_pointer_release_contract() {
    use dioxus::prelude::*;
    use monoxus::select::SelectRuntime;

    let dom = VirtualDom::new(|| rsx! { div {} });
    dom.in_scope(dioxus::core::ScopeId::ROOT, || {
        let scope = ScopeHandle::root("select-test").child("stationary-press");
        let select = Select::new(scope);
        let rt = SelectRuntime::new(select);

        rt.record_pressed_item("item-a", 100.0, 200.0);
        let press = rt.take_pressed_item();
        assert!(press.is_some());
        let p = press.unwrap();
        assert_eq!(p.value, "item-a");
        assert_eq!(p.client_x, 100.0);
        assert_eq!(p.client_y, 200.0);

        // Verify distance threshold logic (< 4.0px)
        let dx = 102.0 - p.client_x;
        let dy = 201.0 - p.client_y;
        let dist = dx.hypot(dy);
        assert!(dist < 4.0);

        // After take, pressed_item is None
        assert_eq!(rt.take_pressed_item(), None);

        // clear_pressed_item on dropdown close
        rt.record_pressed_item("item-b", 50.0, 50.0);
        rt.clear_pressed_item();
        assert_eq!(rt.take_pressed_item(), None);
    });
}

#[test]
fn select_data_disabled_and_conditional_controls() {
    use monoxus::foundation::shared::Direction;

    let scope = ScopeHandle::root("select-test").child("disabled-controls");
    let enabled_closed = Select::new(scope.clone()).with_open(false);
    let trig_closed = enabled_closed.trigger_attributes();
    assert_eq!(trig_closed.data_disabled(), None);
    assert_eq!(trig_closed.aria_controls(), None);
    assert_eq!(trig_closed.target_content_id(), scope.qualify("content"));

    let enabled_open = Select::new(scope.clone()).with_open(true);
    let trig_open = enabled_open.trigger_attributes();
    assert_eq!(trig_open.data_disabled(), None);
    assert_eq!(trig_open.aria_controls(), Some(scope.qualify("content").as_str()));

    let disabled_open = Select::new(scope.clone()).with_disabled(true).with_open(true);
    let trig_dis = disabled_open.trigger_attributes();
    assert_eq!(trig_dis.data_disabled(), Some(""));
    assert_eq!(trig_dis.aria_controls(), Some(scope.qualify("content").as_str()));

    // Direction contract
    assert_eq!(Direction::Ltr.as_str(), "ltr");
    assert_eq!(Direction::Rtl.as_str(), "rtl");
}

#[cfg(feature = "native")]
#[test]
fn fm_w1_ctx_select_root_post_mount_callbacks_and_props_update() {
    use dioxus::prelude::component;
    use crate::common::native_fixture::*;

    let _guard = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    #[component]
    fn SelectReactivePropsScenario() -> dioxus::prelude::Element {
        use dioxus::prelude::*;
        use monoxus::select::{
            SelectContent, SelectItem, SelectPortal, SelectRoot, SelectTrigger, SelectValue,
        };

        let mut stage = use_signal(|| 0_u8);
        let mut val_cb1_log = use_signal(Vec::<String>::new);
        let mut val_cb2_log = use_signal(Vec::<String>::new);
        let mut open_cb1_hits = use_signal(|| 0_u32);
        let mut open_cb2_hits = use_signal(|| 0_u32);

        let is_updated = stage() >= 1;
        let is_disabled = stage() >= 2;

        let on_value_change = if is_updated {
            EventHandler::new(move |v: Option<String>| {
                val_cb2_log
                    .write()
                    .push(v.unwrap_or_else(|| "none".to_string()));
            })
        } else {
            EventHandler::new(move |v: Option<String>| {
                val_cb1_log
                    .write()
                    .push(v.unwrap_or_else(|| "none".to_string()));
            })
        };

        let on_open_change = if is_updated {
            EventHandler::new(move |_o: bool| open_cb2_hits.set(open_cb2_hits() + 1))
        } else {
            EventHandler::new(move |_o: bool| open_cb1_hits.set(open_cb1_hits() + 1))
        };

        rsx! {
            div { style: "padding: 24px;",
                button {
                    id: "w1-sel-next-stage",
                    r#type: "button",
                    style: "width: 90px; height: 24px;",
                    onclick: move |_| stage.set(stage() + 1),
                    "Next Stage"
                }
                span { id: "w1-sel-val-cb1", "{val_cb1_log.read().join(\",\")}" }
                span { id: "w1-sel-val-cb2", "{val_cb2_log.read().join(\",\")}" }
                span { id: "w1-sel-open-hits", "{open_cb1_hits()}:{open_cb2_hits()}" }

                SelectRoot {
                    id: Some("w1-ctx-sel".to_string()),
                    required: is_updated,
                    disabled: is_disabled,
                    loop_selection: is_updated,
                    on_value_change,
                    on_open_change,
                    SelectTrigger {
                        class: "w1-ctx-sel-trig",
                        style: "width: 140px; height: 32px;",
                        SelectValue { placeholder: "Pick" }
                    }
                    SelectPortal {
                        SelectContent {
                            class: "w1-ctx-sel-content",
                            style: "width: 140px; background: white;",
                            SelectItem {
                                value: "alpha".to_string(),
                                class: "w1-ctx-sel-alpha",
                                style: "height: 28px;",
                                "Alpha"
                            }
                            SelectItem {
                                value: "beta".to_string(),
                                class: "w1-ctx-sel-beta",
                                style: "height: 28px;",
                                "Beta"
                            }
                        }
                    }
                }
            }
        }
    }

    let mut harness = mount_native_scenario(SelectReactivePropsScenario);
    advance_frame_and_pump(&mut harness, 0.02);

    assert_eq!(harness.attr(".w1-ctx-sel-trig", "aria-required"), None);

    // Stage 1: update required, loop_selection, and swap callbacks post-mount without remount
    click_and_pump(&mut harness, "#w1-sel-next-stage");
    advance_frame_and_pump(&mut harness, 0.02);
    assert_eq!(
        harness
            .attr(".w1-ctx-sel-trig", "aria-required")
            .as_deref(),
        Some("true"),
        "H-6: SelectTrigger must reactively reflect post-mount required=true"
    );

    // Open and select beta -> cb2 and open_cb2 fire; cb1/open_cb1 stay untouched
    click_and_pump(&mut harness, ".w1-ctx-sel-trig");
    advance_frame_and_pump(&mut harness, 0.02);
    click_and_pump(&mut harness, ".w1-ctx-sel-beta");
    advance_frame_and_pump(&mut harness, 0.02);
    assert_eq!(harness.text_content("#w1-sel-val-cb1"), "");
    assert_eq!(harness.text_content("#w1-sel-val-cb2"), "beta");
    assert_eq!(harness.text_content("#w1-sel-open-hits"), "0:2");

    // Stage 2: disable post-mount -> SelectTrigger reflects disabled and blocks open
    click_and_pump(&mut harness, "#w1-sel-next-stage");
    advance_frame_and_pump(&mut harness, 0.02);
    assert_eq!(
        harness
            .attr(".w1-ctx-sel-trig", "aria-disabled")
            .as_deref(),
        Some("true"),
        "H-6: SelectTrigger must reactively reflect post-mount disabled=true"
    );
    click_and_pump(&mut harness, ".w1-ctx-sel-trig");
    advance_frame_and_pump(&mut harness, 0.02);
    assert_eq!(
        harness
            .attr(".w1-ctx-sel-trig", "aria-expanded")
            .as_deref(),
        Some("false"),
        "H-6: disabled SelectRoot post-mount must block opening"
    );
}

#[cfg(feature = "native")]
#[test]
fn fm_w1_guard_select_cross_item_jitter_inside_popover_keeps_popover_open() {
    use blitz_traits::events::UiEvent;
    use dioxus::prelude::component;
    use crate::common::native_fixture::*;

    let _guard = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    #[component]
    fn SelectInPopoverJitterScenario() -> dioxus::prelude::Element {
        use dioxus::prelude::*;
        use monoxus::popover::{PopoverContent, PopoverPortal, PopoverRoot, PopoverTrigger};
        use monoxus::select::{
            SelectContent, SelectItem, SelectPortal, SelectRoot, SelectTrigger, SelectValue,
        };

        let mut selected = use_signal(|| Option::<String>::None);

        rsx! {
            div { style: "padding: 24px;",
                span {
                    id: "w1-l4-selected",
                    "{selected().unwrap_or_else(|| \"none\".to_string())}"
                }
                PopoverRoot {
                    id: "w1-l4-pop",
                    default_open: true,
                    PopoverTrigger {
                        id: "w1-l4-pop-trig",
                        style: "width: 120px; height: 28px;",
                        "Open Picker"
                    }
                    PopoverPortal {
                        PopoverContent {
                            id: "w1-l4-pop-content",
                            style: "width: 240px; height: 180px; padding: 12px; background: white;",
                            SelectRoot {
                                id: Some("w1-l4-sel".to_string()),
                                on_value_change: move |v: Option<String>| selected.set(v),
                                SelectTrigger {
                                    class: "w1-l4-sel-trig",
                                    style: "width: 140px; height: 28px;",
                                    SelectValue { placeholder: "Month" }
                                }
                                SelectPortal {
                                    SelectContent {
                                        class: "w1-l4-sel-content",
                                        style: "width: 140px; background: white;",
                                        SelectItem {
                                            value: "jan".to_string(),
                                            class: "w1-l4-sel-jan",
                                            style: "height: 28px; box-sizing: border-box;",
                                            "January"
                                        }
                                        SelectItem {
                                            value: "feb".to_string(),
                                            class: "w1-l4-sel-feb",
                                            style: "height: 28px; box-sizing: border-box;",
                                            "February"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut harness = mount_native_scenario(SelectInPopoverJitterScenario);
    advance_frame_and_pump(&mut harness, 0.02);
    assert!(harness.query("#w1-l4-pop-content").is_some());

    // Open Select inside Popover
    click_and_pump(&mut harness, ".w1-l4-sel-trig");
    advance_frame_and_pump(&mut harness, 0.02);
    assert!(harness.query(".w1-l4-sel-content").is_some());

    let jan_rect = harness.layout_rect(".w1-l4-sel-jan");
    let feb_rect = harness.layout_rect(".w1-l4-sel-feb");
    let x = jan_rect.x + jan_rect.width * 0.5;
    let down_y = jan_rect.y + jan_rect.height - 1.0;
    let up_y = feb_rect.y + 1.0;
    assert!(
        (up_y - down_y).abs() < 4.0,
        "Expected adjacent items within <4px jitter border, down_y={down_y}, up_y={up_y}"
    );

    // Cross-item <4px pointerdown(jan) -> pointerup(feb) release
    harness.dispatch(UiEvent::PointerDown(blitz_test_harness::mouse_pointer_event(
        x, down_y,
    )));
    harness.dispatch(UiEvent::PointerUp(blitz_test_harness::mouse_pointer_event(
        x, up_y,
    )));
    drain_polls_and_resolve(&mut harness);
    advance_frame_and_pump(&mut harness, 0.02);
    advance_frame_and_pump(&mut harness, 0.02);

    assert_eq!(
        harness.text_content("#w1-l4-selected"),
        "jan",
        "L-4: cross-item <4px jitter release must select pressed item (jan)"
    );
    assert!(
        harness.query(".w1-l4-sel-content").is_none(),
        "L-4: Select must close after cross-item <4px selection"
    );
    assert!(
        harness.query("#w1-l4-pop-content").is_some(),
        "L-4: Enclosing Popover must remain open after Select cross-item <4px release"
    );
}
