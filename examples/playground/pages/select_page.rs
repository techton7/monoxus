use dioxus::prelude::*;
use crate::scenarios::select::{
    BasicFruitSelectSection, BottomConstrainedSelectSection, FormIntegrationSection,
    GroupedSelectSection, MultipleSelectSection, ScrollableViewportSection, StaticContentSection,
    CARD_STYLE, MUTED_STYLE, SELECT_PLAYGROUND_CSS,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SelectScenarioTab {
    All,
    Basic,
    Grouped,
    Scrollable,
    Form,
    Flip,
    Multiple,
    Static,
}

impl SelectScenarioTab {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::All => "All Scenarios",
            Self::Basic => "1. Basic Fruit",
            Self::Grouped => "2. Grouped",
            Self::Scrollable => "3. Scrollable",
            Self::Form => "4. Form",
            Self::Flip => "5. Flip",
            Self::Multiple => "6. Multiple",
            Self::Static => "7. Static",
        }
    }
}

#[component]
pub fn SelectPage() -> Element {
    let mut current_tab = use_signal(|| SelectScenarioTab::All);

    let tabs = [
        SelectScenarioTab::All,
        SelectScenarioTab::Basic,
        SelectScenarioTab::Grouped,
        SelectScenarioTab::Scrollable,
        SelectScenarioTab::Form,
        SelectScenarioTab::Flip,
        SelectScenarioTab::Multiple,
        SelectScenarioTab::Static,
    ];

    rsx! {
        section {
            style: CARD_STYLE,
            style { "{SELECT_PLAYGROUND_CSS}" }

            div {
                style: "display: flex; flex-direction: column; gap: 0.5rem;",
                h2 {
                    style: "margin: 0; color: #581c87; font-size: 1.5rem;",
                    "Select and Ordered Overlay Selection"
                }
                p {
                    style: MUTED_STYLE,
                    "Headless WAI-ARIA Select primitive with single concrete focus ownership, synchronous 500ms typeahead, roving candidate highlight, grouped overlays, and flip positioning."
                }
            }

            // Scenario Filter Tabs
            nav {
                "aria-label": "Select Scenario Navigation",
                style: "display: flex; flex-wrap: wrap; gap: 0.375rem; padding: 0.375rem; background-color: #f3e8ff; border-radius: 0.5rem; margin-bottom: 0.5rem;",
                for tab in tabs {
                    {
                        let is_active = current_tab() == tab;
                        let active_style = if is_active {
                            "background-color: #9333ea; color: white; font-weight: 600; box-shadow: 0 1px 3px rgba(0,0,0,0.15);"
                        } else {
                            "background-color: transparent; color: #6b21a8; font-weight: 500;"
                        };
                        rsx! {
                            button {
                                key: "{tab.as_str()}",
                                style: "padding: 0.375rem 0.75rem; border: none; border-radius: 0.375rem; font-size: 0.8125rem; cursor: pointer; transition: all 0.15s ease; {active_style}",
                                onclick: move |_| current_tab.set(tab),
                                "{tab.as_str()}"
                            }
                        }
                    }
                }
            }

            // Scenario Display
            div {
                id: "select-scenarios-container",
                style: "display: flex; flex-direction: column; gap: 1.25rem;",

                if current_tab() == SelectScenarioTab::All || current_tab() == SelectScenarioTab::Basic {
                    BasicFruitSelectSection {}
                }
                if current_tab() == SelectScenarioTab::All || current_tab() == SelectScenarioTab::Grouped {
                    GroupedSelectSection {}
                }
                if current_tab() == SelectScenarioTab::All || current_tab() == SelectScenarioTab::Scrollable {
                    ScrollableViewportSection {}
                }
                if current_tab() == SelectScenarioTab::All || current_tab() == SelectScenarioTab::Form {
                    FormIntegrationSection {}
                }
                if current_tab() == SelectScenarioTab::All || current_tab() == SelectScenarioTab::Flip {
                    BottomConstrainedSelectSection {}
                }
                if current_tab() == SelectScenarioTab::All || current_tab() == SelectScenarioTab::Multiple {
                    MultipleSelectSection {}
                }
                if current_tab() == SelectScenarioTab::All || current_tab() == SelectScenarioTab::Static {
                    StaticContentSection {}
                }
            }
        }
    }
}
