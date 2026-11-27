// [KEEP-MONOXUS-BRAND] Compat attribute structs delegating to `ui_sdk::menu::attrs` while preserving `data-monoxus-*` `part_attr`.
use crate::foundation::{
    compose::{Slottable, project_as_child},
    shared::{Direction, Orientation},
    state::DataState,
};

pub use ui_sdk::menu::attrs::{
    MENU_GROUP_ROLE, MENU_ITEM_CHECKBOX_ROLE, MENU_ITEM_RADIO_ROLE, MENU_ITEM_ROLE,
    MENU_ORIENTATION_HORIZONTAL, MENU_ORIENTATION_VERTICAL, MENU_ROLE, MENU_SEPARATOR_ROLE,
    menu_orientation_str,
};

use super::types::{CheckedState, MenuPart, MenuVariant};

pub(crate) fn rewrite_part_attr_to_monoxus(part_attr: String) -> String {
    if let Some(rest) = part_attr.strip_prefix("data-ui-") {
        format!("data-monoxus-{rest}")
    } else {
        part_attr
    }
}

/// Attributes for a menu container / trigger root wrapper.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuRootAttributes {
    pub id: String,
    pub data_state: DataState,
    pub part_attr: String,
}

impl MenuRootAttributes {
    pub fn new(id: String, open: bool, variant: MenuVariant) -> Self {
        Self::from(ui_sdk::menu::attrs::MenuRootAttributes::new(
            id, open, variant,
        ))
    }
}

impl From<ui_sdk::menu::attrs::MenuRootAttributes> for MenuRootAttributes {
    fn from(attrs: ui_sdk::menu::attrs::MenuRootAttributes) -> Self {
        Self {
            id: attrs.id,
            data_state: attrs.data_state,
            part_attr: rewrite_part_attr_to_monoxus(attrs.part_attr),
        }
    }
}

impl From<MenuRootAttributes> for ui_sdk::menu::attrs::MenuRootAttributes {
    fn from(attrs: MenuRootAttributes) -> Self {
        Self {
            id: attrs.id,
            data_state: attrs.data_state,
            part_attr: attrs.part_attr,
        }
    }
}

/// Attributes for a menu trigger button (`DropdownMenu.Trigger`, `Menubar.Trigger`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuTriggerAttributes {
    pub id: String,
    pub aria_haspopup: &'static str,
    pub aria_expanded: bool,
    pub aria_controls: Option<String>,
    pub disabled: bool,
    pub data_state: DataState,
    pub data_disabled: Option<&'static str>,
    pub part_attr: String,
}

impl MenuTriggerAttributes {
    pub fn from_parts(
        id: String,
        content_id: &str,
        open: bool,
        disabled: bool,
        variant: MenuVariant,
    ) -> Self {
        Self::from(ui_sdk::menu::attrs::MenuTriggerAttributes::from_parts(
            id, content_id, open, disabled, variant,
        ))
    }

    pub fn project_as_child<T, C>(&self, target: T, content: Slottable<C>) -> (T, Self, C) {
        let (target, content) = project_as_child(target, content);
        (target, self.clone(), content)
    }
}

impl From<ui_sdk::menu::attrs::MenuTriggerAttributes> for MenuTriggerAttributes {
    fn from(attrs: ui_sdk::menu::attrs::MenuTriggerAttributes) -> Self {
        Self {
            id: attrs.id,
            aria_haspopup: attrs.aria_haspopup,
            aria_expanded: attrs.aria_expanded,
            aria_controls: attrs.aria_controls,
            disabled: attrs.disabled,
            data_state: attrs.data_state,
            data_disabled: attrs.data_disabled,
            part_attr: rewrite_part_attr_to_monoxus(attrs.part_attr),
        }
    }
}

impl From<MenuTriggerAttributes> for ui_sdk::menu::attrs::MenuTriggerAttributes {
    fn from(attrs: MenuTriggerAttributes) -> Self {
        Self {
            id: attrs.id,
            aria_haspopup: attrs.aria_haspopup,
            aria_expanded: attrs.aria_expanded,
            aria_controls: attrs.aria_controls,
            disabled: attrs.disabled,
            data_state: attrs.data_state,
            data_disabled: attrs.data_disabled,
            part_attr: attrs.part_attr,
        }
    }
}

/// Attributes for a floating or static menu content surface (`Content`, `ContentStatic`, `SubContent`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuContentAttributes {
    pub id: String,
    pub wrapper_id: String,
    pub role: &'static str,
    pub aria_orientation: &'static str,
    pub aria_labelledby: Option<String>,
    pub tabindex: i32,
    pub dir: &'static str,
    pub data_state: DataState,
    pub data_side: &'static str,
    pub data_align: &'static str,
    pub part_attr: String,
}

impl MenuContentAttributes {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        wrapper_id: String,
        aria_labelledby: Option<String>,
        direction: Direction,
        open: bool,
        data_side: &'static str,
        data_align: &'static str,
        variant: MenuVariant,
        part: MenuPart,
    ) -> Self {
        Self::from(ui_sdk::menu::attrs::MenuContentAttributes::new(
            id,
            wrapper_id,
            aria_labelledby,
            direction,
            open,
            data_side,
            data_align,
            variant,
            part.into(),
        ))
    }

    pub const fn with_orientation(mut self, orientation: Orientation) -> Self {
        self.aria_orientation = menu_orientation_str(orientation);
        self
    }
}

impl From<ui_sdk::menu::attrs::MenuContentAttributes> for MenuContentAttributes {
    fn from(attrs: ui_sdk::menu::attrs::MenuContentAttributes) -> Self {
        Self {
            id: attrs.id,
            wrapper_id: attrs.wrapper_id,
            role: attrs.role,
            aria_orientation: attrs.aria_orientation,
            aria_labelledby: attrs.aria_labelledby,
            tabindex: attrs.tabindex,
            dir: attrs.dir,
            data_state: attrs.data_state,
            data_side: attrs.data_side,
            data_align: attrs.data_align,
            part_attr: rewrite_part_attr_to_monoxus(attrs.part_attr),
        }
    }
}

impl From<MenuContentAttributes> for ui_sdk::menu::attrs::MenuContentAttributes {
    fn from(attrs: MenuContentAttributes) -> Self {
        Self {
            id: attrs.id,
            wrapper_id: attrs.wrapper_id,
            role: attrs.role,
            aria_orientation: attrs.aria_orientation,
            aria_labelledby: attrs.aria_labelledby,
            tabindex: attrs.tabindex,
            dir: attrs.dir,
            data_state: attrs.data_state,
            data_side: attrs.data_side,
            data_align: attrs.data_align,
            part_attr: attrs.part_attr,
        }
    }
}

/// Attributes for `Group` / `RadioGroup` / `CheckboxGroup` containers (`role="group"`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuGroupAttributes {
    pub id: String,
    pub role: &'static str,
    pub aria_labelledby: Option<String>,
    pub part_attr: String,
}

impl MenuGroupAttributes {
    pub fn new(
        id: String,
        aria_labelledby: Option<String>,
        variant: MenuVariant,
        part: MenuPart,
    ) -> Self {
        Self::from(ui_sdk::menu::attrs::MenuGroupAttributes::new(
            id,
            aria_labelledby,
            variant,
            part.into(),
        ))
    }
}

impl From<ui_sdk::menu::attrs::MenuGroupAttributes> for MenuGroupAttributes {
    fn from(attrs: ui_sdk::menu::attrs::MenuGroupAttributes) -> Self {
        Self {
            id: attrs.id,
            role: attrs.role,
            aria_labelledby: attrs.aria_labelledby,
            part_attr: rewrite_part_attr_to_monoxus(attrs.part_attr),
        }
    }
}

impl From<MenuGroupAttributes> for ui_sdk::menu::attrs::MenuGroupAttributes {
    fn from(attrs: MenuGroupAttributes) -> Self {
        Self {
            id: attrs.id,
            role: attrs.role,
            aria_labelledby: attrs.aria_labelledby,
            part_attr: attrs.part_attr,
        }
    }
}

/// Attributes for `GroupHeading` / `Label`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuHeadingAttributes {
    pub id: String,
    pub part_attr: String,
}

impl MenuHeadingAttributes {
    pub fn new(id: String, variant: MenuVariant, part: MenuPart) -> Self {
        Self::from(ui_sdk::menu::attrs::MenuHeadingAttributes::new(
            id,
            variant,
            part.into(),
        ))
    }
}

impl From<ui_sdk::menu::attrs::MenuHeadingAttributes> for MenuHeadingAttributes {
    fn from(attrs: ui_sdk::menu::attrs::MenuHeadingAttributes) -> Self {
        Self {
            id: attrs.id,
            part_attr: rewrite_part_attr_to_monoxus(attrs.part_attr),
        }
    }
}

impl From<MenuHeadingAttributes> for ui_sdk::menu::attrs::MenuHeadingAttributes {
    fn from(attrs: MenuHeadingAttributes) -> Self {
        Self {
            id: attrs.id,
            part_attr: attrs.part_attr,
        }
    }
}

/// Attributes for `Item` (`role="menuitem"`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuItemAttributes {
    pub id: String,
    pub role: &'static str,
    pub tabindex: i32,
    pub aria_disabled: Option<&'static str>,
    pub data_disabled: Option<&'static str>,
    pub data_highlighted: Option<&'static str>,
    pub data_orientation: &'static str,
    pub part_attr: String,
}

impl MenuItemAttributes {
    pub fn from_parts(
        id: String,
        disabled: bool,
        highlighted: bool,
        variant: MenuVariant,
    ) -> Self {
        Self::from(ui_sdk::menu::attrs::MenuItemAttributes::from_parts(
            id,
            disabled,
            highlighted,
            variant,
        ))
    }

    pub const fn with_orientation(mut self, orientation: Orientation) -> Self {
        self.data_orientation = menu_orientation_str(orientation);
        self
    }

    pub fn project_as_child<T, C>(&self, target: T, content: Slottable<C>) -> (T, Self, C) {
        let (target, content) = project_as_child(target, content);
        (target, self.clone(), content)
    }
}

impl From<ui_sdk::menu::attrs::MenuItemAttributes> for MenuItemAttributes {
    fn from(attrs: ui_sdk::menu::attrs::MenuItemAttributes) -> Self {
        Self {
            id: attrs.id,
            role: attrs.role,
            tabindex: attrs.tabindex,
            aria_disabled: attrs.aria_disabled,
            data_disabled: attrs.data_disabled,
            data_highlighted: attrs.data_highlighted,
            data_orientation: attrs.data_orientation,
            part_attr: rewrite_part_attr_to_monoxus(attrs.part_attr),
        }
    }
}

impl From<MenuItemAttributes> for ui_sdk::menu::attrs::MenuItemAttributes {
    fn from(attrs: MenuItemAttributes) -> Self {
        Self {
            id: attrs.id,
            role: attrs.role,
            tabindex: attrs.tabindex,
            aria_disabled: attrs.aria_disabled,
            data_disabled: attrs.data_disabled,
            data_highlighted: attrs.data_highlighted,
            data_orientation: attrs.data_orientation,
            part_attr: attrs.part_attr,
        }
    }
}

/// Attributes for `CheckboxItem` (`role="menuitemcheckbox"`, `CheckedState` support).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuCheckboxItemAttributes {
    pub id: String,
    pub role: &'static str,
    pub tabindex: i32,
    pub aria_checked: &'static str,
    pub aria_disabled: Option<&'static str>,
    pub data_state: DataState,
    pub data_disabled: Option<&'static str>,
    pub data_highlighted: Option<&'static str>,
    pub data_orientation: &'static str,
    pub part_attr: String,
}

impl MenuCheckboxItemAttributes {
    pub fn from_parts(
        id: String,
        checked: CheckedState,
        disabled: bool,
        highlighted: bool,
        variant: MenuVariant,
    ) -> Self {
        Self::from(ui_sdk::menu::attrs::MenuCheckboxItemAttributes::from_parts(
            id,
            checked,
            disabled,
            highlighted,
            variant,
        ))
    }

    pub const fn with_orientation(mut self, orientation: Orientation) -> Self {
        self.data_orientation = menu_orientation_str(orientation);
        self
    }

    pub fn project_as_child<T, C>(&self, target: T, content: Slottable<C>) -> (T, Self, C) {
        let (target, content) = project_as_child(target, content);
        (target, self.clone(), content)
    }
}

impl From<ui_sdk::menu::attrs::MenuCheckboxItemAttributes> for MenuCheckboxItemAttributes {
    fn from(attrs: ui_sdk::menu::attrs::MenuCheckboxItemAttributes) -> Self {
        Self {
            id: attrs.id,
            role: attrs.role,
            tabindex: attrs.tabindex,
            aria_checked: attrs.aria_checked,
            aria_disabled: attrs.aria_disabled,
            data_state: attrs.data_state,
            data_disabled: attrs.data_disabled,
            data_highlighted: attrs.data_highlighted,
            data_orientation: attrs.data_orientation,
            part_attr: rewrite_part_attr_to_monoxus(attrs.part_attr),
        }
    }
}

impl From<MenuCheckboxItemAttributes> for ui_sdk::menu::attrs::MenuCheckboxItemAttributes {
    fn from(attrs: MenuCheckboxItemAttributes) -> Self {
        Self {
            id: attrs.id,
            role: attrs.role,
            tabindex: attrs.tabindex,
            aria_checked: attrs.aria_checked,
            aria_disabled: attrs.aria_disabled,
            data_state: attrs.data_state,
            data_disabled: attrs.data_disabled,
            data_highlighted: attrs.data_highlighted,
            data_orientation: attrs.data_orientation,
            part_attr: attrs.part_attr,
        }
    }
}

/// Attributes for `RadioItem` (`role="menuitemradio"`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuRadioItemAttributes {
    pub id: String,
    pub role: &'static str,
    pub tabindex: i32,
    pub aria_checked: &'static str,
    pub aria_disabled: Option<&'static str>,
    pub data_state: DataState,
    pub data_disabled: Option<&'static str>,
    pub data_highlighted: Option<&'static str>,
    pub data_orientation: &'static str,
    pub part_attr: String,
}

impl MenuRadioItemAttributes {
    pub fn from_parts(
        id: String,
        checked: bool,
        disabled: bool,
        highlighted: bool,
        variant: MenuVariant,
    ) -> Self {
        Self::from(ui_sdk::menu::attrs::MenuRadioItemAttributes::from_parts(
            id,
            checked,
            disabled,
            highlighted,
            variant,
        ))
    }

    pub const fn with_orientation(mut self, orientation: Orientation) -> Self {
        self.data_orientation = menu_orientation_str(orientation);
        self
    }

    pub fn project_as_child<T, C>(&self, target: T, content: Slottable<C>) -> (T, Self, C) {
        let (target, content) = project_as_child(target, content);
        (target, self.clone(), content)
    }
}

impl From<ui_sdk::menu::attrs::MenuRadioItemAttributes> for MenuRadioItemAttributes {
    fn from(attrs: ui_sdk::menu::attrs::MenuRadioItemAttributes) -> Self {
        Self {
            id: attrs.id,
            role: attrs.role,
            tabindex: attrs.tabindex,
            aria_checked: attrs.aria_checked,
            aria_disabled: attrs.aria_disabled,
            data_state: attrs.data_state,
            data_disabled: attrs.data_disabled,
            data_highlighted: attrs.data_highlighted,
            data_orientation: attrs.data_orientation,
            part_attr: rewrite_part_attr_to_monoxus(attrs.part_attr),
        }
    }
}

impl From<MenuRadioItemAttributes> for ui_sdk::menu::attrs::MenuRadioItemAttributes {
    fn from(attrs: MenuRadioItemAttributes) -> Self {
        Self {
            id: attrs.id,
            role: attrs.role,
            tabindex: attrs.tabindex,
            aria_checked: attrs.aria_checked,
            aria_disabled: attrs.aria_disabled,
            data_state: attrs.data_state,
            data_disabled: attrs.data_disabled,
            data_highlighted: attrs.data_highlighted,
            data_orientation: attrs.data_orientation,
            part_attr: attrs.part_attr,
        }
    }
}

/// Attributes for `ItemIndicator`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuItemIndicatorAttributes {
    pub id: String,
    pub data_state: DataState,
    pub data_disabled: Option<&'static str>,
    pub visible: bool,
    pub part_attr: String,
}

impl MenuItemIndicatorAttributes {
    pub fn from_parts(
        id: String,
        checked: CheckedState,
        disabled: bool,
        force_mount: bool,
        variant: MenuVariant,
    ) -> Self {
        Self::from(
            ui_sdk::menu::attrs::MenuItemIndicatorAttributes::from_parts(
                id,
                checked,
                disabled,
                force_mount,
                variant,
            ),
        )
    }
}

impl From<ui_sdk::menu::attrs::MenuItemIndicatorAttributes> for MenuItemIndicatorAttributes {
    fn from(attrs: ui_sdk::menu::attrs::MenuItemIndicatorAttributes) -> Self {
        Self {
            id: attrs.id,
            data_state: attrs.data_state,
            data_disabled: attrs.data_disabled,
            visible: attrs.visible,
            part_attr: rewrite_part_attr_to_monoxus(attrs.part_attr),
        }
    }
}

impl From<MenuItemIndicatorAttributes> for ui_sdk::menu::attrs::MenuItemIndicatorAttributes {
    fn from(attrs: MenuItemIndicatorAttributes) -> Self {
        Self {
            id: attrs.id,
            data_state: attrs.data_state,
            data_disabled: attrs.data_disabled,
            visible: attrs.visible,
            part_attr: attrs.part_attr,
        }
    }
}

/// Attributes for `Separator`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuSeparatorAttributes {
    pub id: String,
    pub role: &'static str,
    pub aria_orientation: &'static str,
    pub part_attr: String,
}

impl MenuSeparatorAttributes {
    pub fn new(id: String, variant: MenuVariant) -> Self {
        Self::from(ui_sdk::menu::attrs::MenuSeparatorAttributes::new(
            id, variant,
        ))
    }
}

impl From<ui_sdk::menu::attrs::MenuSeparatorAttributes> for MenuSeparatorAttributes {
    fn from(attrs: ui_sdk::menu::attrs::MenuSeparatorAttributes) -> Self {
        Self {
            id: attrs.id,
            role: attrs.role,
            aria_orientation: attrs.aria_orientation,
            part_attr: rewrite_part_attr_to_monoxus(attrs.part_attr),
        }
    }
}

impl From<MenuSeparatorAttributes> for ui_sdk::menu::attrs::MenuSeparatorAttributes {
    fn from(attrs: MenuSeparatorAttributes) -> Self {
        Self {
            id: attrs.id,
            role: attrs.role,
            aria_orientation: attrs.aria_orientation,
            part_attr: attrs.part_attr,
        }
    }
}

/// Attributes for `Arrow`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuArrowAttributes {
    pub id: String,
    pub data_state: DataState,
    pub data_side: &'static str,
    pub data_align: &'static str,
    pub part_attr: String,
}

impl MenuArrowAttributes {
    pub fn new(
        id: String,
        open: bool,
        data_side: &'static str,
        data_align: &'static str,
        variant: MenuVariant,
    ) -> Self {
        Self::from(ui_sdk::menu::attrs::MenuArrowAttributes::new(
            id, open, data_side, data_align, variant,
        ))
    }
}

impl From<ui_sdk::menu::attrs::MenuArrowAttributes> for MenuArrowAttributes {
    fn from(attrs: ui_sdk::menu::attrs::MenuArrowAttributes) -> Self {
        Self {
            id: attrs.id,
            data_state: attrs.data_state,
            data_side: attrs.data_side,
            data_align: attrs.data_align,
            part_attr: rewrite_part_attr_to_monoxus(attrs.part_attr),
        }
    }
}

impl From<MenuArrowAttributes> for ui_sdk::menu::attrs::MenuArrowAttributes {
    fn from(attrs: MenuArrowAttributes) -> Self {
        Self {
            id: attrs.id,
            data_state: attrs.data_state,
            data_side: attrs.data_side,
            data_align: attrs.data_align,
            part_attr: attrs.part_attr,
        }
    }
}

/// Attributes for `SubTrigger`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuSubTriggerAttributes {
    pub id: String,
    pub role: &'static str,
    pub tabindex: i32,
    pub aria_haspopup: &'static str,
    pub aria_expanded: bool,
    pub aria_controls: Option<String>,
    pub aria_disabled: Option<&'static str>,
    pub data_state: DataState,
    pub data_disabled: Option<&'static str>,
    pub data_highlighted: Option<&'static str>,
    pub data_orientation: &'static str,
    pub part_attr: String,
}

impl MenuSubTriggerAttributes {
    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        id: String,
        sub_content_id: &str,
        open: bool,
        disabled: bool,
        highlighted: bool,
        variant: MenuVariant,
    ) -> Self {
        Self::from(ui_sdk::menu::attrs::MenuSubTriggerAttributes::from_parts(
            id,
            sub_content_id,
            open,
            disabled,
            highlighted,
            variant,
        ))
    }

    pub const fn with_orientation(mut self, orientation: Orientation) -> Self {
        self.data_orientation = menu_orientation_str(orientation);
        self
    }

    pub fn project_as_child<T, C>(&self, target: T, content: Slottable<C>) -> (T, Self, C) {
        let (target, content) = project_as_child(target, content);
        (target, self.clone(), content)
    }
}

impl From<ui_sdk::menu::attrs::MenuSubTriggerAttributes> for MenuSubTriggerAttributes {
    fn from(attrs: ui_sdk::menu::attrs::MenuSubTriggerAttributes) -> Self {
        Self {
            id: attrs.id,
            role: attrs.role,
            tabindex: attrs.tabindex,
            aria_haspopup: attrs.aria_haspopup,
            aria_expanded: attrs.aria_expanded,
            aria_controls: attrs.aria_controls,
            aria_disabled: attrs.aria_disabled,
            data_state: attrs.data_state,
            data_disabled: attrs.data_disabled,
            data_highlighted: attrs.data_highlighted,
            data_orientation: attrs.data_orientation,
            part_attr: rewrite_part_attr_to_monoxus(attrs.part_attr),
        }
    }
}

impl From<MenuSubTriggerAttributes> for ui_sdk::menu::attrs::MenuSubTriggerAttributes {
    fn from(attrs: MenuSubTriggerAttributes) -> Self {
        Self {
            id: attrs.id,
            role: attrs.role,
            tabindex: attrs.tabindex,
            aria_haspopup: attrs.aria_haspopup,
            aria_expanded: attrs.aria_expanded,
            aria_controls: attrs.aria_controls,
            aria_disabled: attrs.aria_disabled,
            data_state: attrs.data_state,
            data_disabled: attrs.data_disabled,
            data_highlighted: attrs.data_highlighted,
            data_orientation: attrs.data_orientation,
            part_attr: attrs.part_attr,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_attrs_emit_monoxus_part_attrs_for_all_parts() {
        assert_eq!(
            MenuRootAttributes::new("r".into(), true, MenuVariant::DropdownMenu).part_attr,
            "data-monoxus-dropdown-menu-root"
        );
        assert_eq!(
            MenuTriggerAttributes::from_parts("t".into(), "c", true, false, MenuVariant::Menu)
                .part_attr,
            "data-monoxus-menu-trigger"
        );
        assert_eq!(
            MenuContentAttributes::new(
                "c".into(),
                "w".into(),
                Some("t".into()),
                Direction::Ltr,
                true,
                "bottom",
                "start",
                MenuVariant::DropdownMenu,
                MenuPart::SubContentStatic,
            )
            .part_attr,
            "data-monoxus-dropdown-menu-sub-content-static"
        );
        assert_eq!(
            MenuGroupAttributes::new(
                "g".into(),
                Some("h".into()),
                MenuVariant::Menubar,
                MenuPart::RadioGroup,
            )
            .part_attr,
            "data-monoxus-menubar-radio-group"
        );
        assert_eq!(
            MenuHeadingAttributes::new("h".into(), MenuVariant::ContextMenu, MenuPart::Label)
                .part_attr,
            "data-monoxus-context-menu-label"
        );
        assert_eq!(
            MenuItemAttributes::from_parts("i".into(), false, true, MenuVariant::ContextMenu)
                .part_attr,
            "data-monoxus-context-menu-item"
        );
        assert_eq!(
            MenuCheckboxItemAttributes::from_parts(
                "cb".into(),
                CheckedState::Indeterminate,
                false,
                true,
                MenuVariant::DropdownMenu,
            )
            .part_attr,
            "data-monoxus-dropdown-menu-checkbox-item"
        );
        assert_eq!(
            MenuRadioItemAttributes::from_parts("ri".into(), true, false, false, MenuVariant::Menu)
                .part_attr,
            "data-monoxus-menu-radio-item"
        );
        assert_eq!(
            MenuItemIndicatorAttributes::from_parts(
                "ind".into(),
                CheckedState::Checked,
                false,
                false,
                MenuVariant::Menubar,
            )
            .part_attr,
            "data-monoxus-menubar-item-indicator"
        );
        assert_eq!(
            MenuSeparatorAttributes::new("sep".into(), MenuVariant::DropdownMenu).part_attr,
            "data-monoxus-dropdown-menu-separator"
        );
        assert_eq!(
            MenuArrowAttributes::new("arr".into(), true, "bottom", "center", MenuVariant::Menu)
                .part_attr,
            "data-monoxus-menu-arrow"
        );
        assert_eq!(
            MenuSubTriggerAttributes::from_parts(
                "st".into(),
                "sc",
                true,
                false,
                true,
                MenuVariant::ContextMenu,
            )
            .part_attr,
            "data-monoxus-context-menu-sub-trigger"
        );
    }
}
