pub mod button_demo;
pub mod dialog_demo;
pub mod select_demo;
pub mod tabs_demo;
pub mod tooltip_demo;

use dioxus::prelude::*;

pub fn render_demo(slug: &str) -> Element {
    match slug {
        "button" => rsx! { button_demo::ButtonDemo {} },
        "dialog" => rsx! { dialog_demo::DialogDemo {} },
        "tabs" => rsx! { tabs_demo::TabsDemo {} },
        "select" => rsx! { select_demo::SelectDemo {} },
        "tooltip" => rsx! { tooltip_demo::TooltipDemo {} },
        _ => rsx! {
            div { class: "text-sm text-muted-foreground",
                "Interactive demo not found for `{slug}`."
            }
        },
    }
}

pub fn demo_source(slug: &str) -> &'static str {
    match slug {
        "button" => include_str!("button_demo.rs"),
        "dialog" => include_str!("dialog_demo.rs"),
        "tabs" => include_str!("tabs_demo.rs"),
        "select" => include_str!("select_demo.rs"),
        "tooltip" => include_str!("tooltip_demo.rs"),
        _ => "",
    }
}

pub fn demo_filename(slug: &str) -> &'static str {
    match slug {
        "button" => "button_demo.rs",
        "dialog" => "dialog_demo.rs",
        "tabs" => "tabs_demo.rs",
        "select" => "select_demo.rs",
        "tooltip" => "tooltip_demo.rs",
        _ => "demo.rs",
    }
}
