pub mod arrow;
pub mod content;
pub mod portal;
pub mod provider;
pub mod root;
pub mod trigger;

pub use self::{
    arrow::TooltipArrow,
    content::TooltipContent,
    portal::TooltipPortal,
    provider::TooltipProvider,
    root::TooltipRoot,
    trigger::TooltipTrigger,
};

use dioxus::prelude::*;

use super::{
    relationships::TooltipRelationships,
    runtime::{TooltipProviderRuntime, TooltipRuntime},
};

#[derive(Clone, Copy)]
pub struct TooltipProviderContext {
    pub runtime: Signal<TooltipProviderRuntime>,
}

#[derive(Clone)]
pub struct TooltipRootContext {
    pub runtime: Signal<TooltipRuntime>,
    pub open: Signal<bool>,
    pub on_open_change: Option<EventHandler<bool>>,
    pub relationships: TooltipRelationships,
}
