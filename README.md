<p align="center">
  <img src="https://raw.githubusercontent.com/techton7/monoxus/main/assets/icon.svg" alt="monoxus logo" width="160" height="160" />
</p>

<h1 align="center">monoxus</h1>

<p align="center">
  <strong>Headless Dioxus primitives and shared foundations for accessible, composable component systems.</strong>
</p>

<p align="center">
  <a href="https://crates.io/crates/monoxus"><img src="https://img.shields.io/crates/v/monoxus.svg" alt="Crates.io" /></a>
  <a href="https://docs.rs/monoxus"><img src="https://docs.rs/monoxus/badge.svg" alt="docs.rs" /></a>
  <a href="LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg" alt="License" /></a>
</p>

---

Headless Dioxus primitives and shared foundations for accessible, composable
component systems.


## Status

Early foundation crate. Implemented foundation slices now provide shared
controllable state, composition/projection, shared utility, and
overlay/positioning contracts in `monoxus::foundation`.

The first primitive family surfaces now live in:

- `monoxus::dialog` for headless `Dialog` modal parts, lifecycle snapshots,
  and `use_dialog_runtime` focus/scroll actuation hooks
- `monoxus::alert_dialog` for thin `AlertDialog` action/cancel extensions over
  the same modal backbone plus `use_alert_dialog_runtime`
- `monoxus::popover` for headless positioned-overlay popover parts, placement
  snapshots, explicit focus/scroll/outside-interaction policy publication, and
  `use_popover_runtime` actuation hooks
- `monoxus::tooltip` for headless tooltip provider/root parts, placement
  snapshots, explicit delay/accessibility policy publication, and
  `use_tooltip_provider_runtime` / `use_tooltip_runtime` coordination hooks
- `monoxus::tabs` for headless tabs and selection navigation parts,
  orientation/direction-aware roving focus coordination, decoupled
  automatic/manual activation modes, and `use_tabs_runtime` actuation hooks

## Platform Support & Native Setup

### Web (`wasm32`, default features `default = []`)

- **Zero `blitz` dependency**: `blitz-*` and `dioxus-native` are strictly gated behind `cfg(not(target_arch = "wasm32"))` and the opt-in `native` feature.
- **Works out-of-the-box**: Builds directly against upstream `dioxus` (`0.8.0-alpha.3`) and `oxidase` (JS-free `web-sys` backend), with no forks or `[patch.crates-io]` overrides required.

### Blitz Native (opt-in `native` feature)

- Enabled via `--no-default-features --features native` (or `features = ["native"]` in `Cargo.toml`), which activates `oxidase/native` targeting the downstream Blitz fork (`https://github.com/techton7/blitz.git`, `techton/main` branch, consumed via pinned release tags such as `techton-main-v0.3.8`).
- **Workspace-root `[patch.crates-io]` required**: Cargo does not propagate `[patch.crates-io]` from dependency crates (`oxidase`). Native consumer apps should add a workspace-root `[patch.crates-io]` pinning the matching `techton7/blitz` release tag (prefer a pinned `tag` paired with `oxidase` over a floating `branch = "techton/main"`):

```toml
[patch.crates-io]
blitz-shell = { git = "https://github.com/techton7/blitz.git", tag = "techton-main-v0.3.8" }
blitz-dom = { git = "https://github.com/techton7/blitz.git", tag = "techton-main-v0.3.8" }
blitz-traits = { git = "https://github.com/techton7/blitz.git", tag = "techton-main-v0.3.8" }
blitz-paint = { git = "https://github.com/techton7/blitz.git", tag = "techton-main-v0.3.8" }
dioxus-native = { git = "https://github.com/techton7/blitz.git", tag = "techton-main-v0.3.8" }
dioxus-native-dom = { git = "https://github.com/techton7/blitz.git", tag = "techton-main-v0.3.8" }
```

### Optional Upstream Patches for Native Consumers (Outside `techton7/blitz`)

The following optional fixes belong to separate upstream repositories (`DioxusLabs/dioxus` and `rust-windowing/winit`), so they cannot be merged into `techton7/blitz`'s `techton/main` branch. When opting into either patch, Cargo requires a workspace-root `[patch.crates-io]` so `crates.io` and `git` instances unify across the dependency graph:

1. **Standalone `asset!()` without `dx` (`manganis`, `manganis-core`, `manganis-macro`, `dioxus-asset-resolver`)**: Upstream `dioxus` `0.8.0-alpha.3` expects `dx` to patch `bundled_path` or `CARGO_MANIFEST_DIR` (set by `cargo run`) to resolve source paths; running a `cargo build` binary directly without `CARGO_MANIFEST_DIR` fails to resolve `asset!()` assets. Tracked upstream at https://github.com/DioxusLabs/dioxus/pull/6025 (not required when running via `dx` or `cargo run`, or when inlining CSS with `include_str!`).
2. **macOS Korean IME Input-Source Switch Fix (`winit`, `winit-appkit` `0.31.0-beta.4`)**: Optional patch mitigating macOS Korean IME daemon cold-start timeout on input source switch and returning `nil` from `inputContext` when IME is disabled. Only needed for Native macOS apps hitting Korean IME issues until merged upstream.

## Example playground

Run the minimal Phase 3.1 / 3.2 / 3.3 playground on **Web** with:

```sh
dx serve --example playground --web
```

Run the playground on **Blitz Native** with `cargo run` (which automatically sets `CARGO_MANIFEST_DIR` so unbundled assets resolve without `dx`), or via `dx`:

```sh
cargo run --example playground --no-default-features --features native
```

## Foundation proof artifact

Phase 2.4 handoff proof lives in
`tests/foundation_phase_2_4_handoff.rs`. The artifact exercises the public
`monoxus::foundation::{state, compose, shared, overlay}` exports together
through wrapper-safe outputs such as `data-state`, `data-side`, `data-align`,
and namespaced `--monoxus-*` geometry variables.

Phase 3.1 dialog-family proof now lives in
`tests/dialog_phase_3_1_handoff.rs`. The artifact exercises the public
`monoxus::dialog` and `monoxus::alert_dialog` surfaces together with the
completed foundation exports for state publication, stable IDs, projection,
portal/presence/focus/dismiss reuse, explicit dialog runtime policy
publication for mode/focus/scroll/outside interaction defaults, library-owned
runtime focus/scroll actuation hooks, and alert action/cancel semantics.

Phase 3.2 positioned-overlay proof now lives in
`tests/positioned_overlay_phase_3_2_handoff.rs`. The artifact exercises the
public `monoxus::popover` and `monoxus::tooltip` surfaces for family-boundary
inventory, shared state/composition/portal/presence/dismiss/floating reuse,
runtime-hook export coverage, and wrapper-safe `data-state`, `data-side`,
`data-align`, and `--monoxus-{family}-*` geometry publication. Live browser
proof now covers popover anchor-relative positioning, focus restore, scroll
lock, and outside dismiss behavior plus tooltip keyboard-focus open, hover
delay, one-open-per-provider handoff, close-on-trigger-click, skip-delay
reopen, and descriptive non-focus behavior, with the remaining Bits-specific
and wrapper-specific differences recorded as approved divergences in the Phase
3.2 spec.

## License

MIT
