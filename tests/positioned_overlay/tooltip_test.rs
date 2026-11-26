use std::{cell::RefCell, rc::Rc};

use monoxus::{
    foundation::{
        compose::{EventHandlerOptions, RefHandler, Slottable},
        overlay::{FloatingLayer, PlacementAlign, PlacementSide, PortalHost, Rect, Size},
        shared::ScopeHandle,
        state::DataState,
    },
    tooltip::{
        Tooltip, TooltipProvider, TooltipRuntime, TooltipStateRequest,
        compose_part_event_handlers as compose_tooltip_event_handlers,
        compose_part_refs as compose_tooltip_refs, project_as_child as project_tooltip_as_child,
        use_tooltip_provider_runtime, use_tooltip_runtime,
    },
};

use super::fixtures::TestEvent;

#[test]
fn tooltip_reuses_the_shared_state_composition_and_overlay_backbone() {
    let _ = use_tooltip_provider_runtime;
    let _ = use_tooltip_runtime::<fn(bool)>;
    let _ = TooltipRuntime::mount_trigger;
    let _ = TooltipRuntime::mount_content;
    let _ = TooltipRuntime::placement;
    let _ = TooltipRuntime::escape_keydown;

    let provider = TooltipProvider::new(ScopeHandle::root("tooltip").child("provider"))
        .with_delay_duration_ms(600)
        .with_skip_delay_duration_ms(250)
        .with_disable_hoverable_content(true)
        .with_close_on_trigger_click(false)
        .with_ignore_non_keyboard_focus(true);
    let scope = ScopeHandle::root("tooltip").child("root");
    let mut tooltip = Tooltip::new(scope.clone(), true)
        .with_provider(provider.clone())
        .with_portal_host(PortalHost::inline())
        .with_floating(
            FloatingLayer::new(PlacementSide::Top)
                .with_align(PlacementAlign::Start)
                .with_side_offset(8.0)
                .with_available_space(Size::new(160.0, 120.0)),
        );
    let relationships = tooltip.relationships().clone();
    let geometry = tooltip.geometry_vars(Rect::new(60.0, 50.0, 40.0, 20.0), Size::new(50.0, 24.0));
    let root = tooltip.root();
    let trigger = tooltip.trigger();
    let portal = tooltip.portal();
    let content = tooltip.content();
    let arrow = tooltip.arrow();

    assert_eq!(relationships.scope(), &scope);
    assert_eq!(relationships.root_id(), scope.token());
    assert_eq!(relationships.trigger_id(), scope.qualify("trigger"));
    assert_eq!(relationships.content_id(), scope.qualify("content"));
    assert_eq!(relationships.arrow_id(), scope.qualify("arrow"));

    assert_eq!(provider.delay_duration_ms(), 600);
    assert_eq!(provider.skip_delay_duration_ms(), 250);
    assert!(provider.disable_hoverable_content());
    assert!(!provider.close_on_trigger_click());
    assert!(provider.ignore_non_keyboard_focus());

    assert_eq!(tooltip.data_state(), DataState::Open);
    assert_eq!(root.id(), relationships.root_id());
    assert_eq!(root.data_state(), &DataState::Open);
    assert_eq!(trigger.id(), relationships.trigger_id());
    assert_eq!(trigger.aria_describedby(), Some(relationships.content_id()));
    assert_eq!(trigger.provider_id(), Some(provider.id()));
    assert_eq!(trigger.data_state(), &DataState::Open);
    assert_eq!(trigger.open_request(), TooltipStateRequest::Open);
    assert_eq!(trigger.close_request(), TooltipStateRequest::Close);
    assert!(trigger.open_request().next_open());
    assert_eq!(trigger.close_request().data_state(), DataState::Closed);
    assert!(portal.host().is_inline());
    assert_eq!(content.id(), relationships.content_id());
    assert_eq!(content.role(), "tooltip");
    assert_eq!(content.data_state(), &DataState::Open);
    assert_eq!(content.data_side(), "top");
    assert_eq!(content.data_align(), "start");
    assert!(content.autofocus_suppressed());
    assert_eq!(arrow.id(), relationships.arrow_id());
    assert_eq!(arrow.data_state(), &DataState::Open);
    assert_eq!(arrow.data_side(), "top");
    assert_eq!(arrow.data_align(), "start");

    assert!(tooltip.lifecycle().presence().is_mounted());
    assert_eq!(
        tooltip.lifecycle().dismiss_layer().id(),
        relationships.content_id()
    );
    assert!(
        !tooltip
            .lifecycle()
            .dismiss_layer()
            .blocks_outside_interaction()
    );
    assert_eq!(tooltip.lifecycle().floating().data_side(), "top");
    assert_eq!(tooltip.lifecycle().floating().data_align(), "start");
    assert!(tooltip.lifecycle().content_autofocus_suppressed());
    assert_eq!(tooltip.provider(), Some(&provider));

    let lifecycle = tooltip.lifecycle_mut();
    assert!(lifecycle.register_branch(scope.qualify("content-branch")));

    let stack = vec![relationships.content_id().to_owned()];
    assert!(
        !lifecycle
            .dismiss_layer()
            .should_dismiss_outside_focus(Some(&scope.qualify("content-branch")), &stack)
    );
    assert!(
        lifecycle
            .dismiss_layer()
            .should_dismiss_outside_pointer(Some(&String::from("outside")), &stack)
    );

    assert_eq!(geometry.namespace(), "tooltip");
    assert_eq!(geometry.get("--monoxus-tooltip-floating-x"), Some(60.0));
    assert_eq!(geometry.get("--monoxus-tooltip-floating-y"), Some(18.0));
    assert_eq!(
        geometry.get("--monoxus-tooltip-transform-origin-x"),
        Some(0.0),
    );
    assert_eq!(
        geometry.get("--monoxus-tooltip-transform-origin-y"),
        Some(24.0),
    );

    let (projected_trigger, projected_content) = project_tooltip_as_child(
        relationships.trigger_id().to_owned(),
        Slottable::new(relationships.content_id().to_owned()).map(|id| format!("{id}-tooltip")),
    );
    assert_eq!(projected_trigger, relationships.trigger_id());
    assert_eq!(
        projected_content,
        format!("{}-tooltip", relationships.content_id())
    );

    let mut event = TestEvent::default();
    let mut handlers = compose_tooltip_event_handlers(
        Some(|event: &mut TestEvent| event.calls.push("consumer")),
        Some(|event: &mut TestEvent| event.calls.push("internal")),
        EventHandlerOptions::always_invoke_internal(),
    );
    handlers(&mut event);
    assert_eq!(event.calls, vec!["consumer", "internal"]);

    let seen = Rc::new(RefCell::new(Vec::new()));
    let refs: Vec<Option<RefHandler<String>>> = vec![
        Some(Box::new({
            let seen = Rc::clone(&seen);
            move |value| seen.borrow_mut().push(format!("trigger:{value}"))
        })),
        Some(Box::new({
            let seen = Rc::clone(&seen);
            move |value| seen.borrow_mut().push(format!("content:{value}"))
        })),
    ];
    let mut composed_refs = compose_tooltip_refs(refs);
    composed_refs(relationships.trigger_id().to_owned());
    assert_eq!(
        *seen.borrow(),
        vec![
            format!("trigger:{}", relationships.trigger_id()),
            format!("content:{}", relationships.trigger_id()),
        ],
    );
}


#[cfg(feature = "native")]
mod native_stage2a_tests {
    use dioxus::prelude::*;
    use dioxus_core::{Attribute, AttributeValue};
    use monoxus::tooltip::{
        TooltipArrow, TooltipContent, TooltipPortal, TooltipProvider, TooltipRoot, TooltipTrigger,
    };

    use crate::common::native_fixture::{
        NATIVE_TEST_LOCK, advance_frame_and_pump, drain_polls_and_resolve, focus_node,
        mount_native_scenario, wheel_at_and_pump,
    };

    #[component]
    fn TooltipScrollGuardAndAttrMergeScenario() -> Element {
        let extra_attrs = vec![
            Attribute::new("class", "tip-extra", None, false),
            Attribute::new("style", "color: red", None, false),
            Attribute {
                name: "style",
                value: AttributeValue::None,
                namespace: None,
                volatile: false,
            },
            Attribute::new("data-tip-custom", "ok", None, false),
        ];

        rsx! {
            div {
                id: "fm-ovl-7-scroll-box",
                style: "width: 320px; height: 140px; overflow-y: auto;",
                div {
                    style: "height: 600px; padding-top: 80px;",
                    TooltipProvider {
                        delay_duration: 0,
                        ignore_non_keyboard_focus: false,
                        TooltipRoot {
                            id: "fm-ovl-7-tooltip",
                            TooltipTrigger { id: "fm-ovl-7-trigger", "Focus Tip" }
                            TooltipPortal {
                                TooltipContent {
                                    id: "fm-ovl-7-content",
                                    class: Some("tip-base".to_string()),
                                    style: Some("margin: 0".to_string()),
                                    attributes: extra_attrs,
                                    TooltipArrow { id: "fm-ovl-7-arrow" }
                                    "Scroll guarded tooltip"
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn fm_ovl_7_and_fm_api_3_tooltip_scroll_guard_arrow_and_merge_global_attributes() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(TooltipScrollGuardAndAttrMergeScenario);

        // Focus trigger -> opens tooltip and arms 1-frame focus scroll guard
        focus_node(&mut harness, "#fm-ovl-7-trigger");
        drain_polls_and_resolve(&mut harness);
        assert!(
            harness.query("#fm-ovl-7-content").is_some(),
            "TooltipContent must open on trigger focus"
        );

        // Verify FM-API-3 attribute merging + arrow data attributes
        {
            let cid = harness.node("#fm-ovl-7-content");
            let aid = harness.node("#fm-ovl-7-arrow");
            let base = harness.base();
            let cnode = base.get_node(cid).unwrap();
            let cls = cnode.attr("class".into()).unwrap_or_default();
            assert!(
                cls.contains("tip-base") && cls.contains("tip-extra"),
                "Expected merged classes on TooltipContent, got: {cls}"
            );
            let style = cnode.attr("style".into()).unwrap_or_default();
            assert!(
                style.contains("margin: 0") && style.contains("color: red"),
                "Expected merged styles on TooltipContent, got: {style}"
            );
            assert!(
                style.contains("--monoxus-tooltip-") || style.contains("--radix-tooltip-"),
                "Expected internal tooltip CSS custom properties preserved, got: {style}"
            );
            assert!(
                !style.contains(";;"),
                "Merged TooltipContent style must not contain duplicate semicolons: {style}"
            );
            assert_eq!(cnode.attr("data-tip-custom".into()), Some("ok"));

            let anode = base.get_node(aid).unwrap();
            assert_eq!(anode.attr("id".into()), Some("fm-ovl-7-arrow"));
            assert!(anode.attr("data-side".into()).is_some());
            assert!(anode.attr("data-align".into()).is_some());
            assert_eq!(anode.attr("data-state".into()), Some("open"));
        }

        // Scroll within the 1-frame focus guard window (before frame tick clears guard):
        // dispatch wheel without advancing a frame first -> tooltip stays open.
        let rect = harness.layout_rect("#fm-ovl-7-scroll-box");
        harness.wheel_at(
            rect.x + rect.width - 20.0,
            rect.y + rect.height - 20.0,
            0.0,
            -20.0,
        );
        harness.pump();
        drain_polls_and_resolve(&mut harness);
        assert_eq!(
            harness.attr("#fm-ovl-7-trigger", "data-state").as_deref(),
            Some("open"),
            "Scroll within 1-frame focus guard must keep Tooltip open"
        );

        // Advance past the 1-frame guard, then scroll -> Tooltip must close
        advance_frame_and_pump(&mut harness, 0.05);
        advance_frame_and_pump(&mut harness, 0.05);
        wheel_at_and_pump(
            &mut harness,
            rect.x + rect.width - 20.0,
            rect.y + rect.height - 20.0,
            0.0,
            -50.0,
        );
        advance_frame_and_pump(&mut harness, 0.25);
        assert_eq!(
            harness.attr("#fm-ovl-7-trigger", "data-state").as_deref(),
            Some("closed"),
            "Scroll after 1-frame focus guard must close Tooltip"
        );
    }

    #[component]
    fn TooltipAriaLabelSurfaceScenario() -> Element {
        use monoxus::foundation::overlay::PlacementSide;

        rsx! {
            div {
                style: "padding: 40px;",
                style {
                    "@keyframes tip-aria-exit-kf {{ from {{ opacity: 1; }} to {{ opacity: 0; }} }}
                     .tip-aria-exit[data-state='closed'] {{ animation: tip-aria-exit-kf 0.2s linear; }}"
                }
                TooltipProvider {
                    delay_duration: 0,
                    TooltipRoot {
                        id: "tip-aria-root",
                        side: Some(PlacementSide::Bottom),
                        side_offset: Some(16.0),
                        TooltipTrigger {
                            id: "tip-aria-trigger",
                            style: "display: inline-block; padding: 6px 12px;",
                            "Aria Tip Trigger"
                        }
                        TooltipPortal {
                            TooltipContent {
                                id: "tip-aria-content",
                                aria_label: Some("Screen reader tooltip label".to_string()),
                                class: Some("tip-aria-exit".to_string()),
                                style: Some("width: 200px; height: 48px; display: block;".to_string()),
                                "Visible Tooltip Body"
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn fm_w1_tip_aria_label_measures_visible_surface_for_grace_and_presence_exit() {
        use monoxus::foundation::overlay::{GraceTransit, GraceTransitStatus, Point2D, Rect2D};

        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(TooltipAriaLabelSurfaceScenario);

        // Hover trigger -> opens immediately (delay_duration: 0).
        let (tx, ty) = harness.center_of("#tip-aria-trigger");
        harness.move_mouse_to(tx, ty);
        advance_frame_and_pump(&mut harness, 0.02);
        advance_frame_and_pump(&mut harness, 0.02);

        assert_eq!(
            harness.attr("#tip-aria-trigger", "aria-describedby").as_deref(),
            Some("tip-aria-content"),
            "Trigger aria-describedby must point to the accessible span id"
        );
        assert!(
            harness.query("#tip-aria-content-surface").is_some(),
            "H-2: Visible styled TooltipContent surface must have deterministic id"
        );

        let surface_rect = harness.layout_rect("#tip-aria-content-surface");
        assert!(
            surface_rect.width >= 180.0 && surface_rect.height >= 40.0,
            "H-2: Visible surface must measure full styled dimensions, got {}x{}",
            surface_rect.width,
            surface_rect.height
        );

        // Move pointer into corridor aimed toward the wide visible surface (well right of the 1x1 span).
        let trig_rect = harness.layout_rect("#tip-aria-trigger");
        let gap_y = (trig_rect.y + trig_rect.height + surface_rect.y) * 0.5;
        let corridor_x = surface_rect.x + surface_rect.width * 0.75;
        harness.move_mouse_to(corridor_x, gap_y);
        advance_frame_and_pump(&mut harness, 0.02);
        assert_eq!(
            harness.attr("#tip-aria-trigger", "data-state").as_deref(),
            Some("open"),
            "H-2: Grace corridor must use visible surface rect rather than 1x1 hidden span"
        );

        // Move mouse far away -> closes -> retained in DOM during 0.2s CSS exit animation on visible surface.
        harness.move_mouse_to(2.0, 2.0);
        std::thread::sleep(std::time::Duration::from_millis(140));
        advance_frame_and_pump(&mut harness, 0.02);
        assert_eq!(
            harness.attr("#tip-aria-trigger", "data-state").as_deref(),
            Some("closed"),
            "Tooltip must transition to closed after leaving corridor"
        );
        assert!(
            harness.query("#tip-aria-content-surface").is_some(),
            "H-2: use_presence must retain visible surface during active CSS exit animation"
        );

        // Advance animation clock past 0.2s -> native animationend on visible surface completes unmount.
        advance_frame_and_pump(&mut harness, 0.25);
        advance_frame_and_pump(&mut harness, 0.02);
        assert!(
            harness.query("#tip-aria-content-surface").is_none(),
            "H-2: Visible surface must unmount once its CSS exit animation completes"
        );

        // H-3: Border hit on `from_rect` must stay `InTransit`, never falsely returning `ReachedTarget`.
        let from = Rect2D::new(100.0, 50.0, 50.0, 20.0);
        let to = Rect2D::new(80.0, 100.0, 90.0, 40.0);
        let transit = GraceTransit::new(Point2D::new(125.0, 70.0), from, to, Some(5.0), false);
        assert_eq!(
            transit.inspect_point(Point2D::new(125.0, 70.0)),
            GraceTransitStatus::InTransit
        );
        assert_eq!(
            transit.inspect_point(Point2D::new(125.0, 60.0)),
            GraceTransitStatus::InTransit
        );
        assert_eq!(
            transit.inspect_point(Point2D::new(125.0, 115.0)),
            GraceTransitStatus::ReachedTarget
        );
    }

    #[derive(Clone, Props)]
    struct TooltipBranchOutProps {
        branches: std::rc::Rc<std::cell::RefCell<Vec<String>>>,
    }

    impl PartialEq for TooltipBranchOutProps {
        fn eq(&self, other: &Self) -> bool {
            std::rc::Rc::ptr_eq(&self.branches, &other.branches)
        }
    }

    #[component]
    fn TooltipModalPeAndForceMountScenario(props: TooltipBranchOutProps) -> Element {
        use monoxus::foundation::overlay::OverlayBranchContext;
        use monoxus::popover::{PopoverContent, PopoverPortal, PopoverRoot, PopoverTrigger};

        let reg_branches = props.branches.clone();
        let unreg_branches = props.branches.clone();
        use_context_provider(|| {
            OverlayBranchContext::new(
                move |b: String| reg_branches.borrow_mut().push(b),
                move |b: String| unreg_branches.borrow_mut().retain(|x| x != &b),
            )
        });

        rsx! {
            div {
                style: "padding: 40px;",
                // Closed force_mount Tooltip must NOT register in parent OverlayBranchContext (M-2).
                TooltipRoot {
                    id: "tip-fm-closed-root",
                    default_open: false,
                    TooltipTrigger { id: "tip-fm-closed-trigger", "Closed Tip" }
                    TooltipPortal {
                        force_mount: true,
                        TooltipContent {
                            id: "tip-fm-closed-content",
                            force_mount: true,
                            "Closed Force Mount Tip"
                        }
                    }
                }
                // Active modal Popover holds modal pointer lock -> open TooltipContent emits pointer-events: auto (H-5).
                PopoverRoot {
                    id: "tip-modal-host",
                    default_open: true,
                    modal: Some(true),
                    PopoverTrigger { id: "tip-modal-trigger", "Host" }
                    PopoverPortal {
                        PopoverContent {
                            id: "tip-modal-popover",
                            TooltipRoot {
                                id: "tip-modal-root",
                                default_open: true,
                                TooltipTrigger { id: "tip-modal-tip-trigger", "Nested Tip" }
                                TooltipPortal {
                                    TooltipContent {
                                        id: "tip-modal-content",
                                        "Inside Modal Tip"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn fm_w1_pe_and_guard_tooltip_modal_pointer_events_and_force_mount_branch() {
        use crate::common::native_fixture::mount_native_scenario_with_props;

        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let branches = std::rc::Rc::new(std::cell::RefCell::new(Vec::<String>::new()));

        let mut harness = mount_native_scenario_with_props(
            TooltipModalPeAndForceMountScenario,
            TooltipBranchOutProps {
                branches: branches.clone(),
            },
        );
        advance_frame_and_pump(&mut harness, 0.02);

        // M-2: Closed force_mount TooltipContent must NOT register in parent OverlayBranchContext.
        assert!(
            !branches
                .borrow()
                .iter()
                .any(|b| b == "tip-fm-closed-content"),
            "M-2: Closed force_mount TooltipContent must not register in parent OverlayBranchContext, got {:?}",
            *branches.borrow()
        );

        // H-5: Open TooltipContent under modal pointer lock must emit `pointer-events: auto;`.
        let style = harness
            .attr("#tip-modal-content", "style")
            .unwrap_or_default();
        assert!(
            style.contains("pointer-events: auto;"),
            "H-5: TooltipContent must emit `pointer-events: auto;` under modal pointer lock, got: {style}"
        );
    }
}
