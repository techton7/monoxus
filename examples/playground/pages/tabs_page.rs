use crate::tabs::TabsPlayground;
use dioxus::prelude::*;
use monoxus::{
    foundation::shared::{Direction, DirectionProvider},
    tabs::{TabsContent, TabsList, TabsRoot, TabsTrigger},
};

#[component]
pub fn TabsPage() -> Element {
    rsx! {
        div {
            class: "space-y-8",
            TabsPlayground {}

            // Track 1 Regression Hook (CC-3, DN-7, DN-8, R1-2, R1-5): DirectionProvider cascade, loop_focus=false, TabsContent tabindex=-1
            div {
                class: "max-w-4xl mx-auto rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
                div {
                    class: "space-y-1",
                    h3 {
                        class: "text-base font-semibold text-slate-900",
                        "8. DirectionProvider RTL Cascade, Clamped Loop (loop_focus: false) & TabsContent tabindex=-1"
                    }
                    p {
                        class: "text-xs text-slate-500",
                        "TabsRoot inherits dir=\"rtl\" from DirectionProvider, TabsList clamps at ends, and TabsContent (tabindex=-1) passes Tab directly to its inner button."
                    }
                }
                DirectionProvider {
                    dir: Direction::Rtl,
                    TabsRoot {
                        id: "tabs-dir-provider-root".to_string(),
                        default_value: "first".to_string(),
                        TabsList {
                            id: "tabs-clamped-list".to_string(),
                            loop_focus: false,
                            class: "inline-flex h-10 items-center justify-center rounded-lg bg-slate-100 p-1 text-slate-500 gap-1".to_string(),
                            TabsTrigger {
                                id: "tabs-clamped-1".to_string(),
                                value: "first".to_string(),
                                class: "rounded-md px-3 py-1.5 text-sm font-medium cursor-pointer data-[state=active]:bg-white data-[state=active]:text-slate-900".to_string(),
                                "First (RTL)"
                            }
                            TabsTrigger {
                                id: "tabs-clamped-2".to_string(),
                                value: "second".to_string(),
                                class: "rounded-md px-3 py-1.5 text-sm font-medium cursor-pointer data-[state=active]:bg-white data-[state=active]:text-slate-900".to_string(),
                                "Second"
                            }
                            TabsTrigger {
                                id: "tabs-clamped-3".to_string(),
                                value: "third".to_string(),
                                class: "rounded-md px-3 py-1.5 text-sm font-medium cursor-pointer data-[state=active]:bg-white data-[state=active]:text-slate-900".to_string(),
                                "Third"
                            }
                        }
                        TabsContent {
                            id: "tabs-tabindex-neg1-panel".to_string(),
                            value: "first".to_string(),
                            tabindex: -1,
                            class: "mt-3 rounded-xl border border-slate-200 bg-slate-50 p-4 space-y-2".to_string(),
                            button {
                                id: "tabs-inner-focus-btn",
                                r#type: "button",
                                class: "rounded-lg bg-blue-600 px-3 py-1.5 text-xs font-semibold text-white cursor-pointer",
                                "Inner Interactive Button (Direct Tab Target)"
                            }
                        }
                        TabsContent {
                            id: "tabs-clamped-panel-2".to_string(),
                            value: "second".to_string(),
                            class: "mt-3 rounded-xl border border-slate-200 bg-slate-50 p-4 text-sm text-slate-700".to_string(),
                            "Second panel content."
                        }
                        TabsContent {
                            id: "tabs-clamped-panel-3".to_string(),
                            value: "third".to_string(),
                            class: "mt-3 rounded-xl border border-slate-200 bg-slate-50 p-4 text-sm text-slate-700".to_string(),
                            "Third panel content."
                        }
                    }
                }
            }
        }
    }
}
