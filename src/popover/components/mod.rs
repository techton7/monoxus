pub mod anchor;
pub mod arrow;
pub mod close;
pub mod content;
pub mod description;
pub mod portal;
pub mod root;
pub mod title;
pub mod trigger;

pub use self::{
    anchor::PopoverAnchor,
    arrow::PopoverArrow,
    close::PopoverClose,
    content::PopoverContent,
    description::PopoverDescription,
    portal::PopoverPortal,
    root::PopoverRoot,
    title::PopoverTitle,
    trigger::PopoverTrigger,
};

use dioxus::prelude::*;

use super::{relationships::PopoverRelationships, runtime::PopoverRuntime};

#[derive(Clone)]
pub struct PopoverContext {
    pub runtime: Signal<PopoverRuntime>,
    pub open: Signal<bool>,
    pub on_open_change: Option<EventHandler<bool>>,
    pub relationships: PopoverRelationships,
    pub title_id: Signal<Option<String>>,
    pub description_id: Signal<Option<String>>,
}
