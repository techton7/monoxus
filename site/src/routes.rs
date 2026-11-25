use dioxus::prelude::*;

use crate::components::DocsLayout;
use crate::pages::{ComponentDocPage, GuideDoc, Home, NotFound, OverviewDoc};

#[derive(Routable, Clone, PartialEq, Debug)]
pub enum Route {
    #[route("/")]
    Home {},
    #[route("/docs")]
    DocsIndex {},
    #[layout(DocsLayout)]
        #[route("/docs/overview/:slug")]
        OverviewDoc { slug: String },
        #[route("/docs/guides/:slug")]
        GuideDoc { slug: String },
        #[route("/docs/components/:slug")]
        ComponentDocPage { slug: String },
    #[end_layout]
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

#[component]
pub fn DocsIndex() -> Element {
    let navigator = use_navigator();

    use_effect(move || {
        navigator.replace(Route::OverviewDoc {
            slug: "introduction".to_string(),
        });
    });

    rsx! {}
}
