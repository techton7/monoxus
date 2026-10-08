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
        trigger_controls: Some(trigger.aria_controls().to_string()),
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

    // ARIA controls links trigger to content ID
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

    // ARIA controls links trigger to content ID
    assert_eq!(res.trigger_controls, res.content_id);
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

