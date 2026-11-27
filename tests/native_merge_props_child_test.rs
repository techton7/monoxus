//! S1.1 Failure-Mode Verification: `merge_props` 3-layer `child` composition & `TransitionStatus` presence.

use std::cell::RefCell;

use crate::common;
use common::native_fixture::*;
use dioxus::prelude::*;
use monoxus::foundation::compose::{
    MergePropsConfig, merge_props, merge_props_with_config, push_attr_opt, push_bool_data_attr,
    push_event_handler,
};
use monoxus::foundation::overlay::{
    PresenceState, TransitionStatus, use_presence_with_keep_mounted,
};

#[derive(Props, Clone, PartialEq)]
struct OuterTriggerProps {
    #[props(default)]
    child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    on_click: Option<EventHandler<MouseEvent>>,
    on_trace: EventHandler<&'static str>,
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    children: Element,
}

#[component]
fn OuterTrigger(props: OuterTriggerProps) -> Element {
    let trace = props.on_trace;
    let mut base: Vec<Attribute> = vec![
        Attribute::new("class", "outer-trigger", None, false),
        Attribute::new("style", "--outer: 1", None, false),
        Attribute::new("aria-describedby", "field-desc tip-id", None, false),
        Attribute::new("data-outer-state", "closed", None, false),
    ];
    base = base
        .onclick(move |_evt| trace.call("outer:click"))
        .onmounted(move |_evt| trace.call("outer:mounted"));

    let mut user = props.attributes;
    push_event_handler(&mut user, "onclick", props.on_click);

    let merged = merge_props_with_config(
        base,
        user,
        MergePropsConfig {
            invariant_attrs: &["data-outer-state"],
            bookkeeping_events: &[],
        },
    );
    if let Some(child) = props.child {
        return child.call(merged);
    }
    rsx! {
        button { ..merged, {props.children} }
    }
}

#[derive(Props, Clone, PartialEq)]
struct InnerTriggerProps {
    #[props(default)]
    child: Option<Callback<Vec<Attribute>, Element>>,
    on_trace: EventHandler<&'static str>,
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    children: Element,
}

#[component]
fn InnerTrigger(props: InnerTriggerProps) -> Element {
    let trace = props.on_trace;
    let mut base: Vec<Attribute> = vec![
        Attribute::new("class", "inner-trigger", None, false),
        Attribute::new("style", "--inner: 2", None, false),
        Attribute::new("aria-describedby", "tip-id dialog-desc", None, false),
        Attribute::new("aria-haspopup", "dialog", None, false),
    ];
    push_bool_data_attr(&mut base, "data-inner-trigger", true);
    base = base
        .onclick(move |_evt| trace.call("inner:click"))
        .onmounted(move |_evt| trace.call("inner:mounted"));

    let merged = merge_props(base, props.attributes);
    if let Some(child) = props.child {
        return child.call(merged);
    }
    rsx! {
        button { ..merged, {props.children} }
    }
}

#[derive(Props, Clone, PartialEq)]
struct LeafButtonProps {
    #[props(default)]
    on_click: Option<EventHandler<MouseEvent>>,
    on_trace: EventHandler<&'static str>,
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    children: Element,
}

#[component]
fn LeafButton(props: LeafButtonProps) -> Element {
    let trace = props.on_trace;
    let mut base: Vec<Attribute> = vec![
        Attribute::new("type", "button", None, false),
        Attribute::new("class", "leaf-btn", None, false),
        Attribute::new("style", "--leaf: 3; width: 140px; height: 40px;", None, false),
    ];
    push_attr_opt(&mut base, "data-button-root", Some(""));
    base = base
        .onclick(move |_evt| trace.call("leaf:click"))
        .onpointerdown(move |_evt| trace.call("leaf:bookkeeping-pointerdown"))
        .onmounted(move |_evt| trace.call("leaf:mounted"));

    let mut user = props.attributes;
    push_event_handler(&mut user, "onclick", props.on_click);

    let merged = merge_props_with_config(
        base,
        user,
        MergePropsConfig {
            invariant_attrs: &[],
            bookkeeping_events: &["onpointerdown"],
        },
    );
    rsx! {
        button { ..merged, {props.children} }
    }
}

#[test]
fn fm_s1_1_three_layer_child_composition_merges_attrs_listeners_and_respects_prevent_default() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    thread_local! {
        static LOG: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
    }
    LOG.with(|l| l.borrow_mut().clear());

    fn scenario() -> Element {
        let mut veto = use_signal(|| false);
        let push = |tag: &'static str| {
            LOG.with(|l| l.borrow_mut().push(tag));
        };

        rsx! {
            button {
                id: "toggle-veto",
                style: "width: 100px; height: 30px;",
                onclick: move |_| veto.toggle(),
                "Toggle Veto"
            }
            OuterTrigger {
                id: "composed-trigger",
                class: "user-cls",
                style: "color: red;",
                "aria-describedby": "user-desc field-desc",
                "data-outer-state": "tampered",
                on_click: move |evt: MouseEvent| {
                    push("user:typed-on_click");
                    if veto() {
                        evt.prevent_default();
                    }
                },
                onpointerdown: move |evt: Event<PointerData>| {
                    push("user:pointerdown");
                    if veto() {
                        evt.prevent_default();
                    }
                },
                onmounted: move |_evt: Event<MountedData>| {
                    push("user:mounted");
                },
                on_trace: move |tag| push(tag),
                child: move |outer_props: Vec<Attribute>| rsx! {
                    InnerTrigger {
                        attributes: outer_props,
                        on_trace: move |tag| push(tag),
                        child: move |inner_props: Vec<Attribute>| rsx! {
                            LeafButton {
                                attributes: inner_props,
                                on_trace: move |tag| push(tag),
                                "Action"
                            }
                        }
                    }
                }
            }
        }
    }

    let mut harness = mount_native_scenario(scenario);

    // 1. Single <button id="composed-trigger"> node, zero nested buttons.
    assert!(
        harness.query("#composed-trigger").is_some(),
        "composed trigger button must mount"
    );
    assert_eq!(
        harness.query_all("#composed-trigger button").len(),
        0,
        "must not render nested <button> elements"
    );

    // 2. Attribute merge verification: class, style, aria-describedby union, invariant attr protection.
    let cls = harness.attr("#composed-trigger", "class").unwrap_or_default();
    assert!(cls.contains("leaf-btn"), "missing leaf-btn: {cls}");
    assert!(cls.contains("inner-trigger"), "missing inner-trigger: {cls}");
    assert!(cls.contains("outer-trigger"), "missing outer-trigger: {cls}");
    assert!(cls.contains("user-cls"), "missing user-cls: {cls}");

    let style = harness.attr("#composed-trigger", "style").unwrap_or_default();
    assert!(style.contains("--leaf: 3"), "missing --leaf: {style}");
    assert!(style.contains("--inner: 2"), "missing --inner: {style}");
    assert!(style.contains("--outer: 1"), "missing --outer: {style}");
    assert!(style.contains("color: red"), "missing color: red: {style}");

    let describedby = harness
        .attr("#composed-trigger", "aria-describedby")
        .unwrap_or_default();
    let tokens: Vec<&str> = describedby.split_ascii_whitespace().collect();
    for expected in ["tip-id", "dialog-desc", "field-desc", "user-desc"] {
        assert_eq!(
            tokens.iter().filter(|&&t| t == expected).count(),
            1,
            "aria-describedby must contain `{expected}` exactly once: `{describedby}`"
        );
    }

    // Invariant attr `data-outer-state` must not be clobbered by `"tampered"`.
    assert_eq!(
        harness.attr("#composed-trigger", "data-outer-state").as_deref(),
        Some("closed")
    );
    assert_eq!(
        harness.attr("#composed-trigger", "aria-haspopup").as_deref(),
        Some("dialog")
    );
    assert_eq!(
        harness
            .attr("#composed-trigger", "data-inner-trigger")
            .as_deref(),
        Some("")
    );
    assert_eq!(
        harness
            .attr("#composed-trigger", "data-button-root")
            .as_deref(),
        Some("")
    );

    // 3. All 4 onmounted listeners (user + outer + inner + leaf) fired!
    let mounted = LOG.with(|l| l.borrow().clone());
    for tag in ["user:mounted", "outer:mounted", "inner:mounted", "leaf:mounted"] {
        assert!(
            mounted.contains(&tag),
            "missing `{tag}` in mounted events: {mounted:?}"
        );
    }

    // 4. Non-vetoed click runs in exact order: user -> outer -> inner -> leaf.
    LOG.with(|l| l.borrow_mut().clear());
    click_and_pump(&mut harness, "#composed-trigger");
    let clicks: Vec<&'static str> = LOG.with(|l| {
        l.borrow()
            .iter()
            .copied()
            .filter(|s| s.ends_with(":click") || s.ends_with(":typed-on_click"))
            .collect()
    });
    assert_eq!(
        clicks,
        vec![
            "user:typed-on_click",
            "outer:click",
            "inner:click",
            "leaf:click"
        ]
    );

    // 5. Vetoed click (`evt.prevent_default()`) suppresses outer/inner/leaf default clicks,
    //    while bookkeeping `onpointerdown` STILL runs!
    click_and_pump(&mut harness, "#toggle-veto");
    LOG.with(|l| l.borrow_mut().clear());
    click_and_pump(&mut harness, "#composed-trigger");
    let vetoed = LOG.with(|l| l.borrow().clone());
    assert!(
        vetoed.contains(&"user:typed-on_click"),
        "user click must run: {vetoed:?}"
    );
    assert!(
        !vetoed.contains(&"outer:click")
            && !vetoed.contains(&"inner:click")
            && !vetoed.contains(&"leaf:click"),
        "prevent_default() must suppress chained default clicks: {vetoed:?}"
    );
    assert!(
        vetoed.contains(&"user:pointerdown") && vetoed.contains(&"leaf:bookkeeping-pointerdown"),
        "bookkeeping_events must still fire even when user calls prevent_default(): {vetoed:?}"
    );
}

#[test]
fn fm_s1_1_presence_transition_status_and_keep_mounted() {
    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    fn scenario() -> Element {
        let mut open = use_signal(|| true);
        let mut completed = use_signal(Vec::<bool>::new);
        let presence = use_presence_with_keep_mounted(
            open(),
            "keep-mounted-panel",
            true,
            true,
            Some(EventHandler::new(move |val| {
                completed.write().push(val);
            })),
        );

        let status_label = match presence.transition_status() {
            TransitionStatus::None => "none",
            TransitionStatus::Starting => "starting",
            TransitionStatus::Idle => "idle",
            TransitionStatus::Ending => "ending",
        };
        let state_label = match presence.state() {
            PresenceState::Unmounted => "unmounted",
            PresenceState::Mounted => "mounted",
            PresenceState::Suspended => "suspended",
        };

        rsx! {
            style {
                "@keyframes km-exit {{ from {{ opacity: 1; }} to {{ opacity: 0; }} }}
                 .km-exiting {{ animation: km-exit 0.2s linear forwards; }}"
            }
            button {
                id: "km-toggle",
                style: "width: 100px; height: 30px;",
                onclick: move |_| open.toggle(),
                "Toggle"
            }
            if presence.should_render() {
                div {
                    id: "keep-mounted-panel",
                    class: if presence.state() == PresenceState::Suspended { "km-exiting" } else { "" },
                    hidden: presence.is_hidden(),
                    "data-starting-style": presence.data_starting_style(),
                    "data-ending-style": presence.data_ending_style(),
                    "data-transition": status_label,
                    "data-presence": state_label,
                    "data-completed-count": "{completed.read().len()}",
                    "Panel"
                }
            }
        }
    }

    let mut harness = mount_native_scenario(scenario);
    assert!(
        harness.query("#keep-mounted-panel").is_some(),
        "initial panel mounted"
    );
    assert_eq!(
        harness
            .attr("#keep-mounted-panel", "data-transition")
            .as_deref(),
        Some("idle")
    );
    assert_eq!(harness.attr("#keep-mounted-panel", "data-ending-style"), None);
    assert_eq!(harness.attr("#keep-mounted-panel", "hidden"), None);

    // Close -> enters Suspended / Ending with data-ending-style=""
    click_and_pump(&mut harness, "#km-toggle");
    assert!(
        harness.query("#keep-mounted-panel").is_some(),
        "panel remains mounted while exiting"
    );
    assert_eq!(
        harness
            .attr("#keep-mounted-panel", "data-presence")
            .as_deref(),
        Some("suspended")
    );
    assert_eq!(
        harness
            .attr("#keep-mounted-panel", "data-transition")
            .as_deref(),
        Some("ending")
    );
    assert_eq!(
        harness
            .attr("#keep-mounted-panel", "data-ending-style")
            .as_deref(),
        Some("")
    );

    // Finish exit animation -> controller Unmounted, but keep_mounted keeps DOM node hidden!
    advance_frame_and_pump(&mut harness, 0.25);

    assert!(
        harness.query("#keep-mounted-panel").is_some(),
        "keep_mounted must keep node in DOM after exit"
    );
    assert_eq!(
        harness
            .attr("#keep-mounted-panel", "data-presence")
            .as_deref(),
        Some("unmounted")
    );
    assert_eq!(
        harness
            .attr("#keep-mounted-panel", "data-transition")
            .as_deref(),
        Some("none")
    );
    assert_eq!(harness.attr("#keep-mounted-panel", "data-ending-style"), None);
    assert!(
        harness.attr("#keep-mounted-panel", "hidden").is_some(),
        "keep_mounted closed node must carry hidden attribute"
    );
    assert_eq!(
        harness
            .attr("#keep-mounted-panel", "data-completed-count")
            .as_deref(),
        Some("1")
    );

    // Reopen without advancing rAF yet -> enters Starting (data-starting-style="") and clears hidden
    let (tx, ty) = harness.center_of("#km-toggle");
    let click_evt = blitz_test_harness::mouse_pointer_event(tx, ty);
    harness.dispatch(blitz_traits::events::UiEvent::PointerDown(click_evt.clone()));
    harness.dispatch(blitz_traits::events::UiEvent::PointerUp(click_evt));
    drain_polls_and_resolve(&mut harness);

    assert_eq!(harness.attr("#keep-mounted-panel", "hidden"), None);
    assert_eq!(
        harness
            .attr("#keep-mounted-panel", "data-transition")
            .as_deref(),
        Some("starting")
    );
    assert_eq!(
        harness
            .attr("#keep-mounted-panel", "data-starting-style")
            .as_deref(),
        Some("")
    );

    // Next rAF frame advances Starting -> Idle and removes data-starting-style
    advance_frame_and_pump(&mut harness, 0.02);
    assert_eq!(
        harness
            .attr("#keep-mounted-panel", "data-transition")
            .as_deref(),
        Some("idle")
    );
    assert_eq!(harness.attr("#keep-mounted-panel", "data-starting-style"), None);
}
