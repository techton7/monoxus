use monoxus::{
    accordion::{
        Accordion, AccordionDirection, AccordionItemRegistration, AccordionMode,
        AccordionOrientation,
    },
    foundation::shared::ScopeHandle,
};

#[test]
fn accordion_roving_focus_and_keyboard_navigation() {
    let scope = ScopeHandle::root("accordion-test").child("roving");
    let accordion =
        Accordion::new(scope, AccordionMode::Single { collapsible: true }).with_value("item-1");

    let items = vec![
        AccordionItemRegistration {
            value: "item-1".to_string(),
            disabled: false,
        },
        AccordionItemRegistration {
            value: "item-2".to_string(),
            disabled: true, // disabled item should be skipped!
        },
        AccordionItemRegistration {
            value: "item-3".to_string(),
            disabled: false,
        },
    ];

    // Down arrow from item-1 skips disabled item-2 and reaches item-3
    let next = accordion.resolve_key_navigation(&items, "item-1", "ArrowDown");
    assert_eq!(next, Some("item-3".to_string()));

    // Down arrow from item-3 with loop wraps to item-1
    let wrap_next = accordion.resolve_key_navigation(&items, "item-3", "ArrowDown");
    assert_eq!(wrap_next, Some("item-1".to_string()));

    // Up arrow from item-1 with loop wraps to item-3
    let wrap_prev = accordion.resolve_key_navigation(&items, "item-1", "ArrowUp");
    assert_eq!(wrap_prev, Some("item-3".to_string()));

    // Boundary keys Home / End
    assert_eq!(
        accordion.resolve_key_navigation(&items, "item-3", "Home"),
        Some("item-1".to_string())
    );
    assert_eq!(
        accordion.resolve_key_navigation(&items, "item-1", "End"),
        Some("item-3".to_string())
    );

    // Horizontal with RTL: ArrowRight goes prev, ArrowLeft goes next
    let rtl_accordion = Accordion::new(
        ScopeHandle::root("accordion-test").child("rtl"),
        AccordionMode::Single { collapsible: true },
    )
    .with_orientation(AccordionOrientation::Horizontal)
    .with_direction(AccordionDirection::Rtl);

    let rtl_next = rtl_accordion.resolve_key_navigation(&items, "item-1", "ArrowLeft");
    assert_eq!(rtl_next, Some("item-3".to_string()));

    let rtl_prev = rtl_accordion.resolve_key_navigation(&items, "item-3", "ArrowRight");
    assert_eq!(rtl_prev, Some("item-1".to_string()));
}

#[test]
fn accordion_focused_item_unmount_recovers_tab_stop() {
    use std::cell::RefCell;
    use std::rc::Rc;

    use dioxus::prelude::*;
    use monoxus::accordion::{
        AccordionContent, AccordionContext, AccordionHeader, AccordionItem, AccordionRoot,
        AccordionRuntime, AccordionTrigger,
    };

    #[derive(Clone)]
    struct AccordionRtSlot(Rc<RefCell<Option<AccordionRuntime>>>);
    impl PartialEq for AccordionRtSlot {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.0, &other.0)
        }
    }

    #[component]
    fn RuntimeProbe(out: AccordionRtSlot) -> Element {
        let ctx = use_context::<AccordionContext>();
        *out.0.borrow_mut() = Some(ctx.runtime);
        rsx! {}
    }

    #[component]
    fn DynamicAccordionHost(
        rt_holder: AccordionRtSlot,
        show_holder: Rc<RefCell<Option<Signal<bool>>>>,
    ) -> Element {
        let show_item_2 = use_signal(|| true);
        *show_holder.borrow_mut() = Some(show_item_2);
        rsx! {
            AccordionRoot {
                id: "fm-col-4-acc",
                mode: AccordionMode::Single { collapsible: true },
                AccordionItem {
                    value: "item-1".to_string(),
                    AccordionHeader { AccordionTrigger { id: "acc-t1", "One" } }
                    AccordionContent { "C1" }
                }
                if show_item_2() {
                    AccordionItem {
                        value: "item-2".to_string(),
                        AccordionHeader { AccordionTrigger { id: "acc-t2", "Two" } }
                        AccordionContent { "C2" }
                    }
                }
                AccordionItem {
                    value: "item-3".to_string(),
                    AccordionHeader { AccordionTrigger { id: "acc-t3", "Three" } }
                    AccordionContent { "C3" }
                }
                RuntimeProbe { out: rt_holder }
            }
        }
    }

    let rt_holder = AccordionRtSlot(Rc::new(RefCell::new(None)));
    let show_holder = Rc::new(RefCell::new(None));
    let mut dom = VirtualDom::new_with_props(
        DynamicAccordionHost,
        DynamicAccordionHostProps {
            rt_holder: rt_holder.clone(),
            show_holder: show_holder.clone(),
        },
    );
    dom.rebuild_in_place();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);

    let rt = rt_holder.0.borrow().clone().expect("AccordionRuntime captured");
    let mut show_item_2 = show_holder.borrow().expect("show signal captured");

    dom.in_scope(ScopeId::APP, || {
        rt.roving().set_current_tab_stop(Some("item-2".to_string()));
        assert_eq!(rt.current_tab_stop(), "item-2");
        show_item_2.set(false);
    });
    dom.render_immediate(&mut dioxus::core::NoOpMutations);

    dom.in_scope(ScopeId::APP, || {
        let recovered = rt.current_tab_stop();
        assert!(
            recovered == "item-1" || recovered == "item-3",
            "FM-COL-4: unmounting focused AccordionItem must recover tab stop to a remaining enabled item, got {recovered:?}"
        );
        let t1 = rt.trigger("item-1", false).tabindex();
        let t3 = rt.trigger("item-3", false).tabindex();
        assert_eq!(
            usize::from(t1 == 0) + usize::from(t3 == 0),
            1,
            "exactly one remaining trigger must hold tabindex=0 (t1={t1}, t3={t3})"
        );
    });
}

#[cfg(feature = "native")]
#[test]
fn native_accordion_focused_item_unmount_recovers_dom_tabindex_zero() {
    use dioxus::prelude::*;
    use monoxus::accordion::{
        AccordionContent, AccordionHeader, AccordionItem, AccordionRoot, AccordionTrigger,
    };
    use crate::common::native_fixture::{
        NATIVE_TEST_LOCK, click_and_pump, focus_node, mount_native_scenario,
    };

    #[component]
    fn NativeDynamicAccordionScenario() -> Element {
        let mut show_two = use_signal(|| true);
        rsx! {
            div {
                button {
                    id: "unmount-acc-2",
                    onclick: move |_| show_two.set(false),
                    "Remove Item 2"
                }
                AccordionRoot {
                    id: "native-fm-col-4-acc",
                    mode: AccordionMode::Single { collapsible: true },
                    AccordionItem {
                        value: "item-1".to_string(),
                        AccordionHeader { AccordionTrigger { id: "native-acc-t1", "One" } }
                        AccordionContent { "C1" }
                    }
                    if show_two() {
                        AccordionItem {
                            value: "item-2".to_string(),
                            AccordionHeader { AccordionTrigger { id: "native-acc-t2", "Two" } }
                            AccordionContent { "C2" }
                        }
                    }
                    AccordionItem {
                        value: "item-3".to_string(),
                        AccordionHeader { AccordionTrigger { id: "native-acc-t3", "Three" } }
                        AccordionContent { "C3" }
                    }
                }
            }
        }
    }

    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(NativeDynamicAccordionScenario);

    focus_node(&mut harness, "#native-acc-t2");
    assert_eq!(harness.attr("#native-acc-t2", "tabindex").as_deref(), Some("0"));

    click_and_pump(&mut harness, "#unmount-acc-2");
    assert!(harness.query("#native-acc-t2").is_none());

    let t1_zero = harness.attr("#native-acc-t1", "tabindex").as_deref() == Some("0");
    let t3_zero = harness.attr("#native-acc-t3", "tabindex").as_deref() == Some("0");
    assert_eq!(
        usize::from(t1_zero) + usize::from(t3_zero),
        1,
        "FM-COL-4: remaining accordion triggers must recover exactly one tabindex=0"
    );
}

#[cfg(feature = "native")]
#[test]
fn native_accordion_post_mount_props_preserve_uncontrolled_state_and_force_mount_hidden() {
    use dioxus::prelude::*;
    use monoxus::accordion::{
        AccordionContent, AccordionHeader, AccordionItem, AccordionRoot, AccordionTrigger,
    };
    use crate::common::native_fixture::{NATIVE_TEST_LOCK, click_and_pump, mount_native_scenario};

    #[component]
    fn PostMountAccordionScenario() -> Element {
        let mut root_disabled = use_signal(|| false);
        let mut orientation = use_signal(|| AccordionOrientation::Vertical);
        let mut item_b_disabled = use_signal(|| false);

        rsx! {
            div {
                button {
                    id: "acc-set-horizontal",
                    style: "width: 120px; height: 24px; display: block;",
                    onclick: move |_| orientation.set(AccordionOrientation::Horizontal),
                    "Horizontal"
                }
                button {
                    id: "acc-set-item-b-disabled",
                    style: "width: 120px; height: 24px; display: block;",
                    onclick: move |_| item_b_disabled.set(true),
                    "Disable B"
                }
                button {
                    id: "acc-set-root-disabled",
                    style: "width: 120px; height: 24px; display: block;",
                    onclick: move |_| root_disabled.set(true),
                    "Disable Root"
                }
                AccordionRoot {
                    id: "fm-w2-acc-root",
                    class: "fm-w2-acc-root".to_string(),
                    mode: AccordionMode::Single { collapsible: true },
                    orientation: orientation(),
                    disabled: root_disabled(),
                    AccordionItem {
                        value: "a".to_string(),
                        AccordionHeader {
                            AccordionTrigger {
                                id: "fm-w2-acc-trig-a",
                                style: "width: 120px; height: 24px; display: block;",
                                "Item A"
                            }
                        }
                        AccordionContent {
                            force_mount: true,
                            div { id: "fm-w2-acc-body-a", "Body A" }
                        }
                    }
                    AccordionItem {
                        value: "b".to_string(),
                        disabled: item_b_disabled(),
                        AccordionHeader {
                            AccordionTrigger {
                                id: "fm-w2-acc-trig-b",
                                style: "width: 120px; height: 24px; display: block;",
                                "Item B"
                            }
                        }
                        AccordionContent {
                            force_mount: true,
                            div { id: "fm-w2-acc-body-b", "Body B" }
                        }
                    }
                }
            }
        }
    }

    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(PostMountAccordionScenario);

    // M-7: force_mount=true mounts children while closed, AND emits hidden on closed region.
    assert!(
        harness.query("#fm-w2-acc-body-a").is_some(),
        "force_mount=true must keep Body A mounted when closed"
    );
    assert!(
        harness
            .attr("[role=\"region\"][data-value=\"a\"]", "hidden")
            .is_some(),
        "FM-W2-ACC / M-7: closed AccordionContent(force_mount=true) must emit hidden attribute"
    );
    assert!(
        harness
            .attr("[role=\"region\"][data-value=\"b\"]", "hidden")
            .is_some(),
        "FM-W2-ACC / M-7: closed AccordionContent(force_mount=true) for item b must emit hidden"
    );

    // Open uncontrolled item A via trigger click.
    click_and_pump(&mut harness, "#fm-w2-acc-trig-a");
    assert_eq!(
        harness.attr("#fm-w2-acc-trig-a", "aria-expanded").as_deref(),
        Some("true")
    );
    assert!(
        harness
            .attr("[role=\"region\"][data-value=\"a\"]", "hidden")
            .is_none(),
        "open AccordionContent must remove hidden attribute"
    );
    assert!(
        harness
            .attr("[role=\"region\"][data-value=\"b\"]", "hidden")
            .is_some(),
        "closed sibling AccordionContent(force_mount=true) must retain hidden attribute"
    );

    // H-8a & H-8b: Mutate root orientation post-mount; must propagate without clobbering open item A.
    click_and_pump(&mut harness, "#acc-set-horizontal");
    assert_eq!(
        harness.attr(".fm-w2-acc-root", "data-orientation").as_deref(),
        Some("horizontal")
    );
    assert_eq!(
        harness.attr("#fm-w2-acc-trig-a", "data-orientation").as_deref(),
        Some("horizontal"),
        "FM-W2-ACC: post-mount orientation update must reach AccordionTrigger"
    );
    assert_eq!(
        harness
            .attr("[role=\"region\"][data-value=\"a\"]", "data-orientation")
            .as_deref(),
        Some("horizontal")
    );
    assert_eq!(
        harness.attr("#fm-w2-acc-trig-a", "aria-expanded").as_deref(),
        Some("true"),
        "FM-W2-ACC: mutating root orientation must not clobber uncontrolled open item A"
    );

    // H-8a: Mutate item B disabled post-mount via AccordionItemContext.
    click_and_pump(&mut harness, "#acc-set-item-b-disabled");
    assert_eq!(
        harness.attr("#fm-w2-acc-trig-b", "data-disabled").as_deref(),
        Some(""),
        "FM-W2-ACC: post-mount AccordionItem disabled update must reach AccordionTrigger"
    );
    assert_eq!(harness.attr("#fm-w2-acc-trig-a", "data-disabled"), None);

    // H-8a & H-8b: Mutate root disabled post-mount; propagates to triggers/content without clobbering open state.
    click_and_pump(&mut harness, "#acc-set-root-disabled");
    assert_eq!(
        harness.attr("#fm-w2-acc-trig-a", "data-disabled").as_deref(),
        Some(""),
        "FM-W2-ACC: post-mount root disabled=true must reach AccordionTrigger"
    );
    assert_eq!(
        harness
            .attr("[role=\"region\"][data-value=\"a\"]", "data-disabled")
            .as_deref(),
        Some("")
    );
    assert_eq!(
        harness.attr("#fm-w2-acc-trig-a", "aria-expanded").as_deref(),
        Some("true"),
        "FM-W2-ACC: mutating root disabled must not clobber uncontrolled open item A"
    );

    // Clicking disabled trigger must not toggle item A closed.
    click_and_pump(&mut harness, "#fm-w2-acc-trig-a");
    assert_eq!(
        harness.attr("#fm-w2-acc-trig-a", "aria-expanded").as_deref(),
        Some("true"),
        "FM-W2-ACC: clicking trigger after post-mount disabled=true must not collapse item A"
    );
}
