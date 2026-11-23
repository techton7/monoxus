use std::{cell::Cell, rc::Rc};

use dioxus::prelude::*;

use crate::foundation::{
    roving_focus::{RovingFocusConfig, RovingFocusRuntime, use_roving_focus_group},
    shared::{Direction, Orientation, ScopeHandle},
};

use super::{
    attrs::{
        PaginationNextButtonAttributes, PaginationPageAttributes, PaginationPrevButtonAttributes,
        PaginationRootAttributes,
    },
    relationships::PaginationRelationships,
    state::{PaginationDefinition, clamp_page, compute_total_pages},
    types::PaginationSnapshot,
};

#[derive(Clone)]
pub struct PaginationRuntime {
    pub count: Signal<usize>,
    pub per_page: Signal<usize>,
    pub sibling_count: Signal<usize>,
    pub page: Signal<usize>,
    pub is_controlled: Signal<bool>,
    pub loop_focus: Signal<bool>,
    pub orientation: Signal<Orientation>,
    pub dir: Signal<Direction>,
    pub aria_label: Signal<String>,
    pub scope_id: Signal<String>,
    pub root_dom_id: Signal<String>,
    pub roving: RovingFocusRuntime,
    pub on_page_change: Rc<Cell<Option<EventHandler<usize>>>>,
}

impl PaginationRuntime {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        count: usize,
        per_page: usize,
        sibling_count: usize,
        initial_page: usize,
        is_controlled: bool,
        loop_focus: bool,
        orientation: Orientation,
        dir: Direction,
        aria_label: String,
        scope_id: String,
        root_dom_id: String,
        on_page_change: Option<EventHandler<usize>>,
    ) -> Self {
        let roving_cfg = RovingFocusConfig::new()
            .with_orientation(orientation)
            .with_direction(dir)
            .with_loop_focus(loop_focus)
            .with_enabled(true);
        let roving = use_roving_focus_group(roving_cfg, None, root_dom_id);
        let on_page_change_cell = use_hook(|| Rc::new(Cell::new(on_page_change)));

        Self {
            count: use_signal(|| count),
            per_page: use_signal(|| per_page.max(1)),
            sibling_count: use_signal(|| sibling_count),
            page: use_signal(|| initial_page),
            is_controlled: use_signal(|| is_controlled),
            loop_focus: use_signal(|| loop_focus),
            orientation: use_signal(|| orientation),
            dir: use_signal(|| dir),
            aria_label: use_signal(|| aria_label),
            scope_id: use_signal(|| scope_id),
            root_dom_id: roving.root_dom_id,
            roving,
            on_page_change: on_page_change_cell,
        }
    }

    pub fn roving(&self) -> RovingFocusRuntime {
        self.roving
    }

    pub fn relationships(&self) -> PaginationRelationships {
        let scope = ScopeHandle::root("pagination").child((self.scope_id)());
        PaginationRelationships::new(scope)
    }

    pub fn definition(&self) -> PaginationDefinition {
        let scope = ScopeHandle::root("pagination").child((self.scope_id)());
        PaginationDefinition::new(scope, (self.count)())
            .with_per_page((self.per_page)())
            .with_sibling_count((self.sibling_count)())
            .with_page((self.page)())
            .with_loop_focus((self.loop_focus)())
            .with_orientation((self.orientation)())
            .with_direction((self.dir)())
            .with_aria_label((self.aria_label)())
    }

    pub fn snapshot(&self) -> PaginationSnapshot {
        self.definition().snapshot()
    }

    pub fn set_page(&self, next_raw: usize) {
        let total_pages = compute_total_pages(*self.count.peek(), *self.per_page.peek());
        let next = clamp_page(next_raw, total_pages);
        let current = *self.page.peek();
        if current == next {
            return;
        }
        if !*self.is_controlled.peek() {
            let mut page_sig = self.page;
            page_sig.set(next);
        }
        if let Some(handler) = self.on_page_change.get() {
            handler.call(next);
        }
    }

    pub fn prev_page(&self) {
        let current = *self.page.peek();
        if current > 1 {
            self.set_page(current - 1);
        }
    }

    pub fn next_page(&self) {
        let total_pages = compute_total_pages(*self.count.peek(), *self.per_page.peek());
        let current = *self.page.peek();
        if current < total_pages {
            self.set_page(current + 1);
        }
    }

    pub fn root_attributes(&self, custom_id: Option<String>) -> PaginationRootAttributes {
        self.definition().root_attributes(custom_id)
    }

    pub fn page_attributes(
        &self,
        page: usize,
        custom_id: Option<String>,
        disabled: bool,
    ) -> PaginationPageAttributes {
        self.definition()
            .page_attributes(page, custom_id, disabled)
    }

    pub fn prev_button_attributes(
        &self,
        custom_id: Option<String>,
        disabled: bool,
    ) -> PaginationPrevButtonAttributes {
        self.definition().prev_button_attributes(custom_id, disabled)
    }

    pub fn next_button_attributes(
        &self,
        custom_id: Option<String>,
        disabled: bool,
    ) -> PaginationNextButtonAttributes {
        self.definition().next_button_attributes(custom_id, disabled)
    }
}

#[derive(Clone)]
pub struct PaginationContext {
    pub runtime: PaginationRuntime,
}

impl PaginationContext {
    pub fn snapshot(&self) -> PaginationSnapshot {
        self.runtime.snapshot()
    }

    pub fn set_page(&self, page: usize) {
        self.runtime.set_page(page);
    }

    pub fn prev_page(&self) {
        self.runtime.prev_page();
    }

    pub fn next_page(&self) {
        self.runtime.next_page();
    }
}

pub fn use_pagination_context() -> PaginationContext {
    use_context::<PaginationContext>()
}

pub fn try_use_pagination_context() -> Option<PaginationContext> {
    try_consume_context::<PaginationContext>()
}
