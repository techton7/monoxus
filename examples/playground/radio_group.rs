use dioxus::prelude::*;
use monoxus::button::{Button, ButtonType};
use monoxus::foundation::shared::Direction;
use monoxus::radio_group::{
    RadioGroupIndicator, RadioGroupItem, RadioGroupOrientation, RadioGroupRoot,
};

#[component]
pub fn RadioGroupPlayground() -> Element {
    let mut unselected_val = use_signal(|| None::<String>);
    let mut show_pro = use_signal(|| true);

    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-6",

            div {
                class: "space-y-1",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    "Radio Group"
                }
                p {
                    class: "text-sm text-slate-500",
                    "A set of checkable buttons—known as radio buttons—where no more than one can be checked at a time, featuring arrow-key auto-selection, Enter suppression, and roving focus."
                }
            }

            // SW1B-5: Unselected Group, Tab Entry & Enter Suppression
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-5 shadow-sm space-y-4",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "1. Unselected Group, Tab Entry & Enter Suppression (SW1B-5)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "Tab enters the unselected group without selecting. Enter is suppressed. Space selects the item. Once checked, radios cannot self-uncheck on click."
                    }
                }

                div {
                    class: "flex flex-col gap-3 p-4 bg-slate-50 rounded-lg border border-slate-100",
                    RadioGroupRoot {
                        id: "rg-unselected".to_string(),
                        default_value: None,
                        on_value_change: move |v| unselected_val.set(Some(v)),
                        div {
                            class: "flex flex-wrap gap-4 items-center",
                            label {
                                class: "inline-flex items-center gap-2 text-sm text-slate-700 cursor-pointer select-none",
                                RadioGroupItem {
                                    id: "rg-unselected-a".to_string(),
                                    value: "a".to_string(),
                                    RadioGroupIndicator {
                                        id: "rg-unselected-a-ind".to_string(),
                                    }
                                }
                                span { "Option A" }
                            }
                            label {
                                class: "inline-flex items-center gap-2 text-sm text-slate-700 cursor-pointer select-none",
                                RadioGroupItem {
                                    id: "rg-unselected-b".to_string(),
                                    value: "b".to_string(),
                                    RadioGroupIndicator {
                                        id: "rg-unselected-b-ind".to_string(),
                                    }
                                }
                                span { "Option B" }
                            }
                            label {
                                class: "inline-flex items-center gap-2 text-sm text-slate-700 cursor-pointer select-none",
                                RadioGroupItem {
                                    id: "rg-unselected-c".to_string(),
                                    value: "c".to_string(),
                                    RadioGroupIndicator {
                                        id: "rg-unselected-c-ind".to_string(),
                                    }
                                }
                                span { "Option C" }
                            }
                        }
                    }

                    div {
                        class: "flex items-center gap-1.5 text-xs font-mono text-slate-600 pt-1",
                        "Selected value: "
                        strong {
                            id: "rg-unselected-status",
                            "{unselected_val().as_deref().unwrap_or(\"none\")}"
                        }
                    }
                }
            }

            // SW1B-6: Arrow Navigation, Orientation, RTL & ReadOnly
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-5 shadow-sm space-y-4",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "2. Keyboard Auto-Selection, RTL & Orientation (SW1B-6)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "Arrow keys auto-select focused items. Both-orientation default, RTL key reversal, Horizontal axis lock, non-looping clamp, and ReadOnly focus movement."
                    }
                }

                div {
                    class: "grid grid-cols-1 md:grid-cols-2 gap-4",

                    // Default Both-Orientation Group
                    div {
                        class: "flex flex-col gap-2 p-3 bg-slate-50 rounded-lg border border-slate-100",
                        span { class: "text-xs font-semibold text-slate-700", "Default (Both Arrows, Skips Disabled)" }
                        RadioGroupRoot {
                            id: "rg-default".to_string(),
                            default_value: Some("starter".to_string()),
                            div {
                                class: "flex flex-col gap-2",
                                label {
                                    class: "inline-flex items-center gap-2 text-xs text-slate-700 cursor-pointer select-none",
                                    RadioGroupItem {
                                        id: "rg-default-starter".to_string(),
                                        value: "starter".to_string(),
                                        RadioGroupIndicator { id: "rg-default-starter-ind".to_string() }
                                    }
                                    span { "Starter Tier" }
                                }
                                label {
                                    class: "inline-flex items-center gap-2 text-xs text-slate-400 select-none",
                                    RadioGroupItem {
                                        id: "rg-default-disabled".to_string(),
                                        value: "disabled".to_string(),
                                        disabled: true,
                                        RadioGroupIndicator { id: "rg-default-disabled-ind".to_string() }
                                    }
                                    span { "Legacy Tier (Disabled)" }
                                }
                                label {
                                    class: "inline-flex items-center gap-2 text-xs text-slate-700 cursor-pointer select-none",
                                    RadioGroupItem {
                                        id: "rg-default-pro".to_string(),
                                        value: "pro".to_string(),
                                        RadioGroupIndicator { id: "rg-default-pro-ind".to_string() }
                                    }
                                    span { "Professional Tier" }
                                }
                                label {
                                    class: "inline-flex items-center gap-2 text-xs text-slate-700 cursor-pointer select-none",
                                    RadioGroupItem {
                                        id: "rg-default-enterprise".to_string(),
                                        value: "enterprise".to_string(),
                                        RadioGroupIndicator { id: "rg-default-enterprise-ind".to_string() }
                                    }
                                    span { "Enterprise Tier" }
                                }
                            }
                        }
                    }

                    // RTL, Horizontal, No-Loop & ReadOnly groups
                    div {
                        class: "flex flex-col gap-3 p-3 bg-slate-50 rounded-lg border border-slate-100",

                        // RTL Group
                        div {
                            class: "flex flex-col gap-1",
                            span { class: "text-xs font-semibold text-slate-700", "RTL Direction (Reversed Arrows)" }
                            RadioGroupRoot {
                                id: "rg-rtl".to_string(),
                                dir: Some(Direction::Rtl),
                                default_value: Some("a".to_string()),
                                div {
                                    class: "flex gap-3 items-center",
                                    RadioGroupItem { id: "rg-rtl-a".to_string(), value: "a".to_string(), RadioGroupIndicator {} }
                                    RadioGroupItem { id: "rg-rtl-b".to_string(), value: "b".to_string(), RadioGroupIndicator {} }
                                    RadioGroupItem { id: "rg-rtl-c".to_string(), value: "c".to_string(), RadioGroupIndicator {} }
                                }
                            }
                        }

                        // Horizontal Group
                        div {
                            class: "flex flex-col gap-1 pt-2 border-t border-slate-200/60",
                            span { class: "text-xs font-semibold text-slate-700", "Horizontal Orientation (Left/Right Only)" }
                            RadioGroupRoot {
                                id: "rg-horizontal".to_string(),
                                orientation: Some(RadioGroupOrientation::Horizontal),
                                default_value: Some("a".to_string()),
                                div {
                                    class: "flex gap-3 items-center",
                                    RadioGroupItem { id: "rg-horiz-a".to_string(), value: "a".to_string(), RadioGroupIndicator {} }
                                    RadioGroupItem { id: "rg-horiz-b".to_string(), value: "b".to_string(), RadioGroupIndicator {} }
                                }
                            }
                        }

                        // Non-Looping Group
                        div {
                            class: "flex flex-col gap-1 pt-2 border-t border-slate-200/60",
                            span { class: "text-xs font-semibold text-slate-700", "Non-Looping Focus (Clamped Boundaries)" }
                            RadioGroupRoot {
                                id: "rg-noloop".to_string(),
                                loop_focus: false,
                                default_value: Some("a".to_string()),
                                div {
                                    class: "flex gap-3 items-center",
                                    RadioGroupItem { id: "rg-noloop-a".to_string(), value: "a".to_string(), RadioGroupIndicator {} }
                                    RadioGroupItem { id: "rg-noloop-b".to_string(), value: "b".to_string(), RadioGroupIndicator {} }
                                }
                            }
                        }

                        // ReadOnly Group
                        div {
                            class: "flex flex-col gap-1 pt-2 border-t border-slate-200/60",
                            span { class: "text-xs font-semibold text-slate-700", "ReadOnly Group (Focus Traversal Without Selection)" }
                            RadioGroupRoot {
                                id: "rg-readonly".to_string(),
                                read_only: true,
                                default_value: Some("a".to_string()),
                                div {
                                    class: "flex gap-3 items-center",
                                    RadioGroupItem { id: "rg-ro-a".to_string(), value: "a".to_string(), RadioGroupIndicator {} }
                                    RadioGroupItem { id: "rg-ro-b".to_string(), value: "b".to_string(), RadioGroupIndicator {} }
                                }
                            }
                        }
                    }
                }
            }

            // SW1B-7: Dynamic Item Unmount Recovery
            div {
                class: "rounded-2xl border border-slate-200 bg-white p-5 shadow-sm space-y-4",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "3. Roving Tab-Stop Unmount Recovery (SW1B-7)"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "Unmounting a checked item intentionally leaves remaining radios unchecked (matches Radix/Bits behavior with no silent selection change), while roving tabindex=\"0\" recovers onto the first remaining enabled item (\"free\") so Tab entry remains reachable."
                    }
                }

                div {
                    class: "flex flex-col gap-3 p-4 bg-slate-50 rounded-lg border border-slate-100",
                    div {
                        class: "flex items-center gap-3",
                        Button {
                            id: "rg-remove-active".to_string(),
                            r#type: ButtonType::Button,
                            class: "sw1a-btn-secondary text-xs".to_string(),
                            on_click: move |_| show_pro.set(false),
                            "Unmount Checked Item (\"pro\")"
                        }
                        span {
                            class: "text-xs text-slate-500",
                            if show_pro() { "Mounted: free, pro (checked, tabindex=\"0\"), team" } else { "\"pro\" unmounted — checked: none | fallback tabindex=\"0\" -> \"free\"" }
                        }
                    }

                    RadioGroupRoot {
                        id: "rg-dynamic".to_string(),
                        default_value: Some("pro".to_string()),
                        div {
                            class: "flex gap-4 items-center pt-2",
                            label {
                                class: "inline-flex items-center gap-2 text-xs text-slate-700 cursor-pointer select-none",
                                RadioGroupItem {
                                    id: "rg-dyn-free".to_string(),
                                    value: "free".to_string(),
                                    RadioGroupIndicator { id: "rg-dyn-free-ind".to_string() }
                                }
                                span { "Free" }
                            }
                            if show_pro() {
                                label {
                                    class: "inline-flex items-center gap-2 text-xs text-slate-700 cursor-pointer select-none",
                                    RadioGroupItem {
                                        id: "rg-dyn-pro".to_string(),
                                        value: "pro".to_string(),
                                        RadioGroupIndicator { id: "rg-dyn-pro-ind".to_string() }
                                    }
                                    span { "Pro (Removable)" }
                                }
                            }
                            label {
                                class: "inline-flex items-center gap-2 text-xs text-slate-700 cursor-pointer select-none",
                                RadioGroupItem {
                                    id: "rg-dyn-team".to_string(),
                                    value: "team".to_string(),
                                    RadioGroupIndicator { id: "rg-dyn-team-ind".to_string() }
                                }
                                span { "Team" }
                            }
                        }
                    }
                }
            }
        }
    }
}
