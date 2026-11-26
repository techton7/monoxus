use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use monoxus::{
    collapsible::{
        COLLAPSIBLE_PARTS, Collapsible, CollapsibleContent, CollapsibleContext, CollapsiblePart,
        CollapsibleRelationships, CollapsibleRoot, CollapsibleStateRequest, CollapsibleTrigger,
    },
    foundation::state::DataState,
};

#[component]
fn CollapsibleBasicOpenSample() -> Element {
    rsx! {
        CollapsibleRoot {
            id: "basic-open",
            default_open: true,
            CollapsibleTrigger {
                id: "basic-trigger",
                "Toggle"
            }
            CollapsibleContent {
                id: "basic-content",
                "Content Body"
            }
        }
    }
}

#[component]
fn CollapsibleBasicClosedSample() -> Element {
    rsx! {
        CollapsibleRoot {
            id: "basic-closed",
            default_open: false,
            CollapsibleTrigger {
                id: "closed-trigger",
                "Toggle"
            }
            CollapsibleContent {
                id: "closed-content",
                "Content Body"
            }
        }
    }
}

#[test]
fn test_collapsible_compound_parts_render_cleanly() {
    let mut dom = VirtualDom::new(CollapsibleBasicOpenSample);
    dom.rebuild_in_place();
}

#[test]
fn test_collapsible_closed_by_default_mounts_cleanly() {
    let mut dom = VirtualDom::new(CollapsibleBasicClosedSample);
    dom.rebuild_in_place();
}

#[derive(Clone, Default, PartialEq)]
struct CollapsibleProbeResults {
    root_state: Option<String>,
    root_disabled: Option<bool>,
    trigger_expanded: Option<String>,
    trigger_controls: Option<String>,
    trigger_state: Option<String>,
    trigger_disabled: Option<bool>,
    content_state: Option<String>,
    content_hidden: Option<bool>,
    content_disabled: Option<bool>,
    content_id: Option<String>,
}

#[component]
fn CollapsibleProbe(results: Rc<RefCell<CollapsibleProbeResults>>) -> Element {
    let ctx = use_context::<CollapsibleContext>();
    let runtime = ctx.runtime.read();
    let root = runtime.root();
    let trigger = runtime.trigger();
    let content = runtime.content();

    *results.borrow_mut() = CollapsibleProbeResults {
        root_state: Some(root.data_state_str().to_string()),
        root_disabled: Some(root.is_disabled()),
        trigger_expanded: Some(trigger.aria_expanded().to_string()),
        trigger_controls: trigger.aria_controls().map(ToString::to_string),
        trigger_state: Some(trigger.data_state_str().to_string()),
        trigger_disabled: Some(trigger.is_disabled()),
        content_state: Some(content.data_state_str().to_string()),
        content_hidden: Some(content.is_hidden()),
        content_disabled: Some(content.is_disabled()),
        content_id: Some(content.id().to_string()),
    };

    rsx! {}
}

#[component]
fn CollapsibleAriaHarness(
    is_open: bool,
    disabled: bool,
    results: Rc<RefCell<CollapsibleProbeResults>>,
) -> Element {
    rsx! {
        CollapsibleRoot {
            id: "test-col",
            default_open: is_open,
            disabled: disabled,
            CollapsibleTrigger {
                "Trigger"
            }
            CollapsibleContent {
                "Content"
            }
            CollapsibleProbe {
                results,
            }
        }
    }
}

#[test]
fn test_collapsible_aria_and_data_state_when_open() {
    let probe_out = Rc::new(RefCell::new(CollapsibleProbeResults::default()));
    let mut dom = VirtualDom::new_with_props(
        CollapsibleAriaHarness,
        CollapsibleAriaHarnessProps {
            is_open: true,
            disabled: false,
            results: probe_out.clone(),
        },
    );
    dom.rebuild_in_place();

    let res = probe_out.borrow();
    assert_eq!(res.root_state.as_deref(), Some("open"));
    assert_eq!(res.root_disabled, Some(false));
    assert_eq!(res.trigger_expanded.as_deref(), Some("true"));
    assert_eq!(res.trigger_state.as_deref(), Some("open"));
    assert_eq!(res.trigger_disabled, Some(false));
    assert_eq!(res.content_state.as_deref(), Some("open"));
    assert_eq!(res.content_hidden, Some(false));
    assert_eq!(res.content_disabled, Some(false));

    // ARIA controls links trigger to content ID when open
    assert_eq!(res.trigger_controls, res.content_id);
}

#[test]
fn test_collapsible_aria_and_data_state_when_closed() {
    let probe_out = Rc::new(RefCell::new(CollapsibleProbeResults::default()));
    let mut dom = VirtualDom::new_with_props(
        CollapsibleAriaHarness,
        CollapsibleAriaHarnessProps {
            is_open: false,
            disabled: false,
            results: probe_out.clone(),
        },
    );
    dom.rebuild_in_place();

    let res = probe_out.borrow();
    assert_eq!(res.root_state.as_deref(), Some("closed"));
    assert_eq!(res.root_disabled, Some(false));
    assert_eq!(res.trigger_expanded.as_deref(), Some("false"));
    assert_eq!(res.trigger_state.as_deref(), Some("closed"));
    assert_eq!(res.trigger_disabled, Some(false));
    assert_eq!(res.content_state.as_deref(), Some("closed"));
    assert_eq!(res.content_hidden, Some(true));
    assert_eq!(res.content_disabled, Some(false));

    // ARIA controls is omitted when closed and unmounted
    assert_eq!(res.trigger_controls, None);
}

#[component]
fn ToggleActionProbe(
    toggled_out: Rc<RefCell<bool>>,
) -> Element {
    let ctx = use_context::<CollapsibleContext>();
    let runtime = ctx.runtime.read();
    let rt = runtime.clone();

    use_hook(move || {
        rt.toggle();
        *toggled_out.borrow_mut() = rt.is_open();
    });

    rsx! {}
}

#[component]
fn CollapsibleToggleHarness(
    open_changed: Rc<RefCell<Vec<bool>>>,
    toggled_out: Rc<RefCell<bool>>,
) -> Element {
    let on_change = open_changed.clone();
    rsx! {
        CollapsibleRoot {
            default_open: false,
            on_open_change: move |next_open: bool| {
                on_change.borrow_mut().push(next_open);
            },
            CollapsibleTrigger { "Trigger" }
            CollapsibleContent { "Content" }
            ToggleActionProbe {
                toggled_out,
            }
        }
    }
}

#[test]
fn test_collapsible_toggle_updates_state_and_fires_on_open_change() {
    let open_changed = Rc::new(RefCell::new(Vec::new()));
    let toggled_out = Rc::new(RefCell::new(false));

    let mut dom = VirtualDom::new_with_props(
        CollapsibleToggleHarness,
        CollapsibleToggleHarnessProps {
            open_changed: open_changed.clone(),
            toggled_out: toggled_out.clone(),
        },
    );
    dom.rebuild_in_place();

    assert!(*toggled_out.borrow(), "toggling closed collapsible should make it open");
    assert_eq!(*open_changed.borrow(), vec![true], "on_open_change should be called with true");
}

#[component]
fn CollapsibleDisabledToggleHarness(
    open_changed: Rc<RefCell<Vec<bool>>>,
    toggled_out: Rc<RefCell<bool>>,
) -> Element {
    let on_change = open_changed.clone();
    rsx! {
        CollapsibleRoot {
            default_open: false,
            disabled: true,
            on_open_change: move |next_open: bool| {
                on_change.borrow_mut().push(next_open);
            },
            CollapsibleTrigger { "Trigger" }
            CollapsibleContent { "Content" }
            ToggleActionProbe {
                toggled_out,
            }
        }
    }
}

#[test]
fn test_collapsible_disabled_prevents_toggle() {
    let open_changed = Rc::new(RefCell::new(Vec::new()));
    let toggled_out = Rc::new(RefCell::new(false));

    let mut dom = VirtualDom::new_with_props(
        CollapsibleDisabledToggleHarness,
        CollapsibleDisabledToggleHarnessProps {
            open_changed: open_changed.clone(),
            toggled_out: toggled_out.clone(),
        },
    );
    dom.rebuild_in_place();

    assert!(!*toggled_out.borrow(), "disabled collapsible should remain closed");
    assert!(open_changed.borrow().is_empty(), "on_open_change should not be called when disabled");
}

#[component]
fn ChildMountProbe(mounted: Rc<RefCell<bool>>) -> Element {
    *mounted.borrow_mut() = true;
    rsx! { div { "Mounted Child" } }
}

#[component]
fn CollapsibleForceMountHarness(
    is_open: bool,
    force_mount: bool,
    child_mounted: Rc<RefCell<bool>>,
) -> Element {
    rsx! {
        CollapsibleRoot {
            default_open: is_open,
            CollapsibleTrigger { "Trigger" }
            CollapsibleContent {
                force_mount: force_mount,
                ChildMountProbe {
                    mounted: child_mounted,
                }
            }
        }
    }
}

#[test]
fn test_collapsible_force_mount_behavior() {
    // 1. Closed without force_mount: child NOT mounted
    let child_mounted_closed = Rc::new(RefCell::new(false));
    let mut dom1 = VirtualDom::new_with_props(
        CollapsibleForceMountHarness,
        CollapsibleForceMountHarnessProps {
            is_open: false,
            force_mount: false,
            child_mounted: child_mounted_closed.clone(),
        },
    );
    dom1.rebuild_in_place();
    assert!(!*child_mounted_closed.borrow(), "child must NOT be mounted when closed and force_mount is false");

    // 2. Closed with force_mount: child IS mounted
    let child_mounted_forced = Rc::new(RefCell::new(false));
    let mut dom2 = VirtualDom::new_with_props(
        CollapsibleForceMountHarness,
        CollapsibleForceMountHarnessProps {
            is_open: false,
            force_mount: true,
            child_mounted: child_mounted_forced.clone(),
        },
    );
    dom2.rebuild_in_place();
    assert!(*child_mounted_forced.borrow(), "child MUST be mounted when force_mount is true even if closed");

    // 3. Open without force_mount: child IS mounted
    let child_mounted_open = Rc::new(RefCell::new(false));
    let mut dom3 = VirtualDom::new_with_props(
        CollapsibleForceMountHarness,
        CollapsibleForceMountHarnessProps {
            is_open: true,
            force_mount: false,
            child_mounted: child_mounted_open.clone(),
        },
    );
    dom3.rebuild_in_place();
    assert!(*child_mounted_open.borrow(), "child MUST be mounted when open");
}

#[test]
fn test_collapsible_state_request_transitions() {
    let req_open = CollapsibleStateRequest::Open;
    assert_eq!(req_open.next_open(false), true);
    assert_eq!(req_open.next_open(true), true);
    assert_eq!(req_open.data_state(false), DataState::Open);

    let req_close = CollapsibleStateRequest::Close;
    assert_eq!(req_close.next_open(true), false);
    assert_eq!(req_close.next_open(false), false);
    assert_eq!(req_close.data_state(true), DataState::Closed);

    let req_toggle = CollapsibleStateRequest::Toggle;
    assert_eq!(req_toggle.next_open(false), true);
    assert_eq!(req_toggle.next_open(true), false);
}

#[test]
fn test_collapsible_parts_inventory() {
    assert_eq!(COLLAPSIBLE_PARTS.len(), 3);
    assert_eq!(CollapsiblePart::Root.as_str(), "root");
    assert_eq!(CollapsiblePart::Trigger.as_str(), "trigger");
    assert_eq!(CollapsiblePart::Content.as_str(), "content");
    assert_eq!(Collapsible::parts().len(), 3);
}

#[test]
fn test_collapsible_relationships_deterministic_ids() {
    let scope = monoxus::foundation::shared::ScopeHandle::root("collapsible-test").child("main");
    let relationships = CollapsibleRelationships::new(scope.clone());

    assert_eq!(relationships.scope(), &scope);
    assert_eq!(relationships.root_id(), scope.token());
    assert_eq!(relationships.trigger_id(), scope.qualify("trigger"));
    assert_eq!(relationships.content_id(), scope.qualify("content"));
}

#[component]
fn CollapsibleCallbackSample(call_count: Rc<RefCell<usize>>) -> Element {
    let call_count = call_count.clone();
    rsx! {
        CollapsibleRoot {
            id: "cb-root",
            CollapsibleTrigger {
                id: "cb-trigger",
                onclick: move |_evt: MouseEvent| {
                    *call_count.borrow_mut() += 1;
                },
                "Toggle"
            }
            CollapsibleContent {
                id: "cb-content",
                "Body"
            }
        }
    }
}

#[test]
fn test_collapsible_trigger_mouse_event_callback_contract() {
    let call_count = Rc::new(RefCell::new(0));
    let mut dom = VirtualDom::new_with_props(CollapsibleCallbackSample, CollapsibleCallbackSampleProps {
        call_count: call_count.clone(),
    });
    dom.rebuild_in_place();
    assert_eq!(*call_count.borrow(), 0);
}

#[test]
fn test_collapsible_content_merge_global_attributes_vdom() {
    #[component]
    fn AttrsApp() -> Element {
        rsx! {
            CollapsibleRoot {
                id: "fm-api-3-col-root",
                default_open: true,
                CollapsibleTrigger { id: "fm-api-3-col-trig", "Toggle" }
                CollapsibleContent {
                    id: "fm-api-3-col-content",
                    class: "b",
                    style: "margin: 0",
                    attributes: vec![
                        Attribute::new("class", "a", None, false),
                        Attribute::new("style", "color: red", None, false),
                        Attribute {
                            name: "style",
                            value: dioxus::core::AttributeValue::None,
                            namespace: None,
                            volatile: false,
                        },
                        Attribute::new("background-color", "blue", Some("style"), false),
                    ],
                    "Body"
                }
            }
        }
    }

    let mut dom = VirtualDom::new(AttrsApp);
    dom.rebuild_in_place();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
}

#[test]
fn test_collapsible_content_id_clears_on_none_and_unmount_and_root_skips_unchanged_rerender() {
    use std::cell::Cell;

    #[derive(Clone)]
    struct RenderCounter(Rc<Cell<usize>>);
    impl PartialEq for RenderCounter {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.0, &other.0)
        }
    }

    #[derive(Clone)]
    struct ContentIdSink(Rc<RefCell<Option<String>>>);
    impl PartialEq for ContentIdSink {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.0, &other.0)
        }
    }

    #[component]
    fn CollapsibleRuntimeSubProbe(counter: RenderCounter) -> Element {
        let ctx = use_context::<CollapsibleContext>();
        let _rt = ctx.runtime.read();
        counter.0.set(counter.0.get() + 1);
        rsx! {}
    }

    #[component]
    fn CollapsibleContentIdProbe(sink: ContentIdSink) -> Element {
        let ctx = use_context::<CollapsibleContext>();
        *sink.0.borrow_mut() = ctx.content_id.read().clone();
        rsx! {}
    }

    #[component]
    fn CollapsibleL2Host(
        counter: RenderCounter,
        sink: ContentIdSink,
        tick_out: Rc<RefCell<Option<Signal<u32>>>>,
        id_out: Rc<RefCell<Option<Signal<Option<String>>>>>,
        mount_out: Rc<RefCell<Option<Signal<bool>>>>,
    ) -> Element {
        let tick = use_signal(|| 0_u32);
        let custom_id = use_signal(|| Some("custom-col-a".to_string()));
        let mount_content = use_signal(|| true);
        *tick_out.borrow_mut() = Some(tick);
        *id_out.borrow_mut() = Some(custom_id);
        *mount_out.borrow_mut() = Some(mount_content);
        let _ = tick();

        rsx! {
            CollapsibleRoot {
                id: "l2-col-root",
                default_open: true,
                CollapsibleTrigger { "Toggle" }
                if mount_content() {
                    CollapsibleContent {
                        id: custom_id(),
                        "Panel"
                    }
                }
                CollapsibleRuntimeSubProbe { counter }
                CollapsibleContentIdProbe { sink }
            }
        }
    }

    let counter = RenderCounter(Rc::new(Cell::new(0)));
    let sink = ContentIdSink(Rc::new(RefCell::new(None)));
    let tick_out = Rc::new(RefCell::new(None));
    let id_out = Rc::new(RefCell::new(None));
    let mount_out = Rc::new(RefCell::new(None));

    let mut dom = VirtualDom::new_with_props(
        CollapsibleL2Host,
        CollapsibleL2HostProps {
            counter: counter.clone(),
            sink: sink.clone(),
            tick_out: tick_out.clone(),
            id_out: id_out.clone(),
            mount_out: mount_out.clone(),
        },
    );
    dom.rebuild_in_place();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);

    assert_eq!(counter.0.get(), 1);
    assert_eq!(sink.0.borrow().as_deref(), Some("custom-col-a"));

    // Unchanged parent re-render must not trigger redundant runtime_sig updates.
    let mut tick = tick_out.borrow().expect("tick captured");
    dom.in_scope(ScopeId::APP, || tick.set(1));
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    assert_eq!(
        counter.0.get(),
        1,
        "L-2: CollapsibleRoot unchanged parent re-render must not force child re-render"
    );

    // Changing CollapsibleContent props.id from Some -> None must clear ctx.content_id.
    let mut custom_id = id_out.borrow().expect("id signal captured");
    dom.in_scope(ScopeId::APP, || custom_id.set(None));
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    assert_eq!(
        sink.0.borrow().as_deref(),
        None,
        "L-2: CollapsibleContent id Some->None must clear ctx.content_id"
    );

    // Setting a new custom id and unmounting CollapsibleContent must clear ctx.content_id on drop.
    dom.in_scope(ScopeId::APP, || custom_id.set(Some("custom-col-b".to_string())));
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    assert_eq!(sink.0.borrow().as_deref(), Some("custom-col-b"));

    let mut mount_content = mount_out.borrow().expect("mount signal captured");
    dom.in_scope(ScopeId::APP, || mount_content.set(false));
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    assert_eq!(
        sink.0.borrow().as_deref(),
        None,
        "L-2: unmounting CollapsibleContent must clear own ctx.content_id"
    );
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use crate::common::native_fixture::*;
    use monoxus::accordion::{
        AccordionContent, AccordionHeader, AccordionItem, AccordionMode, AccordionRoot,
        AccordionTrigger,
    };

    #[component]
    fn NativeDisclosureAndAttrMergeScenario() -> Element {
        rsx! {
            div {
                CollapsibleRoot {
                    id: "fm-col-open-root",
                    default_open: true,
                    CollapsibleTrigger { id: "fm-col-open-trig", "Open" }
                    CollapsibleContent {
                        id: "fm-col-open-content",
                        class: "b",
                        style: "margin: 0",
                        attributes: vec![
                            Attribute::new("class", "a", None, false),
                            Attribute::new("style", "color: red", None, false),
                            Attribute {
                                name: "style",
                                value: dioxus::core::AttributeValue::None,
                                namespace: None,
                                volatile: false,
                            },
                            Attribute::new("padding", "4px", Some("style"), false),
                        ],
                        "Open Body"
                    }
                }
                CollapsibleRoot {
                    id: "fm-col-closed-root",
                    default_open: false,
                    CollapsibleTrigger { id: "fm-col-closed-trig", "Closed" }
                    CollapsibleContent {
                        id: "fm-col-closed-content",
                        "Unmounted Body"
                    }
                }
                CollapsibleRoot {
                    id: "fm-col-until-found-root",
                    default_open: false,
                    CollapsibleTrigger { id: "fm-col-uf-trig", "UF" }
                    CollapsibleContent {
                        id: "fm-col-uf-content",
                        force_mount: true,
                        hidden_until_found: true,
                        "Searchable Body"
                    }
                }
                AccordionRoot {
                    id: "fm-acc-root",
                    mode: AccordionMode::Single { collapsible: true },
                    default_value: "a".to_string(),
                    AccordionItem {
                        value: "a".to_string(),
                        AccordionHeader { AccordionTrigger { id: "fm-acc-trig-a", "A" } }
                        AccordionContent { "Body A" }
                    }
                    AccordionItem {
                        value: "b".to_string(),
                        AccordionHeader { AccordionTrigger { id: "fm-acc-trig-b", "B" } }
                        AccordionContent { "Body B" }
                    }
                }
            }
        }
    }

    #[test]
    fn test_native_disclosure_differences_mount_guard_and_attr_merge() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let harness = mount_native_scenario(NativeDisclosureAndAttrMergeScenario);

        // FM-API-3: CollapsibleContent merges class + style without ;; and keeps internal styles.
        assert_eq!(
            harness.attr("#fm-col-open-content", "class").as_deref(),
            Some("b a")
        );
        let merged_style = harness
            .attr("#fm-col-open-content", "style")
            .expect("CollapsibleContent style must exist");
        assert!(merged_style.contains("margin: 0"), "style={merged_style}");
        assert!(merged_style.contains("color: red"), "style={merged_style}");
        assert!(!merged_style.contains(";;"), "style={merged_style}");
        // FM-COL-2: declarative mount-open animation guard present on initial open mount.
        let guard_style = monoxus::foundation::shared::MOUNT_ANIMATION_GUARD_STYLE;
        assert!(
            merged_style.contains(guard_style),
            "default_open=true must include MOUNT_ANIMATION_GUARD_STYLE ({guard_style}) on initial render; got {merged_style}"
        );

        // FM-COL-2: CollapsibleContent unmounts when closed & !force_mount, and supports hidden="until-found".
        assert!(harness.query("#fm-col-closed-content").is_none());
        assert_eq!(
            harness.attr("#fm-col-uf-content", "hidden").as_deref(),
            Some("until-found")
        );

        // FM-COL-2: AccordionContent retains its <div hidden> shell when closed.
        let closed_acc = harness
            .query("[role=\"region\"][data-value=\"b\"]")
            .expect("AccordionContent must keep <div hidden> shell when closed");
        assert!(
            harness
                .base()
                .get_node(closed_acc)
                .unwrap()
                .attr("hidden".into())
                .is_some(),
            "Closed AccordionContent must carry hidden attribute"
        );
        let open_acc = harness
            .query("[role=\"region\"][data-value=\"a\"]")
            .expect("Open AccordionContent must mount");
        let open_acc_style = harness
            .base()
            .get_node(open_acc)
            .unwrap()
            .attr("style".into())
            .unwrap_or_default()
            .to_string();
        assert!(
            open_acc_style.contains(guard_style),
            "Initially open AccordionContent must include MOUNT_ANIMATION_GUARD_STYLE ({guard_style}); got {open_acc_style}"
        );
    }
}

