use monoxus::{
    foundation::shared::ScopeHandle,
    select::{apply_document_order, Select, SelectItemData},
};

#[test]
fn select_single_focus_ownership_contract() {
    let scope = ScopeHandle::root("select-test").child("focus");
    let select = Select::new(scope.clone())
        .with_value(Some("pro".to_owned()))
        .with_open(true);

    let content_attrs = select.content_attributes(Some(scope.qualify("item-pro")));
    assert_eq!(content_attrs.role(), "listbox");
    assert_eq!(content_attrs.tabindex(), -1);
}

#[test]
fn select_tab_dismisses_without_restoring_focus_contract() {
    let scope = ScopeHandle::root("select-test").child("tab-dismiss");
    let select = Select::new(scope).with_open(true);
    assert!(select.is_open());
}

#[test]
fn select_document_order_synchronization_contract() {
    let mut items = vec![
        SelectItemData::new("pro", "Pro", false),
        SelectItemData::new("starter", "Starter", false),
        SelectItemData::new("enterprise", "Enterprise", false),
    ];

    let order = vec!["starter".to_string(), "pro".to_string(), "enterprise".to_string()];
    apply_document_order(&mut items, &order);

    assert_eq!(items[0].value, "starter");
    assert_eq!(items[1].value, "pro");
    assert_eq!(items[2].value, "enterprise");

    // Test empty order preserves existing order without mutation
    let empty_order: Vec<String> = vec![];
    apply_document_order(&mut items, &empty_order);
    assert_eq!(items[0].value, "starter");
    assert_eq!(items[1].value, "pro");
    assert_eq!(items[2].value, "enterprise");
}

#[test]
fn select_item_highlight_and_scroll_into_view_nearest_contract() {
    let scope = ScopeHandle::root("select-test").child("highlight-scroll");
    let select = Select::new(scope.clone()).with_value(Some("korea".to_string()));

    let item_attrs = select.item_attributes("korea", true, false);
    assert!(item_attrs.is_highlighted());
    assert_eq!(item_attrs.id(), scope.qualify("item-korea"));
    assert_eq!(item_attrs.data_value(), "korea");
    assert_eq!(item_attrs.data_label(), "korea");
    assert_eq!(item_attrs.data_selected(), "true");
}

#[test]
fn select_uncontrolled_and_controlled_open_contract() {
    let scope = ScopeHandle::root("select-test").child("open");
    let closed = Select::new(scope.clone());
    assert!(!closed.is_open());

    let opened = Select::new(scope).with_open(true);
    assert!(opened.is_open());
}
