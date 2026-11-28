use dioxus::prelude::*;
use monoxus::toolbar::{
    ToolbarButton, ToolbarLink, ToolbarRoot, ToolbarSeparator, ToolbarToggleGroup,
    ToolbarToggleGroupMode, ToolbarToggleItem,
};

#[component]
pub fn ToolbarDemo() -> Element {
    let mut formats = use_signal(|| vec![String::from("bold")]);
    let mut align = use_signal(|| String::from("left"));
    let mut last_action = use_signal(|| String::from("Ready"));

    let active_formats = if formats().is_empty() {
        String::from("none")
    } else {
        formats().join(", ")
    };

    rsx! {
        div { class: "flex flex-col items-center gap-3",
            ToolbarRoot {
                class: "inline-flex flex-wrap items-center gap-1 rounded-card-sm border border-border-card bg-background-alt p-1.5 shadow-mini",

                ToolbarToggleGroup {
                    r#type: ToolbarToggleGroupMode::Multiple,
                    values: Some(formats()),
                    on_values_change: move |next| formats.set(next),
                    class: "inline-flex items-center gap-1",

                    ToolbarToggleItem {
                        value: "bold",
                        class: "inline-flex h-8 w-8 items-center justify-center rounded-button text-xs font-bold text-foreground transition-colors hover:bg-muted data-[state=on]:bg-dark data-[state=on]:text-background",
                        "B"
                    }
                    ToolbarToggleItem {
                        value: "italic",
                        class: "inline-flex h-8 w-8 items-center justify-center rounded-button text-xs italic text-foreground transition-colors hover:bg-muted data-[state=on]:bg-dark data-[state=on]:text-background",
                        "I"
                    }
                    ToolbarToggleItem {
                        value: "code",
                        class: "inline-flex h-8 px-2 items-center justify-center rounded-button font-mono text-xs text-foreground transition-colors hover:bg-muted data-[state=on]:bg-dark data-[state=on]:text-background",
                        "</>"
                    }
                }

                ToolbarSeparator {
                    class: "mx-1 h-5 w-px bg-border-card",
                }

                ToolbarToggleGroup {
                    r#type: ToolbarToggleGroupMode::Single,
                    value: Some(align()),
                    on_value_change: move |next| align.set(next),
                    class: "inline-flex items-center gap-1",

                    ToolbarToggleItem {
                        value: "left",
                        class: "inline-flex h-8 items-center justify-center rounded-button px-2.5 text-xs font-medium text-foreground transition-colors hover:bg-muted data-[state=on]:bg-dark data-[state=on]:text-background",
                        "Left"
                    }
                    ToolbarToggleItem {
                        value: "center",
                        class: "inline-flex h-8 items-center justify-center rounded-button px-2.5 text-xs font-medium text-foreground transition-colors hover:bg-muted data-[state=on]:bg-dark data-[state=on]:text-background",
                        "Center"
                    }
                    ToolbarToggleItem {
                        value: "right",
                        class: "inline-flex h-8 items-center justify-center rounded-button px-2.5 text-xs font-medium text-foreground transition-colors hover:bg-muted data-[state=on]:bg-dark data-[state=on]:text-background",
                        "Right"
                    }
                }

                ToolbarSeparator {
                    class: "mx-1 h-5 w-px bg-border-card",
                }

                ToolbarButton {
                    class: "inline-flex h-8 items-center justify-center rounded-button bg-dark px-3 text-xs font-medium text-background shadow-btn transition-colors hover:bg-dark/90",
                    on_click: move |_| last_action.set(String::from("Saved draft")),
                    "Save"
                }

                ToolbarLink {
                    href: "#toolbar-api",
                    class: "inline-flex h-8 items-center justify-center rounded-button px-2.5 text-xs font-medium text-muted-foreground transition-colors hover:bg-muted hover:text-foreground",
                    "Help"
                }
            }

            p { class: "text-xs text-muted-foreground",
                "Format: "
                span { class: "font-medium text-foreground", "{active_formats}" }
                " · Align: "
                span { class: "font-medium text-foreground", "{align}" }
                " · "
                span { class: "font-medium text-foreground", "{last_action}" }
            }
        }
    }
}
