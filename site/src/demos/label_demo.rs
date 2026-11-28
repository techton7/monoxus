use dioxus::prelude::*;
use monoxus::label::Label;

#[component]
pub fn LabelDemo() -> Element {
    let mut username = use_signal(|| "monoxus".to_string());

    rsx! {
        div { class: "flex w-full max-w-xs flex-col gap-2",
            Label {
                r#for: "demo-username",
                required: true,
                class: "text-sm font-medium text-foreground select-none",
                "Workspace handle"
                span { class: "ml-1 text-muted-foreground", "*" }
            }
            input {
                id: "demo-username",
                r#type: "text",
                value: "{username}",
                oninput: move |evt| username.set(evt.value()),
                placeholder: "Enter handle",
                class: "h-10 w-full rounded-input border border-border-input bg-background px-3 text-sm text-foreground shadow-mini outline-none transition-colors placeholder:text-muted-foreground focus:border-foreground",
            }
            p { class: "text-xs text-muted-foreground",
                "Click or double-click the label to focus the input without selecting label text."
            }
        }
    }
}
