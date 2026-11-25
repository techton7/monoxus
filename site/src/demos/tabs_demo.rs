use dioxus::prelude::*;
use monoxus::tabs::{TabsContent, TabsList, TabsRoot, TabsTrigger};

#[component]
pub fn TabsDemo() -> Element {
    rsx! {
        TabsRoot {
            default_value: "account".to_string(),
            class: "w-full max-w-md rounded-card-sm border border-border-card bg-background p-2 shadow-mini",

            TabsList {
                class: "grid w-full grid-cols-3 gap-1 rounded-input bg-muted p-1 text-sm font-medium",

                TabsTrigger {
                    value: "account".to_string(),
                    class: "rounded-button px-3 py-1.5 text-muted-foreground transition-colors hover:text-foreground data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-mini",
                    "Account"
                }
                TabsTrigger {
                    value: "password".to_string(),
                    class: "rounded-button px-3 py-1.5 text-muted-foreground transition-colors hover:text-foreground data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-mini",
                    "Password"
                }
                TabsTrigger {
                    value: "notifications".to_string(),
                    class: "rounded-button px-3 py-1.5 text-muted-foreground transition-colors hover:text-foreground data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-mini",
                    "Notifications"
                }
            }

            TabsContent {
                value: "account".to_string(),
                class: "space-y-3 p-3 pt-4 outline-none",
                div { class: "space-y-1",
                    h4 { class: "text-sm font-medium text-foreground", "Profile" }
                    p { class: "text-xs text-muted-foreground",
                        "Manage your public handle and workspace email."
                    }
                }
                div { class: "rounded-input border border-border-input bg-background-alt px-3 py-2 text-xs text-foreground-alt",
                    "handle: @monoxus-dev"
                }
            }

            TabsContent {
                value: "password".to_string(),
                class: "space-y-3 p-3 pt-4 outline-none",
                div { class: "space-y-1",
                    h4 { class: "text-sm font-medium text-foreground", "Security" }
                    p { class: "text-xs text-muted-foreground",
                        "Rotate your password or configure passkeys."
                    }
                }
                div { class: "rounded-input border border-border-input bg-background-alt px-3 py-2 text-xs text-foreground-alt",
                    "Last rotated 14 days ago"
                }
            }

            TabsContent {
                value: "notifications".to_string(),
                class: "space-y-3 p-3 pt-4 outline-none",
                div { class: "space-y-1",
                    h4 { class: "text-sm font-medium text-foreground", "Digest" }
                    p { class: "text-xs text-muted-foreground",
                        "Choose how release and security alerts are delivered."
                    }
                }
                div { class: "rounded-input border border-border-input bg-background-alt px-3 py-2 text-xs text-foreground-alt",
                    "Weekly summary enabled"
                }
            }
        }
    }
}
