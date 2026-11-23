use crate::foundation::{
    compose::{Slottable, project_as_child},
    shared::{Direction, data_disabled_attr},
    state::DataState,
};
use crate::menu::{MENU_ITEM_ROLE, MENU_ORIENTATION_HORIZONTAL, MENU_ROLE, MenuPart, MenuVariant};

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

pub const MENUBAR_ROLE: &str = "menubar";

/// Attributes for `Menubar.Root` (`role="menubar"`, `aria-orientation="horizontal"`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenubarRootAttributes {
    pub id: String,
    pub role: &'static str,
    pub aria_orientation: &'static str,
    pub dir: &'static str,
    pub part_attr: String,
}

impl MenubarRootAttributes {
    pub fn new(id: String, direction: Direction) -> Self {
        Self {
            id,
            role: MENUBAR_ROLE,
            aria_orientation: MENU_ORIENTATION_HORIZONTAL,
            dir: direction.as_str(),
            part_attr: MenuPart::Root.data_attr_name(MenuVariant::Menubar),
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
        let active_highlight = !disabled && (open || highlighted);
        Self {
            id,
            role: MENU_ITEM_ROLE,
            tabindex: if !disabled && is_tabstop { 0 } else { -1 },
            aria_haspopup: MENU_ROLE,
            aria_expanded: open,
            aria_controls: open.then(|| content_id.to_owned()),
            aria_disabled: disabled.then_some("true"),
            disabled,
            data_state: if open {
                DataState::Open
            } else {
                DataState::Closed
            },
            data_highlighted: active_highlight.then_some(""),
            data_disabled: data_disabled_attr(disabled),
            part_attr: MenuPart::Trigger.data_attr_name(MenuVariant::Menubar),
        }
    }

    pub fn project_as_child<T, C>(&self, target: T, content: Slottable<C>) -> (T, Self, C) {
        let (target, content) = project_as_child(target, content);
        (target, self.clone(), content)
    }
}
