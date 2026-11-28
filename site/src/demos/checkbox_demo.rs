use dioxus::prelude::*;
use monoxus::checkbox::{
    CheckboxGroup, CheckboxGroupLabel, CheckboxIndicator, CheckboxRoot, CheckedState,
};
use monoxus::label::Label;

const OPTIONS: [(&str, &str); 3] = [
    ("releases", "Release notes"),
    ("security", "Security advisories"),
    ("digest", "Weekly digest"),
];

#[component]
pub fn CheckboxDemo() -> Element {
    let mut selected = use_signal(|| vec!["releases".to_string(), "security".to_string()]);

    let all_state = match selected().len() {
        0 => CheckedState::Unchecked,
        n if n == OPTIONS.len() => CheckedState::Checked,
        _ => CheckedState::Indeterminate,
    };

    rsx! {
        div { class: "flex w-full max-w-xs flex-col gap-3 rounded-card-sm border border-border-card bg-background p-4 shadow-mini",
            div { class: "flex items-center gap-2.5 border-b border-border-card pb-3",
                CheckboxRoot {
                    id: "checkbox-all",
                    checked: Some(all_state),
                    on_checked_change: move |next: CheckedState| {
                        if next.is_checked() {
                            selected.set(OPTIONS.iter().map(|(v, _)| (*v).to_string()).collect());
                        } else {
                            selected.set(Vec::new());
                        }
                    },
                    class: "flex h-5 w-5 items-center justify-center rounded border border-border-input bg-background text-background shadow-mini transition-colors data-[state=checked]:border-dark data-[state=checked]:bg-dark data-[state=indeterminate]:border-dark data-[state=indeterminate]:bg-dark",
                    CheckboxIndicator {
                        class: "text-xs font-bold leading-none",
                        if all_state.is_indeterminate() { "−" } else { "✓" }
                    }
                }
                Label {
                    r#for: "checkbox-all",
                    class: "text-sm font-medium text-foreground select-none",
                    "Select all notifications"
                }
            }

            CheckboxGroup {
                values: Some(selected()),
                on_values_change: move |next| selected.set(next),
                class: "flex flex-col gap-2.5",

                CheckboxGroupLabel {
                    class: "text-xs font-medium text-muted-foreground",
                    "Email channels"
                }

                for (value, label) in OPTIONS {
                    div { key: "{value}", class: "flex items-center gap-2.5",
                        CheckboxRoot {
                            id: format!("checkbox-{value}"),
                            value: value.to_string(),
                            class: "flex h-4 w-4 items-center justify-center rounded border border-border-input bg-background text-background transition-colors data-[state=checked]:border-dark data-[state=checked]:bg-dark",
                            CheckboxIndicator {
                                class: "text-[10px] font-bold leading-none",
                                "✓"
                            }
                        }
                        Label {
                            r#for: format!("checkbox-{value}"),
                            class: "text-sm text-foreground-alt select-none",
                            "{label}"
                        }
                    }
                }
            }
        }
    }
}
