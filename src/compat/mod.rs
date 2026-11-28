//! Internal Monoxus compatibility projections over [`ui_sdk`].
//!
//! Modules here are explicitly classified as:
//! - `[KEEP-MONOXUS-BRAND]`: Monoxus/Radix/Bits attribute or CSS custom property
//!   projections that stay inside `monoxus` rather than leaking into `ui-sdk`.
//! - `[STAGE1-DROP]`: Legacy v0.1 handler/type shims slated for deletion in Track B `S1.7`.

#![allow(unused_imports, dead_code)]

pub(crate) mod accordion;
pub(crate) mod aspect_ratio;
pub(crate) mod calendar;
pub(crate) mod checkbox;
pub(crate) mod collapsible;
pub(crate) mod command;
pub(crate) mod context_menu;
pub(crate) mod dialog;
pub(crate) mod dropdown_menu;
pub(crate) mod field;
pub(crate) mod hover_card;
pub(crate) mod menu;
pub(crate) mod menubar;
pub(crate) mod navigation_menu;
pub(crate) mod pagination;
pub(crate) mod pin_input;
pub(crate) mod radio_group;
pub(crate) mod range_calendar;
pub(crate) mod select;
pub(crate) mod switch;
pub(crate) mod tabs;
pub(crate) mod toast;
pub(crate) mod toggle_group;
