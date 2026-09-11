use dioxus::prelude::*;
use dioxus_js_interop::{bind_js, reset_module_registry, use_watcher};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoxRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowMetrics {
    pub width: f64,
    pub height: f64,
}

bind_js!("examples/playground/pages/test_bridge.ts"::*);

#[component]
pub fn JsBindgenPage() -> Element {
    let mut command_status = use_signal(|| "Idle".to_string());
    let mut query_result = use_signal(|| "Not measured".to_string());
    let mut query_error = use_signal(|| "None".to_string());
    let mut watcher_enabled = use_signal(|| false);
    let mut watcher_metrics = use_signal(|| None::<WindowMetrics>);
    let mut self_healing_status = use_signal(|| "Idle".to_string());

    // Reactive watcher via use_watcher hook
    use_watcher(move || {
        watcher_enabled().then(|| {
            watch_window_resize(move |metrics| {
                watcher_metrics.set(Some(metrics));
            })
        })
    });

    rsx! {
        div {
            style: "padding: 2rem; max-width: 900px; display: flex; flex-direction: column; gap: 2rem;",

            // Header
            div {
                h1 { style: "font-size: 1.875rem; font-weight: 700; margin-bottom: 0.5rem;", "dioxus-js-bindgen Live Runtime Proof" }
                p { style: "color: #64748b;", "Live verification of CQS 3 Pillars (Command, Query, Watcher), RAII cleanup, and self-healing in the browser." }
            }

            // Pillar 1: Command
            div {
                style: "background: white; padding: 1.5rem; border-radius: 8px; border: 1px solid #e2e8f0; display: flex; flex-direction: column; gap: 1rem;",
                h2 { style: "font-size: 1.25rem; font-weight: 600;", "1. Command (Synchronous Fire-and-Forget)" }
                p { style: "font-size: 0.875rem; color: #64748b;", "Executes immediately without async/await or spawn. Captures errors safely without panicking Rust." }

                div {
                    style: "display: flex; gap: 1rem; align-items: center;",
                    button {
                        id: "btn-run-command",
                        style: "background: #2563eb; color: white; padding: 0.5rem 1rem; border-radius: 6px; border: none; font-weight: 500; cursor: pointer;",
                        onclick: move |_| {
                            focus_element("cmd-target-input");
                            command_status.set("Invoked focus_element synchronously!".to_string());
                        },
                        "Focus Target Input (Sync Command)"
                    }

                    button {
                        id: "btn-fail-command",
                        style: "background: #dc2626; color: white; padding: 0.5rem 1rem; border-radius: 6px; border: none; font-weight: 500; cursor: pointer;",
                        onclick: move |_| {
                            fail_command("nonexistent-element-id");
                            command_status.set("Invoked fail_command (Error caught safely in console)".to_string());
                        },
                        "Trigger Failing Command (Safe Catch)"
                    }
                }

                input {
                    id: "cmd-target-input",
                    placeholder: "I am the target element with id='cmd-target-input'",
                    style: "padding: 0.5rem 0.75rem; border: 1px solid #cbd5e1; border-radius: 6px; width: 100%;",
                }

                div {
                    id: "command-status-display",
                    style: "font-size: 0.875rem; color: #334155;",
                    "Command Status: {command_status}"
                }
            }

            // Pillar 2: Query
            div {
                style: "background: white; padding: 1.5rem; border-radius: 8px; border: 1px solid #e2e8f0; display: flex; flex-direction: column; gap: 1rem;",
                h2 { style: "font-size: 1.25rem; font-weight: 600;", "2. Query (Type-Safe Async RPC)" }
                p { style: "font-size: 0.875rem; color: #64748b;", "Bi-directional async query returning Result<T, JsError> with 1-retry self-healing." }

                div {
                    id: "query-measurement-box",
                    style: "width: 240px; height: 80px; background: #e0e7ff; border: 2px dashed #6366f1; display: flex; align-items: center; justify-content: center; font-size: 0.875rem; font-weight: 500; color: #4338ca;",
                    "Target Box (240x80)"
                }

                div {
                    style: "display: flex; gap: 1rem; align-items: center;",
                    button {
                        id: "btn-run-query",
                        style: "background: #059669; color: white; padding: 0.5rem 1rem; border-radius: 6px; border: none; font-weight: 500; cursor: pointer;",
                        onclick: move |_| {
                            spawn(async move {
                                match get_bounding_rect("query-measurement-box").await {
                                    Ok(rect) => {
                                        query_result.set(format!("x={:.1}, y={:.1}, width={:.1}, height={:.1}", rect.x, rect.y, rect.width, rect.height));
                                        query_error.set("None".to_string());
                                    }
                                    Err(e) => {
                                        query_error.set(format!("{e}"));
                                    }
                                }
                            });
                        },
                        "Measure Box Rect (Query)"
                    }

                    button {
                        id: "btn-fail-query",
                        style: "background: #d97706; color: white; padding: 0.5rem 1rem; border-radius: 6px; border: none; font-weight: 500; cursor: pointer;",
                        onclick: move |_| {
                            spawn(async move {
                                match get_bounding_rect("missing-box-id").await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        query_error.set(format!("{e}"));
                                    }
                                }
                            });
                        },
                        "Trigger Query Exception (Missing ID)"
                    }

                    button {
                        id: "btn-reset-and-heal",
                        style: "background: #7c3aed; color: white; padding: 0.5rem 1rem; border-radius: 6px; border: none; font-weight: 500; cursor: pointer;",
                        onclick: move |_| {
                            spawn(async move {
                                reset_module_registry();
                                self_healing_status.set("Registry reset! Running query to trigger self-healing...".to_string());
                                match get_bounding_rect("query-measurement-box").await {
                                    Ok(rect) => {
                                        self_healing_status.set(format!("Self-healing success! Re-registered & got width={:.1}", rect.width));
                                    }
                                    Err(e) => {
                                        self_healing_status.set(format!("Self-healing failed: {e}"));
                                    }
                                }
                            });
                        },
                        "Reset Module Registry & Self-Heal"
                    }
                }

                div {
                    id: "query-result-display",
                    style: "font-size: 0.875rem; color: #334155;",
                    "Query Result: {query_result}"
                }
                div {
                    id: "query-error-display",
                    style: "font-size: 0.875rem; color: #dc2626;",
                    "Query Error: {query_error}"
                }
                div {
                    id: "self-healing-status-display",
                    style: "font-size: 0.875rem; color: #7c3aed;",
                    "Self-Healing: {self_healing_status}"
                }
            }

            // Pillar 3: Watcher
            div {
                style: "background: white; padding: 1.5rem; border-radius: 8px; border: 1px solid #e2e8f0; display: flex; flex-direction: column; gap: 1rem;",
                h2 { style: "font-size: 1.25rem; font-weight: 600;", "3. Watcher (Reactive Signal-Tracked RAII)" }
                p { style: "font-size: 0.875rem; color: #64748b;", "Streams continuous browser events to Rust. Drops cleanly via use_watcher when signal turns false." }

                div {
                    style: "display: flex; gap: 1rem; align-items: center;",
                    button {
                        id: "btn-toggle-watcher",
                        style: if watcher_enabled() {
                            "background: #ef4444; color: white; padding: 0.5rem 1rem; border-radius: 6px; border: none; font-weight: 500; cursor: pointer;"
                        } else {
                            "background: #10b981; color: white; padding: 0.5rem 1rem; border-radius: 6px; border: none; font-weight: 500; cursor: pointer;"
                        },
                        onclick: move |_| {
                            let curr = watcher_enabled();
                            watcher_enabled.set(!curr);
                        },
                        if watcher_enabled() { "Stop Watcher (Trigger Drop Cleanup)" } else { "Start Window Resize Watcher" }
                    }
                }

                div {
                    id: "watcher-status-display",
                    style: "font-size: 0.875rem; color: #334155;",
                    "Watcher Active: {watcher_enabled}"
                }

                div {
                    id: "watcher-metrics-display",
                    style: "font-size: 0.875rem; color: #059669; font-weight: 600;",
                    if let Some(metrics) = watcher_metrics() {
                        "Live Metrics: width={metrics.width}, height={metrics.height}"
                    } else {
                        "Live Metrics: None"
                    }
                }
            }
        }
    }
}
