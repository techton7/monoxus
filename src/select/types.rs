use dioxus::prelude::Signal;

use super::runtime::SelectRuntime;

pub use ui_sdk::select::types::*;

#[derive(Clone)]
pub struct SelectContext {
    pub runtime: SelectRuntime,
    pub name: Option<String>,
}

#[derive(Clone, Copy, PartialEq)]
pub struct SelectItemContext {
    pub value: Signal<String>,
    pub selected: Signal<bool>,
    pub disabled: Signal<bool>,
}

#[derive(Clone, PartialEq)]
pub struct SelectGroupContext {
    pub group_id: String,
    pub label_id: Signal<Option<String>>,
    pub default_label_id: String,
}
