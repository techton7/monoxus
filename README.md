# monoxus

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

## Example playground

Run the minimal Phase 3.1 / 3.2 / 3.3 playground with:

`dx serve --example playground --web`

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
