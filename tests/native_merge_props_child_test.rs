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
