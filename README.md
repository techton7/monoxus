# monoxus

Headless Dioxus primitives and shared foundations for accessible, composable
component systems.

## Status

Early foundation crate. Implemented foundation slices now provide shared
controllable state, composition/projection, shared utility, and
overlay/positioning contracts in `monoxus::foundation`.

The first primitive family surfaces now live in:

- `monoxus::dialog` for headless `Dialog` modal parts and lifecycle snapshots
- `monoxus::alert_dialog` for thin `AlertDialog` action/cancel extensions over
  the same modal backbone

## Example playground

Run the minimal Phase 3.1 dialog-family playground with:

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
portal/presence/focus/dismiss reuse, and alert action/cancel semantics.

## License

MIT
