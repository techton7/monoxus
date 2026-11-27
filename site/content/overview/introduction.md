---
title: Introduction
description: Headless, accessible, composable UI primitives for Dioxus on Web and Blitz Native.
---

[monoxus](https://github.com/techton7/monoxus) provides unstyled, accessible, and composable Rust UI primitives for [Dioxus](https://dioxuslabs.com) that run identically on both **Web (WASM)** and **[Blitz](https://github.com/DioxusLabs/blitz) Native** from a single component API. It delivers WAI-ARIA state machines, keyboard navigation, focus management, and floating positioning while leaving all styling and visual design to your application.

## Why Monoxus

Building interactive UI components such as dialogs, selects, tabs, and tooltips from scratch requires handling subtle accessibility, focus, and pointer edge cases:

- **WAI-ARIA semantics**: Roles, states, and element relationships are wired automatically across compound parts without manual ID bookkeeping.
- **Keyboard and focus management**: Roving tabindex, arrow-key navigation, typeahead search, focus trapping, and focus restoration on close are built into each primitive's runtime.
- **Zero visual opinions**: Primitives ship without default stylesheets and expose predictable state attributes and CSS custom properties for styling with Tailwind, CSS, or custom tokens.
- **Dual-host Web + Blitz Native parity**: Write your components once and target both browser WASM and [Blitz](https://github.com/DioxusLabs/blitz) Native without diverging RSX or behavior.
- **Composable by default**: Compound parts communicate over [Dioxus](https://dioxuslabs.com) context, and `AsChildSlot` / `Slottable` let you project behavior onto your own elements when needed.

## Dual-Host Architecture

[monoxus](https://github.com/techton7/monoxus) separates pure UI behavior from host rendering so the same component API works across **Web (WASM)** and **[Blitz](https://github.com/DioxusLabs/blitz) Native**:

- **[monoxus-core](https://github.com/techton7/monoxus/tree/main/crates/monoxus-core)**: Pure headless Rust state machines, deterministic ID relationship builders, attribute projectors, and layout math with zero dependency on [Dioxus](https://dioxuslabs.com), `web-sys`, or browser APIs.
- **[monoxus](https://github.com/techton7/monoxus)**: [Dioxus](https://dioxuslabs.com) components, reactive signals, and event handlers that delegate DOM, focus, observer, and timing operations to the host-neutral [oxidase](https://github.com/techton7/oxidase) substrate.

Switch between Web and [Blitz](https://github.com/DioxusLabs/blitz) Native by toggling crate features (`default` vs `native`) without changing your RSX or component code.

> **Looking ahead**: [monoxus-core](https://github.com/techton7/monoxus/tree/main/crates/monoxus-core) is planned to evolve into a standalone framework-agnostic Rust UI behavior core (`ui-sdk`) without breaking [monoxus](https://github.com/techton7/monoxus)'s [Dioxus](https://dioxuslabs.com) API.
