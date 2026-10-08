use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use monoxus::tooltip::{
    TooltipArrow, TooltipContent, TooltipPortal, TooltipProvider, TooltipProviderContext,
    TooltipRoot, TooltipRootContext, TooltipTrigger,
};

#[component]
fn TooltipFullCompoundSample() -> Element {
    rsx! {
        TooltipProvider {
            delay_duration: 500,
            skip_delay_duration: 200,
            TooltipRoot {
                id: "compound-tooltip",
                default_open: true,
                TooltipTrigger {
                    id: "trigger-full",
                    "Hover me"
                }
                TooltipPortal {
                    TooltipContent {
                        id: "content-full",
                        "Tooltip Information",
                        TooltipArrow {
                            id: "arrow-full"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TooltipStandaloneSample() -> Element {
    rsx! {
        TooltipRoot {
            id: "standalone-tooltip",
            default_open: true,
            TooltipTrigger {
                id: "trigger-standalone",
                "Standalone Trigger"
            }
            TooltipPortal {
                TooltipContent {
                    id: "content-standalone",
                    "Standalone Content",
                    TooltipArrow {}
                }
            }
        }
    }
}

#[test]
fn test_tooltip_compound_parts_render_in_provider() {
    let mut dom = VirtualDom::new(TooltipFullCompoundSample);
    dom.rebuild_in_place();
}

#[test]
fn test_tooltip_standalone_fallback_renders() {
    let mut dom = VirtualDom::new(TooltipStandaloneSample);
    dom.rebuild_in_place();
}

#[derive(Clone, Default, PartialEq)]
struct AriaProbeResults {
    role: Option<String>,
    describedby: Option<String>,
    trigger_state: Option<String>,
    content_state: Option<String>,
    arrow_state: Option<String>,
    root_state: Option<String>,
}

#[component]
fn AriaStateProbe(results: Rc<RefCell<AriaProbeResults>>) -> Element {
    let ctx = use_context::<TooltipRootContext>();
    let runtime = ctx.runtime.read();
    let trigger = runtime.trigger();
    let content = runtime.content();
    let arrow = runtime.arrow();
    let root = runtime.root();

    *results.borrow_mut() = AriaProbeResults {
        role: Some(content.role().to_string()),
        describedby: trigger.aria_describedby().map(|s| s.to_string()),
        trigger_state: Some(trigger.data_state().as_str().to_string()),
        content_state: Some(content.data_state().as_str().to_string()),
        arrow_state: Some(arrow.data_state().as_str().to_string()),
        root_state: Some(root.data_state().as_str().to_string()),
    };

    rsx! {}
}

#[component]
fn TooltipAriaTestHarness(is_open: bool, probe_out: Rc<RefCell<AriaProbeResults>>) -> Element {
    rsx! {
        TooltipProvider {
            TooltipRoot {
                id: "aria-test",
                default_open: is_open,
                TooltipTrigger {
                    "Trigger"
                }
                TooltipPortal {
                    TooltipContent {
                        "Content",
                        TooltipArrow {}
                    }
                }
                AriaStateProbe {
                    results: probe_out,
                }
            }
        }
    }
}

#[test]
fn test_tooltip_aria_contract_and_describedby_linkage_when_open() {
    let probe_out = Rc::new(RefCell::new(AriaProbeResults::default()));
    let mut dom = VirtualDom::new_with_props(
        TooltipAriaTestHarness,
        TooltipAriaTestHarnessProps {
            is_open: true,
            probe_out: probe_out.clone(),
        },
    );
    dom.rebuild_in_place();

    let res = probe_out.borrow();
    // ARIA contract: role="tooltip" on content
    assert_eq!(res.role.as_deref(), Some("tooltip"));
    // aria-describedby linkage on trigger when open
    assert!(res.describedby.is_some());
    let expected_content_id = monoxus::foundation::shared::ScopeHandle::root("tooltip")
        .child("aria-test")
        .qualify("content");
    assert_eq!(
        res.describedby.as_deref(),
        Some(expected_content_id.as_str())
    );
    // data-state="open"
    assert_eq!(res.root_state.as_deref(), Some("open"));
    assert_eq!(res.trigger_state.as_deref(), Some("open"));
    assert_eq!(res.content_state.as_deref(), Some("open"));
    assert_eq!(res.arrow_state.as_deref(), Some("open"));
}

#[test]
fn test_tooltip_aria_contract_and_describedby_omitted_when_closed() {
    let probe_out = Rc::new(RefCell::new(AriaProbeResults::default()));
    let mut dom = VirtualDom::new_with_props(
        TooltipAriaTestHarness,
        TooltipAriaTestHarnessProps {
            is_open: false,
            probe_out: probe_out.clone(),
        },
    );
    dom.rebuild_in_place();

    let res = probe_out.borrow();
    // aria-describedby must be None when closed
    assert_eq!(res.describedby, None);
    // data-state="closed"
    assert_eq!(res.root_state.as_deref(), Some("closed"));
    assert_eq!(res.trigger_state.as_deref(), Some("closed"));
    assert_eq!(res.content_state.as_deref(), Some("closed"));
    assert_eq!(res.arrow_state.as_deref(), Some("closed"));
}

#[derive(Clone, Default, PartialEq)]
struct ProviderInheritanceResults {
    provider_delay: u64,
    provider_skip_delay: u64,
    provider_disable_hoverable: bool,
    root_delay: u64,
    root_skip_delay: u64,
    root_disable_hoverable: bool,
}

#[component]
fn ProviderProbe(results: Rc<RefCell<ProviderInheritanceResults>>) -> Element {
    let provider_ctx = use_context::<TooltipProviderContext>();
    let root_ctx = use_context::<TooltipRootContext>();

    let prov_runtime = provider_ctx.runtime.read();
    let root_runtime = root_ctx.runtime.read();

    *results.borrow_mut() = ProviderInheritanceResults {
        provider_delay: prov_runtime.provider().delay_duration_ms(),
        provider_skip_delay: prov_runtime.provider().skip_delay_duration_ms(),
        provider_disable_hoverable: prov_runtime.provider().disable_hoverable_content(),
        root_delay: root_runtime
            .provider()
            .map(|p| p.delay_duration_ms())
            .unwrap_or(0),
        root_skip_delay: root_runtime
            .provider()
            .map(|p| p.skip_delay_duration_ms())
            .unwrap_or(0),
        root_disable_hoverable: root_runtime
            .provider()
            .map(|p| p.disable_hoverable_content())
            .unwrap_or(false),
    };

    rsx! {}
}

#[component]
fn ProviderInheritanceSample(
    provider_delay: u64,
    root_delay_override: Option<u64>,
    results: Rc<RefCell<ProviderInheritanceResults>>,
) -> Element {
    rsx! {
        TooltipProvider {
            delay_duration: provider_delay,
            skip_delay_duration: 150,
            disable_hoverable_content: true,
            TooltipRoot {
                delay_duration: root_delay_override,
                TooltipTrigger { "Trigger" }
                TooltipPortal { TooltipContent { "Content" } }
                ProviderProbe { results }
            }
        }
    }
}

#[test]
fn test_provider_delay_configuration_and_context_inheritance() {
    let results = Rc::new(RefCell::new(ProviderInheritanceResults::default()));
    let mut dom = VirtualDom::new_with_props(
        ProviderInheritanceSample,
        ProviderInheritanceSampleProps {
            provider_delay: 450,
            root_delay_override: None,
            results: results.clone(),
        },
    );
    dom.rebuild_in_place();

    let res = results.borrow();
    assert_eq!(res.provider_delay, 450);
    assert_eq!(res.provider_skip_delay, 150);
    assert!(res.provider_disable_hoverable);
    // Root inherited provider settings
    assert_eq!(res.root_delay, 450);
    assert_eq!(res.root_skip_delay, 150);
    assert!(res.root_disable_hoverable);
}

#[test]
fn test_provider_delay_local_override_precedence() {
    let results = Rc::new(RefCell::new(ProviderInheritanceResults::default()));
    let mut dom = VirtualDom::new_with_props(
        ProviderInheritanceSample,
        ProviderInheritanceSampleProps {
            provider_delay: 700,
            root_delay_override: Some(120),
            results: results.clone(),
        },
    );
    dom.rebuild_in_place();

    let res = results.borrow();
    assert_eq!(res.provider_delay, 700);
    // Local override of 120ms applied to Root
    assert_eq!(res.root_delay, 120);
}

#[component]
fn EscapeDismissProbe(
    dismiss_result: Rc<RefCell<bool>>,
) -> Element {
    let ctx = use_context::<TooltipRootContext>();
    let runtime = ctx.runtime.read();

    let rt = runtime.clone();
    use_hook(move || {
        let dismissed = rt.dismiss_escape();
        *dismiss_result.borrow_mut() = dismissed;
    });

    rsx! {}
}

#[component]
fn TooltipEscapeDismissSample(
    closed_fired: Rc<RefCell<bool>>,
    dismiss_result: Rc<RefCell<bool>>,
) -> Element {
    let on_closed = closed_fired.clone();
    rsx! {
        TooltipProvider {
            TooltipRoot {
                default_open: true,
                on_open_change: move |next_open: bool| {
                    if !next_open {
                        *on_closed.borrow_mut() = true;
                    }
                },
                TooltipTrigger { "Trigger" }
                TooltipPortal { TooltipContent { "Content" } }
                EscapeDismissProbe {
                    dismiss_result,
                }
            }
        }
    }
}

#[test]
fn test_escape_key_dismiss_behavior() {
    let closed_fired = Rc::new(RefCell::new(false));
    let dismiss_result = Rc::new(RefCell::new(false));

    let mut dom = VirtualDom::new_with_props(
        TooltipEscapeDismissSample,
        TooltipEscapeDismissSampleProps {
            closed_fired: closed_fired.clone(),
            dismiss_result: dismiss_result.clone(),
        },
    );
    dom.rebuild_in_place();

    // Verify escape dismissal returned true
    assert!(*dismiss_result.borrow(), "dismiss_escape() should return true when tooltip is open");
}
