use dioxus::prelude::*;
use monoxus::label::Label;
use monoxus::switch::{SwitchRoot, SwitchThumb};

#[component]
pub fn SwitchPlayground() -> Element {
    let mut airplane_checked = use_signal(|| false);
    let mut wifi_checked = use_signal(|| true);

    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-6",

            div {
                class: "space-y-1",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    "Switch"
                }
                p {
                    class: "text-sm text-slate-500",
                    "A binary toggle control that can be toggled via Click, Space, and Enter, maintaining permanent thumb geometry and dual-host track layout alignment."
                }
            }

            // SW1B-4: Binary Switch, Permanent Thumb Geometry & Space/Enter Parity
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-5 shadow-sm space-y-4",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "1. Binary Switch & Thumb Geometry (SW1B-4)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "Both Space and Enter toggle the switch with zero double-fire. The thumb remains permanently mounted, shifting physically along the flex track."
                    }
                }

                div {
                    class: "grid grid-cols-1 md:grid-cols-2 gap-4",

                    // Interactive & Status
                    div {
                        class: "flex flex-col gap-3 p-4 bg-slate-50 rounded-lg border border-slate-100",
                        div {
                            class: "flex items-center justify-between",
                            div {
                                class: "flex items-center gap-3",
                                SwitchRoot {
                                    id: "sw-airplane".to_string(),
                                    "data-gui-test-id": "sw-airplane",
                                    checked: airplane_checked(),
                                    on_checked_change: move |next| airplane_checked.set(next),
                                    SwitchThumb {
                                        id: "sw-airplane-thumb".to_string(),
                                    }
                                }
                                span {
                                    class: "text-sm font-medium text-slate-700 select-none",
                                    "Airplane Mode"
                                }
                            }
                            span {
                                class: "inline-flex items-center gap-1 rounded bg-slate-100 px-2 py-0.5 text-xs font-mono text-slate-600",
                                "Status: "
                                strong {
                                    id: "sw-status",
                                    "data-gui-test-id": "sw-status",
                                    if airplane_checked() { "checked" } else { "unchecked" }
                                }
                            }
                        }

                        div {
                            class: "flex items-center gap-3 pt-2 border-t border-slate-200/60",
                            Label {
                                id: "lbl-sw-wifi".to_string(),
                                r#for: "sw-wifi".to_string(),
                                class: "text-sm text-slate-700 cursor-pointer select-none",
                                "Wi-Fi Network (Label Forwarding)"
                            }
                            SwitchRoot {
                                id: "sw-wifi".to_string(),
                                checked: wifi_checked(),
                                on_checked_change: move |next| wifi_checked.set(next),
                                SwitchThumb {
                                    id: "sw-wifi-thumb".to_string(),
                                }
                            }
                        }
                    }

                    // Disabled & ReadOnly Guards
                    div {
                        class: "flex flex-col gap-3 p-4 bg-slate-50 rounded-lg border border-slate-100",
                        div {
                            class: "flex items-center gap-3",
                            SwitchRoot {
                                id: "sw-disabled".to_string(),
                                "data-gui-test-id": "sw-disabled",
                                disabled: true,
                                default_checked: false,
                                SwitchThumb {
                                    id: "sw-disabled-thumb".to_string(),
                                }
                            }
                            span {
                                class: "text-xs font-mono text-slate-500",
                                "Disabled Switch (Unchecked, Non-interactive)"
                            }
                        }

                        div {
                            class: "flex items-center gap-3 pt-2 border-t border-slate-200/60",
                            SwitchRoot {
                                id: "sw-readonly".to_string(),
                                read_only: true,
                                default_checked: true,
                                SwitchThumb {
                                    id: "sw-readonly-thumb".to_string(),
                                }
                            }
                            span {
                                class: "text-xs font-mono text-slate-500",
                                "ReadOnly Switch (Checked, Focusable, Blocks Toggle)"
                            }
                        }
                    }
                }
            }
        }
    }
}
