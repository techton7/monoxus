pub use monoxus_site::api_schema;
pub use monoxus_site::docs_data;
mod components;
mod demos;
mod pages;
mod routes;
mod theme;

use dioxus::prelude::*;
use monoxus::tooltip::TooltipProvider;

use crate::routes::Route;
use crate::theme::ThemeProvider;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link {
            rel: "icon",
            href: asset!("/assets/favicon.svg"),
        }
        document::Stylesheet {
            href: asset!("/assets/tailwind.css"),
        }
        TooltipProvider {
            delay_duration: 250,
            skip_delay_duration: 150,
            ThemeProvider {
                Router::<Route> {}
            }
        }
    }
}
