use monoxus::{
    foundation::shared::ScopeHandle,
    tabs::{
        ControllableStateProps, Tabs, TabsActivationMode, TabsPart, compose_part_event_handlers,
        compose_part_refs, project_as_child, use_controllable_state,
    },
};

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
