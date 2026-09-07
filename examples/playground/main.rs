#![allow(non_snake_case)]

pub mod accordion;
pub mod alert_dialog;
pub mod dialog;
pub mod pages;
pub mod popover;
pub mod scenarios;
pub mod tabs;
pub mod tooltip;

use dioxus::prelude::*;
use pages::{
    AccordionPage, AlertDialogPage, DialogPage, PopoverPage, SelectPage, TabsPage, TooltipPage,
};

fn main() {
    dioxus::launch(app);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ComponentPage {
    Select,
    Accordion,
    Tabs,
    Dialog,
    AlertDialog,
    Popover,
    Tooltip,
}

impl ComponentPage {
    pub fn title(&self) -> &'static str {
        match self {
            Self::Select => "Select",
            Self::Accordion => "Accordion",
            Self::Tabs => "Tabs",
            Self::Dialog => "Dialog",
            Self::AlertDialog => "Alert Dialog",
            Self::Popover => "Popover",
            Self::Tooltip => "Tooltip",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Select => "▾",
            Self::Accordion => "≡",
            Self::Tabs => "◫",
            Self::Dialog => "◻",
            Self::AlertDialog => "⚠",
            Self::Popover => "⌖",
            Self::Tooltip => "ℹ",
        }
    }

    pub fn badge(&self) -> &'static str {
        match self {
            Self::Select => "7 Scenarios",
            Self::Accordion => "Interactive",
            Self::Tabs => "WAI-ARIA",
            Self::Dialog => "Modal",
            Self::AlertDialog => "Destructive",
            Self::Popover => "Anchored",
            Self::Tooltip => "Hover/Focus",
        }
    }
}

fn app() -> Element {
    let mut current_page = use_signal(|| ComponentPage::Select);

    let pages = [
        ComponentPage::Select,
        ComponentPage::Accordion,
        ComponentPage::Tabs,
        ComponentPage::Dialog,
        ComponentPage::AlertDialog,
        ComponentPage::Popover,
        ComponentPage::Tooltip,
    ];

    rsx! {
        div {
            style: "min-height: 100vh; background-color: #f8fafc; display: flex; flex-direction: row; color: #0f172a; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;",

            // Left Navigation Sidebar
            aside {
                style: "width: 260px; min-width: 260px; background-color: #ffffff; border-right: 1px solid #e2e8f0; display: flex; flex-direction: column; padding: 1.5rem 1rem; gap: 1.5rem;",

                // Header / Branding
                div {
                    style: "display: flex; flex-direction: column; gap: 0.25rem;",
                    div {
                        style: "display: flex; align-items: center; justify-content: space-between;",
                        h1 {
                            style: "margin: 0; font-size: 1.25rem; font-weight: 700; color: #581c87; letter-spacing: -0.025em;",
                            "monoxus"
                        }
                        span {
                            style: "font-size: 0.6875rem; font-weight: 600; padding: 0.125rem 0.375rem; border-radius: 9999px; background-color: #f3e8ff; color: #7e22ce;",
                            "alpha"
                        }
                    }
                    p {
                        style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                        "Headless WAI-ARIA Primitives"
                    }
                }

                // Nav Links
                nav {
                    "aria-label": "Component Navigation",
                    style: "display: flex; flex-direction: column; gap: 0.375rem;",
                    div {
                        style: "font-size: 0.6875rem; font-weight: 700; text-transform: uppercase; color: #94a3b8; letter-spacing: 0.05em; padding-left: 0.5rem; margin-bottom: 0.25rem;",
                        "Components"
                    }
                    for page in pages {
                        {
                            let is_active = current_page() == page;
                            let active_style = if is_active {
                                "background-color: #f3e8ff; color: #7e22ce; font-weight: 600; border: 1px solid #d8b4fe;"
                            } else {
                                "background-color: transparent; color: #475569; font-weight: 500; border: 1px solid transparent;"
                            };
                            let page_id = format!("nav-link-{}", page.title().to_lowercase().replace(' ', "-"));
                            rsx! {
                                button {
                                    id: "{page_id}",
                                    style: "display: flex; align-items: center; justify-content: space-between; width: 100%; padding: 0.5rem 0.75rem; border-radius: 0.5rem; font-size: 0.875rem; cursor: pointer; text-align: left; transition: all 0.15s ease; outline: none; {active_style}",
                                    onclick: move |_| current_page.set(page),
                                    div {
                                        style: "display: flex; align-items: center; gap: 0.625rem;",
                                        span {
                                            style: "font-family: monospace; font-size: 0.9375rem; width: 1rem; text-align: center; color: #9333ea;",
                                            "{page.icon()}"
                                        }
                                        span { "{page.title()}" }
                                    }
                                    span {
                                        style: "font-size: 0.6875rem; padding: 0.125rem 0.375rem; border-radius: 0.25rem; background-color: rgba(147, 51, 234, 0.08); color: #7e22ce;",
                                        "{page.badge()}"
                                    }
                                }
                            }
                        }
                    }
                }

                // Footer Information
                div {
                    style: "margin-top: auto; padding-top: 1rem; border-top: 1px solid #f1f5f9; font-size: 0.75rem; color: #94a3b8;",
                    p { style: "margin: 0 0 0.25rem 0;", "Run with:" }
                    code {
                        style: "display: block; padding: 0.375rem 0.5rem; background-color: #f1f5f9; border-radius: 0.25rem; font-size: 0.6875rem; color: #334155; overflow-x: auto;",
                        "dx serve --example playground"
                    }
                }
            }

            // Main Content Body
            main {
                id: "playground-main-content",
                style: "flex: 1; min-width: 0; padding: 2rem 2.5rem; max-width: 64rem; overflow-y: auto;",

                // Active Page Rendered in Complete Isolation
                match current_page() {
                    ComponentPage::Select => rsx! { SelectPage {} },
                    ComponentPage::Accordion => rsx! { AccordionPage {} },
                    ComponentPage::Tabs => rsx! { TabsPage {} },
                    ComponentPage::Dialog => rsx! { DialogPage {} },
                    ComponentPage::AlertDialog => rsx! { AlertDialogPage {} },
                    ComponentPage::Popover => rsx! { PopoverPage {} },
                    ComponentPage::Tooltip => rsx! { TooltipPage {} },
                }
            }
        }
    }
}
