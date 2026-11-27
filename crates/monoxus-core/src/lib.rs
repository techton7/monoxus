//! Pure Rust headless UI state machines, relationships, attribute publishers,
//! roving-focus logic, `as_child` projection helpers, and overlay geometry.
//!
//! All 47 modules (`foundation`, `menu`, and 45 standalone primitives) are now
//! backed by [`ui_sdk`], with thin tagged compatibility shims (`[KEEP-MONOXUS-BRAND]`
//! / `[STAGE1-DROP]`) preserved only until Phase 5 decommission.

pub use ui_sdk::accessible_icon;
pub mod accordion;
pub use ui_sdk::alert_dialog;
pub mod aspect_ratio;
pub use ui_sdk::avatar;
pub use ui_sdk::button;
pub mod calendar;
pub mod checkbox;
pub mod collapsible;
pub use ui_sdk::combobox;
pub mod command;
pub mod context_menu;
pub use ui_sdk::date_field;
pub use ui_sdk::date_picker;
pub use ui_sdk::date_range_field;
pub use ui_sdk::date_range_picker;
pub mod dialog;
pub mod dropdown_menu;
pub mod field;
pub mod foundation;
pub mod hover_card;
pub use ui_sdk::kernel;
pub use ui_sdk::label;
#[doc(hidden)]
pub mod menu;
pub mod menubar;
pub use ui_sdk::meter;
pub mod navigation_menu;
pub mod pagination;
pub mod pin_input;
pub use ui_sdk::popover;
pub use ui_sdk::progress;
pub mod radio_group;
pub mod range_calendar;
pub use ui_sdk::rating_group;
pub use ui_sdk::scroll_area;
pub mod select;
pub use ui_sdk::separator;
pub use ui_sdk::slider;
pub mod switch;
pub mod tabs;
pub use ui_sdk::time_field;
pub use ui_sdk::time_range_field;
pub mod toast;
pub use ui_sdk::toggle;
pub mod toggle_group;
pub use ui_sdk::toolbar;
pub use ui_sdk::tooltip;
pub use ui_sdk::visually_hidden;
