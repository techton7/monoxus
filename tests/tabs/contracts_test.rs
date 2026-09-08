use monoxus::{
    foundation::{shared::ScopeHandle, state::DataState},
    tabs::{
        TABS_PARTS, Tabs, TabsActivationMode, TabsDirection, TabsOrientation, TabsPart,
        TabsRelationships,
    },
};

#[test]
fn tabs_part_inventory_matches_exhaustive_surface() {
    let parts: Vec<_> = Tabs::parts().iter().map(TabsPart::as_str).collect();
    assert_eq!(parts, vec!["root", "list", "trigger", "content"]);
    assert_eq!(TABS_PARTS.len(), 4);
}

#[test]
fn tabs_relationships_produce_deterministic_cross_referencing_ids() {
    let scope = ScopeHandle::root("tabs-test").child("main");
    let relationships = TabsRelationships::new(scope.clone());

    assert_eq!(relationships.scope(), &scope);
    assert_eq!(relationships.root_id(), scope.token());
    assert_eq!(relationships.list_id(), scope.qualify("list"));
    assert_eq!(
        relationships.trigger_id("account"),
        scope.qualify("trigger-account")
    );
    assert_eq!(
        relationships.content_id("account"),
        scope.qualify("content-account")
    );
    assert_eq!(
        relationships.trigger_id("password"),
        scope.qualify("trigger-password")
    );
    assert_eq!(
        relationships.content_id("password"),
        scope.qualify("content-password")
    );
}

#[test]
fn tabs_attributes_publish_wai_aria_and_data_attributes() {
    let scope = ScopeHandle::root("tabs-test").child("attrs");
    let tabs = Tabs::new(scope.clone(), "account")
        .with_orientation(TabsOrientation::Horizontal)
        .with_direction(TabsDirection::Ltr)
        .with_activation_mode(TabsActivationMode::Automatic);

    let root_attrs = tabs.root();
    assert_eq!(root_attrs.id(), scope.token());
    assert_eq!(root_attrs.data_orientation(), "horizontal");

    let list_attrs = tabs.list();
    assert_eq!(list_attrs.id(), scope.qualify("list"));
    assert_eq!(list_attrs.role(), "tablist");
    assert_eq!(list_attrs.aria_orientation(), "horizontal");
    assert_eq!(list_attrs.data_orientation(), "horizontal");
    assert!(!list_attrs.is_disabled());

    // Active Trigger
    let active_trigger = tabs.trigger("account", false);
    assert_eq!(active_trigger.id(), scope.qualify("trigger-account"));
    assert_eq!(active_trigger.role(), "tab");
    assert!(active_trigger.is_selected());
    assert_eq!(active_trigger.aria_selected(), "true");
    assert_eq!(
        active_trigger.aria_controls(),
        scope.qualify("content-account")
    );
    assert_eq!(active_trigger.tabindex(), 0);
    assert_eq!(active_trigger.data_state(), DataState::Active);
    assert_eq!(active_trigger.data_state_str(), "active");
    assert_eq!(active_trigger.data_value(), "account");
    assert_eq!(active_trigger.data_orientation(), "horizontal");
    assert!(!active_trigger.is_disabled());

    // Inactive Trigger
    let inactive_trigger = tabs.trigger("password", false);
    assert_eq!(inactive_trigger.id(), scope.qualify("trigger-password"));
    assert_eq!(inactive_trigger.role(), "tab");
    assert!(!inactive_trigger.is_selected());
    assert_eq!(inactive_trigger.aria_selected(), "false");
    assert_eq!(
        inactive_trigger.aria_controls(),
        scope.qualify("content-password")
    );
    assert_eq!(inactive_trigger.tabindex(), -1);
    assert_eq!(inactive_trigger.data_state(), DataState::Inactive);
    assert_eq!(inactive_trigger.data_state_str(), "inactive");
    assert_eq!(inactive_trigger.data_value(), "password");
    assert_eq!(inactive_trigger.data_orientation(), "horizontal");
    assert!(!inactive_trigger.is_disabled());

    // Active Content
    let active_content = tabs.content("account");
    assert_eq!(active_content.id(), scope.qualify("content-account"));
    assert_eq!(active_content.role(), "tabpanel");
    assert_eq!(
        active_content.aria_labelledby(),
        scope.qualify("trigger-account")
    );
    assert_eq!(active_content.tabindex(), 0);
    assert!(!active_content.is_hidden());
    assert_eq!(active_content.data_state(), DataState::Active);
    assert_eq!(active_content.data_value(), "account");
    assert_eq!(active_content.data_orientation(), "horizontal");

    // Inactive Content
    let inactive_content = tabs.content("password");
    assert_eq!(inactive_content.id(), scope.qualify("content-password"));
    assert_eq!(inactive_content.role(), "tabpanel");
    assert_eq!(
        inactive_content.aria_labelledby(),
        scope.qualify("trigger-password")
    );
    assert_eq!(inactive_content.tabindex(), 0);
    assert!(inactive_content.is_hidden());
    assert_eq!(inactive_content.data_state(), DataState::Inactive);
    assert_eq!(inactive_content.data_value(), "password");
    assert_eq!(inactive_content.data_orientation(), "horizontal");
}

#[test]
fn tabs_root_level_disabled_cascades_to_list_and_triggers() {
    let scope = ScopeHandle::root("tabs-test").child("disabled");
    let tabs = Tabs::new(scope, "account").with_disabled(true);

    assert!(tabs.list().is_disabled());
    assert!(tabs.trigger("account", false).is_disabled());
    assert!(tabs.trigger("password", false).is_disabled());

    // Trigger-level disabled overrides root
    let normal_tabs = Tabs::new(ScopeHandle::root("tabs-test").child("trig-dis"), "account");
    assert!(!normal_tabs.list().is_disabled());
    assert!(!normal_tabs.trigger("account", false).is_disabled());
    assert!(normal_tabs.trigger("password", true).is_disabled());
}
