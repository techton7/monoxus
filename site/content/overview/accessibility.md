---
title: Accessibility
description: WAI-ARIA compliance, keyboard navigation, FocusScope, RovingFocus, and screen-reader utilities.
---

Monoxus follows the [WAI-ARIA Authoring Practices Guide (APG)](https://www.w3.org/WAI/ARIA/apg/) so interactive primitives work reliably with keyboard navigation and assistive technologies.

## WAI-ARIA Relationships

Compound primitives generate deterministic element IDs and wire relationship attributes automatically:

- **Conditional references**: Attributes such as `aria-controls`, `aria-labelledby`, and `aria-describedby` are emitted only when the referenced part is mounted in the DOM.
- **Disabled semantics**: Disabled triggers and items omit `data-disabled` when enabled and emit `data-disabled=""` when disabled, matching `[data-disabled]` CSS selectors.
- **Polymorphic buttons**: `Button` renders `<button type="button">` by default, or `<a>` when `href` is provided. Disabled links omit `href` and apply `role="link"`, `aria-disabled="true"`, and `tabindex="-1"`.

## FocusScope and Dismissal

Overlay primitives (`Dialog`, `AlertDialog`, `Popover`, `Select`) coordinate focus and dismissal through `monoxus::foundation::overlay`:

- **`FocusScope`**: Tracks active focus boundaries, supports `trap_focus` and `loop_focus`, pauses parent scopes when nested overlays open, and restores focus to the trigger on close.
- **`FocusGuards`**: Installs sentinel boundaries around modal content so `Tab` and `Shift+Tab` cycle cleanly inside the active layer.
- **`DismissLayer`**: Handles `Escape` key dismissal, outside pointer-down detection, and nested branch registration (`OverlayBranchContext`) so interacting with child popovers does not prematurely close parent dialogs.

## RovingFocus

Composite widgets (`Tabs`, `Accordion`, `ToggleGroup`, `RadioGroup`, `Toolbar`) use `RovingFocusRuntime` from `monoxus::foundation::roving_focus`:

- Maintains a single active tab stop (`tabindex="0"`) while inactive items receive `tabindex="-1"`.
- Supports `Horizontal` and `Vertical` orientations, `Home` / `End` jumps, optional `loop_focus` wrapping, and RTL direction awareness via `DirectionProvider`.
- Automatically skips disabled items and reconciles DOM order on mount and unmount.

## Screen-Reader Utilities

Monoxus includes dedicated accessibility primitives for visually hidden labels and decorative icons:

### VisuallyHidden

`VisuallyHidden` renders a `<span>` with a 1x1px clip-rect inline style that removes content from visual layout while keeping it exposed to screen readers:

```rust
use dioxus::prelude::*;
use monoxus::visually_hidden::VisuallyHidden;

rsx! {
    button {
        span { aria_hidden: "true", "×" }
        VisuallyHidden { "Close dialog" }
    }
}
```

### AccessibleIcon

`AccessibleIcon` wraps a decorative icon with `aria-hidden="true"` and renders a sibling `VisuallyHidden` label:

```rust
use dioxus::prelude::*;
use monoxus::accessible_icon::AccessibleIcon;

rsx! {
    AccessibleIcon { label: "Settings",
        svg { width: "16", height: "16", /* ... */ }
    }
}
```
