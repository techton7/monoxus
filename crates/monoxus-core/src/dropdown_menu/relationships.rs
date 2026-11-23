use crate::{foundation::shared::ScopeHandle, menu::MenuRelationships};

/// Deterministic DOM/ARIA ID relationships for a `DropdownMenu` tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DropdownMenuRelationships {
    menu: MenuRelationships,
}

impl DropdownMenuRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            menu: MenuRelationships::new(scope),
        }
    }

    pub fn from_menu(menu: MenuRelationships) -> Self {
        Self { menu }
    }

    pub fn menu(&self) -> &MenuRelationships {
        &self.menu
    }

    pub fn root_id(&self) -> &str {
        self.menu.root_id()
    }

    pub fn trigger_id(&self) -> &str {
        self.menu.trigger_id()
    }

    pub fn portal_id(&self) -> &str {
        self.menu.portal_id()
    }

    pub fn content_id(&self) -> &str {
        self.menu.content_id()
    }

    pub fn wrapper_id(&self) -> &str {
        self.menu.wrapper_id()
    }

    pub fn arrow_id(&self) -> &str {
        self.menu.arrow_id()
    }
}
