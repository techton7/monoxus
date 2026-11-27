use std::{cell::RefCell, rc::Rc};

use monoxus::{
    foundation::{
        compose::{EventHandlerOptions, RefHandler, Slottable},
        overlay::{FloatingLayer, PlacementAlign, PlacementSide, PortalHost, Rect, Size},
        shared::ScopeHandle,
        state::{self, DataState},
    },
    popover::{
        Popover, PopoverCloseFocusPolicy, PopoverOpenFocusPolicy, PopoverOutsideDismissBehavior,
        PopoverOutsideInteractionPolicy, PopoverRuntime, PopoverScrollLockPolicy,
        PopoverStateRequest, compose_part_event_handlers as compose_popover_event_handlers,
        compose_part_refs as compose_popover_refs, project_as_child as project_popover_as_child,
        use_popover_runtime,
    },
};

use super::fixtures::{TestEvent, is_default_prevented};

#[test]
fn popover_reuses_the_shared_state_composition_and_overlay_backbone() {
    let _ = use_popover_runtime::<fn(bool)>;
    let _ = PopoverRuntime::mount_anchor;
    let _ = PopoverRuntime::placement;
    let _ = PopoverRuntime::outside_pointer_down;
    let _ = PopoverRuntime::outside_focus_in;
    let _ = PopoverRuntime::escape_keydown;

    let _ = state::use_controllable_state::<bool, fn(bool)>;
    let _ = state::use_controllable_state_reducer::<bool, bool, fn(&bool, bool) -> bool, fn(bool)>;

    let scope = ScopeHandle::root("popover").child("root");
    let branch_id = scope.qualify("branch");
    let restore_id = scope.qualify("restore");
    let mut popover = Popover::new(scope.clone(), true)
        .with_portal_host(PortalHost::named("layers"))
        .with_floating(
            FloatingLayer::new(PlacementSide::Right)
                .with_align(PlacementAlign::End)
                .with_side_offset(6.0)
                .with_available_space(Size::new(180.0, 140.0)),
        )
        .with_modal(true)
        .with_open_focus_policy(PopoverOpenFocusPolicy::Target(branch_id.clone()))
        .with_close_focus_policy(PopoverCloseFocusPolicy::Target(restore_id.clone()))
        .with_scroll_lock_policy(PopoverScrollLockPolicy::enabled().with_restore_delay(Some(12)))
        .with_outside_interaction_policy(PopoverOutsideInteractionPolicy::new(
            PopoverOutsideDismissBehavior::Ignore,
            PopoverOutsideDismissBehavior::Dismiss,
        ));
    let relationships = popover.relationships().clone();
    let geometry = popover.geometry_vars(Rect::new(40.0, 20.0, 30.0, 10.0), Size::new(50.0, 20.0));
    let root = popover.root();
    let trigger = popover.trigger();
    let anchor = popover.anchor();
    let portal = popover.portal();
    let content = popover.content();
    let arrow = popover.arrow();
    let close = popover.close();

    assert_eq!(relationships.scope(), &scope);
    assert_eq!(relationships.root_id(), scope.token());
    assert_eq!(relationships.trigger_id(), scope.qualify("trigger"));
    assert_eq!(relationships.anchor_id(), scope.qualify("anchor"));
    assert_eq!(relationships.content_id(), scope.qualify("content"));
    assert_eq!(relationships.arrow_id(), scope.qualify("arrow"));
    assert_eq!(relationships.close_id(), scope.qualify("close"));

    assert_eq!(popover.data_state(), DataState::Open);
    assert_eq!(root.id(), relationships.root_id());
    assert_eq!(root.data_state(), &DataState::Open);
    assert_eq!(trigger.id(), relationships.trigger_id());
    assert_eq!(trigger.aria_controls(), Some(relationships.content_id()));
    assert_eq!(trigger.target_content_id(), relationships.content_id());
    assert_eq!(trigger.data_disabled(), None);
    assert!(!trigger.is_disabled());
    assert!(trigger.aria_expanded());
    assert_eq!(trigger.data_state(), &DataState::Open);
    assert_eq!(trigger.open_request(), PopoverStateRequest::Toggle);
    assert!(!trigger.open_request().next_open(true));
    assert_eq!(trigger.open_request().data_state(true), DataState::Closed);

    let closed_popover = Popover::new(scope.clone(), false);
    assert_eq!(closed_popover.trigger().aria_controls(), None);
    assert_eq!(anchor.id(), relationships.anchor_id());
    assert_eq!(portal.host(), &PortalHost::named("layers"));
    assert_eq!(content.id(), relationships.content_id());
    assert_eq!(content.role(), "dialog");
    assert!(content.aria_modal());
    assert_eq!(content.data_state(), &DataState::Open);
    assert_eq!(content.data_side(), "right");
    assert_eq!(content.data_align(), "end");
    assert_eq!(arrow.id(), relationships.arrow_id());
    assert_eq!(arrow.data_state(), &DataState::Open);
    assert_eq!(arrow.data_side(), "right");
    assert_eq!(arrow.data_align(), "end");
    assert_eq!(close.id(), relationships.close_id());
    assert_eq!(close.data_state(), &DataState::Open);
    assert_eq!(close.close_request(), PopoverStateRequest::Close);
    assert!(!close.close_request().next_open(true));

    assert_eq!(
        popover.lifecycle().portal_host(),
        &PortalHost::named("layers")
    );
    assert!(popover.lifecycle().presence().is_mounted());
    assert!(popover.lifecycle().is_modal());
    assert_eq!(
        popover.lifecycle().open_focus_policy(),
        &PopoverOpenFocusPolicy::Target(branch_id.clone()),
    );
    assert_eq!(
        popover.lifecycle().close_focus_policy(),
        &PopoverCloseFocusPolicy::Target(restore_id.clone()),
    );
    assert_eq!(
        popover.lifecycle().scroll_lock_policy(),
        &PopoverScrollLockPolicy::enabled().with_restore_delay(Some(12)),
    );
    assert_eq!(
        popover.lifecycle().outside_interaction_policy(),
        &PopoverOutsideInteractionPolicy::new(
            PopoverOutsideDismissBehavior::Ignore,
            PopoverOutsideDismissBehavior::Dismiss,
        ),
    );
    assert_eq!(
        popover.lifecycle().focus_scope().root(),
        relationships.content_id()
    );
    assert!(popover.lifecycle().focus_scope().traps_focus());
    assert!(popover.lifecycle().focus_scope().loops_focus());
    assert_eq!(
        popover.lifecycle().focus_scope().autofocus_target(),
        Some(&branch_id),
    );
    assert_eq!(
        popover.lifecycle().focus_scope().restore_target(),
        Some(&restore_id),
    );
    assert_eq!(
        popover.lifecycle().dismiss_layer().id(),
        relationships.content_id()
    );
    assert!(
        popover
            .lifecycle()
            .dismiss_layer()
            .blocks_outside_interaction()
    );
    assert_eq!(popover.lifecycle().floating().data_side(), "right");
    assert_eq!(popover.lifecycle().floating().data_align(), "end");

    let lifecycle = popover.lifecycle_mut();
    assert!(lifecycle.register_branch(branch_id.clone()));
    assert_eq!(
        lifecycle.focus_scope_mut().activate(),
        Some(branch_id.clone())
    );
    assert_eq!(lifecycle.focus_scope_mut().deactivate(), Some(restore_id));
    assert_eq!(lifecycle.focus_guards_mut().retain(), 1);

    let stack = vec![
        String::from("background"),
        relationships.content_id().to_owned(),
    ];
    assert!(
        !lifecycle
            .dismiss_layer()
            .should_dismiss_outside_pointer(Some(&String::from("outside")), &stack)
    );
    assert!(
        lifecycle
            .dismiss_layer()
            .should_dismiss_outside_focus(Some(&String::from("outside")), &stack)
    );

    assert_eq!(geometry.namespace(), "popover");
    assert_eq!(geometry.get("--ui-popover-floating-x"), Some(76.0));
    assert_eq!(geometry.get("--ui-popover-floating-y"), Some(10.0));
    assert_eq!(
        geometry.get("--ui-popover-transform-origin-x"),
        Some(0.0),
    );
    assert_eq!(
        geometry.get("--ui-popover-transform-origin-y"),
        Some(20.0),
    );
    let monoxus_geometry = geometry.with_css_prefix("monoxus");
    assert_eq!(
        monoxus_geometry.get("--monoxus-popover-floating-x"),
        Some(76.0)
    );
    assert_eq!(
        monoxus_geometry.get("--monoxus-popover-floating-y"),
        Some(10.0)
    );
    assert_eq!(
        monoxus_geometry.get("--monoxus-popover-transform-origin-x"),
        Some(0.0),
    );
    assert_eq!(
        monoxus_geometry.get("--monoxus-popover-transform-origin-y"),
        Some(20.0),
    );

    let (projected_trigger, projected_content) = project_popover_as_child(
        relationships.trigger_id().to_owned(),
        Slottable::new(relationships.content_id().to_owned()).map(|id| format!("{id}-surface")),
    );
    assert_eq!(projected_trigger, relationships.trigger_id());
    assert_eq!(
        projected_content,
        format!("{}-surface", relationships.content_id())
    );

    let mut event = TestEvent::default();
    let mut handlers = compose_popover_event_handlers(
        Some(|event: &mut TestEvent| {
            event.calls.push("consumer");
            event.default_prevented = true;
        }),
        Some(|event: &mut TestEvent| event.calls.push("internal")),
        EventHandlerOptions::cancelable(is_default_prevented),
    );
    handlers(&mut event);
    assert_eq!(event.calls, vec!["consumer"]);

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
    let mut composed_refs = compose_popover_refs(refs);
    composed_refs(relationships.content_id().to_owned());
    assert_eq!(
        *seen.borrow(),
        vec![
            format!("trigger:{}", relationships.content_id()),
            format!("content:{}", relationships.content_id()),
        ],
    );
}

#[test]
fn fm_w1_pop_pre_placement_content_and_arrow_respect_overrides_and_closed_force_mount_branch() {
    use dioxus::prelude::*;
    use monoxus::{
        foundation::overlay::OverlayBranchContext,
        popover::{
            PopoverArrow, PopoverContent, PopoverContext, PopoverPortal, PopoverRoot,
            PopoverTrigger,
        },
    };

    #[derive(Clone, Default, PartialEq)]
    struct PrePlacementSnapshot {
        has_placement: bool,
        content_side: String,
        content_align: String,
        arrow_side: String,
        arrow_align: String,
    }

    #[component]
    fn PrePlacementProbe(out: Rc<RefCell<PrePlacementSnapshot>>) -> Element {
        let ctx = use_context::<PopoverContext>();
        let runtime = ctx.runtime.read();
        let content = runtime.content();
        let arrow = runtime.arrow();
        *out.borrow_mut() = PrePlacementSnapshot {
            has_placement: runtime.placement().is_some(),
            content_side: content.data_side().to_string(),
            content_align: content.data_align().to_string(),
            arrow_side: arrow.data_side().to_string(),
            arrow_align: arrow.data_align().to_string(),
        };
        rsx! {}
    }

    #[derive(Clone, Props)]
    struct PopoverGuardSampleProps {
        snapshot: Rc<RefCell<PrePlacementSnapshot>>,
        branches: Rc<RefCell<Vec<String>>>,
    }

    impl PartialEq for PopoverGuardSampleProps {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.snapshot, &other.snapshot)
                && Rc::ptr_eq(&self.branches, &other.branches)
        }
    }

    #[component]
    fn PopoverGuardSample(props: PopoverGuardSampleProps) -> Element {
        let reg_branches = props.branches.clone();
        let unreg_branches = props.branches.clone();
        use_context_provider(|| {
            OverlayBranchContext::new(
                move |b: String| reg_branches.borrow_mut().push(b),
                move |b: String| unreg_branches.borrow_mut().retain(|x| x != &b),
            )
        });

        rsx! {
            // M-2: Closed force_mount PopoverContent must not register in parent OverlayBranchContext.
            PopoverRoot {
                id: "pop-fm-closed-root",
                default_open: false,
                PopoverTrigger { id: "pop-fm-closed-trigger", "Closed" }
                PopoverPortal {
                    force_mount: true,
                    PopoverContent {
                        id: "pop-fm-closed-content",
                        force_mount: true,
                        "Closed Force Mount"
                    }
                }
            }
            // M-4: Before Floating UI placement runs, PopoverRuntime::content()/arrow() fall back to overrides.
            PopoverRoot {
                id: "pop-pre-place-root",
                default_open: true,
                sticky: Some(true),
                PopoverTrigger { id: "pop-pre-place-trigger", "Open" }
                PopoverPortal {
                    PopoverContent {
                        id: "pop-pre-place-content",
                        side: Some(PlacementSide::Left),
                        align: Some(PlacementAlign::Start),
                        PopoverArrow { id: "pop-pre-place-arrow" }
                        PrePlacementProbe { out: props.snapshot.clone() }
                    }
                }
            }
        }
    }

    let snapshot = Rc::new(RefCell::new(PrePlacementSnapshot::default()));
    let branches = Rc::new(RefCell::new(Vec::<String>::new()));

    let mut dom = VirtualDom::new_with_props(
        PopoverGuardSample,
        PopoverGuardSampleProps {
            snapshot: snapshot.clone(),
            branches: branches.clone(),
        },
    );
    dom.rebuild_in_place();

    assert!(
        !branches
            .borrow()
            .iter()
            .any(|b| b == "pop-fm-closed-content"),
        "M-2: Closed force_mount PopoverContent must not register in parent OverlayBranchContext, got {:?}",
        *branches.borrow()
    );

    let snap = snapshot.borrow();
    assert!(
        !snap.has_placement,
        "VDOM-only render must exercise pre-placement fallback"
    );
    assert_eq!(snap.content_side, "left");
    assert_eq!(snap.content_align, "start");
    assert_eq!(snap.arrow_side, "left");
    assert_eq!(snap.arrow_align, "start");
}


#[cfg(feature = "native")]
mod native_stage2a_tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use dioxus::prelude::*;
    use dioxus_core::{Attribute, AttributeValue};
    use keyboard_types::{Code, Key, NamedKey};
    use monoxus::popover::{
        EscapeKeyDownEvent, PopoverContent, PopoverPortal, PopoverRoot, PopoverTrigger,
    };

    use crate::common::native_fixture::{
        NATIVE_TEST_LOCK, advance_frame_and_pump, click_and_pump, mount_native_scenario,
        mount_native_scenario_with_props, send_key_and_pump,
    };

    #[component]
    fn PopoverAttrMergeScenario() -> Element {
        let extra_attrs = vec![
            Attribute::new("class", "extra-cls", None, false),
            Attribute::new("style", "color: red", None, false),
            Attribute {
                name: "style",
                value: AttributeValue::None,
                namespace: None,
                volatile: false,
            },
            Attribute::new("data-custom", "ok", None, false),
        ];

        rsx! {
            PopoverRoot {
                id: "attr-merge-popover",
                default_open: true,
                PopoverTrigger { id: "attr-merge-trigger", "Open" }
                PopoverPortal {
                    PopoverContent {
                        id: "attr-merge-content",
                        class: Some("base-cls".to_string()),
                        style: Some("margin: 0".to_string()),
                        attributes: extra_attrs,
                        "Body"
                    }
                }
            }
        }
    }

    #[test]
    fn fm_api_3_popover_content_merge_global_attributes_preserves_css_vars_and_merges_class_style() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(PopoverAttrMergeScenario);
        advance_frame_and_pump(&mut harness, 0.02);

        let cid = harness.node("#attr-merge-content");
        let base = harness.base();
        let node = base.get_node(cid).unwrap();

        let cls = node.attr("class".into()).unwrap_or_default();
        assert!(
            cls.contains("base-cls") && cls.contains("extra-cls"),
            "Expected merged class containing both base-cls and extra-cls, got: {cls}"
        );

        let style = node.attr("style".into()).unwrap_or_default();
        assert!(
            style.contains("margin: 0") && style.contains("color: red"),
            "Expected merged style containing both margin: 0 and color: red, got: {style}"
        );
        assert!(
            style.contains("--monoxus-popover-") || style.contains("--radix-popover-"),
            "Expected internal popover CSS custom properties preserved, got: {style}"
        );
        assert!(
            !style.contains(";;"),
            "Merged style must not contain duplicate semicolons: {style}"
        );
        assert_eq!(node.attr("data-custom".into()), Some("ok"));
    }

    #[derive(Clone, Props)]
    struct PopoverHandlerToggleProps {
        cb1_hits: Arc<AtomicUsize>,
        cb2_hits: Arc<AtomicUsize>,
    }

    impl PartialEq for PopoverHandlerToggleProps {
        fn eq(&self, other: &Self) -> bool {
            Arc::ptr_eq(&self.cb1_hits, &other.cb1_hits)
                && Arc::ptr_eq(&self.cb2_hits, &other.cb2_hits)
        }
    }

    #[component]
    fn PopoverHandlerToggleScenario(props: PopoverHandlerToggleProps) -> Element {
        let mut stage = use_signal(|| 0_u8);
        let open = use_signal(|| true);
        let cb1_hits = props.cb1_hits.clone();
        let cb2_hits = props.cb2_hits.clone();

        let on_escape: Option<EventHandler<EscapeKeyDownEvent>> = match stage() {
            0 => None,
            1 => Some(EventHandler::new(move |evt: EscapeKeyDownEvent| {
                cb1_hits.fetch_add(1, Ordering::SeqCst);
                evt.prevent_default();
            })),
            _ => Some(EventHandler::new(move |evt: EscapeKeyDownEvent| {
                cb2_hits.fetch_add(1, Ordering::SeqCst);
                evt.prevent_default();
            })),
        };

        rsx! {
            div {
                button {
                    id: "advance-handler-stage",
                    onclick: move |_| stage += 1,
                    "Stage {stage()}"
                }
                PopoverRoot {
                    id: "toggle-popover",
                    open: open,
                    outside_interaction: Some(monoxus::popover::PopoverOutsideInteractionPolicy::new(
                        monoxus::popover::PopoverOutsideDismissBehavior::Ignore,
                        monoxus::popover::PopoverOutsideDismissBehavior::Ignore,
                    )),
                    PopoverTrigger { id: "toggle-trigger", "Open" }
                    PopoverPortal {
                        PopoverContent {
                            id: "toggle-content",
                            on_escape_key_down: on_escape,
                            button { id: "toggle-inner", "Inner" }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn fm_api_4_popover_content_handler_toggle_none_to_some_cb1_to_some_cb2() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let cb1_hits = Arc::new(AtomicUsize::new(0));
        let cb2_hits = Arc::new(AtomicUsize::new(0));

        let mut harness = mount_native_scenario_with_props(
            PopoverHandlerToggleScenario,
            PopoverHandlerToggleProps {
                cb1_hits: cb1_hits.clone(),
                cb2_hits: cb2_hits.clone(),
            },
        );
        advance_frame_and_pump(&mut harness, 0.02);

        click_and_pump(&mut harness, "#advance-handler-stage");
        click_and_pump(&mut harness, "#advance-handler-stage");
        advance_frame_and_pump(&mut harness, 0.02);

        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        advance_frame_and_pump(&mut harness, 0.02);

        assert_eq!(cb1_hits.load(Ordering::SeqCst), 0);
        assert_eq!(cb2_hits.load(Ordering::SeqCst), 1);
        assert!(
            harness.query("#toggle-content").is_some(),
            "cb2 called prevent_default(), so PopoverContent must remain open"
        );
    }

    #[derive(Clone, Props)]
    struct PopoverStickyRenderCountProps {
        renders: Arc<AtomicUsize>,
    }

    impl PartialEq for PopoverStickyRenderCountProps {
        fn eq(&self, other: &Self) -> bool {
            Arc::ptr_eq(&self.renders, &other.renders)
        }
    }

    #[component]
    fn PopoverStickyCounter(props: PopoverStickyRenderCountProps) -> Element {
        props.renders.fetch_add(1, Ordering::SeqCst);
        rsx! { span { "Sticky Body" } }
    }

    #[component]
    fn PopoverStickyRootAndOverridesScenario(props: PopoverStickyRenderCountProps) -> Element {
        use monoxus::{
            foundation::overlay::{PlacementAlign, PlacementSide},
            popover::PopoverArrow,
        };

        rsx! {
            div {
                style: "padding-top: 220px; padding-left: 160px;",
                PopoverRoot {
                    id: "pop-sticky-root",
                    default_open: true,
                    sticky: Some(true),
                    PopoverTrigger {
                        id: "pop-sticky-trigger",
                        style: "display: inline-block; padding: 8px 16px;",
                        "Sticky Trigger"
                    }
                    PopoverPortal {
                        PopoverContent {
                            id: "pop-sticky-content",
                            side: Some(PlacementSide::Top),
                            align: Some(PlacementAlign::End),
                            side_offset: Some(12.0),
                            avoid_collisions: Some(false),
                            style: "width: 160px; height: 60px; padding: 8px;",
                            PopoverArrow { id: "pop-sticky-arrow" }
                            PopoverStickyCounter { renders: props.renders.clone() }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn fm_w1_pop_sticky_root_and_content_overrides_no_loop_and_preserves_placement() {
        use crate::common::native_fixture::assert_relative_above;

        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let renders = Arc::new(AtomicUsize::new(0));

        let mut harness = mount_native_scenario_with_props(
            PopoverStickyRootAndOverridesScenario,
            PopoverStickyRenderCountProps {
                renders: renders.clone(),
            },
        );
        advance_frame_and_pump(&mut harness, 0.02);
        advance_frame_and_pump(&mut harness, 0.02);
        advance_frame_and_pump(&mut harness, 0.02);

        let total_renders = renders.load(Ordering::SeqCst);
        assert!(
            total_renders <= 6,
            "H-1: PopoverRoot(sticky: Some(true)) + PopoverContent overrides must not enter a re-render loop (got {total_renders} renders)"
        );

        assert_eq!(
            harness.attr("#pop-sticky-content", "data-side").as_deref(),
            Some("top"),
            "H-1 / M-4: PopoverContent side=Top override must be preserved"
        );
        assert_eq!(
            harness.attr("#pop-sticky-content", "data-align").as_deref(),
            Some("end"),
            "H-1 / M-4: PopoverContent align=End override must be preserved"
        );
        assert_eq!(
            harness.attr("#pop-sticky-arrow", "data-side").as_deref(),
            Some("top"),
            "M-4: PopoverArrow must reflect PopoverContent side=Top"
        );
        assert_eq!(
            harness.attr("#pop-sticky-arrow", "data-align").as_deref(),
            Some("end"),
            "M-4: PopoverArrow must reflect PopoverContent align=End"
        );
        assert_relative_above(&harness, "#pop-sticky-trigger", "#pop-sticky-content", 2.0);
    }
}
