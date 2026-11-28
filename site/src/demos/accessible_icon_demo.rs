use dioxus::prelude::*;
use monoxus::accessible_icon::AccessibleIcon;
use monoxus::button::{Button, ButtonType};

#[component]
pub fn AccessibleIconDemo() -> Element {
    let mut last_action = use_signal(|| "None".to_string());

    rsx! {
        div { class: "flex flex-col items-center gap-4",
            div { class: "flex items-center gap-2",
                Button {
                    r#type: ButtonType::Button,
                    class: "inline-flex h-10 w-10 items-center justify-center rounded-button border border-border-input bg-background text-foreground shadow-mini transition-colors hover:bg-muted active:scale-[0.98]",
                    on_click: move |_| last_action.set("Search triggered".to_string()),
                    AccessibleIcon {
                        label: "Search",
                        child: move |props: Vec<Attribute>| rsx! {
                            svg {
                                width: "16",
                                height: "16",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                ..props,
                                circle { cx: "11", cy: "11", r: "8" }
                                path { d: "m21 21-4.3-4.3" }
                            }
                        },
                    }
                }

                Button {
                    r#type: ButtonType::Button,
                    class: "inline-flex h-10 w-10 items-center justify-center rounded-button border border-border-input bg-background text-foreground shadow-mini transition-colors hover:bg-muted active:scale-[0.98]",
                    on_click: move |_| last_action.set("Notifications opened".to_string()),
                    AccessibleIcon {
                        label: "Notifications",
                        child: move |props: Vec<Attribute>| rsx! {
                            svg {
                                width: "16",
                                height: "16",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                ..props,
                                path { d: "M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9" }
                                path { d: "M10.3 21a1.94 1.94 0 0 0 3.4 0" }
                            }
                        },
                    }
                }

                Button {
                    r#type: ButtonType::Button,
                    class: "inline-flex h-10 w-10 items-center justify-center rounded-button border border-border-input bg-background text-foreground shadow-mini transition-colors hover:bg-muted active:scale-[0.98]",
                    on_click: move |_| last_action.set("Settings opened".to_string()),
                    AccessibleIcon {
                        label: "Settings",
                        child: move |props: Vec<Attribute>| rsx! {
                            svg {
                                width: "16",
                                height: "16",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                ..props,
                                circle { cx: "12", cy: "12", r: "3" }
                                path { d: "M12 2v2" }
                                path { d: "M12 20v2" }
                                path { d: "m4.93 4.93 1.41 1.41" }
                                path { d: "m17.66 17.66 1.41 1.41" }
                                path { d: "M2 12h2" }
                                path { d: "M20 12h2" }
                                path { d: "m6.34 17.66-1.41 1.41" }
                                path { d: "m19.07 4.93-1.41 1.41" }
                            }
                        },
                    }
                }
            }

            p { class: "text-xs text-muted-foreground",
                "Last action: "
                span { class: "font-medium text-foreground", "{last_action}" }
            }
        }
    }
}
