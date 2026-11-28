//! Internal Monoxus compatibility projections over [`ui_sdk`].
//!
//! All modules here are `[KEEP-MONOXUS-BRAND]`: Monoxus/Radix/Bits attribute or CSS custom
//! property projections that stay inside `monoxus` rather than leaking into `ui-sdk`.

#![allow(unused_imports, dead_code)]

pub(crate) mod accordion;
pub(crate) mod aspect_ratio;
pub(crate) mod calendar;
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
pub(crate) mod range_calendar;
pub(crate) mod toast;
