# `monoxus/site` Authoring Guide

Conventions for per-page and per-component files not enforced automatically by shared templates (`DocsLayout`, `DocPageHeader`, `ComponentPreview`, `PropsTable`).

## API Schema (`src/api_schema/<slug>.rs`)

- **Prop grouping**: Classify every prop with `PropGroup`: `Core` (component behavior/state/callbacks), `Common` (`class`, `style`, `id`, `children`, `as_child`), or `HtmlAria` (forwarded HTML/ARIA/data attributes).
- **Types & defaults**: Set `type_simple` by stripping outer `Option<...>` (e.g., `bool`, `Side`) and `type_full` to the exact Rust type (`Option<bool>`). Write enum defaults as `Type::Variant` (e.g., `Side::Bottom`). Preserve exact `r#` raw identifiers (e.g., `r#type`, `r#for`).
- **Source fidelity**: Verify `features`, `anatomy`, `data_attrs`, `css_vars`, and `keyboard` directly against `../src` and `../crates/monoxus-core/src`; never invent unsupported attributes or key bindings.

## Interactive Demos (`src/demos/<slug>_demo.rs`)

- **Idiomatic `monoxus`**: Avoid redundant controlled `on_*_change` callbacks that double-write an already-bound `Signal<T>`. Omit manual `id` props unless a fixed DOM ID is required.
- **Shared providers**: Do not pass per-root `TooltipRoot` delay overrides that detach a tooltip from the shared `TooltipProvider`.
- **Styling**: Use the site's semantic Tailwind tokens (`bg-surface`, `text-foreground`, `border-border`, etc.) so demos follow light/dark themes.

## Markdown Docs (`content/**/*.md`)

- **Overview prose**: Lead overview pages with the Web (WASM) + Blitz Native value proposition. Link project names (`[Dioxus]`, `[monoxus]`, `[monoxus-core]`, `[oxidase]`, `[Blitz]`) instead of using inline backticks. Avoid dumping raw `aria-*` / `data-*` attribute lists in overview prose.
- **Scope**: Keep `Getting Started` focused on installation and a hands-on first example; place deeper state, styling, and composition concepts in `content/guides/`.

## New Component Checklist & Verification

1. Add `src/api_schema/<slug>.rs` and register it in `src/api_schema/mod.rs`.
2. Add `src/demos/<slug>_demo.rs` and wire it in `src/demos/mod.rs`.
3. Verify sidebar/search generation via `src/docs_data.rs` (and `build.rs`).
4. Run checks from `monoxus/site`:
   - `cargo test --test api_drift_test`
   - `node --test worker/redirects.test.ts`
   - `dx build --release --platform web && ./scripts/check_export.sh`
