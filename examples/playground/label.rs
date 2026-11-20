use dioxus::prelude::*;
use monoxus::label::Label;

#[component]
pub fn LabelPlayground() -> Element {
    let mut username = use_signal(|| String::from("gohyeoncheol"));
    let mut nested_val = use_signal(|| String::from("selectable"));
    let mut accepted = use_signal(|| false);
    let mut input_focused = use_signal(|| false);
    let mut nested_input_focused = use_signal(|| false);
    let mut dblclick_counter = use_signal(|| 0_u32);
    let mut nested_action_count = use_signal(|| 0_u32);

    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-8",

            div {
                class: "space-y-1.5",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    "Label"
                }
                p {
                    class: "text-sm text-slate-500",
                    "Renders an accessible <label data-label-root> that forwards focus/activation to associated controls while suppressing accidental double-click text selection on label copy."
                }
            }

            // SW1A-4 Part 1: Explicit `for` forwarding + live focus indicator
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-5",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "1. Explicit Control Association & Focus Forwarding (SW1A-4)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "Single-clicking the label forwards focus to #demo-input. The live indicator reflects real-time focus transfer."
                    }
                }

                div {
                    class: "space-y-2 max-w-md",
                    div {
                        class: "flex items-center justify-between",
                        Label {
                            id: "label-for-input".to_string(),
                            r#for: "demo-input".to_string(),
                            class: "block text-sm font-medium text-slate-800 cursor-pointer".to_string(),
                            "data-gui-test-id": "label-for-input",
                            "Workspace Handle"
                        }
                        span {
                            id: "demo-input-focus-badge",
                            style: if input_focused() {
                                "background-color: #f3e8ff; color: #7e22ce; border: 1px solid #d8b4fe; padding: 0.125rem 0.5rem; border-radius: 9999px; font-family: monospace; font-size: 0.6875rem; font-weight: 600;"
                            } else {
                                "background-color: #f1f5f9; color: #64748b; border: 1px solid #e2e8f0; padding: 0.125rem 0.5rem; border-radius: 9999px; font-family: monospace; font-size: 0.6875rem;"
                            },
                            if input_focused() { "focused" } else { "idle" }
                        }
                    }
                    input {
                        id: "demo-input",
                        r#type: "text",
                        value: "{username()}",
                        oninput: move |evt| username.set(evt.value()),
                        onfocus: move |_| input_focused.set(true),
                        onblur: move |_| input_focused.set(false),
                        class: "w-full rounded-lg border border-slate-300 px-3 py-2 text-sm text-slate-900 shadow-sm",
                    }
                    div {
                        class: "text-xs font-mono text-slate-500 flex items-center gap-1",
                        "Input value: "
                        strong { id: "demo-input-value", "{username()}" }
                    }
                }

                div {
                    class: "flex items-center gap-2.5 pt-3 border-t border-slate-100",
                    input {
                        id: "demo-checkbox",
                        r#type: "checkbox",
                        checked: accepted(),
                        onchange: move |evt| accepted.set(evt.checked()),
                        class: "h-4 w-4 rounded border-slate-300 text-purple-600 cursor-pointer",
                    }
                    Label {
                        id: "label-for-checkbox".to_string(),
                        r#for: "demo-checkbox".to_string(),
                        class: "text-sm font-medium text-slate-700 cursor-pointer".to_string(),
                        "data-gui-test-id": "label-for-checkbox",
                        "Accept telemetry & crash diagnostics"
                    }
                    span {
                        id: "label-checkbox-state",
                        "data-gui-test-id": "label-checkbox-state",
                        style: if accepted() {
                            "background-color: #ecfdf5; color: #059669; border: 1px solid #a7f3d0; padding: 0.125rem 0.5rem; border-radius: 0.25rem; font-family: monospace; font-size: 0.75rem; font-weight: 600;"
                        } else {
                            "background-color: #f1f5f9; color: #64748b; border: 1px solid #e2e8f0; padding: 0.125rem 0.5rem; border-radius: 0.25rem; font-family: monospace; font-size: 0.75rem;"
                        },
                        if accepted() { "checked" } else { "unchecked" }
                    }
                }
            }

            // SW1A-4 Part 2: Double-Click Selection Suppression
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "2. Double-Click Text Selection Guard (SW1A-4)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "Rapid double-clicking the label text suppresses native text highlighting via preventDefault() on mousedown (detail > 1)."
                    }
                }

                div {
                    class: "space-y-3 max-w-md",
                    Label {
                        id: "label-dblclick-guard".to_string(),
                        class: "block p-3 rounded-lg border border-slate-200 bg-slate-50 text-sm font-medium text-slate-800 cursor-pointer select-none".to_string(),
                        "Double-Click Guarded Label Text (Try double clicking rapidly)"
                    }
                    div {
                        class: "flex items-center gap-3",
                        button {
                            id: "dblclick-counter-btn",
                            r#type: "button",
                            class: "sw1a-btn-secondary text-xs",
                            onclick: move |_| dblclick_counter += 1,
                            "Click Counter"
                        }
                        span {
                            class: "text-xs font-mono text-slate-500",
                            "Activation count: "
                            strong { id: "dblclick-count-val", "{dblclick_counter()}" }
                        }
                    }
                }
            }

            // SW1A-4 Part 3: Nested interactive control target guard (`button, input, select, textarea`)
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "3. Nested Interactive Control Target Guard (SW1A-4)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "Interacting inside nested buttons or inputs bypasses Label's preventDefault() via target.closest(\"button, input, select, textarea\"), preserving native control behavior."
                    }
                }

                Label {
                    id: "label-nested-guard".to_string(),
                    class: "flex flex-col gap-3 p-4 rounded-xl border border-slate-200 bg-slate-50/50 max-w-md text-sm font-medium text-slate-800 cursor-pointer".to_string(),
                    div {
                        class: "flex items-center justify-between",
                        span {
                            id: "label-nested-text",
                            class: "text-sm font-semibold text-slate-900",
                            "Implicit Container Label"
                        }
                        span {
                            id: "nested-input-focus-badge",
                            style: if nested_input_focused() {
                                "background-color: #f3e8ff; color: #7e22ce; border: 1px solid #d8b4fe; padding: 0.125rem 0.5rem; border-radius: 9999px; font-family: monospace; font-size: 0.6875rem; font-weight: 600;"
                            } else {
                                "background-color: #f1f5f9; color: #64748b; border: 1px solid #e2e8f0; padding: 0.125rem 0.5rem; border-radius: 9999px; font-family: monospace; font-size: 0.6875rem;"
                            },
                            if nested_input_focused() { "focused" } else { "idle" }
                        }
                    }

                    div {
                        class: "flex items-center gap-2",
                        button {
                            id: "nested-action-btn",
                            r#type: "button",
                            class: "sw1a-btn-secondary text-xs py-1.5 px-3",
                            onclick: move |evt: MouseEvent| {
                                evt.stop_propagation();
                                nested_action_count += 1;
                            },
                            "Nested Action Button"
                        }
                        span {
                            class: "text-xs font-mono text-slate-500",
                            "Clicks: "
                            strong { id: "nested-action-count-val", "{nested_action_count()}" }
                        }
                    }

                    input {
                        id: "nested-input",
                        r#type: "text",
                        value: "{nested_val()}",
                        oninput: move |evt| nested_val.set(evt.value()),
                        onfocus: move |_| nested_input_focused.set(true),
                        onblur: move |_| nested_input_focused.set(false),
                        class: "w-full rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm text-slate-900 shadow-sm",
                    }
                }
            }
        }
    }
}
