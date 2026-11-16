//! Headless stacked notification primitive (`monoxus::toast`).
//!
//! Provides WAI-ARIA live-region notifications (`role="status"` / `role="alert"` inside a
//! `role="region"` landmark `<ol>`), imperative queue dispatch (`toast::*`), live border-box
//! card height measurement, hover/focus stack expansion, timer pause/resume, W3C pointer-capture
//! swipe gestures, and landmark hotkey (`F8`) / `Escape` keyboard navigation across Web (`wasm32`)
//! and Blitz Native substrates without custom JS bridges.
//!
//! # Headless Styling Contract
//!
//! `monoxus::toast` owns behavior, state, WAI-ARIA semantics, and live geometry measurement while
//! publishing deterministic CSS custom properties and `data-*` selectors consumed by downstream
//! design-system crates such as `shadcn-dioxus`.
//!
//! ## Viewport (`ToastViewport` / `<ol data-sonner-toaster="true">`)
//!
//! **CSS Custom Properties:**
//! - `--front-toast-height`: Unscaled border-box height (`px`) of the frontmost toast (`index == 0`),
//!   or `auto` prior to mount measurement. Used to clamp collapsed background cards.
//! - `--gap`: Stack spacing (`px`) between toast cards (`ToastConfig::gap`, default `14px`).
//! - `--width`: Canonical toast card width (`356px`).
//! - `--offset-top`, `--offset-right`, `--offset-bottom`, `--offset-left`: Desktop viewport edge offsets.
//! - `--mobile-offset-top`, `--mobile-offset-right`, `--mobile-offset-bottom`, `--mobile-offset-left`:
//!   Responsive mobile viewport edge offsets.
//!
//! **Data Selectors:**
//! - `data-sonner-toaster="true"`
//! - `data-expanded="true" | "false"`
//! - `data-position="top-left" | "top-center" | "top-right" | "bottom-left" | "bottom-center" | "bottom-right"`
//! - `data-x-position="left" | "center" | "right"`
//! - `data-y-position="top" | "bottom"`
//!
//! ## Toast Card (`ToastRoot` / `<li data-sonner-toast="">`)
//!
//! **CSS Custom Properties:**
//! - `--height`: Measured unscaled border-box height (`px`) of this card, or `auto` prior to mount.
//! - `--offset`: Computed vertical stack offset (`px`): `index * gap` when collapsed, or cumulative
//!   preceding heights plus `index * gap` when expanded.
//! - `--scale`: Progressive collapsed scale (`1.0 - index * 0.05`, clamped at `0.7`), or `1.0` when expanded.
//! - `--z-index`: Descending stack order (`total - index`).
//! - `--index` and `--toasts-before`: Zero-based position in the visible stack.
//! - `--swipe-amount-x`, `--swipe-amount-y`, `--drag-offset`: Unconditionally published pointer drag
//!   translation (`0px` at rest, `<px>` during active swipe).
//!
//! **Data Selectors:**
//! - `data-sonner-toast=""`
//! - `data-mounted="true" | "false"` (Note: `is_mounted` currently flips synchronously in `onmounted`; see `ui/shadcn-dioxus/docs/shadcn_toast_styling_capability_ledger.md` Section 4 Note 6 for co-applying `measure_toast_auto_height` -> `oxidase::frame::next_frame().await` -> `is_mounted.set(true)` during `ah:01M7BTR8FNX62N461B3Q6837YJ`)
//! - `data-id="<u64>"`
//! - `data-state="open" | "closed"`
//! - `data-type="default" | "success" | "info" | "warning" | "error" | "loading"`
//! - `data-visible="true" | "false"`
//! - `data-front="true" | "false"`
//! - `data-expanded="true" | "false"`
//! - `data-dismissible="true" | "false"`
//! - `data-position`, `data-x-position`, `data-y-position`, `data-index`, `data-testid`
//! - `data-swiping="true"`, `data-swiped="true"` (present during active swipe)
//! - `data-swipe-out="true" | "false"`, `data-swipe-direction="left" | "right" | "up" | "down"`
//!
//! ## Compound Sub-Part Selectors
//!
//! - `ToastTitle`: `data-title=""`
//! - `ToastDescription`: `data-description=""`
//! - `ToastAction` (primary): `data-button=""`
//! - `ToastAction` (`cancel: true`): `data-cancel=""`
//! - `ToastClose`: `data-close-button=""`, `data-radix-toast-announce-exclude="true"`
//!
//! # Reference & Governance Links
//!
//! - Feature Ledger: `ui/reference/toast_feature_ledger.md`
//! - Downstream Styling Ledger: `ui/shadcn-dioxus/docs/shadcn_toast_styling_capability_ledger.md` (Section 4 Notes 6 & 7)
//! - Downstream Component Index: `ui/shadcn-dioxus/docs/README.md`
//! - Governing `ah work` Artifacts:
//!   - `monoxus` Specs & Tasks (all completed):
//!     - Spec `01M7BTQG4PQ37CQ07SAM4YAF94` (Tasks `01M7BTQGHVSESV610E8D0X0Y3S`,
//!       `01M7BTQGVJBB13RXAAK9AWGCS8`, `01M7BTQGZQ4SDTR4H1BVT04W5W`)
//!     - Spec `01M7C6YCFVEFWEANSSH9CBS92Z` (Task `01M7C6YTZS93S1METKCJPKJK22`, Wave 1B Option A convergence)
//!     - Spec `01M7CERA8NMHZ3ZSMQ0Q49J4FV` (Option B crate-wide zero-`#[cfg]` convergence)
//!     - Task `01M7CMFHW4V2V3X375AFM0PQKZ` (Parallel per-toast `Delay` monotonic countdown timers)
//!     - Task `01M7D01CYYPNW2CK0YQYBMNZ53` (Pointer vs keyboard post-dismiss focus & viewport focus-ring hygiene)
//!   - `shadcn-dioxus` Spec `01M7BTR87GMCWVGDH8YQZB1FY3` (Task `01M7BTR8FNX62N461B3Q6837YJ`)
//!   - `blitz` Spec `01M7BTP4XHWRJMQ78AA9YYB6DF` (Task `01M7BTP5AGTS2ACJHZ7V0VAGA2`)

pub mod attrs;
pub mod components;
pub mod gesture;
pub mod hotkey;
pub mod layout;
pub mod runtime;
pub mod state;
pub mod types;

pub use self::attrs::*;
pub use self::components::*;
pub use self::gesture::*;
pub use self::hotkey::*;
pub use self::layout::*;
pub use self::runtime::*;
pub use self::state::*;
pub use self::types::*;
