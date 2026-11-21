# `monoxus-core` Extraction Ledger

## 1. Purpose & Workspace Topology

`monoxus-core` is a public MIT-licensed pure Rust crate at `crates/monoxus-core` inside the `monoxus` Cargo workspace (`[workspace] members = [".", "crates/monoxus-core"]`, `default-members = ["."]`, `resolver = "2"`).

### Dependency & Licensing Invariants

- **`monoxus-core` (MIT)**: Zero dependencies on `dioxus`, `oxidase`, `oxidase-floating-ui`, `web-sys`, or any `triui*` crate. Only `serde` (`default = ["serde"]`, optional) is used for pure serializable layout/state DTOs.
- **`monoxus` (`ui/monoxus`, MIT)**: Depends on `monoxus-core`, `dioxus` (`0.8.0-alpha.3`), and `oxidase`. Zero dependencies on `triui*`.
- **`triui-monoxus` (`graphics/triui-monoxus`, PROPRIETARY)**: Depends on `monoxus-core`, `dioxus` (`0.7.0`), and `triui-trioxus`.
- **Zero Broken Public APIs**: Both `monoxus` and `triui-monoxus` re-export all extracted symbols at their existing module paths (`pub use monoxus_core::...`).

---

## 2. Foundation Extraction Boundary

| Module | Extracted to `monoxus-core` | Retained in `ui/monoxus` | Retained in `triui-monoxus` | Reconciliation Notes |
|---|---|---|---|---|
| `foundation/state.rs` | `DataState`, `ControllableStateCell` | `ControllableStateProps`, `ControllableStateHandle`, `ControllableStateReducerProps`, `ControllableStateReducerHandle`, `use_controllable_state*` | `use_controllable_state*` | 6 Dioxus hook/signal lines remain in adapters. |
| `foundation/shared.rs` | `data_disabled_attr`, `ScopeHandle`, `CollectionEntry`, `CollectionRegistry`, `Direction`, `Orientation`, `RovingFocusIntent`, `RovingFocusKey`, `RovingFocusController`, `resolve_direction` | `DirectionContext`, `DirectionProvider`, `use_direction_context`, `use_mount_animation_guard`, `use_stable_id` | `DirectionContext`, `DirectionProvider`, `use_direction_context`, `use_mount_animation_guard`, `use_stable_id`, `resolve_interactive_trigger_class` | `resolve_interactive_trigger_class` is canvas Tailwind-specific if not pure shared; keep in `triui-monoxus` or document parity. |
| `foundation/compose.rs` | `AsChildSlotError`, `AsChildSlot<T>`, `Slottable`, `project_as_child`, `compose_part_event_handlers`, pure slot helpers | `MountedHandle = Rc<MountedData>` | `MountedHandle = Rc<MountedData>` (or canvas equivalent) | Only `MountedHandle` depends on `dioxus::prelude::MountedData`. |
| `foundation/roving_focus.rs` | `RovingFocusConfig`, `RovingCandidate`, `RovingModifiers`, `compute_tab_index`, `is_navigation_key`, pure candidate selection helpers & re-exports | `keyboard_types` key translation, `use_roving_focus_*` Dioxus hooks, `oxidase` DOM queries | `use_roving_focus` over `TriuiHostBridge` | Splits pure roving-focus math from Dioxus hooks & DOM/canvas bridges. |
| `foundation/overlay/dismiss.rs` | `DismissLayer<T>` & pure dismiss types | Adapter wrapper / `oxidase` bridge | Adapter wrapper / `TaffyDriver` hit-path bridge | 100% identical across both crates. |
| `foundation/overlay/focus.rs` | `FocusScope<T>`, `FocusGuards<T>` | `oxidase` focus trap hooks | `FocusScope` / `TaffyDriver` focus bridge | 100% identical across both crates. |
| `foundation/overlay/portal.rs` | `PortalHost` | Adapter portal mount | Adapter portal mount | Superset includes `host_attr_value()` and `as_str()` from `triui-monoxus`. |
| `foundation/overlay/presence.rs` | `PresenceState`, `Presence`, `PresenceCloseCycleId`, `PresenceControllerUpdate`, `PresenceController`, type aliases `PresencePhase`, `PresenceUpdate` | `use_presence*` Dioxus/`oxidase` hooks | `use_presence*` Dioxus/`kinetocore` hooks | Superset reconciles `ui/monoxus/src/foundation/overlay/presence.rs` and `triui-monoxus/src/foundation/presence.rs`. |
| `foundation/overlay/grace.rs` | `Point2D`, `Rect2D`, `CardinalSide`, `DEFAULT_GRACE_PADDING`, polygon & convex hull helpers, `create_grace_area` | Adapter pointer stream | Adapter pointer move | Reconciled superset of both pure geometry modules. |
| `foundation/overlay/floating.rs` | `Rect`, `Size`, `PlacementSide`, `PlacementAlign`, `StickyStrategy`, `FloatingLayer`, `FloatingPlacement`, `PureOverlayPositionConfig`, `compute_pure_overlay_position`, `arrow_position`, `position_in_boundary`, `reference_is_hidden*` | `OverlayPositionConfig`, `compute_overlay_position` (`oxidase_floating_ui` bridge) | `compute_overlay_position` alias to pure helper | `oxidase_floating_ui` stays strictly in `ui/monoxus`. |
| `foundation/browser/`, `foundation/form.rs`, `foundation/host_bridge.rs` | Not extracted | `browser/`, `form.rs` | `host_bridge.rs` | Host-specific DOM/Canvas bridges stay in adapters. |

---

## 3. Component Extraction Boundary (All 21 Components)

### Wave A: High-Similarity Shared Components (6)

1. **`collapsible`**: `types`, `relationships`, `state`, `attrs` (98–100% identical). `runtime.rs` & `components/` stay in adapters.
2. **`popover`**: `types`, `relationships`, `state`, `attrs` (100% identical). `runtime.rs` & `components/` stay in adapters.
3. **`tooltip`**: `types`, `relationships`, `state`, `attrs`. Superset keeps `TooltipContentAttributes::{with_aria_label, aria_label}` and `pub const TOOLTIP_HOVER_TRANSFER_GRACE_MS: u64 = 40`. `TooltipMountedHandle` stays in adapters.
4. **`accordion`**: `types`, `relationships`, `state`, `attrs`. Superset keeps `Copy, Hash` on `AccordionMode`, empty-string guard in `with_value`/`with_values`, and `content_with_disabled` + `disabled`/`is_disabled`/`data_disabled` on `AccordionContentAttributes`.
5. **`tabs`**: `types`, `relationships`, `state`, `attrs`. Superset keeps `Tabs::roving_config(&self) -> RovingFocusConfig`, `Tabs::is_navigation_key`, and `disabled`/`is_disabled`/`data_disabled` on `TabsRootAttributes` & `TabsContentAttributes` (using `data_disabled_attr` so `false` returns `None`).
6. **`select`**: `types`, `relationships`, `state`, `attrs`, `registry`. Superset keeps `stale_ids` replacement in `MountedOptionRegistry::register`. `runtime.rs`, `selection.rs`, `typeahead.rs`, `monitors.rs`, `browser.rs`, `presence.rs`, RSX part files stay in adapters.

### Wave B: Divergent Shared Components (3)

7. **`dialog`**: `types`, `relationships`, `state`, `attrs`. Superset retains ALL policies & accessors from `ui/monoxus` (`open_focus_policy`, `close_focus_policy`, `scroll_lock_policy`, `outside_interaction_policy`, `dismiss_layer`, `focus_scope`, `focus_guards`, `lifecycle`) AND `triui-monoxus` (`with_portal_host`, `content_with_role`, `target_content_id`, `aria_controls`, `is_disabled`, `data_disabled`). `DialogMountedHandle` stays in adapters.
8. **`alert_dialog`**: `types`, `state`, `attrs`. Extracted from `ui/monoxus/src/alert_dialog.rs` and `triui-monoxus/src/alert_dialog/{types,state,attrs}.rs` into `monoxus_core::alert_dialog` preserving all constructors/accessors (`with_portal_host`, `lifecycle`, `action_id`, `cancel_id`, `Action`/`Cancel` attributes).
9. **`toast`**: `types`, `attrs`, `gesture`, `layout` (pure subset only).
   - **Extracted**: `ToastId`, `ToastType`, `ToastPosition`, `SwipeDirection`, `ToastActionEvent`, `SwipeConfig`, `SwipeState`, `ToastComputedStyle` (including `to_style_string()` pure string formatter with `serde` derives), `ToastStackLayout::compute`, pure `Toast*Attributes` & `ToastPhase`.
   - **Retained in Adapters**: `Callback`-carrying `ToastAction`, `ToastOptions`, `ToastData`, `ToastStore`, `ToastRuntime`, `ToastInstant`, `ToastCanvasTransform`/`with_mounted` (`triui-monoxus`), `hotkey.rs`, `components/`.

### Wave C: Unported `ui/monoxus` Components (12, Pure Layer Pre-Extracted)

- **6 Compound Directories** (`avatar`, `checkbox`, `progress`, `radio_group`, `switch`, `toggle_group`): Extract `types`, `relationships`, `state`, `attrs` (`runtime.rs` & `components.rs` stay in `ui/monoxus`). Note: `avatar/types.rs` checks for any `oxidase` usage and excludes DOM-only parts if present.
- **6 Single-File Primitives** (`aspect_ratio.rs`, `button.rs`, `label.rs`, `meter.rs`, `separator.rs`, `toggle.rs`): Extract pure structs, enums, constants, `attributes()`, `project_*_as_child`, and pure unit tests to `monoxus-core`; retain `#[component]`, `*Props`, `Element`, and `dioxus`/`oxidase` imports in `ui/monoxus`.
