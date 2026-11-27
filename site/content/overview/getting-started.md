---
title: Getting Started
description: Install Monoxus in your Dioxus project, configure Web and Blitz Native targets, and run your first accessible primitive.
---

Install [monoxus](https://github.com/techton7/monoxus) in a [Dioxus](https://dioxuslabs.com) project and build an accessible dialog that runs on both **Web (WASM)** and **[Blitz](https://github.com/DioxusLabs/blitz) Native** from a single codebase.

## Installation

Add `monoxus` pinned to the `monoxus-v0.1.2` release tag (compatible with `monoxus = "0.1"` semver):

```bash
cargo add monoxus --git https://github.com/techton7/monoxus --tag monoxus-v0.1.2
```

### Dual-Target Setup (Web + Blitz Native)

Configure a single `Cargo.toml` with `web` and `native` features so you can switch targets from the CLI without editing your manifest:

```toml
[dependencies]
dioxus = "0.8.0-alpha.3"
monoxus = { version = "0.1.2", git = "https://github.com/techton7/monoxus.git", tag = "monoxus-v0.1.2" }

[features]
default = ["web"]
web = ["dioxus/web"]
native = ["dioxus/native", "monoxus/native"]
```

Run either target without changing `Cargo.toml`:

```bash
# Web (WASM) — uses default "web" feature
dx serve --web

# Blitz Native — opts into the "native" feature
dx serve --native --no-default-features --features native
# or directly with Cargo:
cargo run --no-default-features --features native
```

> **Note**: Targeting Blitz Native currently requires the `[patch.crates-io]` block documented in the [monoxus README](https://github.com/techton7/monoxus#blitz-native-opt-in-native-feature).

## Minimal Example

Compose the compound parts of a primitive in RSX. Every part is unstyled by default and accepts standard `class`, `style`, and global HTML attributes:

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

## Style & Try It

Attach your own CSS (or Tailwind utility classes) using each part's `data-state` attribute to style open and closed states:

```css
.dialog-overlay[data-state="open"] {
  position: fixed;
  inset: 0;
  background: rgb(0 0 0 / 0.5);
}

.dialog-content[data-state="open"] {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background: white;
  border-radius: 8px;
  padding: 24px;
}
```

Run the app and interact with the dialog. Out of the box, the primitive handles:

- **Escape dismissal**: Pressing `Escape` closes the dialog.
- **Focus trapping and restoration**: Tabbing stays inside `DialogContent` while open, and focus returns to `DialogTrigger` when closed.
- **Outside-click dismissal**: Clicking on the overlay outside the dialog closes it.
- **`data-state` styling hooks**: `DialogTrigger`, `DialogOverlay`, and `DialogContent` automatically toggle `data-state="open"` and `data-state="closed"` so CSS transitions and selectors stay in sync.

For styling patterns, see [Styling](/docs/guides/styling). For controlled state, `AsChildSlot`, portals, and RTL direction, see [Composition](/docs/guides/composition).
