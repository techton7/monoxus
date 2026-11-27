---
title: Composition
description: Compose custom elements with the single `child` prop and `merge_props`, control state with `Option<T>`, configure portals, text direction, and CSS transition presence.
---

Monoxus separates behavior from DOM structure using compound parts, the single `child` prop with `merge_props`, and foundation composition utilities in `monoxus::foundation`.

## Controlled and Uncontrolled State

Stateful primitives such as `DialogRoot`, `TabsRoot`, `SelectRoot`, and `TooltipRoot` share a unified controlled/uncontrolled pattern:

- **Uncontrolled**: Pass `default_open: true` or `default_value: "account"` and let the primitive manage its internal signal.
- **Controlled**: Pass `Option<T>` (`open: Some(open())` or `value: active()`) alongside `on_open_change` or `on_value_change`. If your callback declines to update the controlled value (controlled veto), the primitive suppresses any 1-frame state flash and keeps the rendered DOM synchronized with your prop.

```rust
use dioxus::prelude::*;
use monoxus::dialog::{DialogContent, DialogOverlay, DialogPortal, DialogRoot, DialogTrigger};

#[component]
pub fn ControlledDialog() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        DialogRoot {
            open: Some(open()),
            on_open_change: move |next| open.set(next),
            DialogTrigger { "Open" }
            DialogPortal {
                DialogOverlay {}
                DialogContent { "Controlled content" }
            }
        }
    }
}
```

## The `child` Prop and `merge_props`

Every DOM-rendering primitive part accepts an optional `child: Option<Callback<Vec<Attribute>, Element>>` prop (replacing `as_child` / `render_child`). Inside each part, Monoxus calls `merge_props` to combine primitive-owned ARIA/data attributes, inline styles, classes, and event listeners with user-supplied props before invoking `child`:

- **`class`**: Concatenates primitive and user classes with whitespace deduplication.
- **`style`**: Merges inline CSS declarations cleanly with user declarations winning on duplicate properties.
- **`aria-describedby`**: Merges ID token lists without clobbering sibling triggers (essential when stacking `TooltipTrigger` + `DialogTrigger`).
- **Event listeners (`onclick`, `onkeydown`, etc.)**: Chains user listeners before primitive listeners so calling `evt.prevent_default()` in a user handler cleanly vetoes the outer primitive's default action.

> **Dioxus 0.8 tip**: Annotate the `child` closure parameter explicitly as `props: Vec<Attribute>` (or `c: FloatingContentChildProps` on floating `*Content` parts) so Rust can infer the `..props` spread without hitting `E0282`. Also avoid naming a `#[component]` function parameter literally `props` (use `attrs: Vec<Attribute>` instead).

### Multi-Layer Trigger Composition (`TooltipTrigger -> DialogTrigger -> Button`)

Because `merge_props` chains event listeners and unions `aria-describedby`, you can stack triggers across primitives onto a single DOM `<button>` without wrapper elements:

```rust
use dioxus::prelude::*;
use monoxus::button::Button;
use monoxus::dialog::{DialogContent, DialogPortal, DialogRoot, DialogTrigger};
use monoxus::tooltip::{TooltipContent, TooltipPortal, TooltipProvider, TooltipRoot, TooltipTrigger};

#[component]
pub fn TooltipDialogButton() -> Element {
    rsx! {
        TooltipProvider {
            DialogRoot {
                TooltipRoot {
                    TooltipTrigger {
                        child: move |tp: Vec<Attribute>| rsx! {
                            DialogTrigger {
                                attributes: tp,
                                child: move |dp: Vec<Attribute>| rsx! {
                                    Button {
                                        attributes: dp,
                                        class: "btn",
                                        "Delete item"
                                    }
                                }
                            }
                        }
                    }
                    TooltipPortal {
                        TooltipContent { "Opens confirmation dialog" }
                    }
                }
                DialogPortal {
                    DialogContent { "Confirm deletion?" }
                }
            }
        }
    }
}
```

### Floating Content Composition (`FloatingContentChildProps`)

Floating overlay content parts (`PopoverContent`, `TooltipContent`, `HoverCardContent`, `SelectContent`, `ComboboxContent`, `DropdownMenuContent`, `ContextMenuContent`, `MenubarContent`) render a positioned outer wrapper plus the inner surface element. Their `child` callback receives `FloatingContentChildProps` containing both `wrapper_props` and `props`:

```rust
use dioxus::prelude::*;
use monoxus::foundation::compose::FloatingContentChildProps;
use monoxus::tooltip::TooltipContent;

rsx! {
    TooltipContent {
        child: move |c: FloatingContentChildProps| rsx! {
            div { ..c.wrapper_props,
                section { ..c.props, "Custom floating surface" }
            }
        }
    }
}
```

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
            TabsRoot { default_value: "one",
                TabsList {
                    TabsTrigger { value: "one", "First" }
                    TabsTrigger { value: "two", "Second" }
                }
            }
        }
    }
}
```

Directional primitives (`Tabs`, `Accordion`, `Select`, `Tooltip`, `Slider`, `RadioGroup`, `ToggleGroup`) read this context automatically and allow per-component `dir` overrides.

## PortalHost and `Portal.disabled`

Overlay portals (`DialogPortal`, `SelectPortal`, `TooltipPortal`, `PopoverPortal`, and menus) accept `host: Option<PortalHost>` from `monoxus::foundation::overlay` and `disabled: bool`:

- `PortalHost::Default`: Resolves `#main` first (for Dioxus root mount parity) and falls back to `document.body` when `#main` is not present.
- `PortalHost::Inline` (or `disabled: true`): Keeps content in-place in the DOM tree without teleporting.
- `PortalHost::named("my-host")`: Teleports content into a specific container element by ID.

## Auto-ID and `GlobalAttributes` Merging

- **Auto-ID (`use_stable_id`)**: Primitives automatically generate deterministic scoped IDs for ARIA relationships (`aria-controls`, `aria-labelledby`, `aria-describedby`) whenever `id` is omitted, while honoring explicit `id` overrides and reactive updates.
- **`GlobalAttributes` Merging**: Every DOM-rendering part extends `GlobalAttributes` and merges user-supplied `class`, `style`, and arbitrary HTML/ARIA/data attributes through `merge_props` without emitting empty `class=""` or `style=""` attributes.

## Preventable Events and `DismissBehavior`

Dismissible overlays expose `PreventableEvent` callbacks (`on_escape_key_down`, `on_pointer_down_outside`, `on_interact_outside`) alongside 4-way `DismissBehavior` props (`escape_key_down_behavior`, `interact_outside_behavior`):

- `DismissBehavior::Close`: Dismisses immediately when the event is not prevented.
- `DismissBehavior::Ignore`: Ignores the dismissal trigger.
- `DismissBehavior::DeferOtherwiseClose` / `DeferOtherwiseIgnore`: Coordinates with nested overlay layers before deciding whether to dismiss.

Calling `event.prevent_default()` inside a preventable handler cancels the default dismissal transition while still allowing custom side effects.

## Presence, `keep_mounted`, and `TransitionStatus`

Overlay content and backdrop parts coordinate mount/unmount lifecycles with CSS animations through `TransitionStatus` and `use_presence_with_keep_mounted`:

- **`data-starting-style`**: Emitted on the initial enter frame (before the first `requestAnimationFrame` settles to idle) so CSS transitions can animate from an initial opacity/transform state.
- **`data-ending-style`**: Emitted while closing CSS animations or transitions run before unmount.
- **`keep_mounted: true`**: Keeps the element in the DOM after exit transitions finish while applying the native `hidden` attribute (unlike `force_mount: true`, which leaves visibility control entirely to CSS).
- **`on_open_change_complete`**: Fires on overlay roots once the enter or exit presence transition settles.
