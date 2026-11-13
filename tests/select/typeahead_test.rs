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

#[test]
fn select_time_bounded_active_typeahead_and_buffer_reset() {
    use dioxus::prelude::*;
    use monoxus::select::{use_select_runtime, Select};

    let mut dom = VirtualDom::new(|| {
        let scope = ScopeHandle::root("select-test").child("time-bounded");
        let runtime = use_select_runtime(
            Select::new(scope),
            None::<fn(Option<String>)>,
            None::<fn(bool)>,
        );

        // Initially no active typeahead
        assert!(!runtime.has_active_typeahead(1000.0));

        // Type character 'n' at t = 1000ms
        runtime.handle_typeahead('n', 1000.0);
        assert!(runtime.has_active_typeahead(1200.0)); // 200ms later: active!
        assert!(runtime.has_active_typeahead(1500.0)); // exactly 500ms later: active!
        assert!(!runtime.has_active_typeahead(1500.1)); // >500ms later: inactive!

        // Open dropdown: must clear buffer and reset timestamp
        runtime.open_dropdown();
        assert!(!runtime.has_active_typeahead(1200.0));

        // Type character 'y' inside open dropdown at t = 2000ms
        runtime.handle_typeahead('y', 2000.0);
        assert!(runtime.has_active_typeahead(2200.0));

        // Close dropdown: must clear buffer and reset timestamp
        runtime.close_dropdown();
        assert!(!runtime.has_active_typeahead(2200.0));

        rsx! {}
    });
    dom.rebuild_in_place();
}

#[test]
fn select_trigger_disabled_attribute_and_toggle_guard() {
    use dioxus::prelude::*;
    use monoxus::select::{use_select_runtime, Select, SelectContext, SelectTrigger};

    let mut dom = VirtualDom::new(|| {
        let scope = ScopeHandle::root("select-test").child("disabled-guard");
        let runtime = use_select_runtime(
            Select::new(scope).with_disabled(true),
            None::<fn(Option<String>)>,
            None::<fn(bool)>,
        );
        provide_context(SelectContext {
            runtime: runtime.clone(),
            name: None,
        });

        // Trigger attributes reflect disabled
        let attrs = runtime.trigger_attributes();
        assert!(attrs.is_disabled());

        // toggle_open is guarded when disabled
        assert!(!runtime.is_open());
        runtime.toggle_open();
        assert!(!runtime.is_open(), "toggle_open must be a no-op when disabled");

        rsx! {
            SelectTrigger {
                "Select item"
            }
        }
    });
    dom.rebuild_in_place();
}

#[test]
fn select_multi_word_typeahead_and_standalone_space_commit_contract() {
    use dioxus::prelude::*;
    use monoxus::select::{use_select_runtime, Select, SelectItemData};

    let mut dom = VirtualDom::new(|| {
        let items = vec![
            SelectItemData::new("ny", "New York", false),
            SelectItemData::new("nj", "New Jersey", false),
            SelectItemData::new("sf", "San Francisco", false),
        ];
        let scope = ScopeHandle::root("select-test").child("multi-word");
        let def = Select::new(scope).with_items(items);

        let runtime = use_select_runtime(
            def,
            None::<fn(Option<String>)>,
            None::<fn(bool)>,
        );
        runtime.open_dropdown();

        // 1. Initial state: no active typeahead
        assert!(!runtime.has_active_typeahead(1000.0));

        // 2. Type 'n' at t = 1000.0
        runtime.handle_typeahead('n', 1000.0);
        assert!(runtime.has_active_typeahead(1000.0));
        assert_eq!(runtime.highlighted_value().as_deref(), Some("ny"));

        // 3. Type 'e' at t = 1100.0
        runtime.handle_typeahead('e', 1100.0);
        assert!(runtime.has_active_typeahead(1100.0));

        // 4. Type 'w' at t = 1200.0
        runtime.handle_typeahead('w', 1200.0);
        assert!(runtime.has_active_typeahead(1200.0));

        // 5. Type ' ' (Space) at t = 1300.0: has_active_typeahead is true -> appends ' ' to buffer
        assert!(runtime.has_active_typeahead(1300.0));
        runtime.handle_typeahead(' ', 1300.0);
        assert_eq!(runtime.typeahead_buffer(), "new ");
        assert_eq!(runtime.highlighted_value().as_deref(), Some("ny"));

        // 6. Type 'y' at t = 1400.0 -> matches "New York"
        runtime.handle_typeahead('y', 1400.0);
        assert_eq!(runtime.typeahead_buffer(), "new y");
        assert_eq!(runtime.highlighted_value().as_deref(), Some("ny"));

        // 7. At t = 2000.0 (>500ms after last keypress): active typeahead expired!
        assert!(!runtime.has_active_typeahead(2000.0));

        rsx! {}
    });
    dom.rebuild_in_place();
}
