use crate::foundation::{
    compose::{Slottable, project_as_child},
    shared::data_disabled_attr,
    state::DataState,
};
use crate::menu::{MenuPart, MenuVariant};

pub use crate::menu::{
    MenuArrowAttributes as ContextMenuArrowAttributes,
    MenuCheckboxItemAttributes as ContextMenuCheckboxItemAttributes,
    MenuContentAttributes as ContextMenuContentAttributes,
    MenuGroupAttributes as ContextMenuGroupAttributes,
    MenuHeadingAttributes as ContextMenuHeadingAttributes,
    MenuItemAttributes as ContextMenuItemAttributes,
    MenuItemIndicatorAttributes as ContextMenuItemIndicatorAttributes,
    MenuRadioItemAttributes as ContextMenuRadioItemAttributes,
    MenuRootAttributes as ContextMenuRootAttributes,
    MenuSeparatorAttributes as ContextMenuSeparatorAttributes,
    MenuSubTriggerAttributes as ContextMenuSubTriggerAttributes,
};

pub const CONTEXT_MENU_TOUCH_CALLOUT_STYLE: &str = "-webkit-touch-callout: none;";

/// Attributes for `ContextMenu.Trigger` region wrapper (`CM-1`, `CM-5`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContextMenuTriggerAttributes {
    pub id: String,
    pub style: &'static str,
    pub disabled: bool,
    pub data_state: DataState,
    pub data_disabled: Option<&'static str>,
    pub part_attr: String,
}

impl ContextMenuTriggerAttributes {
    pub fn from_parts(id: String, open: bool, disabled: bool) -> Self {
        Self {
            id,
            style: CONTEXT_MENU_TOUCH_CALLOUT_STYLE,
            disabled,
            data_state: if open {
                DataState::Open
            } else {
                DataState::Closed
            },
            data_disabled: data_disabled_attr(disabled),
            part_attr: MenuPart::Trigger.data_attr_name(MenuVariant::ContextMenu),
        }
    }

    pub fn project_as_child<T, C>(&self, target: T, content: Slottable<C>) -> (T, Self, C) {
        let (target, content) = project_as_child(target, content);
        (target, self.clone(), content)
    }
}
