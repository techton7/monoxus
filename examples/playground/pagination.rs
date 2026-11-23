use dioxus::prelude::*;
use monoxus::foundation::shared::{Direction, DirectionProvider};
use monoxus::pagination::*;

const BTN_BASE_STYLE: &str = "display: inline-flex; align-items: center; justify-content: center; height: 36px; min-width: 36px; padding: 0 12px; border-radius: 0.5rem; font-size: 0.875rem; font-weight: 500; box-sizing: border-box; transition: all 0.15s ease; outline: none;";
const BTN_DEFAULT_STYLE: &str = "background-color: #ffffff; color: #334155; border: 1px solid #cbd5e1; cursor: pointer;";
const BTN_SELECTED_STYLE: &str = "background-color: #7c3aed; color: #ffffff; border: 1px solid #6d28d9; font-weight: 600; cursor: pointer; box-shadow: 0 1px 2px rgba(124, 58, 237, 0.25);";
const BTN_DISABLED_STYLE: &str = "background-color: #f1f5f9; color: #94a3b8; border: 1px solid #e2e8f0; opacity: 0.65; cursor: not-allowed;";
const ELLIPSIS_STYLE: &str = "display: inline-flex; align-items: center; justify-content: center; height: 36px; min-width: 28px; color: #64748b; font-size: 0.875rem; font-weight: 600; user-select: none;";

fn page_button_style(is_selected: bool, disabled: bool) -> String {
    if disabled {
        format!("{BTN_BASE_STYLE} {BTN_DISABLED_STYLE}")
    } else if is_selected {
        format!("{BTN_BASE_STYLE} {BTN_SELECTED_STYLE}")
    } else {
        format!("{BTN_BASE_STYLE} {BTN_DEFAULT_STYLE}")
    }
}

fn nav_button_style(disabled: bool) -> String {
    if disabled {
        format!("{BTN_BASE_STYLE} {BTN_DISABLED_STYLE}")
    } else {
        format!("{BTN_BASE_STYLE} {BTN_DEFAULT_STYLE}")
    }
}

#[component]
pub fn PaginationPlayground() -> Element {
    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-8",
            style: "max-width: 56rem; margin: 0 auto; display: flex; flex-direction: column; gap: 2rem;",

            // Header Section
            div {
                class: "space-y-1.5",
                style: "display: flex; flex-direction: column; gap: 0.375rem;",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    style: "margin: 0; font-size: 1.5rem; font-weight: 700; color: #0f172a;",
                    "Pagination"
                }
                p {
                    class: "text-sm text-slate-500",
                    style: "margin: 0; font-size: 0.875rem; color: #64748b;",
                    "Headless range-navigation primitive with deterministic ellipsis windowing, boundary disablement, WAI-ARIA aria-current=\"page\", and RTL/looping roving focus."
                }
            }

            BasicPaginationScenario {}
            RtlAndDisabledPaginationScenario {}
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 1: Basic Pagination & Dynamic Ellipsis Windowing (SW4A-1, SW4A-2)
// ---------------------------------------------------------------------------
#[component]
fn BasicPaginationScenario() -> Element {
    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            style: "border-radius: 1rem; border: 1px solid #e2e8f0; background-color: #ffffff; padding: 1.5rem; display: flex; flex-direction: column; gap: 1rem;",

            div {
                class: "space-y-1",
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    class: "text-base font-semibold text-slate-900",
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "1. Dynamic Ellipsis Windowing & Range Arithmetic (SW4A-1, SW4A-2)"
                }
                p {
                    class: "text-xs text-slate-500",
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "100 items at 10 per page (sibling_count = 1). Navigate pages to inspect aria-current=\"page\", boundary Prev/Next disablement, and leading/trailing ellipses."
                }
            }

            PaginationRoot {
                id: "sw4a-pag-basic",
                count: 100,
                per_page: 10,
                sibling_count: 1,
                default_page: 1,
                style: "display: flex; flex-direction: column; gap: 0.875rem;",
                render_children: move |snap: PaginationSnapshot| {
                    let prev_style = nav_button_style(!snap.has_prev_page);
                    let next_style = nav_button_style(!snap.has_next_page);

                    rsx! {
                        div {
                            style: "display: flex; align-items: center; gap: 0.375rem; flex-wrap: wrap;",
                            PaginationPrevButton {
                                id: "sw4a-pag-prev",
                                aria_label: "Go to previous page",
                                style: "{prev_style}",
                                "‹ Prev"
                            }
                            for item in snap.pages.iter() {
                                match item {
                                    PageItem::Page { value, key } => {
                                        let is_selected = snap.current_page == *value;
                                        let style = page_button_style(is_selected, false);
                                        rsx! {
                                            PaginationPage {
                                                key: "{key}",
                                                id: "sw4a-pag-page-{value}",
                                                page: PageData::new(*value),
                                                style: "{style}",
                                            }
                                        }
                                    }
                                    PageItem::Ellipsis { key } => rsx! {
                                        span {
                                            key: "{key}",
                                            id: "sw4a-pag-{key}",
                                            aria_hidden: "true",
                                            style: "{ELLIPSIS_STYLE}",
                                            "…"
                                        }
                                    },
                                }
                            }
                            PaginationNextButton {
                                id: "sw4a-pag-next",
                                aria_label: "Go to next page",
                                style: "{next_style}",
                                "Next ›"
                            }
                        }

                        div {
                            class: "pt-2 flex flex-wrap items-center gap-4 text-xs font-mono",
                            style: "padding-top: 0.5rem; display: flex; flex-wrap: wrap; align-items: center; gap: 1rem; font-size: 0.75rem; font-family: monospace;",
                            div {
                                style: "display: flex; align-items: center; gap: 0.375rem;",
                                span {
                                    class: "text-slate-400 font-semibold",
                                    style: "color: #94a3b8; font-weight: 600;",
                                    "Showing:"
                                }
                                span {
                                    id: "sw4a-pag-range-text",
                                    class: "inline-flex items-center rounded px-2 py-0.5 bg-slate-100 text-slate-700 font-bold",
                                    style: "display: inline-flex; align-items: center; border-radius: 0.25rem; padding: 0.125rem 0.5rem; background-color: #f1f5f9; color: #334155; font-weight: 700;",
                                    "{snap.range.start} - {snap.range.end} of 100"
                                }
                            }
                            div {
                                style: "display: flex; align-items: center; gap: 0.375rem;",
                                span {
                                    class: "text-slate-400 font-semibold",
                                    style: "color: #94a3b8; font-weight: 600;",
                                    "Active Page:"
                                }
                                span {
                                    id: "sw4a-pag-current-page",
                                    class: "inline-flex items-center rounded px-2 py-0.5 bg-purple-50 text-purple-700 font-bold",
                                    style: "display: inline-flex; align-items: center; border-radius: 0.25rem; padding: 0.125rem 0.5rem; background-color: #f5f3ff; color: #6d28d9; font-weight: 700;",
                                    "{snap.current_page} / {snap.total_pages}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 2: RTL + loop_focus & Disabled Pagination (SW4A-3)
// ---------------------------------------------------------------------------
#[component]
fn RtlAndDisabledPaginationScenario() -> Element {
    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-6",
            style: "border-radius: 1rem; border: 1px solid #e2e8f0; background-color: #ffffff; padding: 1.5rem; display: flex; flex-direction: column; gap: 1.5rem;",

            div {
                class: "space-y-1",
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    class: "text-base font-semibold text-slate-900",
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "2. RTL Focus Looping & Disabled Controls (SW4A-3)"
                }
                p {
                    class: "text-xs text-slate-500",
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "RTL direction flips ArrowLeft/ArrowRight roving navigation and wraps at boundaries when loop_focus = true. Disabled triggers ignore clicks and key activation."
                }
            }

            // RTL + loop_focus Card
            div {
                class: "space-y-3 p-4 rounded-xl border border-slate-100 bg-slate-50",
                style: "padding: 1rem; border-radius: 0.75rem; border: 1px solid #f1f5f9; background-color: #f8fafc; display: flex; flex-direction: column; gap: 0.75rem;",
                h4 {
                    class: "text-xs font-bold uppercase tracking-wider text-slate-500",
                    style: "margin: 0; font-size: 0.75rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.05em; color: #64748b;",
                    "Right-to-Left (dir=\"rtl\") with loop_focus = true"
                }

                DirectionProvider {
                    dir: Direction::Rtl,
                    PaginationRoot {
                        id: "sw4a-pag-rtl",
                        count: 50,
                        per_page: 10,
                        sibling_count: 1,
                        default_page: 1,
                        loop_focus: true,
                        dir: Direction::Rtl,
                        aria_label: "RTL pagination",
                        style: "display: flex; flex-direction: column; gap: 0.75rem;",
                        render_children: move |snap: PaginationSnapshot| {
                            let prev_style = nav_button_style(!snap.has_prev_page);
                            let next_style = nav_button_style(!snap.has_next_page);

                            rsx! {
                                div {
                                    style: "display: flex; align-items: center; gap: 0.375rem; flex-wrap: wrap;",
                                    PaginationPrevButton {
                                        id: "sw4a-pag-rtl-prev",
                                        aria_label: "Previous page",
                                        style: "{prev_style}",
                                        "السابق"
                                    }
                                    for item in snap.pages.iter() {
                                        match item {
                                            PageItem::Page { value, key } => {
                                                let is_selected = snap.current_page == *value;
                                                let style = page_button_style(is_selected, false);
                                                rsx! {
                                                    PaginationPage {
                                                        key: "{key}",
                                                        id: "sw4a-pag-rtl-page-{value}",
                                                        page: PageData::new(*value),
                                                        style: "{style}",
                                                    }
                                                }
                                            }
                                            PageItem::Ellipsis { key } => rsx! {
                                                span {
                                                    key: "{key}",
                                                    id: "sw4a-pag-rtl-{key}",
                                                    aria_hidden: "true",
                                                    style: "{ELLIPSIS_STYLE}",
                                                    "…"
                                                }
                                            },
                                        }
                                    }
                                    PaginationNextButton {
                                        id: "sw4a-pag-rtl-next",
                                        aria_label: "Next page",
                                        style: "{next_style}",
                                        "التالي"
                                    }
                                }
                                div {
                                    style: "display: flex; align-items: center; gap: 0.375rem; font-size: 0.75rem; font-family: monospace;",
                                    span { style: "color: #94a3b8; font-weight: 600;", "RTL Page:" }
                                    span {
                                        id: "sw4a-pag-rtl-current",
                                        style: "display: inline-flex; align-items: center; border-radius: 0.25rem; padding: 0.125rem 0.5rem; background-color: #ffffff; border: 1px solid #e2e8f0; color: #334155; font-weight: 700;",
                                        "{snap.current_page}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Disabled Pagination Card
            div {
                class: "space-y-3 p-4 rounded-xl border border-slate-100 bg-slate-50",
                style: "padding: 1rem; border-radius: 0.75rem; border: 1px solid #f1f5f9; background-color: #f8fafc; display: flex; flex-direction: column; gap: 0.75rem;",
                h4 {
                    class: "text-xs font-bold uppercase tracking-wider text-slate-500",
                    style: "margin: 0; font-size: 0.75rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.05em; color: #64748b;",
                    "Disabled Pagination Buttons (disabled = true)"
                }

                PaginationRoot {
                    id: "sw4a-pag-disabled",
                    count: 50,
                    per_page: 10,
                    sibling_count: 1,
                    default_page: 2,
                    aria_label: "Disabled pagination",
                    render_children: move |snap: PaginationSnapshot| {
                        let disabled_nav = nav_button_style(true);

                        rsx! {
                            div {
                                style: "display: flex; align-items: center; gap: 0.375rem; flex-wrap: wrap;",
                                PaginationPrevButton {
                                    id: "sw4a-pag-disabled-prev",
                                    disabled: true,
                                    aria_label: "Previous page",
                                    style: "{disabled_nav}",
                                    "‹ Prev"
                                }
                                for item in snap.pages.iter() {
                                    match item {
                                        PageItem::Page { value, key } => {
                                            let is_selected = snap.current_page == *value;
                                            let style = page_button_style(is_selected, true);
                                            rsx! {
                                                PaginationPage {
                                                    key: "{key}",
                                                    id: "sw4a-pag-disabled-page-{value}",
                                                    page: PageData::new(*value),
                                                    disabled: true,
                                                    style: "{style}",
                                                }
                                            }
                                        }
                                        PageItem::Ellipsis { key } => rsx! {
                                            span {
                                                key: "{key}",
                                                id: "sw4a-pag-disabled-{key}",
                                                aria_hidden: "true",
                                                style: "{ELLIPSIS_STYLE}",
                                                "…"
                                            }
                                        },
                                    }
                                }
                                PaginationNextButton {
                                    id: "sw4a-pag-disabled-next",
                                    disabled: true,
                                    aria_label: "Next page",
                                    style: "{disabled_nav}",
                                    "Next ›"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
