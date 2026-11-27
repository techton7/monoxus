//! Pure Rust headless UI state machines, relationships, attribute publishers,
//! roving-focus logic, `as_child` projection helpers, and overlay geometry.
//!
//! Migrated Wave 0 (`foundation`) and Wave 1A (11 leaf/atomic primitives) are
//! re-exported directly from [`ui_sdk`] with thin compatibility shims where
//! legacy Monoxus/Radix CSS-variable names are preserved.

pub use ui_sdk::accessible_icon;
pub mod accordion;
pub mod alert_dialog;
pub mod aspect_ratio;
pub use ui_sdk::avatar;
pub use ui_sdk::button;
pub mod calendar;
pub mod checkbox;
pub mod collapsible;
pub mod combobox;
pub mod command;
pub mod context_menu;
pub mod date_field;
pub mod date_picker;
pub mod date_range_field;
pub mod date_range_picker;
pub mod dialog;
pub mod dropdown_menu;
pub mod field;
pub mod foundation;
pub mod hover_card;
pub use ui_sdk::label;
#[doc(hidden)]
pub mod menu;
pub mod menubar;
pub use ui_sdk::meter;
pub mod navigation_menu;
pub mod pagination;
pub mod pin_input;
pub mod popover;
pub use ui_sdk::progress;
pub mod radio_group;
pub mod range_calendar;
pub mod rating_group;
pub use ui_sdk::scroll_area;
pub mod select;
pub use ui_sdk::separator;
pub mod slider;
pub mod switch;
pub mod tabs;
pub mod time_field;
pub mod time_range_field;
pub mod toast;
pub mod toggle;
pub mod toggle_group;
pub mod toolbar;
pub mod tooltip;
pub use ui_sdk::visually_hidden;
