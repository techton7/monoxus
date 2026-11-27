---
title: Styling
description: Style Monoxus primitives using data-* state attributes, CSS custom properties, Tailwind CSS v4, or plain CSS.
---

Monoxus primitives ship with zero visual styles. Every component accepts `class` and `style` props and exposes semantic `data-*` attributes and CSS custom properties so you can style any state declaratively.

## Data Attributes

Primitives reflect their interactive state and layout geometry through standard `data-*` attributes:

| Attribute | Common Values | Emitted By |
|---|---|---|
| `data-state` | `"open"` / `"closed"`, `"active"` / `"inactive"`, `"checked"` / `"unchecked"` | `Dialog`, `Select`, `Tabs`, `Tooltip`, `Checkbox`, `Switch` |
| `data-side` | `"top"` \| `"right"` \| `"bottom"` \| `"left"` | `TooltipContent`, `TooltipArrow`, `SelectContent`, `PopoverContent` |
| `data-align` | `"start"` \| `"center"` \| `"end"` | `TooltipContent`, `TooltipArrow`, `SelectContent`, `PopoverContent` |
| `data-orientation` | `"horizontal"` \| `"vertical"` | `TabsRoot`, `TabsList`, `TabsTrigger`, `TabsContent`, `Separator` |
| `data-disabled` | `""` (empty string) when disabled, omitted when enabled | `Button`, `DialogTrigger`, `SelectTrigger`, `SelectItem`, `TabsTrigger`, `TooltipTrigger` |

Because `data-disabled` is omitted entirely when enabled, `[data-disabled]` and `:not([data-disabled])` selectors match reliably without needing `[data-disabled="true"]`.

## CSS Custom Properties

Floating and disclosure primitives expose measured geometry variables in both `--monoxus-*` and `--radix-*` compatibility namespaces:

- **Transform origin**: `--monoxus-<primitive>-transform-origin-x`, `--monoxus-<primitive>-transform-origin-y`, and `--radix-<primitive>-content-transform-origin`
- **Available space**: `--monoxus-<primitive>-available-width`, `--monoxus-<primitive>-available-height`, and `--radix-<primitive>-content-available-{width,height}`
- **Anchor dimensions**: `--monoxus-<primitive>-anchor-width`, `--monoxus-<primitive>-anchor-height`, and `--radix-<primitive>-trigger-{width,height}`
- **Content dimensions**: `--monoxus-<primitive>-content-width`, `--monoxus-<primitive>-content-height`

## Computed `z-index` Hoisting and `GlobalAttributes`

Floating overlays (`PopoverContent`, `TooltipContent`, `SelectContent`, `HoverCardContent`, and menus) render inside a positioned wrapper (`[data-monoxus-floating-content-wrapper]`). Monoxus automatically reads the computed `z-index` from your `*Content` element and hoists it onto the wrapper:

- Apply stacking classes such as `z-50` directly on `*Content`; you do not need to style the outer wrapper or add `relative` unless an `absolute` child (such as `TooltipArrow`) requires a positioned parent.
- Explicit `class`, `style`, and extra `GlobalAttributes` (such as `data-*` or `aria-*`) are merged cleanly with primitive-owned styles and attributes.

## Plain CSS Example

Target `data-state`, `data-side`, and `data-disabled` directly in your stylesheet:

```css
.tabs-trigger {
  padding: 0.5rem 0.75rem;
  border-bottom: 2px solid transparent;
  color: var(--muted-fg);
}

.tabs-trigger[data-state="active"] {
  border-bottom-color: var(--accent);
  color: var(--fg);
}

.tabs-trigger[data-disabled] {
  opacity: 0.45;
  cursor: not-allowed;
}

.select-content {
  max-height: var(--monoxus-select-available-height);
  transform-origin: var(--radix-select-content-transform-origin);
}
```

## Tailwind CSS v4 Example

Use Tailwind's `data-[...]:` variants directly in your RSX `class` prop:

```rust
use dioxus::prelude::*;
use monoxus::tabs::{TabsContent, TabsList, TabsRoot, TabsTrigger};

#[component]
pub fn StyledTabs() -> Element {
    rsx! {
        TabsRoot { default_value: Some("general".into()),
            TabsList { class: "inline-flex gap-1 rounded-lg bg-zinc-100 p-1 dark:bg-zinc-900",
                TabsTrigger {
                    value: "general".to_string(),
                    class: "rounded-md px-3 py-1.5 text-sm font-medium text-zinc-600 data-[state=active]:bg-white data-[state=active]:text-zinc-950 data-[disabled]:opacity-50",
                    "General"
                }
                TabsTrigger {
                    value: "security".to_string(),
                    class: "rounded-md px-3 py-1.5 text-sm font-medium text-zinc-600 data-[state=active]:bg-white data-[state=active]:text-zinc-950 data-[disabled]:opacity-50",
                    "Security"
                }
            }
            TabsContent { value: "general".to_string(), class: "mt-3 text-sm",
                "General account settings."
            }
            TabsContent { value: "security".to_string(), class: "mt-3 text-sm",
                "Password and two-factor authentication."
            }
        }
    }
}
```
