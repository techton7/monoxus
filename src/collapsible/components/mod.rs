pub mod content;
pub mod root;
pub mod trigger;

pub use self::{
    content::{CollapsibleContent, CollapsibleContentProps},
    root::{CollapsibleRoot, CollapsibleRootProps},
    trigger::{CollapsibleTrigger, CollapsibleTriggerProps},
};

use dioxus::prelude::*;

use super::{relationships::CollapsibleRelationships, runtime::CollapsibleRuntime};

#[derive(Clone)]
pub struct CollapsibleContext {
    pub runtime: Signal<CollapsibleRuntime>,
    pub open: Signal<bool>,
    pub disabled: Signal<bool>,
    pub content_id: Signal<Option<String>>,
    pub on_open_change: Option<EventHandler<bool>>,
    pub relationships: CollapsibleRelationships,
}
