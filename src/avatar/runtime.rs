use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use dioxus::prelude::*;
use oxidase::dom::observer::{ImageLoadOptions, ImageLoadStatus, watch_image_load};

use crate::foundation::state::ControllableSignal;

use super::{
    attrs::{AvatarFallbackAttributes, AvatarImageAttributes, AvatarRootAttributes},
    relationships::AvatarRelationships,
    state::AvatarStateMachine,
    types::AvatarImageLoadingStatus,
};

pub use oxidase::WatcherGuard;

#[derive(Clone)]
pub struct AvatarContext {
    pub(crate) status_ctl: ControllableSignal<AvatarImageLoadingStatus>,
    pub(crate) relationships: AvatarRelationships,
    pub(crate) image_count: Signal<usize>,
}

impl AvatarContext {
    pub fn status(&self) -> AvatarImageLoadingStatus {
        self.status_ctl.get()
    }

    pub fn relationships(&self) -> &AvatarRelationships {
        &self.relationships
    }

    pub fn root_attributes(&self) -> AvatarRootAttributes {
        AvatarRootAttributes::new(self.status())
    }

    pub fn image_attributes(&self) -> AvatarImageAttributes {
        AvatarImageAttributes::new(self.status())
    }

    pub fn fallback_attributes(&self) -> AvatarFallbackAttributes {
        AvatarFallbackAttributes::new(self.status())
    }

    pub(crate) fn publish(&self, status: AvatarImageLoadingStatus) {
        self.status_ctl.set(status);
    }

    pub(crate) fn register_image(&self) {
        let mut image_count = self.image_count;
        let next = *image_count.peek() + 1;
        image_count.set(next);
        if next > 1 {
            #[cfg(debug_assertions)]
            eprintln!(
                "monoxus::Avatar: multiple AvatarImage children are unsupported; received {next}"
            );
        }
    }

    pub(crate) fn unregister_image(&self) {
        let mut image_count = self.image_count;
        if let Ok(mut count) = image_count.try_write() {
            *count = count.saturating_sub(1);
        }
    }
}

pub fn use_avatar_context() -> AvatarContext {
    use_context::<AvatarContext>()
}

pub fn try_use_avatar_context() -> Option<AvatarContext> {
    try_consume_context::<AvatarContext>()
}

pub fn watch_avatar_image<F>(
    src: &str,
    options: ImageLoadOptions,
    generation: u64,
    alive: Rc<Cell<bool>>,
    owns_generation: impl Fn() -> bool + 'static,
    mut on_status: F,
) -> oxidase::error::Result<WatcherGuard>
where
    F: FnMut(u64, AvatarImageLoadingStatus) + 'static,
{
    watch_image_load(src, options, move |status: ImageLoadStatus| {
        if alive.get() && owns_generation() {
            on_status(generation, super::types::image_load_status_to_avatar(status));
        }
    })
}

#[derive(Clone)]
pub(crate) struct AvatarImageProbe {
    state: Rc<RefCell<AvatarStateMachine>>,
    watcher: Signal<Option<WatcherGuard>>,
    alive: Rc<Cell<bool>>,
}

impl AvatarImageProbe {
    pub fn new() -> Self {
        Self {
            state: use_hook(|| Rc::new(RefCell::new(AvatarStateMachine::new()))),
            watcher: use_signal(|| None),
            alive: use_hook(|| Rc::new(Cell::new(true))),
        }
    }

    pub fn begin_generation(&self) -> u64 {
        self.state.borrow_mut().begin_generation()
    }

    pub fn owns_generation(&self, generation: u64) -> impl Fn() -> bool + 'static {
        let state = self.state.clone();
        move || state.borrow().owns(generation)
    }

    pub fn transition(
        &self,
        generation: u64,
        status: AvatarImageLoadingStatus,
    ) -> Option<AvatarImageLoadingStatus> {
        self.state.borrow_mut().transition(generation, status)
    }

    pub fn replace_watcher(&mut self, watcher: Option<WatcherGuard>) {
        if let Ok(mut slot) = self.watcher.try_write() {
            if let Some(previous) = slot.take() {
                previous.stop();
            }
            *slot = watcher;
        }
    }

    pub fn watcher_slot(&self) -> Signal<Option<WatcherGuard>> {
        self.watcher
    }

    pub fn alive(&self) -> Rc<Cell<bool>> {
        self.alive.clone()
    }
}
