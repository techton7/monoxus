use dioxus::prelude::*;
use monoxus::field::{
    FieldControl, FieldControlAttrs, FieldDescription, FieldError, FieldLabel, FieldRoot,
};
use monoxus::foundation::compose::ChildProps;

#[component]
pub fn FieldDemo() -> Element {
    let mut email = use_signal(|| String::from("ops@monoxus.dev"));
    let mut touched = use_signal(|| false);

    let trimmed = email().trim().to_string();
    let mut errors = Vec::new();
    if touched() {
        if trimmed.is_empty() {
            errors.push(String::from("Email address is required."));
        } else if !trimmed.contains('@') || !trimmed.contains('.') {
            errors.push(String::from("Enter a valid email address with a domain."));
        }
    }
    let is_invalid = !errors.is_empty();

    rsx! {
        div { class: "w-full max-w-sm",
            FieldRoot {
                class: "flex flex-col gap-2",
                name: "email",
                required: true,
                invalid: is_invalid,

                FieldLabel {
                    class: "text-sm font-medium text-foreground",
                    "Notification email"
                }

                FieldControl {
                    child: move |slot: ChildProps<FieldControlAttrs>| rsx! {
                        input {
                            r#type: "email",
                            value: "{email}",
                            placeholder: "you@example.com",
                            class: "h-10 w-full rounded-input border border-border-input bg-background-alt px-3 text-sm text-foreground outline-none transition-colors focus:border-foreground data-[invalid]:border-foreground",
                            oninput: move |evt| {
                                touched.set(true);
                                email.set(evt.value());
                            },
                            onblur: move |_| touched.set(true),
                            ..slot.props,
                        }
                    },
                }

                FieldDescription {
                    class: "text-xs text-muted-foreground",
                    "Used for release alerts and security notifications."
                }

                FieldError {
                    class: "text-xs font-medium text-foreground",
                    errors,
                }
            }
        }
    }
}
