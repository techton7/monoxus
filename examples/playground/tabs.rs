use dioxus::prelude::*;
use monoxus::tabs::{
    TabsActivationMode, TabsContent, TabsDirection, TabsList, TabsOrientation, TabsRoot,
    TabsTrigger,
};

#[component]
pub fn TabsPlayground() -> Element {
    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-8",

            // Header Section
            div {
                class: "space-y-1.5",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    "Tabs"
                }
                p {
                    class: "text-sm text-slate-500",
                    "A set of layered sections of content—known as tab panels—that are displayed one at a time with declarative roving tabindex and WAI-ARIA compliance."
                }
            }

            // Scenarios
            Scenario1HorizontalAutomatic {}
            Scenario2VerticalManual {}
            Scenario3DisabledSkipping {}
            Scenario4ControlledState {}
            Scenario5RtlDirection {}
            Scenario6DescendantInputIsolation {}
            Scenario7SegmentedPills {}
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 1: Horizontal Automatic Activation
// ---------------------------------------------------------------------------
#[component]
fn Scenario1HorizontalAutomatic() -> Element {
    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            div {
                class: "space-y-1",
                h3 { class: "text-base font-semibold text-slate-900", "1. Horizontal Tabs (Automatic Activation)" }
                p { class: "text-xs text-slate-500",
                    "ArrowLeft / ArrowRight roves focus and automatically activates the focused tab. Home / End jumps to boundaries."
                }
            }

            TabsRoot {
                id: "tabs-horizontal-auto".to_string(),
                default_value: "preview".to_string(),
                orientation: TabsOrientation::Horizontal,
                activation_mode: TabsActivationMode::Automatic,
                TabsList {
                    id: "tabs-auto-list",
                    class: "inline-flex h-10 items-center justify-center rounded-lg bg-slate-100 p-1 text-slate-500",
                    TabsTrigger {
                        id: "tabs-auto-preview",
                        value: "preview".to_string(),
                        class: "inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1.5 text-sm font-medium transition-all cursor-pointer text-slate-600 hover:text-slate-900 data-[state=active]:bg-white data-[state=active]:text-slate-900 data-[state=active]:shadow-sm",
                        "Preview"
                    }
                    TabsTrigger {
                        id: "tabs-auto-code",
                        value: "code".to_string(),
                        class: "inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1.5 text-sm font-medium transition-all cursor-pointer text-slate-600 hover:text-slate-900 data-[state=active]:bg-white data-[state=active]:text-slate-900 data-[state=active]:shadow-sm",
                        "Code"
                    }
                    TabsTrigger {
                        id: "tabs-auto-settings",
                        value: "settings".to_string(),
                        class: "inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1.5 text-sm font-medium transition-all cursor-pointer text-slate-600 hover:text-slate-900 data-[state=active]:bg-white data-[state=active]:text-slate-900 data-[state=active]:shadow-sm",
                        "Settings"
                    }
                }
                TabsContent {
                    id: "tabs-auto-content-preview",
                    value: "preview".to_string(),
                    class: "mt-3 rounded-xl border border-slate-200 bg-slate-50/50 p-5 text-sm text-slate-700 space-y-1",
                    p { class: "font-semibold text-slate-900", "Component Preview" }
                    p { class: "text-xs text-slate-500", "Live rendered component preview with declarative state and styling." }
                }
                TabsContent {
                    id: "tabs-auto-content-code",
                    value: "code".to_string(),
                    class: "mt-3 rounded-xl border border-slate-200 bg-slate-50/50 p-5 text-sm text-slate-700 space-y-1",
                    p { class: "font-semibold text-slate-900", "Source Code" }
                    p { class: "text-xs font-mono text-slate-500", "<TabsRoot default_value=\"preview\">...</TabsRoot>" }
                }
                TabsContent {
                    id: "tabs-auto-content-settings",
                    value: "settings".to_string(),
                    class: "mt-3 rounded-xl border border-slate-200 bg-slate-50/50 p-5 text-sm text-slate-700 space-y-1",
                    p { class: "font-semibold text-slate-900", "Configuration" }
                    p { class: "text-xs text-slate-500", "Component and runtime preferences configured here." }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 2: Vertical Manual Activation
// ---------------------------------------------------------------------------
#[component]
fn Scenario2VerticalManual() -> Element {
    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            div {
                class: "space-y-1",
                h3 { class: "text-base font-semibold text-slate-900", "2. Vertical Tabs (Manual Activation)" }
                p { class: "text-xs text-slate-500",
                    "ArrowUp / ArrowDown roves keyboard focus, but selection requires explicit Space or Enter keypress."
                }
            }

            TabsRoot {
                id: "tabs-vertical-manual".to_string(),
                default_value: "profile".to_string(),
                orientation: TabsOrientation::Vertical,
                activation_mode: TabsActivationMode::Manual,
                class: "flex gap-6 items-start",
                TabsList {
                    id: "tabs-vert-list",
                    class: "flex flex-col w-48 rounded-lg bg-slate-100 p-1 text-slate-500 gap-1",
                    TabsTrigger {
                        id: "tabs-vert-profile",
                        value: "profile".to_string(),
                        class: "flex items-center w-full rounded-md px-3 py-2 text-sm font-medium transition-all cursor-pointer text-slate-600 hover:text-slate-900 data-[state=active]:bg-white data-[state=active]:text-slate-900 data-[state=active]:shadow-sm",
                        "Profile"
                    }
                    TabsTrigger {
                        id: "tabs-vert-notifications",
                        value: "notifications".to_string(),
                        class: "flex items-center w-full rounded-md px-3 py-2 text-sm font-medium transition-all cursor-pointer text-slate-600 hover:text-slate-900 data-[state=active]:bg-white data-[state=active]:text-slate-900 data-[state=active]:shadow-sm",
                        "Notifications"
                    }
                    TabsTrigger {
                        id: "tabs-vert-billing",
                        value: "billing".to_string(),
                        class: "flex items-center w-full rounded-md px-3 py-2 text-sm font-medium transition-all cursor-pointer text-slate-600 hover:text-slate-900 data-[state=active]:bg-white data-[state=active]:text-slate-900 data-[state=active]:shadow-sm",
                        "Billing"
                    }
                }
                div {
                    class: "flex-1 min-w-0",
                    TabsContent {
                        id: "tabs-vert-content-profile",
                        value: "profile".to_string(),
                        class: "rounded-xl border border-slate-200 bg-slate-50/50 p-5 text-sm text-slate-700 space-y-1",
                        p { class: "font-semibold text-slate-900", "Profile Settings" }
                        p { class: "text-xs text-slate-500", "Manage your personal profile and email addresses." }
                    }
                    TabsContent {
                        id: "tabs-vert-content-notifications",
                        value: "notifications".to_string(),
                        class: "rounded-xl border border-slate-200 bg-slate-50/50 p-5 text-sm text-slate-700 space-y-1",
                        p { class: "font-semibold text-slate-900", "Notification Preferences" }
                        p { class: "text-xs text-slate-500", "Choose how and when you receive security alerts." }
                    }
                    TabsContent {
                        id: "tabs-vert-content-billing",
                        value: "billing".to_string(),
                        class: "rounded-xl border border-slate-200 bg-slate-50/50 p-5 text-sm text-slate-700 space-y-1",
                        p { class: "font-semibold text-slate-900", "Billing & Invoices" }
                        p { class: "text-xs text-slate-500", "View invoices and manage payment methods." }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 3: Disabled Tab Skipping
// ---------------------------------------------------------------------------
#[component]
fn Scenario3DisabledSkipping() -> Element {
    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            div {
                class: "space-y-1",
                h3 { class: "text-base font-semibold text-slate-900", "3. Disabled Tab Navigation Skipping" }
                p { class: "text-xs text-slate-500",
                    "Tab 2 is disabled. Pressing ArrowRight from Tab 1 immediately leaps over Tab 2 to Tab 3."
                }
            }

            TabsRoot {
                id: "tabs-disabled-skipping".to_string(),
                default_value: "tab1".to_string(),
                TabsList {
                    id: "tabs-dis-list",
                    class: "inline-flex h-10 items-center justify-center rounded-lg bg-slate-100 p-1 text-slate-500",
                    TabsTrigger {
                        id: "tabs-dis-tab1",
                        value: "tab1".to_string(),
                        class: "inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1.5 text-sm font-medium transition-all cursor-pointer text-slate-600 hover:text-slate-900 data-[state=active]:bg-white data-[state=active]:text-slate-900 data-[state=active]:shadow-sm",
                        "Tab 1 (Active)"
                    }
                    TabsTrigger {
                        id: "tabs-dis-tab2",
                        value: "tab2".to_string(),
                        disabled: true,
                        class: "inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1.5 text-sm font-medium transition-all text-slate-400 cursor-not-allowed opacity-50 data-[state=active]:bg-white",
                        "Tab 2 (Disabled)"
                    }
                    TabsTrigger {
                        id: "tabs-dis-tab3",
                        value: "tab3".to_string(),
                        class: "inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1.5 text-sm font-medium transition-all cursor-pointer text-slate-600 hover:text-slate-900 data-[state=active]:bg-white data-[state=active]:text-slate-900 data-[state=active]:shadow-sm",
                        "Tab 3 (Available)"
                    }
                }
                TabsContent {
                    id: "tabs-dis-content-tab1",
                    value: "tab1".to_string(),
                    class: "mt-3 rounded-xl border border-slate-200 bg-slate-50/50 p-5 text-sm text-slate-700",
                    p { "Tab 1 content is visible." }
                }
                TabsContent {
                    id: "tabs-dis-content-tab3",
                    value: "tab3".to_string(),
                    class: "mt-3 rounded-xl border border-slate-200 bg-slate-50/50 p-5 text-sm text-slate-700",
                    p { "Tab 3 content is visible after skipping disabled Tab 2." }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 4: Controlled State
// ---------------------------------------------------------------------------
#[component]
fn Scenario4ControlledState() -> Element {
    let mut current_tab = use_signal(|| "analytics".to_string());

    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            div {
                class: "space-y-1",
                h3 { class: "text-base font-semibold text-slate-900", "4. Controlled State Synchronization" }
                p { class: "text-xs text-slate-500",
                    "The active tab is driven by an external Signal and can be changed programmatically."
                }
            }

            div {
                class: "flex items-center gap-2",
                span { class: "text-xs font-semibold text-slate-500 uppercase tracking-wider", "External Controls:" }
                button {
                    id: "btn-select-overview",
                    r#type: "button",
                    onclick: move |_| current_tab.set("overview".to_string()),
                    class: "rounded bg-slate-100 px-2.5 py-1 text-xs font-medium text-slate-700 hover:bg-slate-200 cursor-pointer transition",
                    "Select Overview"
                }
                button {
                    id: "btn-select-analytics",
                    r#type: "button",
                    onclick: move |_| current_tab.set("analytics".to_string()),
                    class: "rounded bg-slate-100 px-2.5 py-1 text-xs font-medium text-slate-700 hover:bg-slate-200 cursor-pointer transition",
                    "Select Analytics"
                }
                button {
                    id: "btn-select-reports",
                    r#type: "button",
                    onclick: move |_| current_tab.set("reports".to_string()),
                    class: "rounded bg-slate-100 px-2.5 py-1 text-xs font-medium text-slate-700 hover:bg-slate-200 cursor-pointer transition",
                    "Select Reports"
                }
            }

            TabsRoot {
                id: "tabs-controlled".to_string(),
                value: current_tab,
                on_value_change: move |val| current_tab.set(val),
                TabsList {
                    id: "tabs-ctrl-list",
                    class: "inline-flex h-10 items-center justify-center rounded-lg bg-slate-100 p-1 text-slate-500",
                    TabsTrigger {
                        id: "tabs-ctrl-overview",
                        value: "overview".to_string(),
                        class: "inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1.5 text-sm font-medium transition-all cursor-pointer text-slate-600 hover:text-slate-900 data-[state=active]:bg-white data-[state=active]:text-slate-900 data-[state=active]:shadow-sm",
                        "Overview"
                    }
                    TabsTrigger {
                        id: "tabs-ctrl-analytics",
                        value: "analytics".to_string(),
                        class: "inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1.5 text-sm font-medium transition-all cursor-pointer text-slate-600 hover:text-slate-900 data-[state=active]:bg-white data-[state=active]:text-slate-900 data-[state=active]:shadow-sm",
                        "Analytics"
                    }
                    TabsTrigger {
                        id: "tabs-ctrl-reports",
                        value: "reports".to_string(),
                        class: "inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1.5 text-sm font-medium transition-all cursor-pointer text-slate-600 hover:text-slate-900 data-[state=active]:bg-white data-[state=active]:text-slate-900 data-[state=active]:shadow-sm",
                        "Reports"
                    }
                }
                TabsContent {
                    id: "tabs-ctrl-content-overview",
                    value: "overview".to_string(),
                    class: "mt-3 rounded-xl border border-slate-200 bg-slate-50/50 p-5 text-sm text-slate-700",
                    p { "Overview metrics and executive summary." }
                }
                TabsContent {
                    id: "tabs-ctrl-content-analytics",
                    value: "analytics".to_string(),
                    class: "mt-3 rounded-xl border border-slate-200 bg-slate-50/50 p-5 text-sm text-slate-700",
                    p { "Detailed traffic and conversion analytics." }
                }
                TabsContent {
                    id: "tabs-ctrl-content-reports",
                    value: "reports".to_string(),
                    class: "mt-3 rounded-xl border border-slate-200 bg-slate-50/50 p-5 text-sm text-slate-700",
                    p { "Scheduled performance and compliance reports." }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 5: RTL Direction Tabs
// ---------------------------------------------------------------------------
#[component]
fn Scenario5RtlDirection() -> Element {
    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            div {
                class: "space-y-1",
                h3 { class: "text-base font-semibold text-slate-900", "5. Right-to-Left (RTL) Navigation" }
                p { class: "text-xs text-slate-500",
                    "Under dir=RTL, ArrowLeft navigates forward and ArrowRight navigates backward."
                }
            }

            div {
                dir: "rtl",
                TabsRoot {
                    id: "tabs-rtl".to_string(),
                    default_value: "summary".to_string(),
                    dir: TabsDirection::Rtl,
                    TabsList {
                        id: "tabs-rtl-list",
                        class: "inline-flex h-10 items-center justify-center rounded-lg bg-slate-100 p-1 text-slate-500",
                        TabsTrigger {
                            id: "tabs-rtl-first",
                            value: "summary".to_string(),
                            class: "inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1.5 text-sm font-medium transition-all cursor-pointer text-slate-600 hover:text-slate-900 data-[state=active]:bg-white data-[state=active]:text-slate-900 data-[state=active]:shadow-sm",
                            "ملخص (Summary)"
                        }
                        TabsTrigger {
                            id: "tabs-rtl-second",
                            value: "details".to_string(),
                            class: "inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1.5 text-sm font-medium transition-all cursor-pointer text-slate-600 hover:text-slate-900 data-[state=active]:bg-white data-[state=active]:text-slate-900 data-[state=active]:shadow-sm",
                            "تفاصيل (Details)"
                        }
                        TabsTrigger {
                            id: "tabs-rtl-third",
                            value: "support".to_string(),
                            class: "inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1.5 text-sm font-medium transition-all cursor-pointer text-slate-600 hover:text-slate-900 data-[state=active]:bg-white data-[state=active]:text-slate-900 data-[state=active]:shadow-sm",
                            "دعم (Support)"
                        }
                    }
                    TabsContent {
                        id: "tabs-rtl-content-summary",
                        value: "summary".to_string(),
                        class: "mt-3 rounded-xl border border-slate-200 bg-slate-50/50 p-5 text-sm text-slate-700",
                        p { "لوحة الملخص باللغة العربية (Summary panel in Arabic)." }
                    }
                    TabsContent {
                        id: "tabs-rtl-content-details",
                        value: "details".to_string(),
                        class: "mt-3 rounded-xl border border-slate-200 bg-slate-50/50 p-5 text-sm text-slate-700",
                        p { "لوحة التفاصيل (Details panel)." }
                    }
                    TabsContent {
                        id: "tabs-rtl-content-support",
                        value: "support".to_string(),
                        class: "mt-3 rounded-xl border border-slate-200 bg-slate-50/50 p-5 text-sm text-slate-700",
                        p { "لوحة الدعم الفني (Support panel)." }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 6: Descendant Form Input Isolation
// ---------------------------------------------------------------------------
#[component]
fn Scenario6DescendantInputIsolation() -> Element {
    let mut text_value = use_signal(|| "Type spaces here".to_string());

    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            div {
                class: "space-y-1",
                h3 { class: "text-base font-semibold text-slate-900", "6. Descendant Form Input Boundary Safety" }
                p { class: "text-xs text-slate-500",
                    "Typing Space / Enter inside descendant form controls must NOT trigger tab activation or navigation."
                }
            }

            TabsRoot {
                id: "tabs-input-isolation".to_string(),
                default_value: "form-tab".to_string(),
                TabsList {
                    id: "tabs-form-list",
                    class: "inline-flex h-10 items-center justify-center rounded-lg bg-slate-100 p-1 text-slate-500",
                    TabsTrigger {
                        id: "tabs-form-tab",
                        value: "form-tab".to_string(),
                        class: "inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1.5 text-sm font-medium transition-all cursor-pointer text-slate-600 hover:text-slate-900 data-[state=active]:bg-white data-[state=active]:text-slate-900 data-[state=active]:shadow-sm",
                        "Form Tab"
                    }
                    TabsTrigger {
                        id: "tabs-other-tab",
                        value: "other-tab".to_string(),
                        class: "inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1.5 text-sm font-medium transition-all cursor-pointer text-slate-600 hover:text-slate-900 data-[state=active]:bg-white data-[state=active]:text-slate-900 data-[state=active]:shadow-sm",
                        "Other Tab"
                    }
                }
                TabsContent {
                    id: "tabs-form-content",
                    value: "form-tab".to_string(),
                    class: "mt-3 rounded-xl border border-slate-200 bg-slate-50/50 p-5 text-sm text-slate-700 space-y-3",
                    label {
                        r#for: "playground-nested-input",
                        class: "block text-xs font-semibold text-slate-700",
                        "Nested Input Field (Press Space or Arrow keys here):"
                    }
                    input {
                        id: "playground-nested-input",
                        r#type: "text",
                        value: "{text_value()}",
                        oninput: move |evt| text_value.set(evt.value()),
                        onkeydown: move |evt: KeyboardEvent| {
                            evt.stop_propagation();
                        },
                        class: "w-full max-w-sm rounded-lg border border-slate-300 px-3 py-2 text-sm text-slate-900 shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500",
                    }
                    p { class: "text-xs text-slate-500", "Value: {text_value()}" }
                }
                TabsContent {
                    id: "tabs-other-content",
                    value: "other-tab".to_string(),
                    class: "mt-3 rounded-xl border border-slate-200 bg-slate-50/50 p-5 text-sm text-slate-700",
                    p { "Other tab panel content." }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 7: Segmented Control / Pills Variant
// ---------------------------------------------------------------------------
#[component]
fn Scenario7SegmentedPills() -> Element {
    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            div {
                class: "space-y-1",
                h3 { class: "text-base font-semibold text-slate-900", "7. Segmented Control / Pills Styling" }
                p { class: "text-xs text-slate-500",
                    "Fully unstyled headless primitives styled with Tailwind rounded-full pill variants."
                }
            }

            TabsRoot {
                id: "tabs-pills".to_string(),
                default_value: "weekly".to_string(),
                TabsList {
                    id: "tabs-pills-list",
                    class: "inline-flex rounded-full bg-slate-100 p-1.5 gap-1",
                    TabsTrigger {
                        id: "tabs-pills-daily",
                        value: "daily".to_string(),
                        class: "inline-flex items-center rounded-full px-4 py-1.5 text-xs font-semibold cursor-pointer text-slate-600 hover:text-slate-900 data-[state=active]:bg-blue-600 data-[state=active]:text-white data-[state=active]:shadow-sm transition-all",
                        "Daily"
                    }
                    TabsTrigger {
                        id: "tabs-pills-weekly",
                        value: "weekly".to_string(),
                        class: "inline-flex items-center rounded-full px-4 py-1.5 text-xs font-semibold cursor-pointer text-slate-600 hover:text-slate-900 data-[state=active]:bg-blue-600 data-[state=active]:text-white data-[state=active]:shadow-sm transition-all",
                        "Weekly"
                    }
                    TabsTrigger {
                        id: "tabs-pills-monthly",
                        value: "monthly".to_string(),
                        class: "inline-flex items-center rounded-full px-4 py-1.5 text-xs font-semibold cursor-pointer text-slate-600 hover:text-slate-900 data-[state=active]:bg-blue-600 data-[state=active]:text-white data-[state=active]:shadow-sm transition-all",
                        "Monthly"
                    }
                }
                TabsContent {
                    id: "tabs-pills-content-daily",
                    value: "daily".to_string(),
                    class: "mt-3 rounded-xl border border-slate-200 bg-slate-50/50 p-5 text-sm text-slate-700",
                    p { "Daily activity metrics aggregation." }
                }
                TabsContent {
                    id: "tabs-pills-content-weekly",
                    value: "weekly".to_string(),
                    class: "mt-3 rounded-xl border border-slate-200 bg-slate-50/50 p-5 text-sm text-slate-700",
                    p { "Weekly rollups and trends." }
                }
                TabsContent {
                    id: "tabs-pills-content-monthly",
                    value: "monthly".to_string(),
                    class: "mt-3 rounded-xl border border-slate-200 bg-slate-50/50 p-5 text-sm text-slate-700",
                    p { "Monthly statements and financial summaries." }
                }
            }
        }
    }
}
