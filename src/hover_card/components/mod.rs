pub mod arrow;
pub mod content;
pub mod content_static;
pub mod portal;
pub mod root;
pub mod trigger;

pub use self::{
    arrow::{HoverCardArrow, HoverCardArrowProps},
    content::{HoverCardContent, HoverCardContentProps},
    content_static::{HoverCardContentStatic, HoverCardContentStaticProps},
    portal::{HoverCardPortal, HoverCardPortalProps},
    root::{HoverCard, HoverCardRoot, HoverCardRootProps},
    trigger::{HoverCardTrigger, HoverCardTriggerProps},
};

use dioxus::prelude::*;

use super::{relationships::HoverCardRelationships, runtime::HoverCardRuntime};

#[derive(Clone)]
pub struct HoverCardRootContext {
    pub runtime: Signal<HoverCardRuntime>,
    pub open: Signal<bool>,
    pub on_open_change: Option<EventHandler<bool>>,
    pub relationships: HoverCardRelationships,
}
