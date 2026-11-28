// [KEEP-MONOXUS-BRAND] Compat `MenuPart` preserving `data-monoxus-*` attribute names alongside `ui_sdk::menu::types`.
use serde::{Deserialize, Serialize};

pub use ui_sdk::menu::types::{
    CheckedState, DataState, Direction, DismissBehavior, MENU_CONTEXT_DEFAULT_SIDE_OFFSET,
    MENU_CONTEXT_LONG_PRESS_MS, MENU_DEFAULT_CLOSE_PARENT_ON_ESC, MENU_DEFAULT_COLLISION_PADDING,
    MENU_DEFAULT_CSS_PREFIX, MENU_DEFAULT_HIGHLIGHT_ITEM_ON_HOVER, MENU_DEFAULT_LOOP_FOCUS,
    MENU_DEFAULT_MODAL, MENU_DEFAULT_SIDE_OFFSET, MENU_DEFAULT_SUBMENU_CLOSE_DELAY_MS,
    MENU_DEFAULT_SUBMENU_OPEN_DELAY_MS, MENU_POINTER_SCROLL_GUARD_PX,
    MENU_SUBMENU_CORRIDOR_BUFFER_PX, MENU_SUBMENU_CORRIDOR_FALLBACK_MS,
    MENU_SUBMENU_EDGE_BUFFER_PX, MENU_SUBMENU_GRACE_PADDING_PX, MENU_SUBMENU_MIN_PROGRESS_PX,
    MENU_SUBMENU_RETURN_DISENGAGE_MS, MENU_SUBMENU_TRIGGER_INSET_PX, MENU_TYPEAHEAD_TIMEOUT_MS,
    MenuHighlightReason, MenuInteractionOrigin, MenuItemEntry, MenuItemKind, MenuKey,
    MenuNavigationStep, MenuOpenChangeReason, MenuOpenFocusTarget, MenuOpenRequest,
    MenuOutsideAction, MenuOutsideEvent, MenuPreventableEvent, MenuSelectEvent, MenuVariant,
    Orientation, PlacementAlign, PlacementSide, Point2D, PortalHost, PressedItemSnapshot, Rect2D,
    StickyStrategy, SubmenuCorridorDirection, SubmenuHoverConfig, SubmenuIntentConfig,
    SubmenuPointerEvaluation, TypeaheadConfig,
};

/// Canonical parts exposed across `Menu` and its wrapper primitives, emitting `data-monoxus-*` markers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
pub enum MenuPart {
    Root,
    Trigger,
    Anchor,
    Portal,
    Content,
    ContentStatic,
    Group,
    GroupHeading,
    Label,
    Item,
    CheckboxItem,
    CheckboxGroup,
    RadioGroup,
    RadioItem,
    ItemIndicator,
    Separator,
    Arrow,
    Sub,
    SubTrigger,
    SubContent,
    SubContentStatic,
}

impl MenuPart {
    pub const fn to_ui_sdk(self) -> ui_sdk::menu::MenuPart {
        match self {
            Self::Root => ui_sdk::menu::MenuPart::Root,
            Self::Trigger => ui_sdk::menu::MenuPart::Trigger,
            Self::Anchor => ui_sdk::menu::MenuPart::Anchor,
            Self::Portal => ui_sdk::menu::MenuPart::Portal,
            Self::Content => ui_sdk::menu::MenuPart::Content,
            Self::ContentStatic => ui_sdk::menu::MenuPart::ContentStatic,
            Self::Group => ui_sdk::menu::MenuPart::Group,
            Self::GroupHeading => ui_sdk::menu::MenuPart::GroupHeading,
            Self::Label => ui_sdk::menu::MenuPart::Label,
            Self::Item => ui_sdk::menu::MenuPart::Item,
            Self::CheckboxItem => ui_sdk::menu::MenuPart::CheckboxItem,
            Self::CheckboxGroup => ui_sdk::menu::MenuPart::CheckboxGroup,
            Self::RadioGroup => ui_sdk::menu::MenuPart::RadioGroup,
            Self::RadioItem => ui_sdk::menu::MenuPart::RadioItem,
            Self::ItemIndicator => ui_sdk::menu::MenuPart::ItemIndicator,
            Self::Separator => ui_sdk::menu::MenuPart::Separator,
            Self::Arrow => ui_sdk::menu::MenuPart::Arrow,
            Self::Sub => ui_sdk::menu::MenuPart::Sub,
            Self::SubTrigger => ui_sdk::menu::MenuPart::SubTrigger,
            Self::SubContent => ui_sdk::menu::MenuPart::SubContent,
            Self::SubContentStatic => ui_sdk::menu::MenuPart::SubContentStatic,
        }
    }

    pub const fn from_ui_sdk(part: ui_sdk::menu::MenuPart) -> Self {
        match part {
            ui_sdk::menu::MenuPart::Root => Self::Root,
            ui_sdk::menu::MenuPart::Trigger => Self::Trigger,
            ui_sdk::menu::MenuPart::Anchor => Self::Anchor,
            ui_sdk::menu::MenuPart::Portal => Self::Portal,
            ui_sdk::menu::MenuPart::Content => Self::Content,
            ui_sdk::menu::MenuPart::ContentStatic => Self::ContentStatic,
            ui_sdk::menu::MenuPart::Group => Self::Group,
            ui_sdk::menu::MenuPart::GroupHeading => Self::GroupHeading,
            ui_sdk::menu::MenuPart::Label => Self::Label,
            ui_sdk::menu::MenuPart::Item => Self::Item,
            ui_sdk::menu::MenuPart::CheckboxItem => Self::CheckboxItem,
            ui_sdk::menu::MenuPart::CheckboxGroup => Self::CheckboxGroup,
            ui_sdk::menu::MenuPart::RadioGroup => Self::RadioGroup,
            ui_sdk::menu::MenuPart::RadioItem => Self::RadioItem,
            ui_sdk::menu::MenuPart::ItemIndicator => Self::ItemIndicator,
            ui_sdk::menu::MenuPart::Separator => Self::Separator,
            ui_sdk::menu::MenuPart::Arrow => Self::Arrow,
            ui_sdk::menu::MenuPart::Sub => Self::Sub,
            ui_sdk::menu::MenuPart::SubTrigger => Self::SubTrigger,
            ui_sdk::menu::MenuPart::SubContent => Self::SubContent,
            ui_sdk::menu::MenuPart::SubContentStatic => Self::SubContentStatic,
        }
    }

    pub const fn as_str(self) -> &'static str {
        self.to_ui_sdk().as_str()
    }

    pub const fn static_data_attr_name(self, variant: MenuVariant) -> &'static str {
        match (variant, self) {
            (MenuVariant::Menu, Self::Root) => "data-monoxus-menu-root",
            (MenuVariant::Menu, Self::Trigger) => "data-monoxus-menu-trigger",
            (MenuVariant::Menu, Self::Anchor) => "data-monoxus-menu-anchor",
            (MenuVariant::Menu, Self::Portal) => "data-monoxus-menu-portal",
            (MenuVariant::Menu, Self::Content) => "data-monoxus-menu-content",
            (MenuVariant::Menu, Self::ContentStatic) => "data-monoxus-menu-content-static",
            (MenuVariant::Menu, Self::Group) => "data-monoxus-menu-group",
            (MenuVariant::Menu, Self::GroupHeading) => "data-monoxus-menu-group-heading",
            (MenuVariant::Menu, Self::Label) => "data-monoxus-menu-label",
            (MenuVariant::Menu, Self::Item) => "data-monoxus-menu-item",
            (MenuVariant::Menu, Self::CheckboxItem) => "data-monoxus-menu-checkbox-item",
            (MenuVariant::Menu, Self::CheckboxGroup) => "data-monoxus-menu-checkbox-group",
            (MenuVariant::Menu, Self::RadioGroup) => "data-monoxus-menu-radio-group",
            (MenuVariant::Menu, Self::RadioItem) => "data-monoxus-menu-radio-item",
            (MenuVariant::Menu, Self::ItemIndicator) => "data-monoxus-menu-item-indicator",
            (MenuVariant::Menu, Self::Separator) => "data-monoxus-menu-separator",
            (MenuVariant::Menu, Self::Arrow) => "data-monoxus-menu-arrow",
            (MenuVariant::Menu, Self::Sub) => "data-monoxus-menu-sub",
            (MenuVariant::Menu, Self::SubTrigger) => "data-monoxus-menu-sub-trigger",
            (MenuVariant::Menu, Self::SubContent) => "data-monoxus-menu-sub-content",
            (MenuVariant::Menu, Self::SubContentStatic) => "data-monoxus-menu-sub-content-static",

            (MenuVariant::DropdownMenu, Self::Root) => "data-monoxus-dropdown-menu-root",
            (MenuVariant::DropdownMenu, Self::Trigger) => "data-monoxus-dropdown-menu-trigger",
            (MenuVariant::DropdownMenu, Self::Anchor) => "data-monoxus-dropdown-menu-anchor",
            (MenuVariant::DropdownMenu, Self::Portal) => "data-monoxus-dropdown-menu-portal",
            (MenuVariant::DropdownMenu, Self::Content) => "data-monoxus-dropdown-menu-content",
            (MenuVariant::DropdownMenu, Self::ContentStatic) => {
                "data-monoxus-dropdown-menu-content-static"
            }
            (MenuVariant::DropdownMenu, Self::Group) => "data-monoxus-dropdown-menu-group",
            (MenuVariant::DropdownMenu, Self::GroupHeading) => {
                "data-monoxus-dropdown-menu-group-heading"
            }
            (MenuVariant::DropdownMenu, Self::Label) => "data-monoxus-dropdown-menu-label",
            (MenuVariant::DropdownMenu, Self::Item) => "data-monoxus-dropdown-menu-item",
            (MenuVariant::DropdownMenu, Self::CheckboxItem) => {
                "data-monoxus-dropdown-menu-checkbox-item"
            }
            (MenuVariant::DropdownMenu, Self::CheckboxGroup) => {
                "data-monoxus-dropdown-menu-checkbox-group"
            }
            (MenuVariant::DropdownMenu, Self::RadioGroup) => {
                "data-monoxus-dropdown-menu-radio-group"
            }
            (MenuVariant::DropdownMenu, Self::RadioItem) => "data-monoxus-dropdown-menu-radio-item",
            (MenuVariant::DropdownMenu, Self::ItemIndicator) => {
                "data-monoxus-dropdown-menu-item-indicator"
            }
            (MenuVariant::DropdownMenu, Self::Separator) => "data-monoxus-dropdown-menu-separator",
            (MenuVariant::DropdownMenu, Self::Arrow) => "data-monoxus-dropdown-menu-arrow",
            (MenuVariant::DropdownMenu, Self::Sub) => "data-monoxus-dropdown-menu-sub",
            (MenuVariant::DropdownMenu, Self::SubTrigger) => {
                "data-monoxus-dropdown-menu-sub-trigger"
            }
            (MenuVariant::DropdownMenu, Self::SubContent) => {
                "data-monoxus-dropdown-menu-sub-content"
            }
            (MenuVariant::DropdownMenu, Self::SubContentStatic) => {
                "data-monoxus-dropdown-menu-sub-content-static"
            }

            (MenuVariant::ContextMenu, Self::Root) => "data-monoxus-context-menu-root",
            (MenuVariant::ContextMenu, Self::Trigger) => "data-monoxus-context-menu-trigger",
            (MenuVariant::ContextMenu, Self::Anchor) => "data-monoxus-context-menu-anchor",
            (MenuVariant::ContextMenu, Self::Portal) => "data-monoxus-context-menu-portal",
            (MenuVariant::ContextMenu, Self::Content) => "data-monoxus-context-menu-content",
            (MenuVariant::ContextMenu, Self::ContentStatic) => {
                "data-monoxus-context-menu-content-static"
            }
            (MenuVariant::ContextMenu, Self::Group) => "data-monoxus-context-menu-group",
            (MenuVariant::ContextMenu, Self::GroupHeading) => {
                "data-monoxus-context-menu-group-heading"
            }
            (MenuVariant::ContextMenu, Self::Label) => "data-monoxus-context-menu-label",
            (MenuVariant::ContextMenu, Self::Item) => "data-monoxus-context-menu-item",
            (MenuVariant::ContextMenu, Self::CheckboxItem) => {
                "data-monoxus-context-menu-checkbox-item"
            }
            (MenuVariant::ContextMenu, Self::CheckboxGroup) => {
                "data-monoxus-context-menu-checkbox-group"
            }
            (MenuVariant::ContextMenu, Self::RadioGroup) => "data-monoxus-context-menu-radio-group",
            (MenuVariant::ContextMenu, Self::RadioItem) => "data-monoxus-context-menu-radio-item",
            (MenuVariant::ContextMenu, Self::ItemIndicator) => {
                "data-monoxus-context-menu-item-indicator"
            }
            (MenuVariant::ContextMenu, Self::Separator) => "data-monoxus-context-menu-separator",
            (MenuVariant::ContextMenu, Self::Arrow) => "data-monoxus-context-menu-arrow",
            (MenuVariant::ContextMenu, Self::Sub) => "data-monoxus-context-menu-sub",
            (MenuVariant::ContextMenu, Self::SubTrigger) => "data-monoxus-context-menu-sub-trigger",
            (MenuVariant::ContextMenu, Self::SubContent) => "data-monoxus-context-menu-sub-content",
            (MenuVariant::ContextMenu, Self::SubContentStatic) => {
                "data-monoxus-context-menu-sub-content-static"
            }

            (MenuVariant::Menubar, Self::Root) => "data-monoxus-menubar-root",
            (MenuVariant::Menubar, Self::Trigger) => "data-monoxus-menubar-trigger",
            (MenuVariant::Menubar, Self::Anchor) => "data-monoxus-menubar-anchor",
            (MenuVariant::Menubar, Self::Portal) => "data-monoxus-menubar-portal",
            (MenuVariant::Menubar, Self::Content) => "data-monoxus-menubar-content",
            (MenuVariant::Menubar, Self::ContentStatic) => "data-monoxus-menubar-content-static",
            (MenuVariant::Menubar, Self::Group) => "data-monoxus-menubar-group",
            (MenuVariant::Menubar, Self::GroupHeading) => "data-monoxus-menubar-group-heading",
            (MenuVariant::Menubar, Self::Label) => "data-monoxus-menubar-label",
            (MenuVariant::Menubar, Self::Item) => "data-monoxus-menubar-item",
            (MenuVariant::Menubar, Self::CheckboxItem) => "data-monoxus-menubar-checkbox-item",
            (MenuVariant::Menubar, Self::CheckboxGroup) => "data-monoxus-menubar-checkbox-group",
            (MenuVariant::Menubar, Self::RadioGroup) => "data-monoxus-menubar-radio-group",
            (MenuVariant::Menubar, Self::RadioItem) => "data-monoxus-menubar-radio-item",
            (MenuVariant::Menubar, Self::ItemIndicator) => "data-monoxus-menubar-item-indicator",
            (MenuVariant::Menubar, Self::Separator) => "data-monoxus-menubar-separator",
            (MenuVariant::Menubar, Self::Arrow) => "data-monoxus-menubar-arrow",
            (MenuVariant::Menubar, Self::Sub) => "data-monoxus-menubar-sub",
            (MenuVariant::Menubar, Self::SubTrigger) => "data-monoxus-menubar-sub-trigger",
            (MenuVariant::Menubar, Self::SubContent) => "data-monoxus-menubar-sub-content",
            (MenuVariant::Menubar, Self::SubContentStatic) => {
                "data-monoxus-menubar-sub-content-static"
            }
        }
    }

    pub fn data_attr_name(self, variant: MenuVariant) -> String {
        self.static_data_attr_name(variant).to_owned()
    }

    pub fn data_attr_name_with_prefix(self, prefix: &str, variant: MenuVariant) -> String {
        self.to_ui_sdk().data_attr_name_with_prefix(prefix, variant)
    }
}

impl From<MenuPart> for ui_sdk::menu::MenuPart {
    #[inline]
    fn from(part: MenuPart) -> Self {
        part.to_ui_sdk()
    }
}

impl From<ui_sdk::menu::MenuPart> for MenuPart {
    #[inline]
    fn from(part: ui_sdk::menu::MenuPart) -> Self {
        Self::from_ui_sdk(part)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_part_emits_monoxus_data_attrs_and_roundtrips_with_ui_sdk() {
        let all_parts = [
            MenuPart::Root,
            MenuPart::Trigger,
            MenuPart::Anchor,
            MenuPart::Portal,
            MenuPart::Content,
            MenuPart::ContentStatic,
            MenuPart::Group,
            MenuPart::GroupHeading,
            MenuPart::Label,
            MenuPart::Item,
            MenuPart::CheckboxItem,
            MenuPart::CheckboxGroup,
            MenuPart::RadioGroup,
            MenuPart::RadioItem,
            MenuPart::ItemIndicator,
            MenuPart::Separator,
            MenuPart::Arrow,
            MenuPart::Sub,
            MenuPart::SubTrigger,
            MenuPart::SubContent,
            MenuPart::SubContentStatic,
        ];
        let all_variants = [
            MenuVariant::Menu,
            MenuVariant::DropdownMenu,
            MenuVariant::ContextMenu,
            MenuVariant::Menubar,
        ];

        for part in all_parts {
            let sdk_part: ui_sdk::menu::MenuPart = part.into();
            assert_eq!(MenuPart::from(sdk_part), part);
            assert_eq!(part.as_str(), sdk_part.as_str());

            for variant in all_variants {
                let expected = format!("data-monoxus-{}-{}", variant.as_str(), part.as_str());
                assert_eq!(part.static_data_attr_name(variant), expected);
                assert_eq!(part.data_attr_name(variant), expected);
            }
        }
    }
}
