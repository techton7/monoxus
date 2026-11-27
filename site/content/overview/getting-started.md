---
title: Getting Started
description: Install Monoxus in your Dioxus project and render your first accessible primitive.
---

Add `monoxus` and `dioxus` to your `Cargo.toml` to start building with headless primitives.

## Installation

For a standard Dioxus Web application, pin the `monoxus-v0.1.2` release tag (compatible with `monoxus = "0.1"` semver):

```toml
[dependencies]
dioxus = { version = "0.8.0-alpha.3", features = ["web"] }
monoxus = { version = "0.1.2", git = "https://github.com/techton7/monoxus.git", tag = "monoxus-v0.1.2" }
```

### Blitz Native Target

When targeting **Blitz Native**, disable default features and enable the `native` feature:

```toml
[dependencies]
dioxus = { version = "0.8.0-alpha.3", features = ["native"] }
monoxus = { version = "0.1.2", git = "https://github.com/techton7/monoxus.git", tag = "monoxus-v0.1.2", default-features = false, features = ["native"] }
```

## Minimal Example

Every Monoxus primitive is unstyled. Combine the compound parts in RSX and style them using `class`, `style`, or `data-*` attribute selectors:

```rust
use dioxus::prelude::*;
use monoxus::dialog::{
    DialogClose, DialogContent, DialogDescription, DialogOverlay,
    DialogPortal, DialogRoot, DialogTitle, DialogTrigger,
};

#[component]
pub fn EditProfileDialog() -> Element {
    rsx! {
        DialogRoot {
            DialogTrigger { class: "btn", "Edit Profile" }
            DialogPortal {
                DialogOverlay { class: "dialog-overlay" }
                DialogContent { class: "dialog-content",
                    DialogTitle { "Edit Profile" }
                    DialogDescription {
                        "Update your display name and preferences."
                    }
                    DialogClose { class: "btn", "Done" }
                }
            }
        }
    }
}
```

## Controlled and Uncontrolled State

Stateful primitives such as `DialogRoot`, `TabsRoot`, and `TooltipRoot` support both modes:

- **Uncontrolled**: Pass `default_open: Some(true)` or `default_value: Some("account".into())` and let the primitive manage its internal signal.
- **Controlled**: Pass your own `Signal<T>` via `open` or `value`, and optionally react to changes with `on_open_change` or `on_value_change`.

## Runtime Defaults

- **Auto-ID (`use_stable_id`)**: Compound primitives automatically generate deterministic, collision-free scoped IDs for ARIA relationships (`aria-controls`, `aria-labelledby`, `aria-describedby`), so manual `id` props are optional unless you need a fixed DOM ID.
- **Computed `z-index` Hoisting**: Floating overlays (`Popover`, `Tooltip`, `Select`, `HoverCard`, menus) automatically read the computed `z-index` from `*Content` and hoist it onto the outer floating wrapper, so classes like `z-50` layer cleanly without extra wrapper styles.
