---
title: Introduction
description: Headless, accessible, composable UI primitives for Dioxus on Web and Blitz Native.
---

Monoxus is a collection of unstyled, accessible, and composable UI primitives for [Dioxus](https://dioxuslabs.com). It provides WAI-ARIA compliant state machines, keyboard navigation, focus management, and floating positioning while leaving all styling and visual design to your application.

## Why Monoxus

Building interactive UI components such as dialogs, selects, tabs, and tooltips from scratch requires handling subtle accessibility, focus, and pointer edge cases:

- **WAI-ARIA semantics**: Roles, states, and relationship attributes (`aria-controls`, `aria-labelledby`, `aria-describedby`, `aria-activedescendant`) are wired automatically across compound parts.
- **Keyboard and focus management**: Roving tabindex, arrow-key navigation, typeahead search, focus trapping, and focus restoration on close are built into each primitive's runtime.
- **Zero visual opinions**: Primitives ship without default stylesheets. Every part exposes predictable `data-state`, `data-side`, `data-align`, `data-orientation`, and `data-disabled` attributes along with CSS custom properties.
- **Composable by default**: Compound parts communicate over Dioxus context, and `AsChildSlot` / `Slottable` let you project behavior onto your own elements when needed.

## Dual-Host Architecture

Monoxus is designed from the ground up to run on both **Web (WASM)** and **Blitz Native** from a single component API:

- **`monoxus-core`**: Pure headless Rust state machines, deterministic ID relationship builders, attribute projectors, and layout math with zero dependency on `dioxus`, `web-sys`, or browser APIs.
- **`monoxus`**: Dioxus components, reactive signals, and event handlers that delegate DOM, focus, observer, and timing operations to the host-neutral `oxidase` substrate.

Switch between Web and Blitz Native by toggling crate features (`default` vs `native`) without changing your RSX or component code.
