use dioxus::prelude::*;
use monoxus::{
    accordion::{
        Accordion, AccordionDirection, AccordionMode, AccordionOrientation,
        use_accordion_runtime,
    },
    collapsible::{Collapsible, use_collapsible_runtime},
    foundation::shared::ScopeHandle,
};

const CARD_STYLE: &str = "display: grid; gap: 1rem; padding: 1.25rem; border-radius: 0.75rem; border: 1px solid #93c5fd; background-color: white; box-shadow: 0 10px 30px rgba(30, 58, 138, 0.08);";
const MUTED_STYLE: &str = "margin: 0; color: #1e40af;";
const BADGE_STYLE: &str = "display: inline-block; padding: 0.2rem 0.6rem; border-radius: 9999px; font-size: 0.75rem; font-weight: 600; background-color: #dbeafe; color: #1e40af;";
const ITEM_CARD_STYLE: &str = "border: 1px solid #e2e8f0; border-radius: 0.5rem; overflow: hidden; margin-bottom: 0.5rem; background-color: #ffffff;";

#[component]
pub fn AccordionPlayground() -> Element {
    rsx! {
        section {
            style: CARD_STYLE,
            h2 {
                style: "margin: 0; color: #1e3a8a;",
                "Accordion and Collapsible Disclosure"
            }
            p {
                style: MUTED_STYLE,
                "Headless WAI-ARIA disclosure primitives with Single/Multiple modes, non-collapsible lock, roving focus navigation, and CSS Grid zero-measurement transitions."
            }

            StandaloneCollapsibleSection {}
            SingleCollapsibleSection {}
            SingleNonCollapsibleSection {}
            MultipleAccordionSection {}
            HorizontalRtlSection {}
        }
    }
}

// -------------------------------------------------------------------------
// Scenario 1: Standalone Collapsible
// -------------------------------------------------------------------------

#[component]
fn StandaloneCollapsibleSection() -> Element {
    let scope = ScopeHandle::root("playground").child("collapsible-standalone");
    let is_open_sig = use_signal(|| false);

    let def = Collapsible::new(scope.clone(), is_open_sig());
    let runtime = use_collapsible_runtime(
        def,
        Some(move |open| {
            let mut s = is_open_sig;
            s.set(open);
        }),
    );

    let trig_attrs = runtime.trigger();
    let content_attrs = runtime.content();
    let is_open = runtime.is_open();

    let runtime_click = runtime.clone();

    let state_label = if is_open { "Open" } else { "Closed" };

    rsx! {
        div {
            style: "display: grid; gap: 0.75rem; padding: 1rem; border: 1px solid #e2e8f0; border-radius: 0.5rem;",
            h3 { style: "margin: 0; font-size: 1rem; color: #1e293b;", "1. Standalone Collapsible Disclosure" }
            p { style: "margin: 0; font-size: 0.85rem; color: #64748b;",
                "Simple expandable panel using " code { "Collapsible" } " primitive with WAI-ARIA " code { "aria-expanded" } " and " code { "aria-controls" } "."
            }
            div {
                span { style: BADGE_STYLE, "State: {state_label}" }
            }

            button {
                id: trig_attrs.id(),
                r#type: "button",
                aria_expanded: trig_attrs.aria_expanded(),
                aria_controls: trig_attrs.aria_controls(),
                "data-state": trig_attrs.data_state_str(),
                style: "display: flex; justify-content: space-between; align-items: center; width: 100%; max-width: 20rem; padding: 0.6rem 1rem; background-color: #2563eb; color: white; border: none; border-radius: 0.375rem; font-weight: 500; cursor: pointer;",
                onclick: move |_| {
                    runtime_click.toggle();
                },
                span { "Toggle Release Details" }
                span { style: "font-size: 0.8rem;", if is_open { "▲" } else { "▼" } }
            }

            div {
                id: content_attrs.id(),
                hidden: content_attrs.is_hidden(),
                "data-state": content_attrs.data_state_str(),
                style: "max-width: 20rem; padding: 0.75rem 1rem; background-color: #f1f5f9; border-radius: 0.375rem; border: 1px solid #cbd5e1; font-size: 0.875rem; color: #334155;",
                p { style: "margin: 0 0 0.5rem 0; font-weight: 600;", "Monoxus v0.1.0 Release Notes" }
                p { style: "margin: 0;", "Includes Accordion, Collapsible, Tabs, Dialog, Alert Dialog, Popover, and Tooltip primitives with headless WAI-ARIA semantics." }
            }
        }
    }
}

// -------------------------------------------------------------------------
// Scenario 2: Accordion Single Collapsible Mode
// -------------------------------------------------------------------------

#[component]
fn SingleCollapsibleSection() -> Element {
    let scope = ScopeHandle::root("playground").child("accordion-single-collapsible");
    let active_sig = use_signal(|| vec!["item-1".to_string()]);

    let def = Accordion::new(scope.clone(), AccordionMode::Single { collapsible: true })
        .with_values(active_sig());

    let runtime = use_accordion_runtime(
        def,
        Some(move |vals| {
            let mut s = active_sig;
            s.set(vals);
        }),
    );

    use_effect(use_reactive((), {
        let runtime = runtime.clone();
        move |_| {
            runtime.register_item("item-1", false);
            runtime.register_item("item-2", false);
            runtime.register_item("item-3", false);
        }
    }));

    let items = [
        ("item-1", "What is Monoxus?", "Monoxus is a high-performance, headless UI library for Dioxus web applications."),
        ("item-2", "Is it accessible?", "Yes, all primitives strictly adhere to the WAI-ARIA 1.2 authoring guidelines."),
        ("item-3", "Can it collapse completely?", "Yes, in Single Collapsible mode, clicking the open item collapses it."),
    ];

    let active_desc = format!("{:?}", runtime.active_values());

    rsx! {
        div {
            style: "display: grid; gap: 0.75rem; padding: 1rem; border: 1px solid #e2e8f0; border-radius: 0.5rem;",
            h3 { style: "margin: 0; font-size: 1rem; color: #1e293b;", "2. Accordion Single Mode (Collapsible: true)" }
            p { style: "margin: 0; font-size: 0.85rem; color: #64748b;",
                "Allows only one item open at a time, but clicking the currently open item collapses it. Uses roving focus navigation."
            }
            div {
                span { style: BADGE_STYLE, "Active: {active_desc}" }
                " "
                span { style: BADGE_STYLE, "Tab Stop: {runtime.current_tab_stop()}" }
            }

            div {
                id: runtime.root().id(),
                "data-orientation": runtime.root().data_orientation(),
                style: "max-width: 32rem;",
                for (val, title, body) in items {
                    {
                        let item_attrs = runtime.item(val, false);
                        let header_attrs = runtime.header(val, false);
                        let trig_attrs = runtime.trigger(val, false);
                        let content_attrs = runtime.content(val);
                        let is_open = trig_attrs.is_open();

                        let runtime_click = runtime.clone();
                        let runtime_key = runtime.clone();
                        let val_str = val.to_string();

                        rsx! {
                            div {
                                id: item_attrs.id(),
                                "data-state": item_attrs.data_state_str(),
                                "data-orientation": item_attrs.data_orientation(),
                                "data-value": item_attrs.data_value(),
                                style: ITEM_CARD_STYLE,

                                h3 {
                                    id: header_attrs.id(),
                                    role: header_attrs.role(),
                                    "aria-level": header_attrs.aria_level(),
                                    "data-heading-level": header_attrs.data_heading_level(),
                                    "data-state": header_attrs.data_state_str(),
                                    style: "margin: 0; padding: 0;",

                                    button {
                                        id: trig_attrs.id(),
                                        r#type: "button",
                                        aria_expanded: trig_attrs.aria_expanded(),
                                        aria_controls: trig_attrs.aria_controls(),
                                        tabindex: trig_attrs.tabindex(),
                                        "data-state": trig_attrs.data_state_str(),
                                        "data-value": trig_attrs.data_value(),
                                        style: "display: flex; justify-content: space-between; align-items: center; width: 100%; padding: 0.75rem 1rem; background-color: #f8fafc; border: none; font-size: 0.95rem; font-weight: 600; color: #1e293b; cursor: pointer; text-align: left;",
                                        onclick: move |_| {
                                            runtime_click.toggle_item(&val_str);
                                        },
                                        onkeydown: move |evt: KeyboardEvent| {
                                            let k = evt.key().to_string();
                                            if Accordion::is_navigation_key(&k) {
                                                evt.prevent_default();
                                                runtime_key.navigate_key(&k);
                                            }
                                        },
                                        span { "{title}" }
                                        span { style: "font-size: 0.8rem; color: #64748b;", if is_open { "▲" } else { "▼" } }
                                    }
                                }

                                div {
                                    id: content_attrs.id(),
                                    role: content_attrs.role(),
                                    "aria-labelledby": content_attrs.aria_labelledby(),
                                    hidden: content_attrs.is_hidden(),
                                    "data-state": content_attrs.data_state_str(),
                                    style: "padding: 0.75rem 1rem; border-top: 1px solid #e2e8f0; font-size: 0.875rem; color: #475569; background-color: #ffffff;",
                                    p { style: "margin: 0;", "{body}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// -------------------------------------------------------------------------
// Scenario 3: Accordion Single Non-Collapsible Mode
// -------------------------------------------------------------------------

#[component]
fn SingleNonCollapsibleSection() -> Element {
    let scope = ScopeHandle::root("playground").child("accordion-single-non-collapsible");
    let active_sig = use_signal(|| vec!["step-1".to_string()]);

    let def = Accordion::new(scope.clone(), AccordionMode::Single { collapsible: false })
        .with_values(active_sig());

    let runtime = use_accordion_runtime(
        def,
        Some(move |vals| {
            let mut s = active_sig;
            s.set(vals);
        }),
    );

    use_effect(use_reactive((), {
        let runtime = runtime.clone();
        move |_| {
            runtime.register_item("step-1", false);
            runtime.register_item("step-2", false);
            runtime.register_item("step-3", false);
        }
    }));

    let items = [
        ("step-1", "Step 1: Account Setup", "Configure your organization credentials and profile settings."),
        ("step-2", "Step 2: Billing & Plans", "Select an enterprise subscription tier and payment method."),
        ("step-3", "Step 3: Verification", "Review your submitted information and confirm activation."),
    ];

    let active_desc = format!("{:?}", runtime.active_values());

    rsx! {
        div {
            style: "display: grid; gap: 0.75rem; padding: 1rem; border: 1px solid #e2e8f0; border-radius: 0.5rem;",
            h3 { style: "margin: 0; font-size: 1rem; color: #1e293b;", "3. Accordion Single Mode (Collapsible: false)" }
            p { style: "margin: 0; font-size: 0.85rem; color: #64748b;",
                "Enforces at least one open item. The currently open item cannot be closed (click is a no-op) and receives " code { "aria-disabled=\"true\"" } "."
            }
            div {
                span { style: BADGE_STYLE, "Active: {active_desc}" }
                " "
                span { style: BADGE_STYLE, "Tab Stop: {runtime.current_tab_stop()}" }
            }

            div {
                id: runtime.root().id(),
                "data-orientation": runtime.root().data_orientation(),
                style: "max-width: 32rem;",
                for (val, title, body) in items {
                    {
                        let item_attrs = runtime.item(val, false);
                        let header_attrs = runtime.header(val, false);
                        let trig_attrs = runtime.trigger(val, false);
                        let content_attrs = runtime.content(val);
                        let is_open = trig_attrs.is_open();
                        let aria_dis = trig_attrs.aria_disabled();

                        let runtime_click = runtime.clone();
                        let runtime_key = runtime.clone();
                        let val_str = val.to_string();

                        rsx! {
                            div {
                                id: item_attrs.id(),
                                "data-state": item_attrs.data_state_str(),
                                "data-orientation": item_attrs.data_orientation(),
                                "data-value": item_attrs.data_value(),
                                style: ITEM_CARD_STYLE,

                                h3 {
                                    id: header_attrs.id(),
                                    role: header_attrs.role(),
                                    "aria-level": header_attrs.aria_level(),
                                    "data-heading-level": header_attrs.data_heading_level(),
                                    "data-state": header_attrs.data_state_str(),
                                    style: "margin: 0; padding: 0;",

                                    button {
                                        id: trig_attrs.id(),
                                        r#type: "button",
                                        aria_expanded: trig_attrs.aria_expanded(),
                                        aria_controls: trig_attrs.aria_controls(),
                                        aria_disabled: aria_dis.unwrap_or_default(),
                                        tabindex: trig_attrs.tabindex(),
                                        "data-state": trig_attrs.data_state_str(),
                                        "data-value": trig_attrs.data_value(),
                                        style: "display: flex; justify-content: space-between; align-items: center; width: 100%; padding: 0.75rem 1rem; background-color: #f8fafc; border: none; font-size: 0.95rem; font-weight: 600; color: #1e293b; cursor: pointer; text-align: left;",
                                        onclick: move |_| {
                                            runtime_click.toggle_item(&val_str);
                                        },
                                        onkeydown: move |evt: KeyboardEvent| {
                                            let k = evt.key().to_string();
                                            if Accordion::is_navigation_key(&k) {
                                                evt.prevent_default();
                                                runtime_key.navigate_key(&k);
                                            }
                                        },
                                        span { "{title}" }
                                        div {
                                            style: "display: flex; align-items: center; gap: 0.5rem;",
                                            if let Some(_) = aria_dis {
                                                span { style: "font-size: 0.7rem; color: #94a3b8; font-weight: 400;", "(locked open)" }
                                            }
                                            span { style: "font-size: 0.8rem; color: #64748b;", if is_open { "▲" } else { "▼" } }
                                        }
                                    }
                                }

                                div {
                                    id: content_attrs.id(),
                                    role: content_attrs.role(),
                                    "aria-labelledby": content_attrs.aria_labelledby(),
                                    hidden: content_attrs.is_hidden(),
                                    "data-state": content_attrs.data_state_str(),
                                    style: "padding: 0.75rem 1rem; border-top: 1px solid #e2e8f0; font-size: 0.875rem; color: #475569; background-color: #ffffff;",
                                    p { style: "margin: 0;", "{body}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// -------------------------------------------------------------------------
// Scenario 4: Accordion Multiple Selection Mode
// -------------------------------------------------------------------------

#[component]
fn MultipleAccordionSection() -> Element {
    let scope = ScopeHandle::root("playground").child("accordion-multiple");
    let active_sig = use_signal(|| vec!["notif-email".to_string(), "notif-sms".to_string()]);

    let def = Accordion::new(scope.clone(), AccordionMode::Multiple)
        .with_values(active_sig());

    let runtime = use_accordion_runtime(
        def,
        Some(move |vals| {
            let mut s = active_sig;
            s.set(vals);
        }),
    );

    use_effect(use_reactive((), {
        let runtime = runtime.clone();
        move |_| {
            runtime.register_item("notif-email", false);
            runtime.register_item("notif-sms", false);
            runtime.register_item("notif-push", false);
        }
    }));

    let items = [
        ("notif-email", "Email Notifications", "Receive security alerts, system digests, and invoices via your primary inbox."),
        ("notif-sms", "SMS Critical Alerts", "Get real-time SMS broadcasts for urgent system incidents and 2FA logins."),
        ("notif-push", "Push Notifications", "In-app notifications in your browser tab when critical events occur."),
    ];

    let active_desc = format!("{:?}", runtime.active_values());

    rsx! {
        div {
            style: "display: grid; gap: 0.75rem; padding: 1rem; border: 1px solid #e2e8f0; border-radius: 0.5rem;",
            h3 { style: "margin: 0; font-size: 1rem; color: #1e293b;", "4. Accordion Multiple Selection Mode" }
            p { style: "margin: 0; font-size: 0.85rem; color: #64748b;",
                "Allows multiple panels to be open simultaneously. Toggling one panel does not affect the disclosure state of others."
            }
            div {
                span { style: BADGE_STYLE, "Active: {active_desc}" }
                " "
                span { style: BADGE_STYLE, "Tab Stop: {runtime.current_tab_stop()}" }
            }

            div {
                id: runtime.root().id(),
                "data-orientation": runtime.root().data_orientation(),
                style: "max-width: 32rem;",
                for (val, title, body) in items {
                    {
                        let item_attrs = runtime.item(val, false);
                        let header_attrs = runtime.header(val, false);
                        let trig_attrs = runtime.trigger(val, false);
                        let content_attrs = runtime.content(val);
                        let is_open = trig_attrs.is_open();

                        let runtime_click = runtime.clone();
                        let runtime_key = runtime.clone();
                        let val_str = val.to_string();

                        rsx! {
                            div {
                                id: item_attrs.id(),
                                "data-state": item_attrs.data_state_str(),
                                "data-orientation": item_attrs.data_orientation(),
                                "data-value": item_attrs.data_value(),
                                style: ITEM_CARD_STYLE,

                                h3 {
                                    id: header_attrs.id(),
                                    role: header_attrs.role(),
                                    "aria-level": header_attrs.aria_level(),
                                    "data-heading-level": header_attrs.data_heading_level(),
                                    "data-state": header_attrs.data_state_str(),
                                    style: "margin: 0; padding: 0;",

                                    button {
                                        id: trig_attrs.id(),
                                        r#type: "button",
                                        aria_expanded: trig_attrs.aria_expanded(),
                                        aria_controls: trig_attrs.aria_controls(),
                                        tabindex: trig_attrs.tabindex(),
                                        "data-state": trig_attrs.data_state_str(),
                                        "data-value": trig_attrs.data_value(),
                                        style: "display: flex; justify-content: space-between; align-items: center; width: 100%; padding: 0.75rem 1rem; background-color: #f8fafc; border: none; font-size: 0.95rem; font-weight: 600; color: #1e293b; cursor: pointer; text-align: left;",
                                        onclick: move |_| {
                                            runtime_click.toggle_item(&val_str);
                                        },
                                        onkeydown: move |evt: KeyboardEvent| {
                                            let k = evt.key().to_string();
                                            if Accordion::is_navigation_key(&k) {
                                                evt.prevent_default();
                                                runtime_key.navigate_key(&k);
                                            }
                                        },
                                        span { "{title}" }
                                        span { style: "font-size: 0.8rem; color: #64748b;", if is_open { "▲" } else { "▼" } }
                                    }
                                }

                                div {
                                    id: content_attrs.id(),
                                    role: content_attrs.role(),
                                    "aria-labelledby": content_attrs.aria_labelledby(),
                                    hidden: content_attrs.is_hidden(),
                                    "data-state": content_attrs.data_state_str(),
                                    style: "padding: 0.75rem 1rem; border-top: 1px solid #e2e8f0; font-size: 0.875rem; color: #475569; background-color: #ffffff;",
                                    p { style: "margin: 0;", "{body}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// -------------------------------------------------------------------------
// Scenario 5: Horizontal Accordion with RTL and Disabled Item
// -------------------------------------------------------------------------

#[component]
fn HorizontalRtlSection() -> Element {
    let scope = ScopeHandle::root("playground").child("accordion-horizontal-rtl");
    let active_sig = use_signal(|| vec!["panel-1".to_string()]);

    let def = Accordion::new(scope.clone(), AccordionMode::Single { collapsible: true })
        .with_orientation(AccordionOrientation::Horizontal)
        .with_direction(AccordionDirection::Rtl)
        .with_values(active_sig());

    let runtime = use_accordion_runtime(
        def,
        Some(move |vals| {
            let mut s = active_sig;
            s.set(vals);
        }),
    );

    use_effect(use_reactive((), {
        let runtime = runtime.clone();
        move |_| {
            runtime.register_item("panel-1", false);
            runtime.register_item("panel-2", true); // disabled!
            runtime.register_item("panel-3", false);
        }
    }));

    let items = [
        ("panel-1", "لوحة 1", "محتوى اللوحة الأولى (Panel 1 content in RTL)", false),
        ("panel-2", "لوحة 2 (معطلة)", "هذه اللوحة معطلة (Disabled Panel 2)", true),
        ("panel-3", "لوحة 3", "محتوى اللوحة الثالثة (Panel 3 content in RTL)", false),
    ];

    let active_desc = format!("{:?}", runtime.active_values());

    rsx! {
        div {
            style: "display: grid; gap: 0.75rem; padding: 1rem; border: 1px solid #e2e8f0; border-radius: 0.5rem;",
            h3 { style: "margin: 0; font-size: 1rem; color: #1e293b;", "5. Horizontal RTL Accordion with Disabled Item" }
            p { style: "margin: 0; font-size: 0.85rem; color: #64748b;",
                "Horizontal layout with " code { "dir=\"rtl\"" } ". Keyboard arrow navigation swaps Left/Right directions and skips the disabled panel."
            }
            div {
                span { style: BADGE_STYLE, "Active: {active_desc}" }
                " "
                span { style: BADGE_STYLE, "Tab Stop: {runtime.current_tab_stop()}" }
            }

            div {
                id: runtime.root().id(),
                dir: "rtl",
                "data-orientation": runtime.root().data_orientation(),
                style: "display: flex; gap: 0.5rem; max-width: 40rem; border: 1px solid #e2e8f0; padding: 0.75rem; border-radius: 0.5rem; background-color: #f8fafc;",
                for (val, title, body, disabled) in items {
                    {
                        let item_attrs = runtime.item(val, disabled);
                        let header_attrs = runtime.header(val, disabled);
                        let trig_attrs = runtime.trigger(val, disabled);
                        let content_attrs = runtime.content(val);
                        let is_open = trig_attrs.is_open();
                        let cursor_style = if disabled { "not-allowed" } else { "pointer" };

                        let runtime_click = runtime.clone();
                        let runtime_key = runtime.clone();
                        let val_str = val.to_string();

                        let bg = if disabled { "#f1f5f9" } else if is_open { "#dbeafe" } else { "#ffffff" };
                        let fg = if disabled { "#94a3b8" } else if is_open { "#1e40af" } else { "#1e293b" };

                        rsx! {
                            div {
                                id: item_attrs.id(),
                                "data-state": item_attrs.data_state_str(),
                                "data-orientation": item_attrs.data_orientation(),
                                "data-value": item_attrs.data_value(),
                                style: "flex: 1; border: 1px solid #cbd5e1; border-radius: 0.375rem; overflow: hidden; background-color: {bg};",

                                h3 {
                                    id: header_attrs.id(),
                                    role: header_attrs.role(),
                                    "aria-level": header_attrs.aria_level(),
                                    "data-heading-level": header_attrs.data_heading_level(),
                                    "data-state": header_attrs.data_state_str(),
                                    style: "margin: 0; padding: 0;",

                                    button {
                                        id: trig_attrs.id(),
                                        r#type: "button",
                                        aria_expanded: trig_attrs.aria_expanded(),
                                        aria_controls: trig_attrs.aria_controls(),
                                        tabindex: trig_attrs.tabindex(),
                                        disabled: trig_attrs.is_disabled(),
                                        "data-state": trig_attrs.data_state_str(),
                                        "data-value": trig_attrs.data_value(),
                                        style: "width: 100%; padding: 0.5rem; border: none; font-size: 0.875rem; font-weight: 600; color: {fg}; background: transparent; cursor: {cursor_style}; text-align: center;",
                                        onclick: move |_| {
                                            if !disabled {
                                                runtime_click.toggle_item(&val_str);
                                            }
                                        },
                                        onkeydown: move |evt: KeyboardEvent| {
                                            let k = evt.key().to_string();
                                            if Accordion::is_navigation_key(&k) {
                                                evt.prevent_default();
                                                runtime_key.navigate_key(&k);
                                            }
                                        },
                                        span { "{title}" }
                                    }
                                }

                                div {
                                    id: content_attrs.id(),
                                    role: content_attrs.role(),
                                    "aria-labelledby": content_attrs.aria_labelledby(),
                                    hidden: content_attrs.is_hidden(),
                                    "data-state": content_attrs.data_state_str(),
                                    style: "padding: 0.5rem; border-top: 1px solid #cbd5e1; font-size: 0.8rem; color: #334155; background-color: #ffffff;",
                                    p { style: "margin: 0;", "{body}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
