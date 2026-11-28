// [KEEP-MONOXUS-BRAND]
use ui_sdk::foundation::{
    roving_focus::{RovingFocusConfig, RovingModifiers},
    shared::{CollectionRegistry, Direction, Orientation, ScopeHandle},
};

pub use ui_sdk::pagination::state::{
    clamp_page, compute_page_items, compute_range, compute_total_pages,
};

use super::{
    attrs::{
        PaginationNextButtonAttributes, PaginationPageAttributes, PaginationPrevButtonAttributes,
        PaginationRootAttributes,
    },
    relationships::PaginationRelationships,
    types::{
        PageItem, PageRange, PaginationItemRegistration, PaginationPart, PaginationSnapshot,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaginationDefinition(ui_sdk::pagination::state::PaginationDefinition);

impl PaginationDefinition {
    pub fn new(scope: ScopeHandle, count: usize) -> Self {
        Self(ui_sdk::pagination::state::PaginationDefinition::new(
            scope, count,
        ))
    }

    pub const fn parts() -> &'static [PaginationPart] {
        ui_sdk::pagination::state::PaginationDefinition::parts()
    }

    pub fn relationships(&self) -> &PaginationRelationships {
        self.0.relationships()
    }

    pub fn count(&self) -> usize {
        self.0.count()
    }

    pub fn per_page(&self) -> usize {
        self.0.per_page()
    }

    pub fn sibling_count(&self) -> usize {
        self.0.sibling_count()
    }

    pub fn total_pages(&self) -> usize {
        self.0.total_pages()
    }

    pub fn current_page(&self) -> usize {
        self.0.current_page()
    }

    pub fn range(&self) -> PageRange {
        self.0.range()
    }

    pub fn pages(&self) -> Vec<PageItem> {
        self.0.pages()
    }

    pub fn has_prev_page(&self) -> bool {
        self.0.has_prev_page()
    }

    pub fn has_next_page(&self) -> bool {
        self.0.has_next_page()
    }

    pub fn loop_focus(&self) -> bool {
        self.0.loop_focus()
    }

    pub fn orientation(&self) -> Orientation {
        self.0.orientation()
    }

    pub fn dir(&self) -> Direction {
        self.0.dir()
    }

    pub fn aria_label(&self) -> &str {
        self.0.aria_label()
    }

    pub fn snapshot(&self) -> PaginationSnapshot {
        self.0.snapshot()
    }

    pub fn with_per_page(self, per_page: usize) -> Self {
        Self(self.0.with_per_page(per_page))
    }

    pub fn with_sibling_count(self, sibling_count: usize) -> Self {
        Self(self.0.with_sibling_count(sibling_count))
    }

    pub fn with_page(self, page: usize) -> Self {
        Self(self.0.with_page(page))
    }

    pub fn with_loop_focus(self, loop_focus: bool) -> Self {
        Self(self.0.with_loop_focus(loop_focus))
    }

    pub fn with_orientation(self, orientation: Orientation) -> Self {
        Self(self.0.with_orientation(orientation))
    }

    pub fn with_direction(self, dir: Direction) -> Self {
        Self(self.0.with_direction(dir))
    }

    pub fn with_aria_label(self, aria_label: impl Into<String>) -> Self {
        Self(self.0.with_aria_label(aria_label))
    }

    pub fn set_page(&mut self, page: usize) -> usize {
        self.0.set_page(page)
    }

    pub fn prev_page(&mut self) -> usize {
        self.0.prev_page()
    }

    pub fn next_page(&mut self) -> usize {
        self.0.next_page()
    }

    pub const fn roving_config(&self) -> RovingFocusConfig {
        self.0.roving_config()
    }

    pub fn navigate_key(
        &self,
        registry: &CollectionRegistry<String, PaginationItemRegistration>,
        current_key: Option<&str>,
        key: &str,
        modifiers: RovingModifiers,
    ) -> Option<String> {
        self.0.navigate_key(registry, current_key, key, modifiers)
    }

    pub fn root_attributes(&self, custom_id: Option<String>) -> PaginationRootAttributes {
        self.0.root_attributes(custom_id).into()
    }

    pub fn page_attributes(
        &self,
        page: usize,
        custom_id: Option<String>,
        disabled: bool,
    ) -> PaginationPageAttributes {
        self.0.page_attributes(page, custom_id, disabled).into()
    }

    pub fn prev_button_attributes(
        &self,
        custom_id: Option<String>,
        disabled: bool,
    ) -> PaginationPrevButtonAttributes {
        self.0.prev_button_attributes(custom_id, disabled).into()
    }

    pub fn next_button_attributes(
        &self,
        custom_id: Option<String>,
        disabled: bool,
    ) -> PaginationNextButtonAttributes {
        self.0.next_button_attributes(custom_id, disabled).into()
    }
}

pub type Pagination = PaginationDefinition;
