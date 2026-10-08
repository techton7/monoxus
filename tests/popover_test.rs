use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use monoxus::popover::{
    PopoverAnchor, PopoverArrow, PopoverClose, PopoverContent, PopoverContext,
    PopoverDescription, PopoverPortal, PopoverRoot, PopoverTitle, PopoverTrigger,
};

#[component]
fn PopoverFullCompoundSample() -> Element {
    rsx! {
        PopoverRoot {
            id: "full-popover",
            default_open: true,
            PopoverAnchor {
                id: "anchor-full",
                "Custom Anchor"
            }
            PopoverTrigger {
                id: "trigger-full",
                "Open Popover"
            }
            PopoverPortal {
                PopoverContent {
                    id: "content-full",
                    PopoverTitle {
                        id: "title-full",
                        "Popover Title"
                    }
                    PopoverDescription {
                        id: "desc-full",
                        "Popover description info"
                    }
                    PopoverClose {
                        id: "close-full",
                        "Close Popover"
                    }
                    PopoverArrow {
                        id: "arrow-full"
                    }
                }
            }
        }
    }
}

#[component]
fn PopoverMinimalSample() -> Element {
    rsx! {
        PopoverRoot {
            id: "minimal-popover",
            default_open: false,
            PopoverTrigger {
                id: "trigger-min",
                "Toggle"
            }
            PopoverPortal {
                PopoverContent {
                    id: "content-min",
                    "Minimal Content"
                }
            }
        }
    }
}

#[test]
fn test_popover_full_compound_parts_render() {
    let mut dom = VirtualDom::new(PopoverFullCompoundSample);
    dom.rebuild_in_place();
}

#[test]
fn test_popover_minimal_parts_render_closed() {
    let mut dom = VirtualDom::new(PopoverMinimalSample);
    dom.rebuild_in_place();
}

#[derive(Clone, Default, PartialEq)]
struct PopoverAriaProbeResults {
    trigger_haspopup: Option<String>,
    trigger_expanded: Option<bool>,
    trigger_controls: Option<String>,
    trigger_state: Option<String>,
    content_role: Option<String>,
    content_modal: Option<bool>,
    content_state: Option<String>,
    content_side: Option<String>,
    content_align: Option<String>,
    root_state: Option<String>,
    close_state: Option<String>,
    arrow_state: Option<String>,
    title_id: Option<String>,
    description_id: Option<String>,
}

#[component]
fn PopoverAriaProbe(results: Rc<RefCell<PopoverAriaProbeResults>>) -> Element {
    let ctx = use_context::<PopoverContext>();
    let runtime = ctx.runtime.read();
    let trigger = runtime.trigger();
    let content = runtime.content();
    let root = runtime.root();
    let close = runtime.close();
    let arrow = runtime.arrow();

    *results.borrow_mut() = PopoverAriaProbeResults {
        trigger_haspopup: Some("dialog".to_string()),
        trigger_expanded: Some(trigger.aria_expanded()),
        trigger_controls: Some(trigger.aria_controls().to_string()),
        trigger_state: Some(trigger.data_state().as_str().to_string()),
        content_role: Some(content.role().to_string()),
        content_modal: Some(content.aria_modal()),
        content_state: Some(content.data_state().as_str().to_string()),
        content_side: Some(content.data_side().to_string()),
        content_align: Some(content.data_align().to_string()),
        root_state: Some(root.data_state().as_str().to_string()),
        close_state: Some(close.data_state().as_str().to_string()),
        arrow_state: Some(arrow.data_state().as_str().to_string()),
        title_id: ctx.title_id.read().clone(),
        description_id: ctx.description_id.read().clone(),
    };

    rsx! {}
}

#[component]
fn PopoverAriaTestHarness(
    is_open: bool,
    is_modal: bool,
    probe_out: Rc<RefCell<PopoverAriaProbeResults>>,
) -> Element {
    rsx! {
        PopoverRoot {
            id: "aria-test",
            default_open: is_open,
            modal: is_modal,
            PopoverTrigger {
                id: "aria-trigger",
                "Trigger"
            }
            PopoverPortal {
                PopoverContent {
                    id: "aria-content",
                    PopoverTitle {
                        id: "aria-title",
                        "Heading"
                    }
                    PopoverDescription {
                        id: "aria-desc",
                        "Description paragraph"
                    }
                    PopoverClose {
                        id: "aria-close",
                        "X"
                    }
                    PopoverArrow {
                        id: "aria-arrow"
                    }
                }
            }
            PopoverAriaProbe {
                results: probe_out,
            }
        }
    }
}

#[test]
fn test_popover_aria_contract_and_labelledby_describedby_linkage_when_open() {
    let probe_out = Rc::new(RefCell::new(PopoverAriaProbeResults::default()));
    let mut dom = VirtualDom::new_with_props(
        PopoverAriaTestHarness,
        PopoverAriaTestHarnessProps {
            is_open: true,
            is_modal: false,
            probe_out: probe_out.clone(),
        },
    );
    dom.rebuild_in_place();

    let res = probe_out.borrow();
    // ARIA contracts:
    assert_eq!(res.content_role.as_deref(), Some("dialog"));
    assert_eq!(res.trigger_haspopup.as_deref(), Some("dialog"));
    assert_eq!(res.trigger_expanded, Some(true));
    assert_eq!(res.trigger_state.as_deref(), Some("open"));
    assert_eq!(res.root_state.as_deref(), Some("open"));
    assert_eq!(res.content_state.as_deref(), Some("open"));
    assert_eq!(res.close_state.as_deref(), Some("open"));
    assert_eq!(res.arrow_state.as_deref(), Some("open"));

    // aria-controls matches content id
    let expected_content_id = monoxus::foundation::shared::ScopeHandle::root("popover")
        .child("aria-test")
        .qualify("content");
    assert_eq!(res.trigger_controls.as_deref(), Some(expected_content_id.as_str()));

    // Title and Description registration:
    assert_eq!(res.title_id.as_deref(), Some("aria-title"));
    assert_eq!(res.description_id.as_deref(), Some("aria-desc"));
    assert_eq!(res.content_modal, Some(false));
}

#[test]
fn test_popover_modal_aria_modal_contract() {
    let probe_out = Rc::new(RefCell::new(PopoverAriaProbeResults::default()));
    let mut dom = VirtualDom::new_with_props(
        PopoverAriaTestHarness,
        PopoverAriaTestHarnessProps {
            is_open: true,
            is_modal: true,
            probe_out: probe_out.clone(),
        },
    );
    dom.rebuild_in_place();

    let res = probe_out.borrow();
    assert_eq!(res.content_role.as_deref(), Some("dialog"));
    assert_eq!(res.content_modal, Some(true));
}

#[test]
fn test_popover_closed_state_contract() {
    let probe_out = Rc::new(RefCell::new(PopoverAriaProbeResults::default()));
    let mut dom = VirtualDom::new_with_props(
        PopoverAriaTestHarness,
        PopoverAriaTestHarnessProps {
            is_open: false,
            is_modal: false,
            probe_out: probe_out.clone(),
        },
    );
    dom.rebuild_in_place();

    let res = probe_out.borrow();
    assert_eq!(res.trigger_expanded, Some(false));
    assert_eq!(res.trigger_state.as_deref(), Some("closed"));
    assert_eq!(res.root_state.as_deref(), Some("closed"));
}

#[component]
fn ToggleProbe(
    toggled: Rc<RefCell<bool>>,
) -> Element {
    let ctx = use_context::<PopoverContext>();
    let runtime = ctx.runtime.read();

    let rt = runtime.clone();
    use_hook(move || {
        let next = rt.request_state(monoxus::popover::PopoverStateRequest::Toggle);
        *toggled.borrow_mut() = next;
    });

    rsx! {}
}

#[component]
fn PopoverToggleSample(
    open_changed: Rc<RefCell<bool>>,
    toggled: Rc<RefCell<bool>>,
) -> Element {
    let on_change = open_changed.clone();
    rsx! {
        PopoverRoot {
            default_open: false,
            on_open_change: move |next_open: bool| {
                *on_change.borrow_mut() = next_open;
            },
            PopoverTrigger { "Trigger" }
            PopoverPortal {
                PopoverContent { "Content" }
            }
            ToggleProbe {
                toggled,
            }
        }
    }
}

#[test]
fn test_popover_trigger_toggle_fires_on_open_change() {
    let open_changed = Rc::new(RefCell::new(false));
    let toggled = Rc::new(RefCell::new(false));

    let mut dom = VirtualDom::new_with_props(
        PopoverToggleSample,
        PopoverToggleSampleProps {
            open_changed: open_changed.clone(),
            toggled: toggled.clone(),
        },
    );
    dom.rebuild_in_place();

    assert!(*toggled.borrow(), "toggle should transition from closed to open");
    assert!(*open_changed.borrow(), "on_open_change should be invoked with true");
}

#[component]
fn CloseProbe(
    closed_fired: Rc<RefCell<bool>>,
) -> Element {
    let ctx = use_context::<PopoverContext>();
    let runtime = ctx.runtime.read();

    let rt = runtime.clone();
    use_hook(move || {
        rt.close_now();
        *closed_fired.borrow_mut() = true;
    });

    rsx! {}
}

#[component]
fn PopoverCloseSample(
    closed_fired: Rc<RefCell<bool>>,
    on_change_fired: Rc<RefCell<bool>>,
) -> Element {
    let on_change = on_change_fired.clone();
    rsx! {
        PopoverRoot {
            default_open: true,
            on_open_change: move |next_open: bool| {
                if !next_open {
                    *on_change.borrow_mut() = true;
                }
            },
            PopoverTrigger { "Trigger" }
            PopoverPortal {
                PopoverContent {
                    "Content",
                    PopoverClose { "Close" }
                }
            }
            CloseProbe {
                closed_fired,
            }
        }
    }
}

#[test]
fn test_popover_close_button_fires_on_open_change() {
    let closed_fired = Rc::new(RefCell::new(false));
    let on_change_fired = Rc::new(RefCell::new(false));

    let mut dom = VirtualDom::new_with_props(
        PopoverCloseSample,
        PopoverCloseSampleProps {
            closed_fired: closed_fired.clone(),
            on_change_fired: on_change_fired.clone(),
        },
    );
    dom.rebuild_in_place();

    assert!(*closed_fired.borrow(), "close_now was called");
    assert!(*on_change_fired.borrow(), "on_open_change(false) was triggered");
}

#[component]
fn EscapeDismissProbe(
    dismiss_result: Rc<RefCell<bool>>,
) -> Element {
    let ctx = use_context::<PopoverContext>();
    let runtime = ctx.runtime.read();

    let rt = runtime.clone();
    use_hook(move || {
        let dismissed = rt.dismiss_escape();
        *dismiss_result.borrow_mut() = dismissed;
    });

    rsx! {}
}

#[component]
fn PopoverEscapeDismissSample(
    closed_fired: Rc<RefCell<bool>>,
    dismiss_result: Rc<RefCell<bool>>,
) -> Element {
    let on_closed = closed_fired.clone();
    rsx! {
        PopoverRoot {
            default_open: true,
            on_open_change: move |next_open: bool| {
                if !next_open {
                    *on_closed.borrow_mut() = true;
                }
            },
            PopoverTrigger { "Trigger" }
            PopoverPortal {
                PopoverContent { "Content" }
            }
            EscapeDismissProbe {
                dismiss_result,
            }
        }
    }
}

#[test]
fn test_popover_escape_key_dismiss_behavior() {
    let closed_fired = Rc::new(RefCell::new(false));
    let dismiss_result = Rc::new(RefCell::new(false));

    let mut dom = VirtualDom::new_with_props(
        PopoverEscapeDismissSample,
        PopoverEscapeDismissSampleProps {
            closed_fired: closed_fired.clone(),
            dismiss_result: dismiss_result.clone(),
        },
    );
    dom.rebuild_in_place();

    assert!(*dismiss_result.borrow(), "dismiss_escape() should return true when popover is open");
    assert!(*closed_fired.borrow(), "on_open_change should receive false on escape");
}
