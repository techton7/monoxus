use dioxus::prelude::*;
use monoxus::{
    foundation::shared::ScopeHandle,
    tabs::{Tabs, TabsActivationMode, TabsDirection, TabsOrientation, use_tabs_runtime},
};

const CARD_STYLE: &str = "display: grid; gap: 1rem; padding: 1.25rem; border-radius: 0.75rem; border: 1px solid #93c5fd; background-color: white; box-shadow: 0 10px 30px rgba(30, 58, 138, 0.08);";
const MUTED_STYLE: &str = "margin: 0; color: #1e40af;";
const TAB_LIST_HORIZONTAL: &str =
    "display: flex; gap: 0.5rem; border-bottom: 2px solid #e2e8f0; padding-bottom: 0.25rem;";
const TAB_LIST_VERTICAL: &str = "display: flex; flex-direction: column; gap: 0.5rem; width: 12rem; border-right: 2px solid #e2e8f0; padding-right: 0.75rem;";
const TAB_PANEL_STYLE: &str = "padding: 1.25rem; background-color: #f8fafc; border-radius: 0.5rem; border: 1px solid #e2e8f0; margin-top: 0.5rem;";
const BADGE_STYLE: &str = "display: inline-block; padding: 0.2rem 0.6rem; border-radius: 9999px; font-size: 0.75rem; font-weight: 600; background-color: #dbeafe; color: #1e40af;";

#[component]
pub fn TabsPlayground() -> Element {
    rsx! {
        section {
            style: CARD_STYLE,
            h2 {
                style: "margin: 0; color: #1e3a8a;",
                "Tabs and Selection Navigation"
            }
            p {
                style: MUTED_STYLE,
                "Headless WAI-ARIA Tabs primitives with roving tabindex, direction-aware arrow navigation, manual/automatic activation modes, and zero-mock runtime proof."
            }

            HorizontalAutomaticSection {}
            VerticalManualSection {}
            RtlSection {}
            DisabledTabsSection {}
            DescendantInputRegressionSection {}
        }
    }
}

#[component]
fn HorizontalAutomaticSection() -> Element {
    let selected = use_signal(|| "account".to_string());
    let scope = ScopeHandle::root("playground").child("tabs-horizontal");

    let tabs_def = Tabs::new(scope.clone(), selected())
        .with_orientation(TabsOrientation::Horizontal)
        .with_direction(TabsDirection::Ltr)
        .with_activation_mode(TabsActivationMode::Automatic);

    let runtime = use_tabs_runtime(
        tabs_def,
        Some(move |new_val| {
            let mut selected = selected;
            selected.set(new_val);
        }),
    );

    use_effect(use_reactive((), {
        let runtime = runtime.clone();
        move |_| {
            runtime.register_trigger("account", false);
            runtime.register_trigger("password", false);
            runtime.register_trigger("settings", false);
        }
    }));

    let list_attrs = runtime.list();

    rsx! {
        div {
            style: "display: grid; gap: 0.75rem; padding: 1rem; border: 1px solid #e2e8f0; border-radius: 0.5rem;",
            h3 { style: "margin: 0; font-size: 1rem; color: #1e293b;", "1. Horizontal Tabs (Automatic Activation)" }
            p { style: "margin: 0; font-size: 0.85rem; color: #64748b;",
                "Navigate with " code { "ArrowLeft" } "/" code { "ArrowRight" } ", " code { "Home" } ", " code { "End" } ". Selection changes immediately on focus."
            }
            div {
                span { style: BADGE_STYLE, "Active: {runtime.active_value()}" }
                " "
                span { style: BADGE_STYLE, "Tab Stop: {runtime.current_tab_stop()}" }
            }

            div {
                id: list_attrs.id(),
                role: list_attrs.role(),
                aria_orientation: list_attrs.aria_orientation(),
                "data-orientation": list_attrs.data_orientation(),
                style: TAB_LIST_HORIZONTAL,
                tabindex: "-1",
                onkeydown: {
                    let runtime = runtime.clone();
                    move |evt: KeyboardEvent| {
                        if Tabs::is_navigation_key(&evt.key().to_string()) {
                            evt.prevent_default();
                            runtime.navigate_key(&evt.key().to_string());
                        }
                    }
                },

                for val in ["account", "password", "settings"] {
                    {
                        let trig_attrs = runtime.trigger(val, false);
                        let is_active = trig_attrs.is_selected();
                        let bg = if is_active { "#2563eb" } else { "#f1f5f9" };
                        let fg = if is_active { "#ffffff" } else { "#334155" };
                        let runtime_click = runtime.clone();
                        let runtime_down = runtime.clone();
                        let val_str = val.to_string();
                        let val_down = val.to_string();

                        rsx! {
                            button {
                                id: trig_attrs.id(),
                                role: trig_attrs.role(),
                                aria_selected: trig_attrs.aria_selected(),
                                aria_controls: trig_attrs.aria_controls(),
                                tabindex: "{trig_attrs.tabindex()}",
                                "data-state": trig_attrs.data_state_str(),
                                "data-orientation": trig_attrs.data_orientation(),
                                "data-value": trig_attrs.data_value(),
                                style: "padding: 0.5rem 1rem; border-radius: 0.375rem; border: none; font-weight: 500; cursor: pointer; background-color: {bg}; color: {fg}; transition: all 0.15s ease;",
                                onmousedown: move |_| {
                                    runtime_down.select_tab(&val_down);
                                },
                                onclick: move |_| {
                                    runtime_click.select_tab(&val_str);
                                },
                                "{val}"
                            }
                        }
                    }
                }
            }

            for val in ["account", "password", "settings"] {
                {
                    let content_attrs = runtime.content(val);
                    rsx! {
                        div {
                            id: content_attrs.id(),
                            role: content_attrs.role(),
                            aria_labelledby: content_attrs.aria_labelledby(),
                            tabindex: "{content_attrs.tabindex()}",
                            hidden: content_attrs.is_hidden(),
                            "data-state": content_attrs.data_state_str(),
                            "data-orientation": content_attrs.data_orientation(),
                            "data-value": content_attrs.data_value(),
                            style: TAB_PANEL_STYLE,
                            p { style: "margin: 0; font-weight: 600;", "Panel content for {val}" }
                            p { style: "margin: 0.25rem 0 0; color: #64748b;", "Role: tabpanel, tabindex: 0, hidden: {content_attrs.is_hidden()}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn VerticalManualSection() -> Element {
    let selected = use_signal(|| "profile".to_string());
    let scope = ScopeHandle::root("playground").child("tabs-vertical");

    let tabs_def = Tabs::new(scope.clone(), selected())
        .with_orientation(TabsOrientation::Vertical)
        .with_activation_mode(TabsActivationMode::Manual);

    let runtime = use_tabs_runtime(
        tabs_def,
        Some(move |new_val| {
            let mut selected = selected;
            selected.set(new_val);
        }),
    );

    use_effect(use_reactive((), {
        let runtime = runtime.clone();
        move |_| {
            runtime.register_trigger("profile", false);
            runtime.register_trigger("notifications", false);
            runtime.register_trigger("billing", false);
        }
    }));

    let list_attrs = runtime.list();

    rsx! {
        div {
            style: "display: grid; gap: 0.75rem; padding: 1rem; border: 1px solid #e2e8f0; border-radius: 0.5rem;",
            h3 { style: "margin: 0; font-size: 1rem; color: #1e293b;", "2. Vertical Tabs (Manual Activation)" }
            p { style: "margin: 0; font-size: 0.85rem; color: #64748b;",
                "Navigate with " code { "ArrowUp" } "/" code { "ArrowDown" } ". Focus & roving tabindex change, but selection requires " code { "Space" } " or " code { "Enter" } "."
            }
            div {
                span { style: BADGE_STYLE, "Active (Selected): {runtime.active_value()}" }
                " "
                span { style: BADGE_STYLE, "Focused Tab Stop: {runtime.current_tab_stop()}" }
            }

            div {
                style: "display: flex; gap: 1.5rem;",
                div {
                    id: list_attrs.id(),
                    role: list_attrs.role(),
                    aria_orientation: list_attrs.aria_orientation(),
                    "data-orientation": list_attrs.data_orientation(),
                    style: TAB_LIST_VERTICAL,
                    tabindex: "-1",
                    onkeydown: {
                        let runtime = runtime.clone();
                        move |evt: KeyboardEvent| {
                            if Tabs::is_navigation_key(&evt.key().to_string()) {
                                evt.prevent_default();
                                runtime.navigate_key(&evt.key().to_string());
                            }
                        }
                    },

                    for val in ["profile", "notifications", "billing"] {
                        {
                            let trig_attrs = runtime.trigger(val, false);
                            let is_active = trig_attrs.is_selected();
                            let bg = if is_active { "#7c3aed" } else { "#f1f5f9" };
                            let fg = if is_active { "#ffffff" } else { "#334155" };
                            let runtime_click = runtime.clone();
                            let runtime_key = runtime.clone();
                            let val_str = val.to_string();
                            let val_key = val.to_string();

                            rsx! {
                                button {
                                    id: trig_attrs.id(),
                                    role: trig_attrs.role(),
                                    aria_selected: trig_attrs.aria_selected(),
                                    aria_controls: trig_attrs.aria_controls(),
                                    tabindex: "{trig_attrs.tabindex()}",
                                    "data-state": trig_attrs.data_state_str(),
                                    "data-orientation": trig_attrs.data_orientation(),
                                    "data-value": trig_attrs.data_value(),
                                    style: "padding: 0.5rem 1rem; border-radius: 0.375rem; border: none; text-align: left; font-weight: 500; cursor: pointer; background-color: {bg}; color: {fg}; transition: all 0.15s ease;",
                                    onclick: move |_| {
                                        runtime_click.select_tab(&val_str);
                                    },
                                    onkeydown: move |evt: KeyboardEvent| {
                                        let key = evt.key().to_string();
                                        if key == " " || key == "Enter" {
                                            evt.prevent_default();
                                            runtime_key.select_tab(&val_key);
                                        }
                                    },
                                    "{val}"
                                }
                            }
                        }
                    }
                }

                div {
                    style: "flex: 1;",
                    for val in ["profile", "notifications", "billing"] {
                        {
                            let content_attrs = runtime.content(val);
                            rsx! {
                                div {
                                    id: content_attrs.id(),
                                    role: content_attrs.role(),
                                    aria_labelledby: content_attrs.aria_labelledby(),
                                    tabindex: "{content_attrs.tabindex()}",
                                    hidden: content_attrs.is_hidden(),
                                    "data-state": content_attrs.data_state_str(),
                                    "data-orientation": content_attrs.data_orientation(),
                                    "data-value": content_attrs.data_value(),
                                    style: TAB_PANEL_STYLE,
                                    p { style: "margin: 0; font-weight: 600;", "Vertical Panel: {val}" }
                                    p { style: "margin: 0.25rem 0 0; color: #64748b;", "Current active tab is {runtime.active_value()}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn RtlSection() -> Element {
    let selected = use_signal(|| "overview".to_string());
    let scope = ScopeHandle::root("playground").child("tabs-rtl");

    let tabs_def = Tabs::new(scope.clone(), selected())
        .with_orientation(TabsOrientation::Horizontal)
        .with_direction(TabsDirection::Rtl);

    let runtime = use_tabs_runtime(
        tabs_def,
        Some(move |new_val| {
            let mut selected = selected;
            selected.set(new_val);
        }),
    );

    use_effect(use_reactive((), {
        let runtime = runtime.clone();
        move |_| {
            runtime.register_trigger("overview", false);
            runtime.register_trigger("billing", false);
            runtime.register_trigger("support", false);
        }
    }));

    let list_attrs = runtime.list();

    rsx! {
        div {
            dir: "rtl",
            style: "display: grid; gap: 0.75rem; padding: 1rem; border: 1px solid #e2e8f0; border-radius: 0.5rem;",
            h3 { style: "margin: 0; font-size: 1rem; color: #1e293b;", "3. RTL Direction Tabs" }
            p { style: "margin: 0; font-size: 0.85rem; color: #64748b;",
                "In RTL mode, " code { "ArrowLeft" } " moves forward, and " code { "ArrowRight" } " moves backward."
            }

            div {
                id: list_attrs.id(),
                role: list_attrs.role(),
                aria_orientation: list_attrs.aria_orientation(),
                "data-orientation": list_attrs.data_orientation(),
                style: TAB_LIST_HORIZONTAL,
                tabindex: "-1",
                onkeydown: {
                    let runtime = runtime.clone();
                    move |evt: KeyboardEvent| {
                        if Tabs::is_navigation_key(&evt.key().to_string()) {
                            evt.prevent_default();
                            runtime.navigate_key(&evt.key().to_string());
                        }
                    }
                },

                for (val, label) in [("overview", "نظرة عامة"), ("billing", "الفواتير"), ("support", "الدعم")] {
                    {
                        let trig_attrs = runtime.trigger(val, false);
                        let is_active = trig_attrs.is_selected();
                        let bg = if is_active { "#059669" } else { "#f1f5f9" };
                        let fg = if is_active { "#ffffff" } else { "#334155" };
                        let runtime_click = runtime.clone();
                        let val_str = val.to_string();

                        rsx! {
                            button {
                                id: trig_attrs.id(),
                                role: trig_attrs.role(),
                                aria_selected: trig_attrs.aria_selected(),
                                aria_controls: trig_attrs.aria_controls(),
                                tabindex: "{trig_attrs.tabindex()}",
                                "data-state": trig_attrs.data_state_str(),
                                "data-orientation": trig_attrs.data_orientation(),
                                "data-value": trig_attrs.data_value(),
                                style: "padding: 0.5rem 1rem; border-radius: 0.375rem; border: none; font-weight: 500; cursor: pointer; background-color: {bg}; color: {fg};",
                                onclick: move |_| {
                                    runtime_click.select_tab(&val_str);
                                },
                                "{label}"
                            }
                        }
                    }
                }
            }

            for val in ["overview", "billing", "support"] {
                {
                    let content_attrs = runtime.content(val);
                    rsx! {
                        div {
                            id: content_attrs.id(),
                            role: content_attrs.role(),
                            aria_labelledby: content_attrs.aria_labelledby(),
                            tabindex: "{content_attrs.tabindex()}",
                            hidden: content_attrs.is_hidden(),
                            "data-state": content_attrs.data_state_str(),
                            "data-orientation": content_attrs.data_orientation(),
                            "data-value": content_attrs.data_value(),
                            style: TAB_PANEL_STYLE,
                            p { style: "margin: 0; font-weight: 600;", "RTL Panel: {val}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn DisabledTabsSection() -> Element {
    let selected = use_signal(|| "tab1".to_string());
    let scope = ScopeHandle::root("playground").child("tabs-disabled");

    let tabs_def = Tabs::new(scope.clone(), selected());
    let runtime = use_tabs_runtime(
        tabs_def,
        Some(move |new_val| {
            let mut selected = selected;
            selected.set(new_val);
        }),
    );

    use_effect(use_reactive((), {
        let runtime = runtime.clone();
        move |_| {
            runtime.register_trigger("tab1", false);
            runtime.register_trigger("tab2", true); // disabled!
            runtime.register_trigger("tab3", false);
        }
    }));

    let list_attrs = runtime.list();

    rsx! {
        div {
            style: "display: grid; gap: 0.75rem; padding: 1rem; border: 1px solid #e2e8f0; border-radius: 0.5rem;",
            h3 { style: "margin: 0; font-size: 1rem; color: #1e293b;", "4. Disabled Tab Navigation Skipping" }
            p { style: "margin: 0; font-size: 0.85rem; color: #64748b;",
                "Tab 2 is disabled. Pressing " code { "ArrowRight" } " on Tab 1 immediately jumps over Tab 2 to Tab 3."
            }

            div {
                id: list_attrs.id(),
                role: list_attrs.role(),
                aria_orientation: list_attrs.aria_orientation(),
                "data-orientation": list_attrs.data_orientation(),
                style: TAB_LIST_HORIZONTAL,
                tabindex: "-1",
                onkeydown: {
                    let runtime = runtime.clone();
                    move |evt: KeyboardEvent| {
                        if Tabs::is_navigation_key(&evt.key().to_string()) {
                            evt.prevent_default();
                            runtime.navigate_key(&evt.key().to_string());
                        }
                    }
                },

                for (val, is_dis) in [("tab1", false), ("tab2", true), ("tab3", false)] {
                    {
                        let trig_attrs = runtime.trigger(val, is_dis);
                        let is_active = trig_attrs.is_selected();
                        let bg = if is_dis { "#e2e8f0" } else if is_active { "#2563eb" } else { "#f1f5f9" };
                        let fg = if is_dis { "#94a3b8" } else if is_active { "#ffffff" } else { "#334155" };
                        let cursor = if is_dis { "not-allowed" } else { "pointer" };
                        let runtime_click = runtime.clone();
                        let val_str = val.to_string();

                        rsx! {
                            button {
                                id: trig_attrs.id(),
                                role: trig_attrs.role(),
                                aria_selected: trig_attrs.aria_selected(),
                                aria_controls: trig_attrs.aria_controls(),
                                tabindex: "{trig_attrs.tabindex()}",
                                disabled: is_dis,
                                "data-disabled": is_dis.then_some(""),
                                "data-state": trig_attrs.data_state_str(),
                                "data-orientation": trig_attrs.data_orientation(),
                                "data-value": trig_attrs.data_value(),
                                style: "padding: 0.5rem 1rem; border-radius: 0.375rem; border: none; font-weight: 500; cursor: {cursor}; background-color: {bg}; color: {fg};",
                                onclick: move |_| {
                                    if !is_dis {
                                        runtime_click.select_tab(&val_str);
                                    }
                                },
                                if is_dis { "{val} (Disabled)" } else { "{val}" }
                            }
                        }
                    }
                }
            }

            for val in ["tab1", "tab2", "tab3"] {
                {
                    let content_attrs = runtime.content(val);
                    rsx! {
                        div {
                            id: content_attrs.id(),
                            role: content_attrs.role(),
                            aria_labelledby: content_attrs.aria_labelledby(),
                            tabindex: "{content_attrs.tabindex()}",
                            hidden: content_attrs.is_hidden(),
                            "data-state": content_attrs.data_state_str(),
                            style: TAB_PANEL_STYLE,
                            p { style: "margin: 0; font-weight: 600;", "Panel: {val}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn DescendantInputRegressionSection() -> Element {
    let selected = use_signal(|| "form-tab".to_string());
    let mut input_text = use_signal(|| "Hello World".to_string());
    let scope = ScopeHandle::root("playground").child("tabs-regression");

    let tabs_def = Tabs::new(scope.clone(), selected());
    let runtime = use_tabs_runtime(
        tabs_def,
        Some(move |new_val| {
            let mut selected = selected;
            selected.set(new_val);
        }),
    );

    use_effect(use_reactive((), {
        let runtime = runtime.clone();
        move |_| {
            runtime.register_trigger("form-tab", false);
            runtime.register_trigger("other-tab", false);
        }
    }));

    let list_attrs = runtime.list();

    rsx! {
        div {
            style: "display: grid; gap: 0.75rem; padding: 1rem; border: 1px solid #e2e8f0; border-radius: 0.5rem;",
            h3 { style: "margin: 0; font-size: 1rem; color: #1e293b;", "5. Descendant Input Event Boundary & Blur Safety" }
            p { style: "margin: 0; font-size: 0.85rem; color: #64748b;",
                "Typing Space/Enter inside the text input must NOT trigger tab navigation (Bugs #3232, #2915). Departing input blurs before tab unmount (Bug #3600)."
            }

            div {
                id: list_attrs.id(),
                role: list_attrs.role(),
                aria_orientation: list_attrs.aria_orientation(),
                style: TAB_LIST_HORIZONTAL,
                tabindex: "-1",
                onkeydown: {
                    let runtime = runtime.clone();
                    move |evt: KeyboardEvent| {
                        if Tabs::is_navigation_key(&evt.key().to_string()) {
                            evt.prevent_default();
                            runtime.navigate_key(&evt.key().to_string());
                        }
                    }
                },

                for val in ["form-tab", "other-tab"] {
                    {
                        let trig_attrs = runtime.trigger(val, false);
                        let is_active = trig_attrs.is_selected();
                        let bg = if is_active { "#2563eb" } else { "#f1f5f9" };
                        let fg = if is_active { "#ffffff" } else { "#334155" };
                        let runtime_click = runtime.clone();
                        let runtime_down = runtime.clone();
                        let val_str = val.to_string();
                        let val_down = val.to_string();

                        rsx! {
                            button {
                                id: trig_attrs.id(),
                                role: trig_attrs.role(),
                                aria_selected: trig_attrs.aria_selected(),
                                aria_controls: trig_attrs.aria_controls(),
                                tabindex: "{trig_attrs.tabindex()}",
                                "data-state": trig_attrs.data_state_str(),
                                style: "padding: 0.5rem 1rem; border-radius: 0.375rem; border: none; font-weight: 500; cursor: pointer; background-color: {bg}; color: {fg};",
                                onmousedown: move |_| {
                                    runtime_down.select_tab(&val_down);
                                },
                                onclick: move |_| {
                                    runtime_click.select_tab(&val_str);
                                },
                                "{val}"
                            }
                        }
                    }
                }
            }

            div {
                id: runtime.content("form-tab").id(),
                role: runtime.content("form-tab").role(),
                aria_labelledby: runtime.content("form-tab").aria_labelledby(),
                tabindex: "{runtime.content(\"form-tab\").tabindex()}",
                hidden: runtime.content("form-tab").is_hidden(),
                style: TAB_PANEL_STYLE,
                label {
                    style: "display: block; font-weight: 600; margin-bottom: 0.5rem;",
                    "Nested Input Field (Press Space/Enter here):"
                }
                input {
                    id: "playground-nested-input",
                    r#type: "text",
                    value: "{input_text()}",
                    oninput: move |evt| input_text.set(evt.value()),
                    onkeydown: move |evt: KeyboardEvent| {
                        // Crucial: descendant stops propagation to prevent parent tablist from handling space/enter
                        evt.stop_propagation();
                    },
                    style: "padding: 0.5rem; border: 1px solid #cbd5e1; border-radius: 0.375rem; width: 100%; max-width: 24rem;"
                }
                p { style: "margin: 0.5rem 0 0; color: #64748b; font-size: 0.85rem;", "Input value: {input_text()}" }
            }

            div {
                id: runtime.content("other-tab").id(),
                role: runtime.content("other-tab").role(),
                aria_labelledby: runtime.content("other-tab").aria_labelledby(),
                tabindex: "{runtime.content(\"other-tab\").tabindex()}",
                hidden: runtime.content("other-tab").is_hidden(),
                style: TAB_PANEL_STYLE,
                p { style: "margin: 0; font-weight: 600;", "Other Tab Panel Content" }
            }
        }
    }
}
