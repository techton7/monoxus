# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.1](https://github.com/techton7/monoxus/compare/monoxus-v0.2.0...monoxus-v0.2.1) - 2026-11-28

### Added

- *(monoxus)* upgrade `ui-sdk` to `0.2.2` and re-export `monoxus::agent` alongside `monoxus::kernel`
- *(monoxus)* add `on_*_change_details: Option<EventHandler<ChangeEvent<T>>>` across all stateful primitive roots backed by `TransitionGate` LIFO `Propose -> Veto -> Commit` with `ev.cancel()` side-effect ordering guarantees and dynamic overlay Escape / Outside-Press `ev.allow_propagation()`
- *(monoxus)* add `monoxus::agent` runtime (`AgentProvider`, `<AgentBoundary>`, `AgentBridgeHandle::{handle_request, handle_json, handle_request_settled, handle_json_settled}`, `use_agent_surface`, and `wasm32` WebMCP bridge strictly exposing `["snapshot", "query", "act"]`) wired across all interactive primitives with zero-allocation fast path when `<AgentProvider>` is absent
- *(monoxus)* add dev-only diagnostics (`monoxus::foundation::diagnostics`) covering `ControlledRefusal` (with 5 false-positive guards), `ControlledModeSwitch`, `ControlledAndDefaultProvided`, `DetachedHandleAction`, `HandleOverlap`, and `InvalidProp` (compiling to inline no-ops in release)
- *(monoxus)* add detached triggers and typed `OverlayHandle<K, P = ()>` (`DialogHandle`, `AlertDialogHandle`, `PopoverHandle`, `TooltipHandle`, `HoverCardHandle`) with non-generic `OverlayHandleProp<K>`, reactive `handle.payload()`, multi-trigger switching without closing, floating re-anchoring, and LIFO attach stack
- *(monoxus)* add `<MonoxusConfig>` cascading context (`dir`, `default_portal_to`, `default_locale`) with bidirectional `DirectionProvider` inheritance, 7 `*Portal` host inheritance, `HoverCard`/`ToastViewport` `dir` inheritance, and `locale: Option<CalendarLocale>` across all 8 Date/Time/Calendar roots
- *(monoxus)* add nested `DialogOverlay` auto-suppression (`force_render: bool = false`), `is_topmost_dialog()` backdrop click guard, `data-nested`/`data-nested-dialog-open`, and `--nested-dialogs` CSS custom property across `Dialog` and `AlertDialog`
- *(monoxus)* add async pending `Button` (`pending`, `focusable_when_disabled`, `pending_announcement`, `on_press`, `on_action`, `use_async_action()`) with shared polite live-region announcer (`monoxus::foundation::announce`) excluded from modal DOM shielding
- *(monoxus)* expose `ProgressRoot.min: f64 = 0.0` with dynamic `data-min` on Root/Indicator, `runtime.min()`/`runtime.percentage()`, and `ProgressValidation::InvalidMin` diagnostics
- *(site)* sync all 45 component API schemas and documentation to `v0.2.1` (0 drift)

## [0.2.0](https://github.com/techton7/monoxus/compare/monoxus-v0.1.2...monoxus-v0.2.0) - 2026-11-28

### Changed

- *(monoxus)* depend directly on published `ui-sdk = "0.2.0"` (crates.io) across all 45 public primitives + internal `menu`, relocating `[KEEP-MONOXUS-BRAND]` shims under `src/compat/` and deprecating `crates/monoxus-core`
- *(monoxus)* normalize all 45 primitives to single `child: Option<Callback<Vec<Attribute>, Element>>` (and `FloatingContentChildProps` for floating overlays) powered by `merge_props` (`class`/`style`/`aria-describedby` merging and user-first `ListenerCallback` chaining with `prevent_default()` veto)
- *(monoxus)* standardize controlled state to value-based `Option<T>` + `default_*` + `on_*_change` (removing `Option<Signal<T>>` / `controlled_*` props) with zero-frame-flash controlled veto
- *(monoxus)* relocate floating placement, portal host, focus, scroll-lock, and 4-way `DismissBehavior` policies from overlay/picker roots to `*Content` / `*Portal` and convert overlay & `Select` roots to non-DOM fragments
- *(monoxus)* modernize `Select` (unit `SelectMode::Single | Multiple`, context-wired `SelectGroup`/`SelectLabel`, auto-hiding `SelectItemIndicator`, auto-scrolling `SelectScroll{Up,Down}Button`, explicit `SelectPortalProps`/`SelectHiddenInputProps`, `Select` root alias)
- *(monoxus)* add `TransitionStatus` (`data-starting-style` / `data-ending-style`), `keep_mounted`, `Portal.disabled`, `TabsRoot.disabled`, `TooltipRoot` shared provider override preservation, and remove redundant pass-through `aria_*` / `test_id` props
- *(site)* sync `monoxus/site` MVP 5 component schemas, interactive demos, Composition/Styling guides, and API Reference tables to `v0.2.0`

## [0.1.1](https://github.com/techton7/monoxus/compare/monoxus-v0.1.0...monoxus-v0.1.1) - 2026-11-25

### Added

- *(monoxus)* implement Batch 3 (Sub-wave 4B command/combobox and Sub-wave 5C date/time fields and pickers)
- *(monoxus)* add Calendar/RangeCalendar select as_child composition, Select disabled sync, and playground parity
- *(monoxus)* implement Batch 2 (3B menu/dropdown/context/menubar, 3C navigation_menu, 5B calendar/range_calendar), dual-host QA fixes, README host guide, and sync deps
- *(monoxus)* implement Batch 1 (3A hover_card/toolbar, 4A pagination/pin_input, 5A visually_hidden/accessible_icon/rating_group) and sync deps
- *(monoxus-core)* sync main Tier 2B, extract slider/scroll_area and disclosure helpers, update verify gate
- *(monoxus)* implement Tier 2 Sub-wave 2B primitives, live fixes, and blitz v0.3.4 sync
- *(monoxus-core)* extract pure Rust core crate and wire monoxus workspace
- *(playground)* redesign Progress/Meter demos and approve passing Tier 2A GUI baselines
- *(monoxus)* implement Tier 1 Sub-wave 1B checkbox, switch, radio_group, and shared form substrate
- *(monoxus)* complete Track 1-A remediation and Tier 1 Sub-wave 1A primitives
- *(monoxus)* complete reference-aligned toast runtime and crate-wide zero-cfg oxidase convergence
- *(monoxus)* integrate cross-platform pointer capture, focus-visible modality, and unmount safety
- *(monoxus)* complete Space/Enter keyboard activation parity across primitives
- *(select)* align floating auto-update mount lifecycle with W3C reference and prove native click visibility
- *(overlay)* guard floating readiness against unmeasured content dimensions
- *(assets)* patch upstream manganis standalone assets and add verification test
- *(monoxus)* standardize floating origin measurement and align select scroll with radix isPositioned lifecycle
- *(native)* integrate oxidase v0.2.0 animation/transition lifecycle and eliminate test sleep fallbacks
- *(playground)* inject Pretendard font-face via inline style for Blitz Native resolution
- *(css)* embed data URI for Pretendard font-face to support Blitz Native resolution
- *(playground)* integrate Pretendard v1.3.9 variable font across web and native
- *(overlay)* configure fixed positioning strategy for upstream oxidase-floating-ui integration
- *(verify)* add dual-host data/gesture tests, web runner scripts, and native screenshot
- *(testing)* add cross-host verification infrastructure with headless blitz harness and ego-browser web driver
- *(playground)* normalize single scenario source with #[oxidase::main], CollapsiblePage, and 9-primitive semantic telemetry
- *(select)* wire host-tree descendant order query with fail-closed semantics and black-box verification
- *(monoxus)* complete Milestone T-1 declarative compound surface and pure Rust runtime convergence
- *(foundation)* downstream migration to oxidase watchers and pure rust overlay positioning
- *(playground)* unify declarative compound components and integrate Tailwind CSS v4
- *(components)* implement declarative compound components for Dialog, AlertDialog, and Tabs (Track A)
- integrate oxidase dependency and apply #[watcher(raf)] coalescing
- *(browser)* integrate dioxus-js-interop geometry and stabilize overlay runtime foundations
- complete dioxus-js-bindgen TS migration and add release-plz pipeline
- *(browser)* integrate dioxus-js-bindgen for dom helpers and form_reset watcher
- *(playground)* add symmetrical showcase motion and single-owned keyframes for dialog and alert-dialog
- *(tooltip)* implement safe polygon and hover transit protection (phase 3.7.B)
- *(popover)* gate showcase motion on live positioning
- make playground exit animations visible
- align playground overlay animations with radix showcase
- retrofit overlay presence runtimes
- add shared presence runtime core
- *(toast)* support 6 screen positions and direction-specific swipe-out animations
- *(toast)* implement monoxus toast and stacked notification primitive
- *(select)* add focused runtime proofs and unit tests for force_mount, prevent_overflow_text_selection, and custom_anchor
- *(select)* complete DEC-P3.5-008 content surface, floating props, and preventable outside dismissal
- *(select)* complete Phase 3.5 parity remediation and scrollIntoView nearest
- *(select)* dynamic flip on scroll using floating auto-update monitor
- *(select)* conditional focus restoration on tab and highlighted/disabled styling
- *(select)* implement viewport collision flip, outside click dismiss, and portal support
- *(select)* implement Phase 3.5 Select and Ordered Overlay Selection primitive family
- *(accordion)* implement Collapsible and Accordion disclosure primitives
- implement reference-faithful tabs primitive family with roving focus
- add reference-aligned popover and tooltip runtime
- *(dialog)* add runtime focus and scroll actuation
- *(examples)* improve playground modal presentation
- *(examples)* add playground dialog demo
- *(dialog)* add Phase 3.1 modal family surface
- *(foundation)* add overlay and positioning backbone
- *(foundation)* add composition utilities substrate

### Fixed

- *(monoxus)* fix tooltip custom trigger id, slider focus ring, and scroll area viewport focus styling
- *(select)* guard open alignment, skip touch hover highlight, and deduplicate floating updates
- *(playground)* checkbox checked hover specificity; gui defaults cross-host-parity, animations disabled
- *(select)* restore scrollable auto-scroll, static content keyboard navigation, and post-scroll dismiss/anchor coverage
- *(playground)* visualize progress/meter bars and sync tailwind css
- *(ci)* delegate manganis patch to gitignored .cargo/config.toml for CI compatibility
- *(overlay)* sync floating recalculation on scroll, wrap frame in scope, and add wheel alignment test
- *(monoxus)* align ScrollBehavior to Auto and update dogfooding test baselines
- *(select)* resolve clippy collapsible if in set_highlighted
- *(popover)* eliminate synthetic viewport geometry and compact playground demo
- *(select)* use mounted measurements for native floating positioning
- *(playground)* align dependencies and satisfy Dioxus hook rules
- *(overlay)* refine dual-host tooltip placement timing and popover outside dismiss
- *(dialog)* repair declarative presence ID-sync, lifecycle gating, and exit animations
- specify crates.io dependency in Cargo.toml and use local patch config
- finalize overlay presence runtime proof
- *(toast)* prevent text selection during swipe gesture with data-swiped and user-select none
- *(select)* restore 1804199 scroll_element_into_view_nearest on highlight update
- *(select)* fix floating collision flip evaluation and prevent prop stomping on re-render
- *(select)* settle preventable escape, floating props decoupling, label formatting, and uncontrolled form
- *(select)* resolve portal teleport, escape prevention, and label parity
- *(select)* unconditionally track is_open reactive effect for multi-open flip positioning
- *(select)* document-order roving synchronization and tab dismiss unit test
- *(select)* use restore_focus_element_by_id with preventScroll: true to eliminate screen jump on open/close

### Other

- *(monoxus)* sync blitz-host dev-dependency to blitz-host-v0.3.7
- *(monoxus)* restore crates.io winit source in Cargo.lock
- *(gui)* update GUI manifest coverage guard for all 44 primitives
- Merge branch 'chore/gui-tests-backfill'
- *(monoxus)* sync blitz v0.3.6, oxidase v0.4.3, and blitz-host v0.3.3 remote tags
- *(monoxus)* depend on oxidase-v0.4.2 and blitz-host-v0.3.2 tags
- *(monoxus)* sync blitz tag to techton-main-v0.3.5
- Merge branch 'feat/tier2-subwave-2b' into main
- *(gui)* backfill tabs-initial baselines and remove polling thread from playground
- *(gui)* backfill dual-host visual parity baselines across component suites
- *(gui)* backfill GUI manifests for portions 2+3
- *(gui)* add Tier 2A dioxus-gui-test manifests and test ids
- Add Tier 2 progress meter aspect ratio and avatar
- Verify event-driven presence and clarify demos
- Align monoxus test coverage guard name
- Keep fixture warning suppression local
- Optimize monoxus build and test infrastructure
- lock manganis and dioxus-asset-resolver to local v0.8.0-alpha.3 PR #6025 patches
- *(deps)* update Cargo.lock after dx build
- *(deps)* upgrade monoxus to Dioxus 0.8.0-alpha.3, blitz-host-v0.3.0, oxidase-v0.4.0
- *(deps)* bump blitz dependencies to techton-main-v0.2.13
- *(deps)* bump blitz to techton-main-v0.2.12 for falsy boolean attribute fix
- *(deps)* bump oxidase pin to oxidase-v0.3.3
- *(select)* add bottom collision upward flip regression test without wheel scroll
- *(deps)* bump blitz to techton-main-v0.2.9 and oxidase to oxidase-v0.3.2
- *(deps)* bump oxidase to oxidase-v0.2.1
- *(deps)* bump blitz packages to techton-main-v0.2.7
- *(deps)* bump blitz dependencies to techton-main-v0.2.6
- *(monoxus)* dogfood native animationend exit unmounting without sleep fallbacks
- *(deps)* update blitz dependencies to techton-main-v0.2.4
- *(ignore)* ignore test-results in monoxus
- *(package)* exclude GUI test assets from crate archives
- *(gui)* add package-local monoxus GUI suite
- *(foundation)* introduce spawn_root_task for root-scoped async tasks and eliminate temporary safe_spawn
- *(gesture)* verify select typeahead and toast swipe gesture across hosts
- *(monoxus)* add dual-host verification for overlay and floating primitives (T-2.4)
- *(monoxus)* add dual-host verification for non-floating primitives (T-2.3)
- add dev package opt-level 3 profile
- *(deps)* align oxidase dependency to tagged release v0.1.8 with passthrough features
- *(deps)* update oxidase lockfile to 0.1.8
- *(cargo)* document local patch workflow and CI crates.io dependency rules
- *(tests)* modularize root integration test suites preserving entry crate files
- *(tabs)* modularize tabs compound primitive subsystem
- *(dialog_accordion)* modularize dialog and accordion subsystems
- *(positioned_overlay)* modularize popover and tooltip subsystems
- *(foundation)* modularize overlay subsystem and colocate select browser RPCs
- *(browser)* migrate inline JS strings and eval sites to dioxus-use-js and modular foundation
- *(select)* fundamental split, contract fixes & multi-page playground
- decouple popover and tooltip from dialog into foundation utilities
- *(foundation)* add Phase 2.4 handoff proof
