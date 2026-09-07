use monoxus::{
    foundation::{
        overlay::PortalHost,
        shared::ScopeHandle,
        state::DataState,
    },
    select::{Select, SelectPart, SelectRelationships, SELECT_PARTS},
};

#[test]
fn select_part_inventory_matches_exhaustive_surface() {
    let parts: Vec<_> = Select::parts()
        .iter()
        .map(SelectPart::as_str)
        .collect();
    assert_eq!(
        parts,
        vec![
            "root",
            "trigger",
            "value",
            "icon",
            "portal",
            "content",
            "content-static",
            "viewport",
            "group",
            "label",
            "item",
            "item-text",
            "item-indicator",
            "separator",
            "arrow",
            "hidden-input",
        ]
    );
    assert_eq!(SELECT_PARTS.len(), 16);
}

#[test]
fn select_relationships_produce_deterministic_ids() {
    let scope = ScopeHandle::root("select-test").child("main");
    let relationships = SelectRelationships::new(scope.clone());

    assert_eq!(relationships.scope(), &scope);
    assert_eq!(relationships.root_id(), scope.token());
    assert_eq!(relationships.trigger_id(), scope.qualify("trigger"));
    assert_eq!(relationships.content_id(), scope.qualify("content"));
    assert_eq!(
        relationships.item_id("apple"),
        scope.qualify("item-apple")
    );
    assert_eq!(
        relationships.group_id("fruits"),
        scope.qualify("group-fruits")
    );
    assert_eq!(
        relationships.label_id("fruits"),
        scope.qualify("label-fruits")
    );
}

#[test]
fn select_attributes_publish_wai_aria_and_data_state() {
    let scope = ScopeHandle::root("select-test").child("attrs");
    let select = Select::new(scope.clone())
        .with_value(Some("banana".to_owned()))
        .with_open(true);

    let root_attrs = select.root_attributes();
    assert_eq!(root_attrs.id(), scope.token());
    assert_eq!(root_attrs.data_state(), DataState::Open);
    assert_eq!(root_attrs.data_state_str(), "open");
    assert!(!root_attrs.is_disabled());

    let trigger_attrs = select.trigger_attributes();
    assert_eq!(trigger_attrs.id(), scope.qualify("trigger"));
    assert_eq!(trigger_attrs.role(), "combobox");
    assert_eq!(trigger_attrs.aria_haspopup(), "listbox");
    assert_eq!(trigger_attrs.aria_expanded(), "true");
    assert_eq!(trigger_attrs.aria_controls(), scope.qualify("content"));
    assert_eq!(trigger_attrs.data_state_str(), "open");
    assert_eq!(trigger_attrs.tabindex(), 0);
    assert_eq!(trigger_attrs.aria_disabled(), None);

    let candidate_id = scope.qualify("item-banana");
    let content_attrs = select.content_attributes(Some(candidate_id.clone()));
    assert_eq!(content_attrs.id(), scope.qualify("content"));
    assert_eq!(content_attrs.role(), "listbox");
    assert_eq!(content_attrs.tabindex(), -1);
    assert_eq!(
        content_attrs.aria_activedescendant(),
        Some(candidate_id.as_str())
    );
    assert_eq!(content_attrs.data_state_str(), "open");
    assert_eq!(content_attrs.data_side_str(), "bottom");
    assert_eq!(content_attrs.data_align_str(), "start");

    let item_banana = select.item_attributes("banana", true, false);
    assert_eq!(item_banana.role(), "option");
    assert_eq!(item_banana.aria_selected(), "true");
    assert_eq!(item_banana.data_state(), "checked");
    assert!(item_banana.is_highlighted());
    assert!(!item_banana.is_disabled());

    let item_apple = select.item_attributes("apple", false, false);
    assert_eq!(item_apple.role(), "option");
    assert_eq!(item_apple.aria_selected(), "false");
    assert_eq!(item_apple.data_state(), "unchecked");
    assert!(!item_apple.is_highlighted());
    assert!(!item_apple.is_disabled());
}

#[test]
fn select_groups_and_labels_accessibility_contract() {
    let scope = ScopeHandle::root("select-test").child("grouped");
    let relationships = SelectRelationships::new(scope.clone());

    let group_id = relationships.group_id("veggies");
    let label_id = relationships.label_id("veggies");

    assert_eq!(group_id, scope.qualify("group-veggies"));
    assert_eq!(label_id, scope.qualify("label-veggies"));
}

#[test]
fn select_static_content_lane_contract() {
    let scope = ScopeHandle::root("select-test").child("static");
    let select = Select::new(scope.clone())
        .with_value(Some("static-val".to_string()))
        .with_open(true);

    let attrs = select.content_attributes(None);
    assert_eq!(attrs.id(), scope.qualify("content"));
    assert_eq!(attrs.role(), "listbox");
    assert_eq!(attrs.tabindex(), -1);
    assert_eq!(attrs.data_state_str(), "open");
}

#[test]
fn select_portal_configurable_routing_contract() {
    let scope = ScopeHandle::root("select-test").child("portal-config");

    let def_select = Select::new(scope.clone());
    assert!(def_select.portal_host().is_default_host());

    let custom_host = PortalHost::named("custom-container");
    let custom_select = Select::new(scope.clone()).with_portal_host(custom_host.clone());
    assert_eq!(custom_select.portal_host(), &custom_host);
    assert_eq!(custom_select.portal_host().id(), Some("custom-container"));

    let inline_host = PortalHost::inline();
    let inline_select = Select::new(scope).with_portal_host(inline_host.clone());
    assert!(inline_select.portal_host().is_inline());
}
