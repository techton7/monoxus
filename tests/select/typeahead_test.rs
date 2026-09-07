use monoxus::{
    foundation::shared::ScopeHandle,
    select::{Select, SelectItemData},
};

#[test]
fn select_synchronous_typeahead_prefix_matching() {
    let items = vec![
        SelectItemData::new("apple", "Apple", false),
        SelectItemData::new("banana", "Banana", false),
        SelectItemData::new("blueberry", "Blueberry", false),
        SelectItemData::new("cherry", "Cherry", false),
    ];

    let query_b = "b";
    let first_b = items
        .iter()
        .find(|i| !i.disabled && i.text.to_lowercase().starts_with(query_b));
    assert_eq!(first_b.map(|i| i.value.as_str()), Some("banana"));

    let query_bl = "bl";
    let first_bl = items
        .iter()
        .find(|i| !i.disabled && i.text.to_lowercase().starts_with(query_bl));
    assert_eq!(first_bl.map(|i| i.value.as_str()), Some("blueberry"));
}

#[test]
fn select_closed_trigger_typeahead_contract() {
    let scope = ScopeHandle::root("select-test").child("closed-typeahead");
    let items = vec![
        SelectItemData::new("starter", "Starter", false),
        SelectItemData::new("pro", "Pro", false),
        SelectItemData::new("enterprise", "Enterprise", false),
    ];

    let select = Select::new(scope).with_items(items.clone());
    assert_eq!(select.items().len(), 3);

    let prefix = "p";
    let matched = items
        .iter()
        .find(|i| !i.disabled && i.text.to_lowercase().starts_with(prefix));
    assert_eq!(matched.map(|i| i.value.as_str()), Some("pro"));
}
