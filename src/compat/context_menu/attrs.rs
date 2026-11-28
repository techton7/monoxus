// [KEEP-MONOXUS-BRAND] Compat `ContextMenu` attribute aliases and `ContextMenuTriggerAttributes` emitting `data-monoxus-*`.
use ui_sdk::foundation::{
    compose::{Slottable, project_as_child},
    state::DataState,
};
use crate::compat::menu::attrs::rewrite_part_attr_to_monoxus;

pub use crate::compat::menu::{
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

pub use ui_sdk::context_menu::attrs::CONTEXT_MENU_TOUCH_CALLOUT_STYLE;

/// Attributes for `ContextMenu.Trigger` region wrapper.
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
        Self::from(ui_sdk::context_menu::attrs::ContextMenuTriggerAttributes::from_parts(
            id, open, disabled,
        ))
    }

    pub fn project_as_child<T, C>(&self, target: T, content: Slottable<C>) -> (T, Self, C) {
        let (target, content) = project_as_child(target, content);
        (target, self.clone(), content)
    }
}

impl From<ui_sdk::context_menu::attrs::ContextMenuTriggerAttributes>
    for ContextMenuTriggerAttributes
{
    fn from(attrs: ui_sdk::context_menu::attrs::ContextMenuTriggerAttributes) -> Self {
        Self {
            id: attrs.id,
            style: attrs.style,
            disabled: attrs.disabled,
            data_state: attrs.data_state,
            data_disabled: attrs.data_disabled,
            part_attr: rewrite_part_attr_to_monoxus(attrs.part_attr),
        }
    }
}

impl From<ContextMenuTriggerAttributes>
    for ui_sdk::context_menu::attrs::ContextMenuTriggerAttributes
{
    fn from(attrs: ContextMenuTriggerAttributes) -> Self {
        Self {
            id: attrs.id,
            style: attrs.style,
            disabled: attrs.disabled,
            data_state: attrs.data_state,
            data_disabled: attrs.data_disabled,
            part_attr: attrs.part_attr,
        }
    }
}
