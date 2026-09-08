use monoxus::{
    foundation::shared::ScopeHandle,
    tabs::{Tabs, TabsDirection, TabsOrientation, TriggerRegistration},
};

#[test]
fn tabs_orientation_aware_arrow_navigation_ltr_and_rtl() {
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
