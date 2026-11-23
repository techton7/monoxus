use crate::foundation::{
    compose::{Slottable, project_as_child},
    shared::{Direction, data_disabled_attr},
    state::DataState,
};

use super::types::{CheckedState, MenuPart, MenuVariant};

pub const MENU_ROLE: &str = "menu";
pub const MENU_ITEM_ROLE: &str = "menuitem";
pub const MENU_ITEM_CHECKBOX_ROLE: &str = "menuitemcheckbox";
pub const MENU_ITEM_RADIO_ROLE: &str = "menuitemradio";
pub const MENU_GROUP_ROLE: &str = "group";
pub const MENU_SEPARATOR_ROLE: &str = "separator";
pub const MENU_ORIENTATION_VERTICAL: &str = "vertical";
pub const MENU_ORIENTATION_HORIZONTAL: &str = "horizontal";

/// Attributes for a menu container / trigger root wrapper.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuRootAttributes {
    pub id: String,
    pub data_state: DataState,
    pub part_attr: String,
}

impl MenuRootAttributes {
    pub fn new(id: String, open: bool, variant: MenuVariant) -> Self {
        Self {
            id,
            data_state: if open {
                DataState::Open
            } else {
                DataState::Closed
            },
            part_attr: MenuPart::Root.data_attr_name(variant),
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
        Self {
            id,
            aria_haspopup: MENU_ROLE,
            aria_expanded: open,
            aria_controls: open.then(|| content_id.to_owned()),
            disabled,
            data_state: if open {
                DataState::Open
            } else {
                DataState::Closed
            },
            data_disabled: data_disabled_attr(disabled),
            part_attr: MenuPart::Trigger.data_attr_name(variant),
        }
    }

    pub fn project_as_child<T, C>(&self, target: T, content: Slottable<C>) -> (T, Self, C) {
        let (target, content) = project_as_child(target, content);
        (target, self.clone(), content)
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
        Self {
            id,
            wrapper_id,
            role: MENU_ROLE,
            aria_orientation: MENU_ORIENTATION_VERTICAL,
            aria_labelledby,
            tabindex: -1,
            dir: direction.as_str(),
            data_state: if open {
                DataState::Open
            } else {
                DataState::Closed
            },
            data_side,
            data_align,
            part_attr: part.data_attr_name(variant),
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
        Self {
            id,
            role: MENU_GROUP_ROLE,
            aria_labelledby,
            part_attr: part.data_attr_name(variant),
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
        Self {
            id,
            part_attr: part.data_attr_name(variant),
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
        let active = !disabled && highlighted;
        Self {
            id,
            role: MENU_ITEM_ROLE,
            tabindex: if active { 0 } else { -1 },
            aria_disabled: disabled.then_some("true"),
            data_disabled: data_disabled_attr(disabled),
            data_highlighted: active.then_some(""),
            data_orientation: MENU_ORIENTATION_VERTICAL,
            part_attr: MenuPart::Item.data_attr_name(variant),
        }
    }

    pub fn project_as_child<T, C>(&self, target: T, content: Slottable<C>) -> (T, Self, C) {
        let (target, content) = project_as_child(target, content);
        (target, self.clone(), content)
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
        let active = !disabled && highlighted;
        Self {
            id,
            role: MENU_ITEM_CHECKBOX_ROLE,
            tabindex: if active { 0 } else { -1 },
            aria_checked: checked.aria_checked(),
            aria_disabled: disabled.then_some("true"),
            data_state: checked.data_state(),
            data_disabled: data_disabled_attr(disabled),
            data_highlighted: active.then_some(""),
            data_orientation: MENU_ORIENTATION_VERTICAL,
            part_attr: MenuPart::CheckboxItem.data_attr_name(variant),
        }
    }

    pub fn project_as_child<T, C>(&self, target: T, content: Slottable<C>) -> (T, Self, C) {
        let (target, content) = project_as_child(target, content);
        (target, self.clone(), content)
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
        let active = !disabled && highlighted;
        Self {
            id,
            role: MENU_ITEM_RADIO_ROLE,
            tabindex: if active { 0 } else { -1 },
            aria_checked: if checked { "true" } else { "false" },
            aria_disabled: disabled.then_some("true"),
            data_state: if checked {
                DataState::Checked
            } else {
                DataState::Unchecked
            },
            data_disabled: data_disabled_attr(disabled),
            data_highlighted: active.then_some(""),
            data_orientation: MENU_ORIENTATION_VERTICAL,
            part_attr: MenuPart::RadioItem.data_attr_name(variant),
        }
    }

    pub fn project_as_child<T, C>(&self, target: T, content: Slottable<C>) -> (T, Self, C) {
        let (target, content) = project_as_child(target, content);
        (target, self.clone(), content)
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
        Self {
            id,
            data_state: checked.data_state(),
            data_disabled: data_disabled_attr(disabled),
            visible: force_mount || !matches!(checked, CheckedState::Unchecked),
            part_attr: MenuPart::ItemIndicator.data_attr_name(variant),
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
        Self {
            id,
            role: MENU_SEPARATOR_ROLE,
            aria_orientation: MENU_ORIENTATION_HORIZONTAL,
            part_attr: MenuPart::Separator.data_attr_name(variant),
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
        Self {
            id,
            data_state: if open {
                DataState::Open
            } else {
                DataState::Closed
            },
            data_side,
            data_align,
            part_attr: MenuPart::Arrow.data_attr_name(variant),
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
        let active = !disabled && highlighted;
        Self {
            id,
            role: MENU_ITEM_ROLE,
            tabindex: if active { 0 } else { -1 },
            aria_haspopup: MENU_ROLE,
            aria_expanded: open,
            aria_controls: open.then(|| sub_content_id.to_owned()),
            aria_disabled: disabled.then_some("true"),
            data_state: if open {
                DataState::Open
            } else {
                DataState::Closed
            },
            data_disabled: data_disabled_attr(disabled),
            data_highlighted: active.then_some(""),
            data_orientation: MENU_ORIENTATION_VERTICAL,
            part_attr: MenuPart::SubTrigger.data_attr_name(variant),
        }
    }

    pub fn project_as_child<T, C>(&self, target: T, content: Slottable<C>) -> (T, Self, C) {
        let (target, content) = project_as_child(target, content);
        (target, self.clone(), content)
    }
}
