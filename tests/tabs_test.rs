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
fn tabs_decoupled_tab_stop_supports_manual_activation_mode() {
    let scope = ScopeHandle::root("tabs-test").child("manual");
    let mut tabs =
        Tabs::new(scope.clone(), "account").with_activation_mode(TabsActivationMode::Manual);

    assert_eq!(tabs.active_value(), "account");
    assert_eq!(tabs.current_tab_stop(), "account");

    // Move tab-stop to "password" while active_value remains "account"
    tabs.set_tab_stop("password");
    assert_eq!(tabs.active_value(), "account");
    assert_eq!(tabs.current_tab_stop(), "password");

    // Now "password" has tabindex="0", but "account" is still aria-selected="true"
    let account_trigger = tabs.trigger("account", false);
    let password_trigger = tabs.trigger("password", false);

    assert!(account_trigger.is_selected());
    assert_eq!(account_trigger.tabindex(), -1);
    assert_eq!(account_trigger.data_state_str(), "active");

    assert!(!password_trigger.is_selected());
    assert_eq!(password_trigger.tabindex(), 0);
    assert_eq!(password_trigger.data_state_str(), "inactive");

    // Panels reflect active_value, not tab-stop
    assert!(!tabs.content("account").is_hidden());
    assert!(tabs.content("password").is_hidden());

    // Explicit selection synchronizes active_value
    tabs.select_tab("password");
    assert_eq!(tabs.active_value(), "password");
    assert_eq!(tabs.current_tab_stop(), "password");
    assert!(tabs.content("account").is_hidden());
    assert!(!tabs.content("password").is_hidden());
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

#[test]
fn tabs_orientation_aware_arrow_navigation_ltr_and_rtl() {
    use monoxus::tabs::TriggerRegistration;

    let scope = ScopeHandle::root("tabs-test").child("nav");
    let triggers = vec![
        TriggerRegistration {
            value: "tab1".to_string(),
            disabled: false,
        },
        TriggerRegistration {
            value: "tab2".to_string(),
            disabled: false,
        },
        TriggerRegistration {
            value: "tab3".to_string(),
            disabled: false,
        },
    ];

    // Horizontal LTR
    let horiz_ltr = Tabs::new(scope.child("h-ltr"), "tab1")
        .with_orientation(TabsOrientation::Horizontal)
        .with_direction(TabsDirection::Ltr)
        .with_loop_focus(true);

    assert_eq!(
        horiz_ltr.resolve_key_navigation(&triggers, "tab1", "ArrowRight"),
        Some("tab2".to_string())
    );
    assert_eq!(
        horiz_ltr.resolve_key_navigation(&triggers, "tab1", "ArrowLeft"),
        Some("tab3".to_string())
    );
    // Vertical keys ignored in horizontal orientation
    assert_eq!(
        horiz_ltr.resolve_key_navigation(&triggers, "tab1", "ArrowUp"),
        None
    );
    assert_eq!(
        horiz_ltr.resolve_key_navigation(&triggers, "tab1", "ArrowDown"),
        None
    );

    // Horizontal RTL
    let horiz_rtl = Tabs::new(scope.child("h-rtl"), "tab1")
        .with_orientation(TabsOrientation::Horizontal)
        .with_direction(TabsDirection::Rtl)
        .with_loop_focus(true);

    // In RTL, ArrowRight moves to previous (wrapped to tab3), ArrowLeft moves to next (tab2)
    assert_eq!(
        horiz_rtl.resolve_key_navigation(&triggers, "tab1", "ArrowRight"),
        Some("tab3".to_string())
    );
    assert_eq!(
        horiz_rtl.resolve_key_navigation(&triggers, "tab1", "ArrowLeft"),
        Some("tab2".to_string())
    );

    // Vertical
    let vert = Tabs::new(scope.child("v"), "tab1")
        .with_orientation(TabsOrientation::Vertical)
        .with_loop_focus(true);

    assert_eq!(
        vert.resolve_key_navigation(&triggers, "tab1", "ArrowDown"),
        Some("tab2".to_string())
    );
    assert_eq!(
        vert.resolve_key_navigation(&triggers, "tab1", "ArrowUp"),
        Some("tab3".to_string())
    );
    // Horizontal keys ignored in vertical orientation
    assert_eq!(
        vert.resolve_key_navigation(&triggers, "tab1", "ArrowLeft"),
        None
    );
    assert_eq!(
        vert.resolve_key_navigation(&triggers, "tab1", "ArrowRight"),
        None
    );
}

#[test]
fn tabs_roving_focus_skips_disabled_triggers() {
    use monoxus::tabs::TriggerRegistration;

    let scope = ScopeHandle::root("tabs-test").child("disabled-nav");
    let triggers = vec![
        TriggerRegistration {
            value: "tab1".to_string(),
            disabled: false,
        },
        TriggerRegistration {
            value: "tab2".to_string(),
            disabled: true,
        },
        TriggerRegistration {
            value: "tab3".to_string(),
            disabled: false,
        },
    ];

    let tabs = Tabs::new(scope, "tab1")
        .with_orientation(TabsOrientation::Horizontal)
        .with_direction(TabsDirection::Ltr)
        .with_loop_focus(true);

    // ArrowRight from tab1 skips disabled tab2 and lands on tab3
    assert_eq!(
        tabs.resolve_key_navigation(&triggers, "tab1", "ArrowRight"),
        Some("tab3".to_string())
    );

    // ArrowLeft from tab3 skips disabled tab2 and lands on tab1
    assert_eq!(
        tabs.resolve_key_navigation(&triggers, "tab3", "ArrowLeft"),
        Some("tab1".to_string())
    );
}

#[test]
fn tabs_roving_focus_looping_vs_clamping() {
    use monoxus::tabs::TriggerRegistration;

    let scope = ScopeHandle::root("tabs-test").child("looping");
    let triggers = vec![
        TriggerRegistration {
            value: "first".to_string(),
            disabled: false,
        },
        TriggerRegistration {
            value: "last".to_string(),
            disabled: false,
        },
    ];

    // With loop_focus = true
    let looping_tabs = Tabs::new(scope.child("loop"), "last")
        .with_orientation(TabsOrientation::Horizontal)
        .with_loop_focus(true);
    assert_eq!(
        looping_tabs.resolve_key_navigation(&triggers, "last", "ArrowRight"),
        Some("first".to_string())
    );
    assert_eq!(
        looping_tabs.resolve_key_navigation(&triggers, "first", "ArrowLeft"),
        Some("last".to_string())
    );

    // With loop_focus = false
    let clamped_tabs = Tabs::new(scope.child("clamp"), "last")
        .with_orientation(TabsOrientation::Horizontal)
        .with_loop_focus(false);
    assert_eq!(
        clamped_tabs.resolve_key_navigation(&triggers, "last", "ArrowRight"),
        None
    );
    assert_eq!(
        clamped_tabs.resolve_key_navigation(&triggers, "first", "ArrowLeft"),
        None
    );
}

#[test]
fn tabs_home_and_end_boundary_navigation() {
    use monoxus::tabs::TriggerRegistration;

    let scope = ScopeHandle::root("tabs-test").child("boundary");
    let triggers = vec![
        TriggerRegistration {
            value: "t1".to_string(),
            disabled: true, // first is disabled
        },
        TriggerRegistration {
            value: "t2".to_string(),
            disabled: false, // first enabled
        },
        TriggerRegistration {
            value: "t3".to_string(),
            disabled: false, // last enabled
        },
        TriggerRegistration {
            value: "t4".to_string(),
            disabled: true, // last is disabled
        },
    ];

    let tabs = Tabs::new(scope, "t3");

    // Home jumps to first non-disabled ("t2")
    assert_eq!(
        tabs.resolve_key_navigation(&triggers, "t3", "Home"),
        Some("t2".to_string())
    );
    assert_eq!(
        tabs.resolve_key_navigation(&triggers, "t3", "PageUp"),
        Some("t2".to_string())
    );

    // End jumps to last non-disabled ("t3")
    assert_eq!(
        tabs.resolve_key_navigation(&triggers, "t2", "End"),
        Some("t3".to_string())
    );
    assert_eq!(
        tabs.resolve_key_navigation(&triggers, "t2", "PageDown"),
        Some("t3".to_string())
    );
}

#[test]
fn tabs_idempotent_activation_and_state_transitions() {
    let scope = ScopeHandle::root("tabs-test").child("idempotent");
    let mut tabs = Tabs::new(scope, "account");

    // Selecting current value is idempotent (returns false for changed)
    assert!(!tabs.select_tab("account"));
    assert_eq!(tabs.active_value(), "account");
    assert_eq!(tabs.current_tab_stop(), "account");

    // Selecting new value returns true
    assert!(tabs.select_tab("password"));
    assert_eq!(tabs.active_value(), "password");
    assert_eq!(tabs.current_tab_stop(), "password");

    // Selecting it again immediately is a no-op (idempotent for dual mousedown/click)
    assert!(!tabs.select_tab("password"));
}

#[test]
fn tabs_composition_and_controllable_state_foundation_exports() {
    use monoxus::tabs::{
        ControllableStateProps, compose_part_event_handlers, compose_part_refs, project_as_child,
        use_controllable_state,
    };

    let _ = use_controllable_state::<String, fn(String)>;
    let _ = ControllableStateProps::<String, fn(String)> {
        value: None,
        default_value: "test".to_string(),
        on_change: None,
    };
    let called = std::rc::Rc::new(std::cell::Cell::new(false));
    let mut handler = {
        let called = std::rc::Rc::clone(&called);
        compose_part_event_handlers(
            Some(move |_: &mut ()| called.set(true)),
            None::<fn(&mut ())>,
            monoxus::foundation::compose::EventHandlerOptions::default(),
        )
    };
    handler(&mut ());
    assert!(called.get());

    let ref_called = std::rc::Rc::new(std::cell::Cell::new(false));
    let refs: Vec<Option<monoxus::foundation::compose::RefHandler<()>>> = vec![Some(Box::new({
        let ref_called = std::rc::Rc::clone(&ref_called);
        move |_| ref_called.set(true)
    }))];
    let mut composed_refs = compose_part_refs(refs);
    composed_refs(());
    assert!(ref_called.get());

    let (projected_target, projected_child) = project_as_child(
        "target",
        monoxus::foundation::compose::Slottable::new("child"),
    );
    assert_eq!(projected_target, "target");
    assert_eq!(projected_child, "child");

    // Boundary check: deferred product items
    let parts: Vec<_> = Tabs::parts().iter().map(TabsPart::as_str).collect();
    for deferred in ["header", "footer", "card", "nav-tabs", "tab-indicator"] {
        assert!(!parts.contains(&deferred));
    }
}
