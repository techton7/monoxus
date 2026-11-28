use dioxus::prelude::*;
use ui_sdk::kernel::{ChangeContext, ChangeEvent, ChangeReason, ChangeSource};

use crate::foundation::{
    roving_focus::{RovingFocusConfig, RovingFocusRuntime, use_roving_focus_group},
    shared::{Direction, Orientation, ScopeHandle},
    state::{
        ChangeOutcome, ControllableConfig, ControllableSignal, sync_signal,
        use_controllable_signal_config,
    },
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
    pub(crate) page_ctl: ControllableSignal<usize>,
    pub is_controlled: Signal<bool>,
    pub loop_focus: Signal<bool>,
    pub orientation: Signal<Orientation>,
    pub dir: Signal<Direction>,
    pub aria_label: Signal<String>,
    pub scope_id: Signal<String>,
    pub root_dom_id: Signal<String>,
    pub roving: RovingFocusRuntime,
}

#[allow(clippy::too_many_arguments)]
pub fn use_pagination_runtime(
    count: usize,
    per_page: usize,
    sibling_count: usize,
    page: Option<usize>,
    default_page: usize,
    on_page_change: Option<EventHandler<usize>>,
    loop_focus: bool,
    orientation: Orientation,
    dir: Direction,
    aria_label: String,
    scope_id: String,
    root_dom_id: String,
) -> PaginationRuntime {
    use_pagination_runtime_with_details(
        count,
        per_page,
        sibling_count,
        page,
        default_page,
        on_page_change,
        None,
        loop_focus,
        orientation,
        dir,
        aria_label,
        scope_id,
        root_dom_id,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn use_pagination_runtime_with_details(
    count: usize,
    per_page: usize,
    sibling_count: usize,
    page: Option<usize>,
    default_page: usize,
    on_page_change: Option<EventHandler<usize>>,
    on_page_change_details: Option<EventHandler<ChangeEvent<usize>>>,
    loop_focus: bool,
    orientation: Orientation,
    dir: Direction,
    aria_label: String,
    scope_id: String,
    root_dom_id: String,
) -> PaginationRuntime {
    let effective_per_page = per_page.max(1);
    let total_pages = compute_total_pages(count, effective_per_page);
    let page_ctl = use_controllable_signal_config(
        ControllableConfig::new(page, default_page)
            .with_explicit_default(default_page != 1)
            .with_on_change(on_page_change)
            .with_on_change_details(on_page_change_details)
            .with_sanitize(move |p| clamp_page(p, total_pages))
            .with_component("PaginationRoot")
            .with_prop("page"),
    );

    if !page_ctl.is_controlled() {
        let mut page_sig = page_ctl.signal();
        let current = *page_sig.peek();
        let clamped = clamp_page(current, total_pages);
        sync_signal(&mut page_sig, clamped);
    }

    let roving_cfg = RovingFocusConfig::new()
        .with_orientation(orientation)
        .with_direction(dir)
        .with_loop_focus(loop_focus)
        .with_enabled(true);
    let roving = use_roving_focus_group(roving_cfg, None, root_dom_id);

    let is_controlled = page_ctl.is_controlled();
    let mut rt = PaginationRuntime {
        count: use_signal(|| count),
        per_page: use_signal(|| effective_per_page),
        sibling_count: use_signal(|| sibling_count),
        page: page_ctl.signal(),
        page_ctl,
        is_controlled: use_signal(|| is_controlled),
        loop_focus: use_signal(|| loop_focus),
        orientation: use_signal(|| orientation),
        dir: use_signal(|| dir),
        aria_label: use_signal(|| aria_label.clone()),
        scope_id: use_signal(|| scope_id.clone()),
        root_dom_id: roving.root_dom_id,
        roving,
    };

    sync_signal(&mut rt.count, count);
    sync_signal(&mut rt.per_page, effective_per_page);
    sync_signal(&mut rt.sibling_count, sibling_count);
    sync_signal(&mut rt.is_controlled, is_controlled);
    sync_signal(&mut rt.loop_focus, loop_focus);
    sync_signal(&mut rt.orientation, orientation);
    sync_signal(&mut rt.dir, dir);
    sync_signal(&mut rt.aria_label, aria_label);
    sync_signal(&mut rt.scope_id, scope_id);

    rt
}

impl PaginationRuntime {
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
        let _ = self.set_page_with_context(
            next_raw,
            ChangeContext::new(ChangeReason::Navigation, ChangeSource::Programmatic),
        );
    }

    pub fn set_page_with_context(&self, next_raw: usize, ctx: ChangeContext) -> ChangeOutcome {
        let total_pages = compute_total_pages(*self.count.peek(), *self.per_page.peek());
        let next = clamp_page(next_raw, total_pages);
        self.page_ctl.set_with_context(next, ctx)
    }

    pub fn prev_page(&self) {
        let _ = self.prev_page_with_context(ChangeContext::new(
            ChangeReason::Navigation,
            ChangeSource::Programmatic,
        ));
    }

    pub fn prev_page_with_context(&self, ctx: ChangeContext) -> ChangeOutcome {
        let current = *self.page.peek();
        if current > 1 {
            self.set_page_with_context(current - 1, ctx)
        } else {
            ChangeOutcome::Unchanged
        }
    }

    pub fn next_page(&self) {
        let _ = self.next_page_with_context(ChangeContext::new(
            ChangeReason::Navigation,
            ChangeSource::Programmatic,
        ));
    }

    pub fn next_page_with_context(&self, ctx: ChangeContext) -> ChangeOutcome {
        let total_pages = compute_total_pages(*self.count.peek(), *self.per_page.peek());
        let current = *self.page.peek();
        if current < total_pages {
            self.set_page_with_context(current + 1, ctx)
        } else {
            ChangeOutcome::Unchanged
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
