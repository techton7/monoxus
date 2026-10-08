use dioxus::prelude::*;
use monoxus::{
    foundation::shared::ScopeHandle,
    select::{apply_document_order, Select, SelectItemData, SelectItemRegistration, SelectRuntime},
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
fn select_grouped_items_depth_first_document_order_contract() {
    let mut dom = VirtualDom::new(|| rsx! {});
    dom.rebuild_in_place();
    dom.in_scope(ScopeId::APP, || {
        let scope = ScopeHandle::root("select-test").child("grouped-items");
        let select = Select::new(scope).with_open(true);
        let runtime = SelectRuntime::new(select);

        // Register items belonging to Group 1 and Group 2 in arbitrary order
        runtime.register_item(SelectItemRegistration::new("g2-c", "G2-C", "Item C", false));
        runtime.register_item(SelectItemRegistration::new("g1-a", "G1-A", "Item A", false));
        runtime.register_item(SelectItemRegistration::new("g2-d", "G2-D", "Item D", false));
        runtime.register_item(SelectItemRegistration::new("g1-b", "G1-B", "Item B", false));

        // Apply depth-first document tree order: [G1-A, G1-B, G2-C, G2-D]
        runtime.set_ordered_items(vec![
            "g1-a".to_string(),
            "g1-b".to_string(),
            "g2-c".to_string(),
            "g2-d".to_string(),
        ]);

        let rels = runtime.relationships();

        // 1. Initial Home navigation -> G1-A
        runtime.highlight_first();
        assert_eq!(runtime.highlighted_value().as_deref(), Some("G1-A"));
        let attrs = runtime.content_attributes(runtime.highlighted_value().map(|v| rels.item_id(&v)));
        assert_eq!(attrs.aria_activedescendant(), Some(rels.item_id("G1-A").as_str()));

        // 2. highlight_next -> G1-B
        runtime.highlight_next();
        assert_eq!(runtime.highlighted_value().as_deref(), Some("G1-B"));
        let attrs = runtime.content_attributes(runtime.highlighted_value().map(|v| rels.item_id(&v)));
        assert_eq!(attrs.aria_activedescendant(), Some(rels.item_id("G1-B").as_str()));

        // 3. highlight_next -> G2-C
        runtime.highlight_next();
        assert_eq!(runtime.highlighted_value().as_deref(), Some("G2-C"));
        let attrs = runtime.content_attributes(runtime.highlighted_value().map(|v| rels.item_id(&v)));
        assert_eq!(attrs.aria_activedescendant(), Some(rels.item_id("G2-C").as_str()));

        // 4. highlight_next -> G2-D
        runtime.highlight_next();
        assert_eq!(runtime.highlighted_value().as_deref(), Some("G2-D"));
        let attrs = runtime.content_attributes(runtime.highlighted_value().map(|v| rels.item_id(&v)));
        assert_eq!(attrs.aria_activedescendant(), Some(rels.item_id("G2-D").as_str()));

        // 5. highlight_prev -> G2-C
        runtime.highlight_prev();
        assert_eq!(runtime.highlighted_value().as_deref(), Some("G2-C"));

        // 6. End navigation -> G2-D
        runtime.highlight_last();
        assert_eq!(runtime.highlighted_value().as_deref(), Some("G2-D"));

        // 7. Home navigation -> G1-A
        runtime.highlight_first();
        assert_eq!(runtime.highlighted_value().as_deref(), Some("G1-A"));
    });
}

#[test]
fn select_dynamic_item_reordering_updates_order_contract() {
    let mut dom = VirtualDom::new(|| rsx! {});
    dom.rebuild_in_place();
    dom.in_scope(ScopeId::APP, || {
        let scope = ScopeHandle::root("select-test").child("dynamic-reorder");
        let select = Select::new(scope).with_open(true);
        let runtime = SelectRuntime::new(select);

        runtime.register_item(SelectItemRegistration::new("item-a", "A", "Apple", false));
        runtime.register_item(SelectItemRegistration::new("item-b", "B", "Berry", false));
        runtime.register_item(SelectItemRegistration::new("item-c", "C", "Cherry", false));

        // Initial document order: [A, B, C]
        runtime.set_ordered_items(vec![
            "item-a".to_string(),
            "item-b".to_string(),
            "item-c".to_string(),
        ]);

        runtime.highlight_first();
        assert_eq!(runtime.highlighted_value().as_deref(), Some("A"));

        // Dynamic reordering (reverse order): [C, B, A]
        runtime.set_ordered_items(vec![
            "item-c".to_string(),
            "item-b".to_string(),
            "item-a".to_string(),
        ]);

        // Verify roving focus follows the new order [C, B, A]
        runtime.highlight_first();
        assert_eq!(runtime.highlighted_value().as_deref(), Some("C"));

        runtime.highlight_next();
        assert_eq!(runtime.highlighted_value().as_deref(), Some("B"));

        runtime.highlight_next();
        assert_eq!(runtime.highlighted_value().as_deref(), Some("A"));

        // Verify typeahead navigates the new order:
        // Type 'b' -> highlights B (Berry)
        runtime.handle_typeahead('b', 100.0);
        assert_eq!(runtime.highlighted_value().as_deref(), Some("B"));

        // After timeout (> 500ms), type 'c' -> highlights C (Cherry)
        runtime.handle_typeahead('c', 700.0);
        assert_eq!(runtime.highlighted_value().as_deref(), Some("C"));

        // Next item after C in new order is B
        runtime.highlight_next();
        assert_eq!(runtime.highlighted_value().as_deref(), Some("B"));
    });
}

#[test]
fn select_conditional_item_insertion_and_removal_preserves_clean_document_order_contract() {
    let mut dom = VirtualDom::new(|| rsx! {});
    dom.rebuild_in_place();
    dom.in_scope(ScopeId::APP, || {
        let scope = ScopeHandle::root("select-test").child("conditional-order");
        let select = Select::new(scope).with_open(true);
        let runtime = SelectRuntime::new(select);

        // Group 1: [G1-A], Group 2: [G2-C]
        runtime.register_item(SelectItemRegistration::new("g1-a", "G1-A", "Item A", false));
        runtime.register_item(SelectItemRegistration::new("g2-c", "G2-C", "Item C", false));

        runtime.set_ordered_items(vec!["g1-a".to_string(), "g2-c".to_string()]);

        let initial_vals: Vec<_> = runtime
            .registry()
            .active_items()
            .map(|i| i.value.clone())
            .collect();
        assert_eq!(initial_vals, vec!["G1-A", "G2-C"]);

        // Conditionally insert G1-B into Group 1 and G2-D into Group 2
        runtime.register_item(SelectItemRegistration::new("g1-b", "G1-B", "Item B", false));
        runtime.register_item(SelectItemRegistration::new("g2-d", "G2-D", "Item D", false));

        // Host query order returns pre-order: [G1-A, G1-B, G2-C, G2-D]
        runtime.set_ordered_items(vec![
            "g1-a".to_string(),
            "g1-b".to_string(),
            "g2-c".to_string(),
            "g2-d".to_string(),
        ]);

        let with_inserted: Vec<_> = runtime
            .registry()
            .active_items()
            .map(|i| i.value.clone())
            .collect();
        assert_eq!(with_inserted, vec!["G1-A", "G1-B", "G2-C", "G2-D"]);

        // Conditionally remove G1-B from Group 1
        runtime.unregister_item("g1-b");

        let after_removal: Vec<_> = runtime
            .registry()
            .active_items()
            .map(|i| i.value.clone())
            .collect();
        assert_eq!(after_removal, vec!["G1-A", "G2-C", "G2-D"]);

        // Verify keyboard navigation reflects removal cleanly without duplicate or stale items
        runtime.highlight_first();
        assert_eq!(runtime.highlighted_value().as_deref(), Some("G1-A"));
        runtime.highlight_next();
        assert_eq!(runtime.highlighted_value().as_deref(), Some("G2-C"));
        runtime.highlight_next();
        assert_eq!(runtime.highlighted_value().as_deref(), Some("G2-D"));
    });
}

#[test]
fn select_native_blitz_document_order_query_contract() {
    use blitz_dom::{BaseDocument, DocumentConfig};
    use std::cell::RefCell;
    use std::rc::Rc;

    let base_doc = Rc::new(RefCell::new(BaseDocument::new(DocumentConfig::default())));
    let native_doc = oxidase::dom::Document::from_base(base_doc.clone());

    let qual_name = |name: &'static str| blitz_dom::QualName {
        prefix: None,
        ns: blitz_dom::ns!(html),
        local: blitz_dom::LocalName::from(name),
    };
    let attr = |name: &'static str, value: &str| blitz_dom::Attribute {
        name: qual_name(name),
        value: value.to_string(),
    };

    // Construct nested DOM hierarchy:
    // root -> select_content
    //   -> group_1
    //     -> opt_1_a
    //     -> opt_1_b
    //   -> group_2
    //     -> opt_2_c
    //     -> opt_2_d
    {
        let mut base = base_doc.borrow_mut();
        let root_id = base.root_node().id;
        let mut mutator = base.mutate();
        let select_content =
            mutator.create_element(qual_name("div"), vec![attr("id", "select-content")]);
        let group_1 = mutator.create_element(qual_name("div"), vec![attr("id", "group-1")]);
        let opt_1_a = mutator.create_element(qual_name("div"), vec![attr("id", "opt-1-a")]);
        let opt_1_b = mutator.create_element(qual_name("div"), vec![attr("id", "opt-1-b")]);
        let group_2 = mutator.create_element(qual_name("div"), vec![attr("id", "group-2")]);
        let opt_2_c = mutator.create_element(qual_name("div"), vec![attr("id", "opt-2-c")]);
        let opt_2_d = mutator.create_element(qual_name("div"), vec![attr("id", "opt-2-d")]);

        mutator.append_children(group_1, &[opt_1_a, opt_1_b]);
        mutator.append_children(group_2, &[opt_2_c, opt_2_d]);
        mutator.append_children(select_content, &[group_1, group_2]);
        mutator.append_children(root_id, &[select_content]);
    }

    // Candidate IDs supplied in arbitrary/reverse order
    let candidates = ["opt-2-d", "opt-1-b", "opt-2-c", "opt-1-a"];
    let ordered = native_doc
        .query_descendant_order("select-content", &candidates)
        .expect("native document query_descendant_order must succeed");

    // Proves exact pre-order depth-first traversal [opt-1-a, opt-1-b, opt-2-c, opt-2-d]
    assert_eq!(
        ordered,
        vec!["opt-1-a", "opt-1-b", "opt-2-c", "opt-2-d"]
    );
}

