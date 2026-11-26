use dioxus::prelude::*;

macro_rules! register_demos {
    ($( $slug:literal => $module:ident :: $component:ident ),* $(,)?) => {
        $( pub mod $module; )*

        pub fn render_demo(slug: &str) -> Element {
            match slug {
                $( $slug => rsx! { $module::$component {} }, )*
                _ => rsx! {
                    div { class: "text-sm text-muted-foreground",
                        "Interactive demo not found for `{slug}`."
                    }
                },
            }
        }

        pub fn demo_source(slug: &str) -> &'static str {
            match slug {
                $( $slug => include_str!(concat!(stringify!($module), ".rs")), )*
                _ => "",
            }
        }

        pub fn demo_filename(slug: &str) -> &'static str {
            match slug {
                $( $slug => concat!(stringify!($module), ".rs"), )*
                _ => "demo.rs",
            }
        }
    };
}

register_demos! {
    "button" => button_demo::ButtonDemo,
    "dialog" => dialog_demo::DialogDemo,
    "select" => select_demo::SelectDemo,
    "tabs" => tabs_demo::TabsDemo,
    "tooltip" => tooltip_demo::TooltipDemo,
}
