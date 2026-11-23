use std::collections::BTreeSet;

use crate::foundation::{
    roving_focus::{
        RovingFocusConfig, RovingModifiers, is_navigation_key_for_orientation,
        resolve_next_stop_in_registry,
    },
    shared::{CollectionRegistry, Direction, Orientation, ScopeHandle},
};

use super::{
    attrs::{
        PaginationNextButtonAttributes, PaginationPageAttributes, PaginationPrevButtonAttributes,
        PaginationRootAttributes,
    },
    relationships::PaginationRelationships,
    types::{
        PAGINATION_PARTS, PageItem, PageRange, PaginationItemRegistration, PaginationPart,
        PaginationSnapshot, orientation_as_str,
    },
};

pub fn compute_total_pages(count: usize, per_page: usize) -> usize {
    if count == 0 {
        1
    } else {
        count.div_ceil(per_page.max(1))
    }
}

pub fn clamp_page(page: usize, total_pages: usize) -> usize {
    let max_page = total_pages.max(1);
    page.clamp(1, max_page)
}

pub fn compute_range(page: usize, per_page: usize, count: usize) -> PageRange {
    if count == 0 {
        return PageRange { start: 0, end: 0 };
    }
    let effective_per_page = per_page.max(1);
    let total_pages = compute_total_pages(count, effective_per_page);
    let current_page = clamp_page(page, total_pages);
    let zero_based_start = (current_page - 1).saturating_mul(effective_per_page);
    let start = zero_based_start.saturating_add(1).min(count);
    let end = zero_based_start.saturating_add(effective_per_page).min(count);
    PageRange { start, end }
}

/// Computes the deterministic sequence of `PageItem::Page` and `PageItem::Ellipsis` entries
/// using checked/saturating arithmetic so small `total_pages` never underflow `usize`.
pub fn compute_page_items(
    page: usize,
    total_pages: usize,
    sibling_count: usize,
) -> Vec<PageItem> {
    let total = total_pages.max(1);
    let current_page = clamp_page(page, total);

    let mut pages_to_show = BTreeSet::new();
    pages_to_show.insert(1usize);
    pages_to_show.insert(total);

    let first_item_with_siblings = 3usize.saturating_add(sibling_count);
    let last_item_with_siblings = total.saturating_sub(2).saturating_sub(sibling_count);

    if first_item_with_siblings > last_item_with_siblings {
        for i in 2..total {
            pages_to_show.insert(i);
        }
    } else if current_page < first_item_with_siblings {
        let end = first_item_with_siblings.min(total);
        for i in 2..=end {
            pages_to_show.insert(i);
        }
    } else if current_page > last_item_with_siblings {
        let start = last_item_with_siblings.max(2);
        for i in start..total {
            pages_to_show.insert(i);
        }
    } else {
        let start = current_page.saturating_sub(sibling_count).max(2);
        let end = current_page.saturating_add(sibling_count).min(total);
        for i in start..=end {
            pages_to_show.insert(i);
        }
    }

    let mut items = Vec::with_capacity(pages_to_show.len() + 2);
    let mut last_number = 0usize;
    let mut ellipsis_slot = 0usize;

    for p in pages_to_show {
        if p.saturating_sub(last_number) > 1 {
            items.push(PageItem::Ellipsis {
                key: format!("ellipsis-{ellipsis_slot}"),
            });
            ellipsis_slot += 1;
        }
        items.push(PageItem::Page {
            value: p,
            key: format!("page-{p}"),
        });
        last_number = p;
    }

    items
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaginationDefinition {
    relationships: PaginationRelationships,
    count: usize,
    per_page: usize,
    sibling_count: usize,
    page: usize,
    loop_focus: bool,
    orientation: Orientation,
    dir: Direction,
    aria_label: String,
}

impl PaginationDefinition {
    pub fn new(scope: ScopeHandle, count: usize) -> Self {
        Self {
            relationships: PaginationRelationships::new(scope),
            count,
            per_page: 1,
            sibling_count: 1,
            page: 1,
            loop_focus: false,
            orientation: Orientation::Horizontal,
            dir: Direction::Ltr,
            aria_label: String::from("pagination"),
        }
    }

    pub const fn parts() -> &'static [PaginationPart] {
        &PAGINATION_PARTS
    }

    pub fn relationships(&self) -> &PaginationRelationships {
        &self.relationships
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn per_page(&self) -> usize {
        self.per_page
    }

    pub fn sibling_count(&self) -> usize {
        self.sibling_count
    }

    pub fn total_pages(&self) -> usize {
        compute_total_pages(self.count, self.per_page)
    }

    pub fn current_page(&self) -> usize {
        clamp_page(self.page, self.total_pages())
    }

    pub fn range(&self) -> PageRange {
        compute_range(self.current_page(), self.per_page, self.count)
    }

    pub fn pages(&self) -> Vec<PageItem> {
        compute_page_items(self.current_page(), self.total_pages(), self.sibling_count)
    }

    pub fn has_prev_page(&self) -> bool {
        self.current_page() > 1
    }

    pub fn has_next_page(&self) -> bool {
        self.current_page() < self.total_pages()
    }

    pub fn loop_focus(&self) -> bool {
        self.loop_focus
    }

    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    pub fn dir(&self) -> Direction {
        self.dir
    }

    pub fn aria_label(&self) -> &str {
        &self.aria_label
    }

    pub fn snapshot(&self) -> PaginationSnapshot {
        let total_pages = self.total_pages();
        let current_page = clamp_page(self.page, total_pages);
        PaginationSnapshot {
            pages: compute_page_items(current_page, total_pages, self.sibling_count),
            range: compute_range(current_page, self.per_page, self.count),
            current_page,
            total_pages,
            has_prev_page: current_page > 1,
            has_next_page: current_page < total_pages,
        }
    }

    pub fn with_per_page(mut self, per_page: usize) -> Self {
        self.per_page = per_page.max(1);
        self.page = clamp_page(self.page, self.total_pages());
        self
    }

    pub fn with_sibling_count(mut self, sibling_count: usize) -> Self {
        self.sibling_count = sibling_count;
        self
    }

    pub fn with_page(mut self, page: usize) -> Self {
        self.page = clamp_page(page, self.total_pages());
        self
    }

    pub fn with_loop_focus(mut self, loop_focus: bool) -> Self {
        self.loop_focus = loop_focus;
        self
    }

    pub fn with_orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn with_direction(mut self, dir: Direction) -> Self {
        self.dir = dir;
        self
    }

    pub fn with_aria_label(mut self, aria_label: impl Into<String>) -> Self {
        self.aria_label = aria_label.into();
        self
    }

    pub fn set_page(&mut self, page: usize) -> usize {
        let clamped = clamp_page(page, self.total_pages());
        self.page = clamped;
        clamped
    }

    pub fn prev_page(&mut self) -> usize {
        let prev = self.current_page().saturating_sub(1).max(1);
        self.set_page(prev)
    }

    pub fn next_page(&mut self) -> usize {
        let next = self.current_page().saturating_add(1);
        self.set_page(next)
    }

    pub const fn roving_config(&self) -> RovingFocusConfig {
        RovingFocusConfig::new()
            .with_orientation(self.orientation)
            .with_direction(self.dir)
            .with_loop_focus(self.loop_focus)
            .with_enabled(true)
    }

    pub fn navigate_key(
        &self,
        registry: &CollectionRegistry<String, PaginationItemRegistration>,
        current_key: Option<&str>,
        key: &str,
        modifiers: RovingModifiers,
    ) -> Option<String> {
        let config = self.roving_config();
        if !is_navigation_key_for_orientation(config.orientation, key) {
            return None;
        }

        resolve_next_stop_in_registry(
            &config,
            registry,
            current_key,
            key,
            modifiers,
            |reg| !reg.disabled,
        )
    }

    pub fn root_attributes(&self, custom_id: Option<String>) -> PaginationRootAttributes {
        PaginationRootAttributes {
            id: custom_id.unwrap_or_else(|| self.relationships.root_id().to_string()),
            role: "navigation",
            aria_label: self.aria_label.clone(),
            dir: self.dir.as_str(),
            data_orientation: orientation_as_str(self.orientation),
            data_monoxus_pagination_root: "",
        }
    }

    pub fn page_attributes(
        &self,
        page: usize,
        custom_id: Option<String>,
        disabled: bool,
    ) -> PaginationPageAttributes {
        let id = custom_id.unwrap_or_else(|| self.relationships.page_id(page));
        let is_selected = self.current_page() == page;
        PaginationPageAttributes::new(id, page, is_selected, disabled)
    }

    pub fn prev_button_attributes(
        &self,
        custom_id: Option<String>,
        disabled: bool,
    ) -> PaginationPrevButtonAttributes {
        let id = custom_id.unwrap_or_else(|| self.relationships.prev_id().to_string());
        let effective_disabled = disabled || !self.has_prev_page();
        PaginationPrevButtonAttributes::new(id, effective_disabled)
    }

    pub fn next_button_attributes(
        &self,
        custom_id: Option<String>,
        disabled: bool,
    ) -> PaginationNextButtonAttributes {
        let id = custom_id.unwrap_or_else(|| self.relationships.next_id().to_string());
        let effective_disabled = disabled || !self.has_next_page();
        PaginationNextButtonAttributes::new(id, effective_disabled)
    }
}

pub type Pagination = PaginationDefinition;
