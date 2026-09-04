# monoxus

Headless Dioxus primitives and shared foundations for accessible, composable
component systems.

## Status

Early foundation crate. Implemented foundation slices now provide shared
controllable state, composition/projection, shared utility, and
overlay/positioning contracts in `monoxus::foundation`.

## Foundation proof artifact

Phase 2.4 handoff proof lives in
`tests/foundation_phase_2_4_handoff.rs`. The artifact exercises the public
`monoxus::foundation::{state, compose, shared, overlay}` exports together
through wrapper-safe outputs such as `data-state`, `data-side`, `data-align`,
and namespaced `--monoxus-*` geometry variables.

## License

MIT
