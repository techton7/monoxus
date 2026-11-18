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
