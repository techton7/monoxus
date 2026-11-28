use ui_sdk::foundation::compose::project_attrs_as_child;
use ui_sdk::foundation::{
    compose::Slottable,
    shared::{Direction, Orientation, ScopeHandle},
    state::DataState,
};

use super::{
    relationships::NavigationMenuRelationships,
    state::{
        NavigationMenuDef, NavigationMenuFocusProxyDecision, NavigationMenuHoverAction,
        NavigationMenuHoverMachine, NavigationMenuSubDef, NavigationMenuTimerCommand,
        compute_motion_direction, is_content_entry_key, is_item_hover_enabled,
        resolve_content_arrow_target, resolve_focus_proxy_decision,
        should_dismiss_on_focus_outside,
    },
    types::{
        NavigationMenuActivationMode, NavigationMenuFocusProxySide, NavigationMenuGeometryVars,
        NavigationMenuIndicatorPosition, NavigationMenuLinkSelectDetail,
        NavigationMenuLinkSelectEvent, NavigationMenuMotion, NavigationMenuPart,
        NavigationMenuViewportSize,
    },
};

#[test]
fn hover_machine_executes_3_timer_model_and_skip_delay_window() {
    let mut machine = NavigationMenuHoverMachine::new(200, 300, 150);
    assert!(machine.is_open_delayed());

    // Enter trigger "a" while delayed -> starts 200ms open timer.
    let t1 = machine.dispatch(NavigationMenuHoverAction::TriggerEnter {
        item_value: "a".to_string(),
        activation_mode: NavigationMenuActivationMode::Automatic,
        item_open_on_hover: None,
    });
    assert_eq!(t1.next_value, None);
    assert_eq!(
        t1.commands,
        vec![NavigationMenuTimerCommand::StartOpenTimer {
            item_value: "a".to_string(),
            delay_ms: 200,
            generation: 1,
        }]
    );

    // Leave trigger "a" early -> cancels open timer and starts 150ms close grace.
    let t2 = machine.dispatch(NavigationMenuHoverAction::TriggerLeave {
        activation_mode: NavigationMenuActivationMode::Automatic,
        item_open_on_hover: None,
    });
    assert_eq!(
        t2.commands,
        vec![
            NavigationMenuTimerCommand::CancelOpenTimer,
            NavigationMenuTimerCommand::StartCloseTimer {
                delay_ms: 150,
                generation: 1,
            },
        ]
    );

    // Stale OpenTimerFired(gen=1) is ignored after cancellation.
    let stale_open = machine.dispatch(NavigationMenuHoverAction::OpenTimerFired {
        item_value: "a".to_string(),
        generation: 1,
    });
    assert_eq!(stale_open.next_value, None);

    // Enter trigger "a" again and let open timer fire.
    let t3 = machine.dispatch(NavigationMenuHoverAction::TriggerEnter {
        item_value: "a".to_string(),
        activation_mode: NavigationMenuActivationMode::Automatic,
        item_open_on_hover: None,
    });
    let open_gen = match &t3.commands[0] {
        NavigationMenuTimerCommand::StartOpenTimer { generation, .. } => *generation,
        other => panic!("unexpected command: {other:?}"),
    };
    let t4 = machine.dispatch(NavigationMenuHoverAction::OpenTimerFired {
        item_value: "a".to_string(),
        generation: open_gen,
    });
    assert_eq!(t4.next_value.as_deref(), Some("a"));

    // ValueCommitted("a") disables open delay while skipDelayDuration > 0.
    machine.dispatch(NavigationMenuHoverAction::ValueCommitted {
        next_value: "a".to_string(),
    });
    assert!(!machine.is_open_delayed());

    // Leave trigger "a" -> close grace starts; entering content cancels it.
    let t5 = machine.dispatch(NavigationMenuHoverAction::TriggerLeave {
        activation_mode: NavigationMenuActivationMode::Automatic,
        item_open_on_hover: None,
    });
    assert!(matches!(
        t5.commands.as_slice(),
        [NavigationMenuTimerCommand::StartCloseTimer { delay_ms: 150, .. }]
    ));
    let t6 = machine.dispatch(NavigationMenuHoverAction::ContentEnter {
        activation_mode: NavigationMenuActivationMode::Automatic,
    });
    assert_eq!(t6.commands, vec![NavigationMenuTimerCommand::CancelCloseTimer]);

    // Immediate switch to sibling trigger "b" while open skips 200ms delay.
    let t7 = machine.dispatch(NavigationMenuHoverAction::TriggerEnter {
        item_value: "b".to_string(),
        activation_mode: NavigationMenuActivationMode::Automatic,
        item_open_on_hover: None,
    });
    assert_eq!(t7.next_value.as_deref(), Some("b"));
    assert!(t7.commands.is_empty());

    // Content leave -> close timer fires -> ValueCommitted("") starts 300ms skipDelayTimer.
    machine.dispatch(NavigationMenuHoverAction::ValueCommitted {
        next_value: "b".to_string(),
    });
    let t8 = machine.dispatch(NavigationMenuHoverAction::ContentLeave {
        activation_mode: NavigationMenuActivationMode::Automatic,
        item_open_on_hover: None,
    });
    let close_gen = match &t8.commands[0] {
        NavigationMenuTimerCommand::StartCloseTimer { generation, .. } => *generation,
        other => panic!("unexpected command: {other:?}"),
    };
    let t9 = machine.dispatch(NavigationMenuHoverAction::CloseTimerFired {
        generation: close_gen,
    });
    assert_eq!(t9.next_value.as_deref(), Some(""));
    let t10 = machine.dispatch(NavigationMenuHoverAction::ValueCommitted {
        next_value: String::new(),
    });
    assert!(!machine.is_open_delayed());
    let skip_gen = match &t10.commands[0] {
        NavigationMenuTimerCommand::StartSkipDelayTimer {
            delay_ms: 300,
            generation,
        } => *generation,
        other => panic!("unexpected command: {other:?}"),
    };

    // Entering within 300ms skip window opens immediately.
    let t11 = machine.dispatch(NavigationMenuHoverAction::TriggerEnter {
        item_value: "a".to_string(),
        activation_mode: NavigationMenuActivationMode::Automatic,
        item_open_on_hover: None,
    });
    assert_eq!(t11.next_value.as_deref(), Some("a"));
    machine.dispatch(NavigationMenuHoverAction::ValueCommitted {
        next_value: "a".to_string(),
    });

    // After skip timer fires when closed, delay is restored.
    let t_reclose = machine.dispatch(NavigationMenuHoverAction::ValueCommitted {
        next_value: String::new(),
    });
    let skip_gen_2 = match &t_reclose.commands[0] {
        NavigationMenuTimerCommand::StartSkipDelayTimer {
            delay_ms: 300,
            generation,
        } => *generation,
        other => panic!("unexpected command: {other:?}"),
    };
    assert!(skip_gen_2 > skip_gen);
    let t12 = machine.dispatch(NavigationMenuHoverAction::SkipDelayTimerFired {
        generation: skip_gen_2,
    });
    assert!(t12.commands.is_empty());
    assert!(machine.is_open_delayed());
}

#[test]
fn hover_machine_respects_manual_mode_and_open_on_hover_false() {
    let mut machine = NavigationMenuHoverMachine::default();

    let t_manual = machine.dispatch(NavigationMenuHoverAction::TriggerEnter {
        item_value: "a".to_string(),
        activation_mode: NavigationMenuActivationMode::Manual,
        item_open_on_hover: None,
    });
    assert_eq!(t_manual.next_value, None);
    assert!(t_manual.commands.is_empty());

    let t_item_off = machine.dispatch(NavigationMenuHoverAction::TriggerEnter {
        item_value: "a".to_string(),
        activation_mode: NavigationMenuActivationMode::Automatic,
        item_open_on_hover: Some(false),
    });
    assert_eq!(t_item_off.next_value, None);
    assert!(t_item_off.commands.is_empty());

    assert!(!is_item_hover_enabled(NavigationMenuActivationMode::Manual, None));
    assert!(!is_item_hover_enabled(
        NavigationMenuActivationMode::Automatic,
        Some(false)
    ));
    assert!(is_item_hover_enabled(
        NavigationMenuActivationMode::Automatic,
        None
    ));
}

#[test]
fn motion_direction_handles_initial_open_close_ltr_rtl_and_3_item_preservation() {
    let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];

    // Initial open -> None
    assert_eq!(
        compute_motion_direction(&items, Direction::Ltr, "a", "", "a", None),
        None
    );

    // Full close -> None
    assert_eq!(
        compute_motion_direction(&items, Direction::Ltr, "", "a", "a", Some(NavigationMenuMotion::FromStart)),
        None
    );

    // Empty-state reset
    assert_eq!(
        compute_motion_direction(&items, Direction::Ltr, "", "", "c", Some(NavigationMenuMotion::FromEnd)),
        None
    );

    // LTR: "a" -> "b"
    assert_eq!(
        compute_motion_direction(&items, Direction::Ltr, "b", "a", "b", None),
        Some(NavigationMenuMotion::FromEnd)
    );
    assert_eq!(
        compute_motion_direction(&items, Direction::Ltr, "b", "a", "a", None),
        Some(NavigationMenuMotion::ToStart)
    );
    // Uninvolved item "c" preserves prev_motion
    assert_eq!(
        compute_motion_direction(
            &items,
            Direction::Ltr,
            "b",
            "a",
            "c",
            Some(NavigationMenuMotion::ToEnd)
        ),
        Some(NavigationMenuMotion::ToEnd)
    );

    // LTR: "b" -> "a"
    assert_eq!(
        compute_motion_direction(&items, Direction::Ltr, "a", "b", "a", None),
        Some(NavigationMenuMotion::FromStart)
    );
    assert_eq!(
        compute_motion_direction(&items, Direction::Ltr, "a", "b", "b", None),
        Some(NavigationMenuMotion::ToEnd)
    );

    // RTL reversal: "a" -> "b" becomes FromStart / ToEnd
    assert_eq!(
        compute_motion_direction(&items, Direction::Rtl, "b", "a", "b", None),
        Some(NavigationMenuMotion::FromStart)
    );
    assert_eq!(
        compute_motion_direction(&items, Direction::Rtl, "b", "a", "a", None),
        Some(NavigationMenuMotion::ToEnd)
    );
}

#[test]
fn sub_menu_inherits_direction_and_activation_mode_and_honors_disable_toggle() {
    let root_scope = ScopeHandle::root("navigation-menu").child("main");
    let rels = NavigationMenuRelationships::new(root_scope.clone());
    let sub_def = NavigationMenuSubDef::new(
        rels.sub_scope("nested"),
        Direction::Rtl,
        NavigationMenuActivationMode::Manual,
    )
    .with_value("sub-1")
    .into_def();

    assert!(!sub_def.is_root_menu());
    assert_eq!(sub_def.dir(), Direction::Rtl);
    assert_eq!(sub_def.activation_mode(), NavigationMenuActivationMode::Manual);
    assert!(sub_def.disable_toggle());
    assert_eq!(sub_def.aria_label(), None);

    // disable_toggle = true keeps active item open on re-click
    assert_eq!(sub_def.toggle_value("sub-1"), "sub-1");
    assert_eq!(sub_def.toggle_value("sub-2"), "sub-2");

    // disable_toggle = false allows closing on re-click
    let toggleable = sub_def.with_disable_toggle(false);
    assert_eq!(toggleable.toggle_value("sub-1"), "");

    // Override activation_mode
    let sub_auto = NavigationMenuSubDef::new(
        rels.sub_scope("nested-2"),
        Direction::Ltr,
        NavigationMenuActivationMode::Manual,
    )
    .with_activation_mode(Some(NavigationMenuActivationMode::Automatic))
    .into_def();
    assert_eq!(
        sub_auto.activation_mode(),
        NavigationMenuActivationMode::Automatic
    );
}

#[test]
fn attrs_relationships_css_vars_and_as_child_match_contract() {
    assert_eq!(NavigationMenuDef::parts().len(), 9);
    assert_eq!(NavigationMenuPart::Viewport.as_str(), "viewport");

    let scope = ScopeHandle::root("navigation-menu").child("nav");
    let def = NavigationMenuDef::new(scope)
        .with_orientation(Orientation::Horizontal)
        .with_dir(Direction::Ltr)
        .with_value("getting-started");

    let root_attrs = def.root_attributes(None);
    assert_eq!(root_attrs.aria_label(), Some("Main"));
    assert_eq!(root_attrs.data_orientation(), "horizontal");
    assert_eq!(root_attrs.dir(), "ltr");

    // Open trigger sets aria-expanded="true" and aria-controls; closed omits aria-controls; disabled emits data-disabled=""
    let open_trigger = def.trigger_attributes("getting-started", false, None, None);
    assert_eq!(open_trigger.aria_expanded(), "true");
    assert_eq!(
        open_trigger.aria_controls(),
        Some(def.relationships().content_id("getting-started").as_str())
    );
    assert_eq!(open_trigger.data_state(), &DataState::Open);
    assert_eq!(open_trigger.data_disabled(), None);

    let closed_trigger = def.trigger_attributes("components", true, None, None);
    assert_eq!(closed_trigger.aria_expanded(), "false");
    assert_eq!(closed_trigger.aria_controls(), None);
    assert_eq!(closed_trigger.data_state(), &DataState::Closed);
    assert_eq!(closed_trigger.data_disabled(), Some(""));

    // Link attrs
    let active_link = def.link_attributes(
        "intro",
        Some("/docs".to_string()),
        None,
        None,
        true,
        true,
        None,
    );
    assert_eq!(active_link.aria_current(), Some("page"));
    assert_eq!(active_link.data_active(), Some(""));
    assert_eq!(active_link.data_focused(), Some(""));
    assert_eq!(active_link.tabindex(), 0);

    // Viewport & Indicator dual CSS variables (--monoxus-* + --radix-*)
    let vp_attrs = def.viewport_attributes(
        false,
        Some(NavigationMenuViewportSize {
            width: 420.0,
            height: 240.0,
        }),
        None,
        None,
    );
    assert!(vp_attrs.style().contains("pointer-events: none;"));
    assert!(
        vp_attrs
            .style()
            .contains("--monoxus-navigation-menu-viewport-width: 420px;")
    );
    assert!(
        vp_attrs
            .style()
            .contains("--radix-navigation-menu-viewport-height: 240px;")
    );

    let ind_attrs = def.indicator_attributes(
        true,
        NavigationMenuIndicatorPosition {
            size: 110.0,
            offset: 48.0,
        },
        None,
        Some("transition: transform 200ms;"),
    );
    assert_eq!(ind_attrs.data_state(), "visible");
    assert!(ind_attrs.style().contains("position: absolute;"));
    assert!(ind_attrs.style().contains("width: 110px;"));
    assert!(ind_attrs.style().contains("translateX(48px)"));
    assert!(
        ind_attrs
            .style()
            .contains("--monoxus-navigation-menu-indicator-translate-x: 48px;")
    );
    assert!(
        ind_attrs
            .style()
            .contains("--radix-navigation-menu-indicator-translate-x: 48px;")
    );
    assert!(ind_attrs.style().contains("transition: transform 200ms;"));

    let geom = NavigationMenuGeometryVars::new(Orientation::Vertical)
        .with_indicator_position(Some(NavigationMenuIndicatorPosition {
            size: 32.0,
            offset: 64.0,
        }));
    let ind_vars = geom.indicator_css_variables();
    assert_eq!(ind_vars.len(), 2);
    assert_eq!(
        ind_vars[0].0,
        "--monoxus-navigation-menu-indicator-translate-y"
    );
    assert_eq!(
        ind_vars[1].0,
        "--radix-navigation-menu-indicator-translate-y"
    );

    // AsChild projections
    let (rt, ra, rc) =
        project_attrs_as_child(&root_attrs, "nav", Slottable::new("body"));
    assert_eq!(rt, "nav");
    assert_eq!(ra, root_attrs);
    assert_eq!(rc, "body");

    let (_, ta, _) =
        project_attrs_as_child(&open_trigger, "button", Slottable::new("t"));
    assert_eq!(ta, open_trigger);

    let content_attrs = def.content_attributes(
        "getting-started",
        true,
        Some(NavigationMenuMotion::FromEnd),
        true,
        None,
        None,
        None,
    );
    let (_, ca, _) =
        project_attrs_as_child(&content_attrs, "section", Slottable::new("c"));
    assert_eq!(ca.data_motion(), Some("from-end"));

    let (_, la, _) =
        project_attrs_as_child(&active_link, "a", Slottable::new("l"));
    assert_eq!(la, active_link);
    let (_, ia, _) =
        project_attrs_as_child(&ind_attrs, "div", Slottable::new("i"));
    assert_eq!(ia, ind_attrs);
    let (_, va, _) =
        project_attrs_as_child(&vp_attrs, "section", Slottable::new("v"));
    assert_eq!(va, vp_attrs);
}

#[test]
fn entry_keys_content_arrows_focus_proxy_and_link_select_work_cleanly() {
    assert!(is_content_entry_key(
        Orientation::Horizontal,
        Direction::Ltr,
        "ArrowDown"
    ));
    assert!(!is_content_entry_key(
        Orientation::Horizontal,
        Direction::Ltr,
        "ArrowRight"
    ));
    assert!(is_content_entry_key(
        Orientation::Vertical,
        Direction::Ltr,
        "ArrowRight"
    ));
    assert!(is_content_entry_key(
        Orientation::Vertical,
        Direction::Rtl,
        "ArrowLeft"
    ));

    let candidates = vec!["l1".to_string(), "l2".to_string(), "l3".to_string()];
    assert_eq!(
        resolve_content_arrow_target(&candidates, Some("l1"), "ArrowRight", Direction::Ltr),
        Some("l2".to_string())
    );
    assert_eq!(
        resolve_content_arrow_target(&candidates, Some("l1"), "ArrowDown", Direction::Ltr),
        Some("l2".to_string())
    );
    assert_eq!(
        resolve_content_arrow_target(&candidates, Some("l1"), "ArrowUp", Direction::Ltr),
        None
    );
    assert_eq!(
        resolve_content_arrow_target(&candidates, Some("l1"), "End", Direction::Ltr),
        Some("l3".to_string())
    );

    assert_eq!(
        resolve_focus_proxy_decision(Some("trig-1"), "trig-1", false),
        NavigationMenuFocusProxyDecision::Enter(NavigationMenuFocusProxySide::Start)
    );
    assert_eq!(
        resolve_focus_proxy_decision(Some("next-btn"), "trig-1", false),
        NavigationMenuFocusProxyDecision::Enter(NavigationMenuFocusProxySide::End)
    );
    assert_eq!(
        resolve_focus_proxy_decision(Some("inner-link"), "trig-1", true),
        NavigationMenuFocusProxyDecision::Ignore
    );

    assert!(!should_dismiss_on_focus_outside(true, true));
    assert!(!should_dismiss_on_focus_outside(false, false));
    assert!(should_dismiss_on_focus_outside(false, true));

    let evt = NavigationMenuLinkSelectEvent::new(NavigationMenuLinkSelectDetail {
        href: Some("/docs".to_string()),
        meta_key: false,
    });
    assert!(evt.detail().should_dismiss_root(evt.is_default_prevented()));
    evt.prevent_default();
    assert!(!evt.detail().should_dismiss_root(evt.is_default_prevented()));

    let meta_evt = NavigationMenuLinkSelectEvent::new(NavigationMenuLinkSelectDetail {
        href: Some("/docs".to_string()),
        meta_key: true,
    });
    assert!(
        !meta_evt
            .detail()
            .should_dismiss_root(meta_evt.is_default_prevented())
    );
}
