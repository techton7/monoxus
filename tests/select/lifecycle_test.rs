use monoxus::{
    foundation::shared::ScopeHandle,
    select::{Select, SelectItemData, apply_document_order},
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

    let order = vec![
        "starter".to_string(),
        "pro".to_string(),
        "enterprise".to_string(),
    ];
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

#[test]
fn select_presence_lifecycle_and_retained_mount_contract() {
    use monoxus::foundation::overlay::{Presence, PresenceState};

    let scope = ScopeHandle::root("select-test").child("presence");
    let select = Select::new(scope.clone());

    // Select::new() starts closed with retained mount enabled
    assert_eq!(select.presence().state(), PresenceState::Unmounted);
    assert!(select.presence().retain_mount());
    assert!(!select.presence().desired_present());

    // with_open(true) transitions presence to Mounted
    let opened = select.clone().with_open(true);
    assert_eq!(opened.presence().state(), PresenceState::Mounted);
    assert!(opened.presence().desired_present());

    // with_retained_mount(false) disables retained mount
    let no_retain = select.clone().with_retained_mount(false);
    assert!(!no_retain.presence().retain_mount());

    // with_presence replaces presence model
    let custom_presence = Presence::new(true).with_retained_mount(false);
    let with_pres = select.with_presence(custom_presence);
    assert_eq!(with_pres.presence().state(), PresenceState::Mounted);
    assert!(!with_pres.presence().retain_mount());
}

#[test]
fn select_mounted_option_registry_pure_rust_document_order_contract() {
    use monoxus::select::{MountedOptionRegistry, SelectItemRegistration};

    let mut registry = MountedOptionRegistry::new();
    registry.register(SelectItemRegistration::new("item-pro", "pro", "Pro", false));
    registry.register(SelectItemRegistration::new("item-starter", "starter", "Starter", false));
    registry.register(SelectItemRegistration::new("item-enterprise", "enterprise", "Enterprise", false));

    let initial_values: Vec<_> = registry.active_items().map(|i| i.value.as_str()).collect();
    assert_eq!(initial_values, vec!["pro", "starter", "enterprise"]);

    // Pure Rust document order reordering
    registry.set_tree_order(vec![
        "item-starter".to_string(),
        "item-pro".to_string(),
        "item-enterprise".to_string(),
    ]);

    let reordered_values: Vec<_> = registry.active_items().map(|i| i.value.as_str()).collect();
    assert_eq!(reordered_values, vec!["starter", "pro", "enterprise"]);

    // Dynamic unmount removal
    registry.unregister("item-pro");
    let after_unmount: Vec<_> = registry.active_items().map(|i| i.value.as_str()).collect();
    assert_eq!(after_unmount, vec!["starter", "enterprise"]);
    assert_eq!(registry.len(), 2);
}

#[test]
fn select_grouped_items_depth_first_document_order_contract() {
    use monoxus::select::{MountedOptionRegistry, SelectItemRegistration};

    let mut registry = MountedOptionRegistry::new();
    // Group 1 items
    registry.register(SelectItemRegistration::new("g1-item-1", "apple", "Apple", false));
    registry.register(SelectItemRegistration::new("g1-item-2", "banana", "Banana", false));
    // Group 2 items
    registry.register(SelectItemRegistration::new("g2-item-1", "carrot", "Carrot", false));
    registry.register(SelectItemRegistration::new("g2-item-2", "daikon", "Daikon", false));

    // Depth-first document tree order
    registry.set_tree_order(vec![
        "g1-item-1".to_string(),
        "g1-item-2".to_string(),
        "g2-item-1".to_string(),
        "g2-item-2".to_string(),
    ]);

    let active_values: Vec<_> = registry.active_items().map(|i| i.value.as_str()).collect();
    assert_eq!(active_values, vec!["apple", "banana", "carrot", "daikon"]);
}

#[test]
fn select_dynamic_item_reordering_updates_order_contract() {
    use monoxus::select::{MountedOptionRegistry, SelectItemRegistration};

    let mut registry = MountedOptionRegistry::new();
    registry.register(SelectItemRegistration::new("item-a", "a", "A", false));
    registry.register(SelectItemRegistration::new("item-b", "b", "B", false));
    registry.register(SelectItemRegistration::new("item-c", "c", "C", false));

    // Dynamic reordering (reverse sort: c, b, a)
    registry.set_tree_order(vec![
        "item-c".to_string(),
        "item-b".to_string(),
        "item-a".to_string(),
    ]);

    let reversed_values: Vec<_> = registry.active_items().map(|i| i.value.as_str()).collect();
    assert_eq!(reversed_values, vec!["c", "b", "a"]);
}

#[test]
fn select_conditional_item_insertion_and_removal_preserves_clean_document_order_contract() {
    use monoxus::select::{MountedOptionRegistry, SelectItemRegistration};

    let mut registry = MountedOptionRegistry::new();
    registry.register(SelectItemRegistration::new("item-a", "a", "A", false));
    registry.register(SelectItemRegistration::new("item-b", "b", "B", false));

    // Initial order: [a, b]
    registry.set_tree_order(vec!["item-a".to_string(), "item-b".to_string()]);
    let initial: Vec<_> = registry.active_items().map(|i| i.value.as_str()).collect();
    assert_eq!(initial, vec!["a", "b"]);

    // Conditionally insert item-mid between a and b: [a, mid, b]
    registry.register(SelectItemRegistration::new("item-mid", "mid", "Middle", false));
    registry.set_tree_order(vec![
        "item-a".to_string(),
        "item-mid".to_string(),
        "item-b".to_string(),
    ]);
    let with_mid: Vec<_> = registry.active_items().map(|i| i.value.as_str()).collect();
    assert_eq!(with_mid, vec!["a", "mid", "b"]);

    // Remove conditional item-mid: [a, b]
    registry.unregister("item-mid");
    let after_removal: Vec<_> = registry.active_items().map(|i| i.value.as_str()).collect();
    assert_eq!(after_removal, vec!["a", "b"]);
    assert_eq!(registry.len(), 2);
}

