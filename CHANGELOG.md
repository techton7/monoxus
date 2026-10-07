# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1](https://github.com/techton7/monoxus/compare/v0.1.0...v0.1.1) - 2026-10-07

### Added

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
