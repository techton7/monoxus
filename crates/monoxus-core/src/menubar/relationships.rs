use crate::{foundation::shared::ScopeHandle, menu::MenuRelationships};

/// Deterministic DOM/ARIA ID relationships for a `Menubar` container and child menus.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenubarRelationships {
    scope: ScopeHandle,
    root_id: String,
}

impl MenubarRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            root_id: scope.token(),
            scope,
        }
    }

    pub fn scope(&self) -> &ScopeHandle {
        &self.scope
    }

    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    pub fn menu_relationships(&self, menu_scope: ScopeHandle) -> MenuRelationships {
        MenuRelationships::new(menu_scope)
    }
}
