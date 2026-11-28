pub use ui_sdk::combobox::types::*;

use dioxus::prelude::Signal;

use super::runtime::ComboboxRuntime;

#[derive(Clone, PartialEq)]
pub struct ComboboxContext {
    pub runtime: ComboboxRuntime,
}

#[derive(Clone, PartialEq)]
pub struct ComboboxGroupContext {
    pub group_id: String,
    pub heading_id: Signal<Option<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComboboxPortalContext {
    pub host: PortalHost,
    pub disabled: bool,
    pub force_mount: bool,
}
