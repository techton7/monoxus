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
    anchor::{PopoverAnchor, PopoverAnchorProps},
    arrow::{PopoverArrow, PopoverArrowProps},
    close::{PopoverClose, PopoverCloseProps},
    content::{PopoverContent, PopoverContentProps},
    description::{PopoverDescription, PopoverDescriptionProps},
    portal::{PopoverPortal, PopoverPortalProps},
    root::{PopoverRoot, PopoverRootProps},
    title::{PopoverTitle, PopoverTitleProps},
    trigger::{PopoverTrigger, PopoverTriggerProps},
};

use dioxus::prelude::*;

use crate::foundation::state::ControllableSignal;

use super::{relationships::PopoverRelationships, runtime::PopoverRuntime};

#[derive(Clone)]
pub struct PopoverContext {
    pub runtime: Signal<PopoverRuntime>,
    pub open: Signal<bool>,
    pub open_ctl: ControllableSignal<bool>,
    pub on_open_change: Option<EventHandler<bool>>,
    pub relationships: PopoverRelationships,
    pub title_id: Signal<Option<String>>,
    pub description_id: Signal<Option<String>>,
    pub content_id: Signal<Option<String>>,
}
