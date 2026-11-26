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
