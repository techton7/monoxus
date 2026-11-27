// [KEEP-MONOXUS-BRAND] Compat `Menubar` attribute aliases and root/trigger attrs emitting `data-monoxus-*`.
use crate::foundation::{
    compose::{Slottable, project_as_child},
    shared::{Direction, Orientation, data_disabled_attr, data_presence_attr},
    state::DataState,
};
use crate::menu::attrs::{menu_orientation_str, rewrite_part_attr_to_monoxus};

pub use crate::menu::{
    MenuArrowAttributes as MenubarArrowAttributes,
    MenuCheckboxItemAttributes as MenubarCheckboxItemAttributes,
    MenuContentAttributes as MenubarContentAttributes,
    MenuGroupAttributes as MenubarGroupAttributes,
    MenuHeadingAttributes as MenubarHeadingAttributes,
    MenuItemAttributes as MenubarItemAttributes,
    MenuItemIndicatorAttributes as MenubarItemIndicatorAttributes,
    MenuRadioItemAttributes as MenubarRadioItemAttributes,
    MenuSeparatorAttributes as MenubarSeparatorAttributes,
    MenuSubTriggerAttributes as MenubarSubTriggerAttributes,
};

pub use ui_sdk::menubar::attrs::MENUBAR_ROLE;

/// Attributes for `Menubar.Root` (`role="menubar"`, `aria-orientation`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenubarRootAttributes {
    pub id: String,
    pub role: &'static str,
    pub aria_orientation: &'static str,
    pub data_orientation: &'static str,
    pub data_modal: Option<&'static str>,
    pub data_has_submenu_open: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
    pub dir: &'static str,
    pub part_attr: String,
}

impl MenubarRootAttributes {
    pub fn new(id: String, direction: Direction) -> Self {
        Self::from(ui_sdk::menubar::attrs::MenubarRootAttributes::new(
            id, direction,
        ))
    }

    pub const fn with_orientation(mut self, orientation: Orientation) -> Self {
        let token = menu_orientation_str(orientation);
        self.aria_orientation = token;
        self.data_orientation = token;
        self
    }

    pub const fn with_modal(mut self, modal: bool) -> Self {
        self.data_modal = data_presence_attr(modal);
        self
    }

    pub const fn with_has_submenu_open(mut self, has_submenu_open: bool) -> Self {
        self.data_has_submenu_open = data_presence_attr(has_submenu_open);
        self
    }

    pub const fn with_disabled(mut self, disabled: bool) -> Self {
        self.data_disabled = data_disabled_attr(disabled);
        self
    }
}

impl From<ui_sdk::menubar::attrs::MenubarRootAttributes> for MenubarRootAttributes {
    fn from(attrs: ui_sdk::menubar::attrs::MenubarRootAttributes) -> Self {
        Self {
            id: attrs.id,
            role: attrs.role,
            aria_orientation: attrs.aria_orientation,
            data_orientation: attrs.data_orientation,
            data_modal: attrs.data_modal,
            data_has_submenu_open: attrs.data_has_submenu_open,
            data_disabled: attrs.data_disabled,
            dir: attrs.dir,
            part_attr: rewrite_part_attr_to_monoxus(attrs.part_attr),
        }
    }
}

impl From<MenubarRootAttributes> for ui_sdk::menubar::attrs::MenubarRootAttributes {
    fn from(attrs: MenubarRootAttributes) -> Self {
        Self {
            id: attrs.id,
            role: attrs.role,
            aria_orientation: attrs.aria_orientation,
            data_orientation: attrs.data_orientation,
            data_modal: attrs.data_modal,
            data_has_submenu_open: attrs.data_has_submenu_open,
            data_disabled: attrs.data_disabled,
            dir: attrs.dir,
            part_attr: attrs.part_attr,
        }
    }
}

/// Attributes for `Menubar.Trigger` (`role="menuitem"`, roving `tabindex`, `aria-haspopup="menu"`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenubarTriggerAttributes {
    pub id: String,
    pub role: &'static str,
    pub tabindex: i32,
    pub aria_haspopup: &'static str,
    pub aria_expanded: bool,
    pub aria_controls: Option<String>,
    pub aria_disabled: Option<&'static str>,
    pub disabled: bool,
    pub data_state: DataState,
    pub data_highlighted: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
    pub part_attr: String,
}

impl MenubarTriggerAttributes {
    pub fn from_parts(
        id: String,
        content_id: &str,
        open: bool,
        disabled: bool,
        highlighted: bool,
        is_tabstop: bool,
    ) -> Self {
        Self::from(ui_sdk::menubar::attrs::MenubarTriggerAttributes::from_parts(
            id,
            content_id,
            open,
            disabled,
            highlighted,
            is_tabstop,
        ))
    }

    pub fn project_as_child<T, C>(&self, target: T, content: Slottable<C>) -> (T, Self, C) {
        let (target, content) = project_as_child(target, content);
        (target, self.clone(), content)
    }
}

impl From<ui_sdk::menubar::attrs::MenubarTriggerAttributes> for MenubarTriggerAttributes {
    fn from(attrs: ui_sdk::menubar::attrs::MenubarTriggerAttributes) -> Self {
        Self {
            id: attrs.id,
            role: attrs.role,
            tabindex: attrs.tabindex,
            aria_haspopup: attrs.aria_haspopup,
            aria_expanded: attrs.aria_expanded,
            aria_controls: attrs.aria_controls,
            aria_disabled: attrs.aria_disabled,
            disabled: attrs.disabled,
            data_state: attrs.data_state,
            data_highlighted: attrs.data_highlighted,
            data_disabled: attrs.data_disabled,
            part_attr: rewrite_part_attr_to_monoxus(attrs.part_attr),
        }
    }
}

impl From<MenubarTriggerAttributes> for ui_sdk::menubar::attrs::MenubarTriggerAttributes {
    fn from(attrs: MenubarTriggerAttributes) -> Self {
        Self {
            id: attrs.id,
            role: attrs.role,
            tabindex: attrs.tabindex,
            aria_haspopup: attrs.aria_haspopup,
            aria_expanded: attrs.aria_expanded,
            aria_controls: attrs.aria_controls,
            aria_disabled: attrs.aria_disabled,
            disabled: attrs.disabled,
            data_state: attrs.data_state,
            data_highlighted: attrs.data_highlighted,
            data_disabled: attrs.data_disabled,
            part_attr: attrs.part_attr,
        }
    }
}
