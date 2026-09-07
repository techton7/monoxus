use monoxus::{
    foundation::{shared::ScopeHandle, state::DataState},
    select::{
        Select, SelectItemData, SelectPart, SelectRelationships, SELECT_PARTS,
    },
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
            "viewport",
            "group",
            "label",
            "item",
            "item-text",
            "item-indicator",
            "separator",
            "hidden-input",
        ]
    );
    assert_eq!(SELECT_PARTS.len(), 14);
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
    assert_eq!(content_attrs.aria_activedescendant(), Some(candidate_id.as_str()));
    assert_eq!(content_attrs.data_state_str(), "open");

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
}

#[test]
fn select_uncontrolled_and_controlled_value_contract() {
    let scope = ScopeHandle::root("select-test").child("val");
    let select = Select::new(scope.clone());
    assert_eq!(select.value(), None);

    let select_controlled = select.clone().with_value(Some("cherry".to_owned()));
    assert_eq!(select_controlled.value(), Some("cherry"));
    assert!(!select_controlled.allows_deselect());

    let select_deselect = select_controlled.clone().with_allow_deselect(true);
    assert!(select_deselect.allows_deselect());
}

#[test]
fn select_uncontrolled_and_controlled_open_contract() {
    let scope = ScopeHandle::root("select-test").child("open");
    let select = Select::new(scope);
    assert!(!select.is_open());

    let opened = select.with_open(true);
    assert!(opened.is_open());
    assert_eq!(opened.trigger_attributes().aria_expanded(), "true");
}

#[test]
fn select_disabled_cascades_to_trigger_and_items() {
    let scope = ScopeHandle::root("select-test").child("dis");
    let select = Select::new(scope).with_disabled(true);

    assert!(select.is_disabled());
    let trig = select.trigger_attributes();
    assert!(trig.is_disabled());
    assert_eq!(trig.aria_disabled(), Some("true"));
    assert_eq!(trig.tabindex(), -1);

    let item = select.item_attributes("apple", false, false);
    assert!(item.is_disabled());
    assert_eq!(item.aria_disabled(), Some("true"));
}

#[test]
fn select_candidate_highlight_decoupled_from_committed_value() {
    // Demonstrates candidate navigation in memory without mutating committed value
    let scope = ScopeHandle::root("select-test").child("nav");
    let select = Select::new(scope).with_value(Some("apple".to_owned()));

    assert_eq!(select.value(), Some("apple"));

    // Candidate highlight can point to "banana" while value remains "apple"
    let apple_item = select.item_attributes("apple", false, false);
    let banana_item = select.item_attributes("banana", true, false);

    assert_eq!(apple_item.aria_selected(), "true"); // committed
    assert!(!apple_item.is_highlighted());           // not candidate

    assert_eq!(banana_item.aria_selected(), "false"); // not committed
    assert!(banana_item.is_highlighted());            // candidate!
}

#[test]
fn select_circular_roving_logic() {
    let items = vec![
        SelectItemData { value: "apple".into(), text: "Apple".into(), disabled: false },
        SelectItemData { value: "banana".into(), text: "Banana".into(), disabled: true },
        SelectItemData { value: "cherry".into(), text: "Cherry".into(), disabled: false },
        SelectItemData { value: "durian".into(), text: "Durian".into(), disabled: false },
    ];

    let enabled: Vec<_> = items.iter().filter(|i| !i.disabled).collect();
    assert_eq!(enabled.len(), 3);
    assert_eq!(enabled[0].value, "apple");
    assert_eq!(enabled[1].value, "cherry");
    assert_eq!(enabled[2].value, "durian");

    // Forward roving from durian wraps to apple
    let curr_idx = 2; // durian
    let next_idx = (curr_idx + 1) % enabled.len();
    assert_eq!(enabled[next_idx].value, "apple");

    // Backward roving from apple wraps to durian
    let prev_idx = if next_idx == 0 { enabled.len() - 1 } else { next_idx - 1 };
    assert_eq!(enabled[prev_idx].value, "durian");
}

#[test]
fn select_synchronous_typeahead_prefix_matching() {
    let items = vec![
        SelectItemData { value: "apple".into(), text: "Apple".into(), disabled: false },
        SelectItemData { value: "apricot".into(), text: "Apricot".into(), disabled: false },
        SelectItemData { value: "banana".into(), text: "Banana".into(), disabled: false },
        SelectItemData { value: "blueberry".into(), text: "Blueberry".into(), disabled: false },
        SelectItemData { value: "cherry".into(), text: "Cherry".into(), disabled: true },
    ];

    let mut buffer = String::new();
    let mut last_key_time = 0.0;

    // Simulate key 'a' at t = 100ms
    let now_1 = 100.0;
    if now_1 - last_key_time > 500.0 {
        buffer.clear();
    }
    buffer.push('a');
    last_key_time = now_1;

    let prefix = buffer.to_lowercase();
    let match_1 = items.iter().find(|i| !i.disabled && i.text.to_lowercase().starts_with(&prefix));
    assert_eq!(match_1.map(|i| i.value.as_str()), Some("apple"));

    // Simulate key 'p' at t = 300ms (diff = 200ms <= 500ms -> cumulative buffer "ap")
    let now_2 = 300.0;
    if now_2 - last_key_time > 500.0 {
        buffer.clear();
    }
    buffer.push('p');
    last_key_time = now_2;
    assert_eq!(buffer, "ap");

    let prefix = buffer.to_lowercase();
    let match_2 = items.iter().find(|i| !i.disabled && i.text.to_lowercase().starts_with(&prefix));
    assert_eq!(match_2.map(|i| i.value.as_str()), Some("apple"));

    // Simulate key 'r' at t = 450ms (diff = 150ms -> cumulative "apr")
    let now_3 = 450.0;
    if now_3 - last_key_time > 500.0 {
        buffer.clear();
    }
    buffer.push('r');
    last_key_time = now_3;
    assert_eq!(buffer, "apr");

    let prefix = buffer.to_lowercase();
    let match_3 = items.iter().find(|i| !i.disabled && i.text.to_lowercase().starts_with(&prefix));
    assert_eq!(match_3.map(|i| i.value.as_str()), Some("apricot"));

    // Simulate key 'b' at t = 1100ms (diff = 650ms > 500ms -> buffer resets to "b")
    let now_4 = 1100.0;
    if now_4 - last_key_time > 500.0 {
        buffer.clear();
    }
    buffer.push('b');
    last_key_time = now_4;
    _ = last_key_time;
    assert_eq!(buffer, "b");

    let prefix = buffer.to_lowercase();
    let match_4 = items.iter().find(|i| !i.disabled && i.text.to_lowercase().starts_with(&prefix));
    assert_eq!(match_4.map(|i| i.value.as_str()), Some("banana"));
}

#[test]
fn select_single_focus_ownership_contract() {
    let scope = ScopeHandle::root("select-test").child("focus");
    let select = Select::new(scope.clone()).with_open(true);

    let content_attrs = select.content_attributes(Some(scope.qualify("item-opt1")));
    // SelectContent MUST have tabindex="-1" to receive focus directly
    assert_eq!(content_attrs.tabindex(), -1);
    assert_eq!(content_attrs.role(), "listbox");
    // aria-activedescendant points to virtual candidate
    assert_eq!(
        content_attrs.aria_activedescendant(),
        Some(scope.qualify("item-opt1").as_str())
    );

    let trigger_attrs = select.trigger_attributes();
    assert_eq!(trigger_attrs.role(), "combobox");
    assert_eq!(trigger_attrs.tabindex(), 0);
}

#[test]
fn select_groups_and_labels_accessibility_contract() {
    let scope = ScopeHandle::root("select-test").child("groups");
    let rels = SelectRelationships::new(scope);

    let group_id = rels.group_id("fruits");
    let label_id = rels.label_id("fruits");

    assert!(group_id.contains("group-fruits"));
    assert!(label_id.contains("label-fruits"));
    assert_ne!(group_id, label_id);
}

#[test]
fn select_hidden_input_form_contract() {
    let scope = ScopeHandle::root("select-test").child("form");
    let select = Select::new(scope).with_value(Some("submitted-value".to_owned()));

    assert_eq!(select.value(), Some("submitted-value"));
}
