---
title: Composition
description: Compose custom elements, portals, text direction, and mount presence with Monoxus foundation utilities.
---

Monoxus separates behavior from DOM structure using compound parts and foundation composition utilities in `monoxus::foundation`.

## AsChildSlot and Slottable

When building your own design system wrapper on top of Monoxus, `AsChildSlot` and `Slottable` (in `monoxus::foundation::compose`) let you project primitive attributes and event handlers onto a single child target without adding extra wrapper elements:

- **`AsChildSlot<T>`**: Validates that an `as_child` projection receives a single child target via `AsChildSlot::try_from_option` or `AsChildSlot::try_from_iter`.
- **`Slottable<T>`**: Marks the inner content slot that should remain inside the projected target when `AsChildSlot::with_slottable` or `project_as_child` merges the two.
- **`compose_event_handlers` & `compose_refs`**: Chains user-supplied event handlers and mount refs with primitive-owned handlers while respecting `prevent_default` semantics.

```rust
use monoxus::foundation::compose::{AsChildSlot, Slottable};

let slot = AsChildSlot::try_from_option(Some("custom-trigger"))?;
let (target, content) = slot.with_slottable(Slottable::new("Open menu"));
```

For single-element primitives such as `VisuallyHidden` or `AccessibleIcon`, you can also pass `as_child: true` with a `render_child` callback that receives the computed attributes directly.

## DirectionProvider

`DirectionProvider` (in `monoxus::foundation::shared`) sets the reading direction (`Direction::Ltr` or `Direction::Rtl`) for all descendant primitives:

```rust
use dioxus::prelude::*;
use monoxus::foundation::shared::{Direction, DirectionProvider};
use monoxus::tabs::{TabsList, TabsRoot, TabsTrigger};

#[component]
pub fn RtlTabs() -> Element {
    rsx! {
        DirectionProvider { dir: Direction::Rtl,
            TabsRoot { default_value: Some("one".into()),
                TabsList {
                    TabsTrigger { value: "one".to_string(), "First" }
                    TabsTrigger { value: "two".to_string(), "Second" }
                }
            }
        }
    }
}
```

Directional primitives (`Tabs`, `Accordion`, `Select`, `Tooltip`, `Slider`, `RadioGroup`, `ToggleGroup`) read this context automatically and allow per-component `dir` overrides.

## PortalHost

Overlay portals (`DialogPortal`, `SelectPortal`, `TooltipPortal`) accept a `host: Option<PortalHost>` prop from `monoxus::foundation::overlay`:

- `PortalHost::Default`: Teleports content to the default document overlay root.
- `PortalHost::Inline`: Keeps content in-place in the DOM tree without teleporting.
- `PortalHost::named("my-host")`: Teleports content into a specific container element by ID.

## Presence

`Presence` and `PresenceController` (in `monoxus::foundation::overlay`) coordinate mount/unmount lifecycles with CSS exit animations. Overlays stay mounted while transitioning through `PresenceState` (`Mounted`, `Unmounting`, `Suspended`, `Unmounted`), and `force_mount: true` on `DialogPortal`, `DialogContent`, `SelectPortal`, `SelectContent`, `TabsContent`, or `TooltipContent` keeps the node mounted when closed so CSS transitions can run.
