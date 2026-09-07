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
    assert_eq!(select_none.trigger_attributes().data_placeholder_str(), "true");

    let select_empty = Select::new(scope.clone()).with_value(Some("".to_string()));
    assert_eq!(select_empty.trigger_attributes().data_placeholder_str(), "true");

    let select_val = Select::new(scope).with_value(Some("apple".to_string()));
    assert_eq!(select_val.trigger_attributes().data_placeholder_str(), "false");
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
    let select = Select::new(scope)
        .with_loop(false)
        .with_items(vec![
            SelectItemData::new("a", "A", false),
            SelectItemData::new("b", "B", false),
        ]);

    assert!(!select.loop_selection());
}
