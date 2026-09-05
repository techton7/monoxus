#![allow(non_snake_case)]

mod accordion;
mod alert_dialog;
mod dialog;
mod popover;
mod tabs;
mod tooltip;

use dioxus::prelude::*;

fn main() {
    dioxus::launch(app);
}

fn app() -> Element {
    rsx! {
        main {
            style: "font-family: system-ui, sans-serif; max-width: 72rem; margin: 0 auto; padding: 2rem; color: #111827; background-color: #f8fafc; min-height: 100vh;",
            h1 {
                style: "margin-bottom: 0.5rem;",
                "monoxus playground"
            }
            p {
                style: "margin-top: 0; margin-bottom: 1.5rem; color: #475569;",
                "A tiny Phase 3.1 / 3.2 / 3.3 / 3.4 harness for dialogs, overlays, tabs, accordions, and collapsibles. Run it with "
                code { "dx serve --example playground --web" }
                "."
            }
            div {
                style: "display: grid; gap: 1.5rem;",
                accordion::AccordionPlayground {}
                tabs::TabsPlayground {}
                dialog::DialogPlayground {}
                alert_dialog::AlertDialogPlayground {}
                popover::PopoverPlayground {}
                tooltip::TooltipPlayground {}
            }
        }
    }
}
