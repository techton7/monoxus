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

#[test]
fn fm_s1_2_controlled_veto_disabled_guards_child_forwarding_and_empty_attr_hygiene() {
    use monoxus::accessible_icon::AccessibleIcon;
    use monoxus::accordion::{
        Accordion, AccordionContent, AccordionHeader, AccordionItem, AccordionMode,
        AccordionTrigger,
    };
    use monoxus::avatar::{Avatar, AvatarFallback, AvatarImage};
    use monoxus::collapsible::{Collapsible, CollapsibleContent, CollapsibleTrigger};
    use monoxus::field::{Field, FieldDescription, FieldError, FieldLabel};
    use monoxus::pagination::{
        PageData, Pagination, PaginationNextButton, PaginationPage, PaginationPrevButton,
    };
    use monoxus::progress::{Progress, ProgressIndicator};
    use monoxus::tabs::{Tabs, TabsContent, TabsList, TabsTrigger};
    use monoxus::toolbar::{Toolbar, ToolbarButton};

    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    fn scenario() -> Element {
        let mut accordion_vetoed = use_signal(|| 0usize);
        let mut tabs_vetoed = use_signal(|| 0usize);
        let mut col_vetoed = use_signal(|| 0usize);
        let mut col_disabled_clicks = use_signal(|| 0usize);
        let mut col_pd_clicks = use_signal(|| 0usize);
        let mut field_child_clicks = use_signal(|| 0usize);

        rsx! {
            // 1. Controlled AccordionRoot single-mode veto (stays on "a")
            Accordion {
                id: "s12-acc",
                mode: AccordionMode::Single { collapsible: true },
                value: "a",
                on_value_change: move |_next| accordion_vetoed += 1,
                AccordionItem {
                    value: "a",
                    AccordionHeader {
                        AccordionTrigger { id: "s12-acc-trig-a", "A" }
                    }
                    AccordionContent { id: "s12-acc-panel-a", "Body A" }
                }
                AccordionItem {
                    value: "b",
                    AccordionHeader {
                        AccordionTrigger { id: "s12-acc-trig-b", "B" }
                    }
                    AccordionContent { id: "s12-acc-panel-b", "Body B" }
                }
            }
            div { id: "s12-acc-veto-count", "data-count": "{accordion_vetoed}" }

            // 2. Controlled TabsRoot veto + TabsRoot.disabled
            Tabs {
                id: "s12-tabs-ctrl",
                value: "t1",
                on_value_change: move |_next| tabs_vetoed += 1,
                TabsList {
                    TabsTrigger { id: "s12-tab-1", value: "t1", "Tab 1" }
                    TabsTrigger { id: "s12-tab-2", value: "t2", "Tab 2" }
                }
                TabsContent { id: "s12-panel-1", value: "t1", "P1" }
                TabsContent { id: "s12-panel-2", value: "t2", "P2" }
            }
            div { id: "s12-tabs-veto-count", "data-count": "{tabs_vetoed}" }
            Tabs {
                id: "s12-tabs-dis",
                default_value: "d1",
                disabled: true,
                TabsList {
                    TabsTrigger { id: "s12-tab-d1", value: "d1", "D1" }
                    TabsTrigger { id: "s12-tab-d2", value: "d2", "D2" }
                }
                TabsContent { id: "s12-panel-d1", value: "d1", "DP1" }
                TabsContent { id: "s12-panel-d2", value: "d2", "DP2" }
            }

            // 3. Collapsible: controlled veto, disabled trigger guard, prevent_default guard, zero empty class/style
            Collapsible {
                id: "s12-col-ctrl",
                "data-testid": "s12-col-ctrl",
                open: false,
                on_open_change: move |_next| col_vetoed += 1,
                CollapsibleTrigger { id: "s12-col-ctrl-trig", "Ctrl" }
                CollapsibleContent { id: "s12-col-ctrl-body", "Body" }
            }
            div { id: "s12-col-veto-count", "data-count": "{col_vetoed}" }
            Collapsible {
                id: "s12-col-dis",
                "data-testid": "s12-col-dis",
                disabled: true,
                CollapsibleTrigger {
                    id: "s12-col-dis-trig",
                    on_click: move |_evt| col_disabled_clicks += 1,
                    "Dis"
                }
                CollapsibleContent { id: "s12-col-dis-body", "Body" }
            }
            div { id: "s12-col-dis-clicks", "data-count": "{col_disabled_clicks}" }
            Collapsible {
                id: "s12-col-pd",
                "data-testid": "s12-col-pd",
                CollapsibleTrigger {
                    id: "s12-col-pd-trig",
                    on_click: move |evt: MouseEvent| {
                        col_pd_clicks += 1;
                        evt.prevent_default();
                    },
                    "Prevent"
                }
                CollapsibleContent { id: "s12-col-pd-body", "Body" }
            }
            div { id: "s12-col-pd-clicks", "data-count": "{col_pd_clicks}" }

            // 4. AvatarImage void <img> + Avatar/Progress/Pagination/Field/AccessibleIcon child forwarding
            Avatar {
                id: "s12-avatar",
                loading_status: monoxus::avatar::AvatarImageLoadingStatus::Loaded,
                class: "av-cls",
                child: move |props: Vec<Attribute>| rsx! {
                    span {
                        ..props,
                        AvatarImage { id: "s12-avatar-img", src: "test.png" }
                        AvatarFallback { id: "s12-avatar-fb", "FB" }
                    }
                },
            }
            Progress {
                id: "s12-progress",
                value: Some(50.0),
                class: "prog-cls",
                style: "height: 8px;",
                "data-prog-extra": "ok",
                child: move |props: Vec<Attribute>| rsx! {
                    div {
                        ..props,
                        ProgressIndicator {
                            id: "s12-progress-ind",
                            class: "ind-cls",
                            child: move |ind_props: Vec<Attribute>| rsx! {
                                span { ..ind_props }
                            },
                        }
                    }
                },
            }
            Field {
                id: "s12-field",
                invalid: true,
                class: "fld-cls",
                style: "gap: 4px;",
                "data-fld-extra": "yes",
                child: move |props: Vec<Attribute>| rsx! {
                    section {
                        ..props,
                        FieldLabel {
                            id: "s12-field-label",
                            class: "lbl-cls",
                            on_click: move |_evt| field_child_clicks += 1,
                            child: move |lp: Vec<Attribute>| rsx! {
                                label { ..lp, "Email" }
                            },
                        }
                        FieldDescription { id: "s12-field-desc", "Hint" }
                        FieldError { id: "s12-field-err", errors: vec!["Required".to_string()] }
                    }
                },
            }
            div { id: "s12-field-clicks", "data-count": "{field_child_clicks}" }
            AccessibleIcon {
                label: "Star",
                "data-icon-extra": "preserved",
                child: move |props: Vec<Attribute>| rsx! {
                    svg { id: "s12-icon-svg", ..props }
                },
            }

            // 5. Unstyled Toolbar & Pagination emit zero empty class="" / style=""
            Toolbar {
                id: "s12-tb",
                ToolbarButton { id: "s12-tb-btn", "Bold" }
            }
            Pagination {
                id: "s12-pg",
                count: 30,
                per_page: 10,
                PaginationPrevButton { id: "s12-pg-prev", "Prev" }
                PaginationPage { id: "s12-pg-1", page: PageData { value: 1 }, "1" }
                PaginationNextButton { id: "s12-pg-next", "Next" }
            }
        }
    }

    let mut harness = mount_native_scenario(scenario);

    // 1. Controlled Accordion veto: click B -> callback fires, DOM stays on A (B remains closed)
    click_and_pump(&mut harness, "#s12-acc-trig-b");
    assert_eq!(harness.attr("#s12-acc-veto-count", "data-count").as_deref(), Some("1"));
    assert_eq!(harness.attr("#s12-acc-trig-a", "data-state").as_deref(), Some("open"));
    assert_eq!(harness.attr("#s12-acc-trig-b", "data-state").as_deref(), Some("closed"));

    // 2. Controlled Tabs veto + disabled TabsRoot
    click_and_pump(&mut harness, "#s12-tab-2");
    assert!(
        harness
            .attr("#s12-tabs-veto-count", "data-count")
            .as_deref()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0)
            >= 1,
        "controlled Tabs on_value_change must fire on user activation"
    );
    assert_eq!(harness.attr("#s12-tab-1", "data-state").as_deref(), Some("active"));
    assert_eq!(harness.attr("#s12-tab-2", "data-state").as_deref(), Some("inactive"));

    assert_eq!(harness.attr("#s12-tab-d2", "data-disabled").as_deref(), Some(""));
    click_and_pump(&mut harness, "#s12-tab-d2");
    assert_eq!(harness.attr("#s12-tab-d1", "data-state").as_deref(), Some("active"));
    assert_eq!(harness.attr("#s12-tab-d2", "data-state").as_deref(), Some("inactive"));

    // 3. Collapsible controlled veto, disabled trigger guard, prevent_default guard
    click_and_pump(&mut harness, "#s12-col-ctrl-trig");
    assert_eq!(harness.attr("#s12-col-veto-count", "data-count").as_deref(), Some("1"));
    assert_eq!(
        harness
            .attr("[data-testid=\"s12-col-ctrl\"]", "data-state")
            .as_deref(),
        Some("closed")
    );

    click_and_pump(&mut harness, "#s12-col-dis-trig");
    assert_eq!(harness.attr("#s12-col-dis-clicks", "data-count").as_deref(), Some("0"));
    assert_eq!(
        harness
            .attr("[data-testid=\"s12-col-dis\"]", "data-state")
            .as_deref(),
        Some("closed")
    );

    click_and_pump(&mut harness, "#s12-col-pd-trig");
    assert_eq!(harness.attr("#s12-col-pd-clicks", "data-count").as_deref(), Some("1"));
    assert_eq!(
        harness
            .attr("[data-testid=\"s12-col-pd\"]", "data-state")
            .as_deref(),
        Some("closed")
    );

    // 4. AvatarImage void <img> has zero children; child forwarding preserves class/style/attrs/events
    let img_nid = harness.node("#s12-avatar-img");
    assert!(
        harness.base().get_node(img_nid).unwrap().children.is_empty(),
        "AvatarImage void <img> must render zero child nodes"
    );
    assert_eq!(harness.attr("#s12-avatar", "class").as_deref(), Some("av-cls"));
    assert_eq!(harness.attr("#s12-progress", "class").as_deref(), Some("prog-cls"));
    assert_eq!(harness.attr("#s12-progress", "data-prog-extra").as_deref(), Some("ok"));
    assert_eq!(harness.attr("#s12-progress-ind", "class").as_deref(), Some("ind-cls"));
    assert_eq!(harness.attr("#s12-field", "class").as_deref(), Some("fld-cls"));
    assert_eq!(harness.attr("#s12-field", "data-fld-extra").as_deref(), Some("yes"));
    assert_eq!(harness.attr("#s12-field-label", "class").as_deref(), Some("lbl-cls"));
    click_and_pump(&mut harness, "#s12-field-label");
    assert_eq!(harness.attr("#s12-field-clicks", "data-count").as_deref(), Some("1"));
    assert_eq!(
        harness.attr("#s12-icon-svg", "data-icon-extra").as_deref(),
        Some("preserved")
    );
    assert_eq!(harness.attr("#s12-icon-svg", "aria-hidden").as_deref(), Some("true"));

    // 5. Zero empty class="" or style="" on unstyled Collapsible, Toolbar, Pagination
    for sel in [
        "[data-testid=\"s12-col-ctrl\"]",
        "#s12-col-ctrl-trig",
        "#s12-tb",
        "#s12-tb-btn",
        "#s12-pg",
        "#s12-pg-prev",
        "#s12-pg-1",
        "#s12-pg-next",
    ] {
        assert_ne!(harness.attr(sel, "class").as_deref(), Some(""), "{sel} must not emit empty class");
        assert_ne!(harness.attr(sel, "style").as_deref(), Some(""), "{sel} must not emit empty style");
    }
}

#[test]
fn fm_s1_3_three_layer_tooltip_dialog_button_composition() {
    use monoxus::button::Button;
    use monoxus::dialog::{DialogContent, DialogRoot, DialogTrigger};
    use monoxus::tooltip::{TooltipContent, TooltipProvider, TooltipRoot, TooltipTrigger};

    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    fn scenario() -> Element {
        let mut open_clicks = use_signal(|| 0usize);
        let mut veto_clicks = use_signal(|| 0usize);

        rsx! {
            TooltipProvider {
                // Variant 1: 3-layer composition opens Dialog and runs user click handler
                DialogRoot {
                    TooltipRoot {
                        TooltipTrigger {
                            id: "tri-btn",
                            class: "tt-cls",
                            style: "--tt: 1;",
                            "data-tt-layer": "yes",
                            child: move |tp: Vec<Attribute>| rsx! {
                                DialogTrigger {
                                    id: "tri-btn",
                                    class: "dlg-cls",
                                    style: "--dlg: 2;",
                                    "data-dlg-layer": "yes",
                                    attributes: tp,
                                    child: move |dp: Vec<Attribute>| rsx! {
                                        Button {
                                            id: "tri-btn",
                                            class: "usr-cls",
                                            style: "padding: 4px;",
                                            attributes: dp,
                                            on_click: move |_evt: MouseEvent| {
                                                open_clicks += 1;
                                            },
                                            "Open"
                                        }
                                    },
                                }
                            },
                        }
                        TooltipContent { id: "tri-tt-content", "Tooltip Hint" }
                    }
                    DialogContent { id: "tri-dialog-content", "Dialog Body" }
                }
                div { id: "tri-open-clicks", "data-count": "{open_clicks}" }

                // Variant 2: user prevent_default() suppresses Dialog opening while still running user handler
                DialogRoot {
                    TooltipRoot {
                        TooltipTrigger {
                            id: "tri-btn-veto",
                            class: "tt-veto-cls",
                            child: move |tp: Vec<Attribute>| rsx! {
                                DialogTrigger {
                                    id: "tri-btn-veto",
                                    attributes: tp,
                                    on_click: move |evt: MouseEvent| {
                                        veto_clicks += 1;
                                        evt.prevent_default();
                                    },
                                    child: move |dp: Vec<Attribute>| rsx! {
                                        Button {
                                            id: "tri-btn-veto",
                                            class: "usr-veto-cls",
                                            style: "padding: 4px;",
                                            attributes: dp,
                                            "Blocked Open"
                                        }
                                    },
                                }
                            },
                        }
                        TooltipContent { id: "tri-tt-veto-content", "Blocked Hint" }
                    }
                    DialogContent { id: "tri-dialog-veto-content", "Should Not Open" }
                }
                div { id: "tri-veto-clicks", "data-count": "{veto_clicks}" }
            }
        }
    }

    let mut harness = mount_native_scenario(scenario);

    // 1. Single <button id="tri-btn"> with zero nested <button> descendants.
    let btn_nid = harness.node("#tri-btn");
    let btn_tag = harness
        .base()
        .get_node(btn_nid)
        .and_then(|n| n.element_data())
        .map(|el| el.name.local.to_string());
    assert_eq!(btn_tag.as_deref(), Some("button"));
    assert_eq!(
        harness.query_all("#tri-btn button").len(),
        0,
        "3-layer composition must not emit nested <button> elements"
    );

    // 2. Class, style, and data/ARIA attrs from all 3 layers are merged.
    let cls = harness.attr("#tri-btn", "class").unwrap_or_default();
    assert!(cls.contains("tt-cls"), "missing TooltipTrigger class: {cls}");
    assert!(cls.contains("dlg-cls"), "missing DialogTrigger class: {cls}");
    assert!(cls.contains("usr-cls"), "missing Button class: {cls}");

    let style = harness.attr("#tri-btn", "style").unwrap_or_default();
    assert!(style.contains("--tt: 1"), "missing TooltipTrigger style: {style}");
    assert!(style.contains("--dlg: 2"), "missing DialogTrigger style: {style}");
    assert!(style.contains("padding: 4px"), "missing Button style: {style}");

    assert_eq!(harness.attr("#tri-btn", "data-tt-layer").as_deref(), Some("yes"));
    assert_eq!(harness.attr("#tri-btn", "data-dlg-layer").as_deref(), Some("yes"));
    assert_eq!(harness.attr("#tri-btn", "data-button-root").as_deref(), Some(""));
    assert_eq!(harness.attr("#tri-btn", "aria-haspopup").as_deref(), Some("dialog"));
    assert_eq!(harness.attr("#tri-btn", "data-state").as_deref(), Some("closed"));

    // 3. Variant 2: prevent_default() suppresses Dialog opening while still incrementing user counter.
    assert!(harness.query("#tri-dialog-veto-content").is_none());
    click_and_pump(&mut harness, "#tri-btn-veto");
    assert_eq!(
        harness.attr("#tri-veto-clicks", "data-count").as_deref(),
        Some("1")
    );
    assert!(
        harness.query("#tri-dialog-veto-content").is_none(),
        "prevent_default() in composed trigger must suppress Dialog opening"
    );

    // 4. Variant 1: normal click opens Dialog AND increments user click counter.
    assert!(harness.query("#tri-dialog-content").is_none());
    click_and_pump(&mut harness, "#tri-btn");
    assert_eq!(
        harness.attr("#tri-open-clicks", "data-count").as_deref(),
        Some("1")
    );
    assert!(
        harness.query("#tri-dialog-content").is_some(),
        "composed 3-layer trigger click must open DialogContent"
    );
}

#[test]
fn fm_s1_3_overlay_non_dom_roots_tooltip_isolation_and_controlled_veto() {
    use keyboard_types::{Code, Key, NamedKey};
    use monoxus::alert_dialog::AlertDialogRoot;
    use monoxus::combobox::{ComboboxContent, ComboboxInput, ComboboxRoot, ComboboxTrigger};
    use monoxus::date_picker::{DatePickerContent, DatePickerRoot, DatePickerTrigger};
    use monoxus::dialog::{DialogClose, DialogContent, DialogRoot, DialogTrigger, EscapeKeyDownEvent};
    use monoxus::foundation::overlay::DismissBehavior;
    use monoxus::hover_card::{HoverCardRoot, HoverCardTrigger};
    use monoxus::popover::{PopoverContent, PopoverRoot, PopoverTrigger};
    use monoxus::select::{SelectContent, SelectItem, SelectRoot, SelectTrigger};
    use monoxus::tooltip::{TooltipContent, TooltipProvider, TooltipRoot, TooltipTrigger};

    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    fn scenario() -> Element {
        let mut dlg_veto = use_signal(|| 0usize);
        let mut pop_veto = use_signal(|| 0usize);
        let mut sel_veto = use_signal(|| 0usize);
        let mut cbx_veto = use_signal(|| 0usize);
        let mut dp_veto = use_signal(|| 0usize);
        let mut esc_pd_count = use_signal(|| 0usize);

        rsx! {
            // 1. Non-DOM root wrappers check (zero <div style="display: contents;">)
            div {
                id: "wrap-dialog",
                DialogRoot {
                    DialogTrigger { id: "nd-dlg-trig", "D" }
                }
            }
            div {
                id: "wrap-alert-dialog",
                AlertDialogRoot {
                    DialogTrigger { id: "nd-adlg-trig", "AD" }
                }
            }
            div {
                id: "wrap-popover",
                PopoverRoot {
                    PopoverTrigger { id: "nd-pop-trig", "P" }
                }
            }
            div {
                id: "wrap-tooltip",
                TooltipProvider {
                    TooltipRoot {
                        TooltipTrigger { id: "nd-tt-trig", "T" }
                    }
                }
            }
            div {
                id: "wrap-hover-card",
                HoverCardRoot {
                    HoverCardTrigger { id: "nd-hc-trig", "HC" }
                }
            }
            div {
                id: "wrap-select",
                SelectRoot {
                    SelectTrigger { id: "nd-sel-trig", "S" }
                }
            }

            // 2. Controlled open: Some(false) veto across 5 overlay primitives
            DialogRoot {
                open: Some(false),
                on_open_change: move |_next: bool| dlg_veto += 1,
                DialogTrigger { id: "veto-dlg-trig", "Veto Dialog" }
                DialogContent { id: "veto-dlg-content", "Body" }
            }
            div { id: "veto-dlg-count", "data-count": "{dlg_veto}" }

            PopoverRoot {
                open: Some(false),
                on_open_change: move |_next: bool| pop_veto += 1,
                PopoverTrigger { id: "veto-pop-trig", "Veto Popover" }
                PopoverContent { id: "veto-pop-content", "Body" }
            }
            div { id: "veto-pop-count", "data-count": "{pop_veto}" }

            SelectRoot {
                open: Some(false),
                on_open_change: move |_next: bool| sel_veto += 1,
                SelectTrigger { id: "veto-sel-trig", "Veto Select" }
                SelectContent {
                    id: "veto-sel-content",
                    SelectItem { value: "a", "A" }
                }
            }
            div { id: "veto-sel-count", "data-count": "{sel_veto}" }

            ComboboxRoot {
                open: Some(false),
                on_open_change: move |_next: bool| cbx_veto += 1,
                ComboboxInput { id: "veto-cbx-input" }
                ComboboxTrigger { id: "veto-cbx-trig", "Veto Combobox" }
                ComboboxContent { id: "veto-cbx-content", "Body" }
            }
            div { id: "veto-cbx-count", "data-count": "{cbx_veto}" }

            DatePickerRoot {
                open: Some(false),
                on_open_change: move |_next: bool| dp_veto += 1,
                DatePickerTrigger { id: "veto-dp-trig", "Veto DatePicker" }
                DatePickerContent { id: "veto-dp-content", "Body" }
            }
            div { id: "veto-dp-count", "data-count": "{dp_veto}" }

            // 3. TooltipRoot single override under TooltipProvider + TooltipRoot/PopoverRoot shared trigger
            TooltipProvider {
                delay_duration: 300,
                TooltipRoot {
                    id: "iso-tt-override",
                    delay_duration: Some(150),
                    TooltipTrigger { id: "iso-tt-trig", "Override Tip" }
                    TooltipContent { id: "iso-tt-content", "Tip Body" }
                }
                PopoverRoot {
                    TooltipRoot {
                        delay_duration: Some(150),
                        TooltipTrigger {
                            id: "tp-shared-btn",
                            child: move |tp: Vec<Attribute>| rsx! {
                                PopoverTrigger {
                                    id: "tp-shared-btn",
                                    attributes: tp,
                                    "Shared Trigger"
                                }
                            },
                        }
                        TooltipContent { id: "tp-shared-tt", "Shared Tip" }
                    }
                    PopoverContent { id: "tp-shared-pop", "Shared Popover" }
                }
            }

            // 4. DialogContent Escape prevent_default() and DismissBehavior::Ignore
            DialogRoot {
                DialogTrigger { id: "esc-pd-trig", "Open Esc Prevent" }
                DialogContent {
                    id: "esc-pd-content",
                    on_escape_key_down: move |evt: EscapeKeyDownEvent| {
                        esc_pd_count += 1;
                        evt.prevent_default();
                    },
                    DialogClose { id: "esc-pd-close", "Close" }
                }
            }
            div { id: "esc-pd-count", "data-count": "{esc_pd_count}" }

            DialogRoot {
                DialogTrigger { id: "esc-ign-trig", "Open Esc Ignore" }
                DialogContent {
                    id: "esc-ign-content",
                    escape_key_down_behavior: Some(DismissBehavior::Ignore),
                    DialogClose { id: "esc-ign-close", "Close" }
                }
            }
        }
    }

    let mut harness = mount_native_scenario(scenario);

    // 1. Assert zero <div style="display: contents;"> root wrappers in direct children.
    for wrap_sel in [
        "#wrap-dialog",
        "#wrap-alert-dialog",
        "#wrap-popover",
        "#wrap-tooltip",
        "#wrap-hover-card",
        "#wrap-select",
    ] {
        let wrap_nid = harness.node(wrap_sel);
        let doc = harness.base();
        let wrap_node = doc.get_node(wrap_nid).unwrap();
        for &child_id in &wrap_node.children {
            let Some(child_node) = doc.get_node(child_id) else {
                continue;
            };
            let Some(el) = child_node.element_data() else {
                continue;
            };
            let is_div = el.name.local == blitz_dom::local_name!("div");
            let is_portal = child_node.attr("data-portal-anchor".into()).is_some()
                || child_node.attr("data-portal-host".into()).is_some()
                || child_node
                    .attr(blitz_dom::local_name!("id"))
                    .is_some_and(|id| id.contains("portal"));
            let has_display_contents = child_node
                .attr(blitz_dom::local_name!("style"))
                .is_some_and(|s| s.contains("display: contents") || s.contains("display:contents"));
            assert!(
                !(is_div && !is_portal && has_display_contents),
                "{wrap_sel} must not contain a non-portal <div style=\"display: contents\"> root wrapper"
            );
        }
    }

    // 2. Controlled open: Some(false) veto across Dialog, Popover, Select, Combobox, DatePicker.
    for (trig_sel, content_sel, count_sel) in [
        ("#veto-dlg-trig", "#veto-dlg-content", "#veto-dlg-count"),
        ("#veto-pop-trig", "#veto-pop-content", "#veto-pop-count"),
        ("#veto-sel-trig", "#veto-sel-content", "#veto-sel-count"),
        ("#veto-cbx-trig", "#veto-cbx-content", "#veto-cbx-count"),
        ("#veto-dp-trig", "#veto-dp-content", "#veto-dp-count"),
    ] {
        assert!(harness.query(content_sel).is_none(), "{content_sel} must start unmounted");
        click_and_pump(&mut harness, trig_sel);
        let count = harness
            .attr(count_sel, "data-count")
            .as_deref()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        assert!(count >= 1, "{trig_sel} click must fire on_open_change (count={count})");
        assert!(
            harness.query(content_sel).is_none(),
            "{content_sel} must stay unmounted when controlled open=Some(false) vetoes"
        );
        assert_eq!(
            harness.attr(trig_sel, "data-state").as_deref(),
            Some("closed"),
            "{trig_sel} must remain data-state=closed"
        );
    }

    // 3. TooltipRoot override under TooltipProvider + TooltipRoot/PopoverRoot shared trigger.
    assert!(harness.query("#iso-tt-trig").is_some());
    assert_eq!(harness.query_all("#tp-shared-btn button").len(), 0);
    assert!(harness.query("#tp-shared-pop").is_none());
    click_and_pump(&mut harness, "#tp-shared-btn");
    assert!(
        harness.query("#tp-shared-pop").is_some(),
        "PopoverContent must open when sharing trigger with TooltipRoot"
    );
    assert_eq!(
        harness.attr("#tp-shared-pop", "data-state").as_deref(),
        Some("open")
    );
    // Close shared popover before dialog Escape checks.
    click_and_pump(&mut harness, "#tp-shared-btn");

    // 4. DialogContent on_escape_key_down prevent_default() blocks Escape close.
    click_and_pump(&mut harness, "#esc-pd-trig");
    assert!(harness.query("#esc-pd-content").is_some());
    focus_node(&mut harness, "#esc-pd-content");
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
    let esc_count = harness
        .attr("#esc-pd-count", "data-count")
        .as_deref()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);
    assert!(esc_count >= 1, "on_escape_key_down must fire on Escape");
    assert!(
        harness.query("#esc-pd-content").is_some(),
        "prevent_default() in on_escape_key_down must keep DialogContent open"
    );
    click_and_pump(&mut harness, "#esc-pd-close");
    assert!(harness.query("#esc-pd-content").is_none());

    // 5. DialogContent escape_key_down_behavior: Some(DismissBehavior::Ignore) blocks Escape close.
    click_and_pump(&mut harness, "#esc-ign-trig");
    assert!(harness.query("#esc-ign-content").is_some());
    focus_node(&mut harness, "#esc-ign-content");
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
    assert!(
        harness.query("#esc-ign-content").is_some(),
        "DismissBehavior::Ignore must block Escape close on DialogContent"
    );
}

#[test]
fn fm_s1_4_select_modernization_range_calendar_child_and_menu_hygiene() {
    use monoxus::dropdown_menu::{
        DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger,
    };
    use monoxus::foundation::date_time::CalendarDate;
    use monoxus::range_calendar::{
        RangeCalendar, RangeCalendarCell, RangeCalendarDay, RangeCalendarGrid,
        RangeCalendarGridBody, RangeCalendarGridRow, RangeCalendarSnapshot,
    };
    use monoxus::select::{
        Select, SelectContent, SelectGroup, SelectHiddenInput, SelectItem, SelectItemIndicator,
        SelectLabel, SelectSeparator, SelectTrigger,
    };

    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    fn scenario() -> Element {
        let mut sel_ctrl_veto = use_signal(|| 0usize);

        rsx! {
            // 1. Select modernization: Select alias, SelectItemIndicator, SelectGroup+SelectLabel auto-link,
            //    SelectSeparator style hygiene, SelectHiddenInput GlobalAttributes forwarding.
            Select {
                default_open: true,
                default_value: "apple",
                SelectTrigger { id: "s14-sel-open-trig", "Apple" }
                SelectContent {
                    id: "s14-sel-open-content",
                    SelectGroup {
                        id: "s14-sel-grp",
                        SelectLabel { id: "s14-sel-grp-lbl", "Fruits" }
                        SelectItem {
                            id: "s14-sel-item-apple",
                            value: "apple",
                            SelectItemIndicator { id: "s14-sel-ind-apple" }
                            "Apple"
                        }
                        SelectSeparator { id: "s14-sel-sep" }
                        SelectItem {
                            id: "s14-sel-item-banana",
                            value: "banana",
                            SelectItemIndicator { id: "s14-sel-ind-banana" }
                            "Banana"
                        }
                        SelectItem {
                            id: "s14-sel-item-cherry",
                            value: "cherry",
                            SelectItemIndicator { id: "s14-sel-ind-cherry", force_mount: true }
                            "Cherry"
                        }
                    }
                }
            }
            SelectHiddenInput {
                id: "s14-sel-hidden",
                name: "fruit",
                value: "apple",
                "data-hidden-extra": "yes",
            }

            // Uncontrolled Select { default_open: false } opens on trigger click
            Select {
                default_open: false,
                SelectTrigger { id: "s14-sel-unctl-trig", "Pick" }
                SelectContent {
                    id: "s14-sel-unctl-content",
                    SelectItem { value: "x", "X" }
                }
            }

            // Controlled Select { open: Some(false) } with vetoing on_open_change stays closed
            Select {
                open: Some(false),
                on_open_change: move |_next: bool| sel_ctrl_veto += 1,
                SelectTrigger { id: "s14-sel-ctl-trig", "Locked" }
                SelectContent {
                    id: "s14-sel-ctl-content",
                    SelectItem { value: "y", "Y" }
                }
            }
            div { id: "s14-sel-ctl-count", "data-count": "{sel_ctrl_veto}" }

            // 2. RangeCalendarDay with `child` preserving class, style, attributes, and focus/hover preview
            RangeCalendar {
                id: "s14-rc",
                default_placeholder: CalendarDate::new(2024, 1, 1).unwrap(),
                render_children: move |snap: RangeCalendarSnapshot| rsx! {
                    RangeCalendarGrid {
                        RangeCalendarGridBody {
                            for (r_idx, week) in snap.months[0].weeks.iter().enumerate() {
                                RangeCalendarGridRow {
                                    key: "{r_idx}",
                                    for date in week.iter().copied() {
                                        RangeCalendarCell {
                                            key: "{date}",
                                            date,
                                            month: snap.months[0].value,
                                            RangeCalendarDay {
                                                id: "s14-rc-day-{date}",
                                                class: "rc-day-cls",
                                                style: "min-width: 24px;",
                                                "data-day-extra": "preserved",
                                                child: move |props: Vec<Attribute>| rsx! {
                                                    button {
                                                        ..props,
                                                        "{date.day}"
                                                    }
                                                },
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
            }

            // 3. Unstyled DropdownMenu emits zero empty class="" or style=""
            DropdownMenu {
                default_open: true,
                modal: false,
                DropdownMenuTrigger { id: "s14-dm-trig", "Actions" }
                DropdownMenuContent {
                    id: "s14-dm-content",
                    DropdownMenuItem { id: "s14-dm-item", "Copy" }
                }
            }
        }
    }

    let mut harness = mount_native_scenario(scenario);

    // 1a. SelectItemIndicator: rendered when selected, omitted when unselected, force_mount renders unchecked.
    assert!(
        harness.query("#s14-sel-ind-apple").is_some(),
        "selected SelectItemIndicator must render"
    );
    assert_eq!(
        harness.attr("#s14-sel-ind-apple", "data-state").as_deref(),
        Some("checked")
    );
    assert!(
        harness.query("#s14-sel-ind-banana").is_none(),
        "unselected SelectItemIndicator without force_mount must omit DOM"
    );
    assert!(
        harness.query("#s14-sel-ind-cherry").is_some(),
        "force_mount SelectItemIndicator must render when unselected"
    );
    assert_eq!(
        harness.attr("#s14-sel-ind-cherry", "data-state").as_deref(),
        Some("unchecked")
    );

    // 1b. SelectGroup + SelectLabel auto-link aria-labelledby via context without group_key.
    assert_eq!(harness.attr("#s14-sel-grp", "role").as_deref(), Some("group"));
    assert_eq!(
        harness.attr("#s14-sel-grp", "aria-labelledby").as_deref(),
        Some("s14-sel-grp-lbl")
    );

    // 1c. SelectSeparator emits no inline #e2e8f0 style.
    assert!(harness.query("#s14-sel-sep").is_some());
    let sep_style = harness.attr("#s14-sel-sep", "style").unwrap_or_default();
    assert!(
        !sep_style.contains("#e2e8f0"),
        "SelectSeparator must not emit inline #e2e8f0 style: {sep_style}"
    );

    // 1d. SelectHiddenInput forwards GlobalAttributes.
    assert_eq!(
        harness.attr("#s14-sel-hidden", "data-hidden-extra").as_deref(),
        Some("yes")
    );

    // Close the initially-open Select before testing uncontrolled/controlled triggers.
    click_and_pump(&mut harness, "#s14-sel-open-trig");

    // 1e. Uncontrolled Select opens on trigger click; controlled Select veto stays closed.
    assert!(harness.query("#s14-sel-unctl-content").is_none());
    click_and_pump(&mut harness, "#s14-sel-unctl-trig");
    assert!(
        harness.query("#s14-sel-unctl-content").is_some(),
        "uncontrolled Select must open on trigger click"
    );
    click_and_pump(&mut harness, "#s14-sel-unctl-trig");

    assert!(harness.query("#s14-sel-ctl-content").is_none());
    click_and_pump(&mut harness, "#s14-sel-ctl-trig");
    assert_eq!(
        harness.attr("#s14-sel-ctl-count", "data-count").as_deref(),
        Some("1")
    );
    assert!(
        harness.query("#s14-sel-ctl-content").is_none(),
        "controlled Select open=Some(false) with vetoing on_open_change must stay closed"
    );

    // 2. RangeCalendarDay `child` preserves class, style, attributes, and hover/focus range preview.
    let day10_nid = harness.node("#s14-rc-day-2024-01-10");
    let day10_tag = harness
        .base()
        .get_node(day10_nid)
        .and_then(|n| n.element_data())
        .map(|el| el.name.local.to_string());
    assert_eq!(day10_tag.as_deref(), Some("button"));
    assert!(
        harness
            .attr("#s14-rc-day-2024-01-10", "class")
            .unwrap_or_default()
            .contains("rc-day-cls")
    );
    assert!(
        harness
            .attr("#s14-rc-day-2024-01-10", "style")
            .unwrap_or_default()
            .contains("min-width: 24px")
    );
    assert_eq!(
        harness
            .attr("#s14-rc-day-2024-01-10", "data-day-extra")
            .as_deref(),
        Some("preserved")
    );

    // Select start date Jan 10, then focus Jan 14 -> range preview highlights Jan 10..=14.
    click_and_pump(&mut harness, "#s14-rc-day-2024-01-10");
    assert_eq!(
        harness
            .attr("#s14-rc-day-2024-01-10", "data-selection-start")
            .as_deref(),
        Some("")
    );
    focus_node(&mut harness, "#s14-rc-day-2024-01-14");
    for day in 10..=14 {
        let sel = format!("#s14-rc-day-2024-01-{day:02}");
        assert_eq!(
            harness.attr(&sel, "data-highlighted").as_deref(),
            Some(""),
            "{sel} must carry data-highlighted during range preview"
        );
    }
    assert_eq!(
        harness.attr("#s14-rc-day-2024-01-15", "data-highlighted"),
        None
    );

    // 3. Unstyled DropdownMenu emits zero empty class="" or style="" attributes.
    click_and_pump(&mut harness, "#s14-dm-trig");
    for sel in ["#s14-dm-trig", "#s14-dm-content", "#s14-dm-item"] {
        assert!(harness.query(sel).is_some(), "{sel} must be mounted");
        assert_ne!(
            harness.attr(sel, "class").as_deref(),
            Some(""),
            "{sel} must not emit empty class"
        );
        assert_ne!(
            harness.attr(sel, "style").as_deref(),
            Some(""),
            "{sel} must not emit empty style"
        );
    }
}
