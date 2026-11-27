// [KEEP-MONOXUS-BRAND] Legacy Radix compat constants and `MenuPart` slice alongside `ui_sdk::dropdown_menu::types`.
use crate::menu::MenuPart;

pub use ui_sdk::dropdown_menu::types::{
    DROPDOWN_MENU_DEFAULT_HOVER_CLOSE_DELAY_MS, DROPDOWN_MENU_DEFAULT_HOVER_OPEN_DELAY_MS,
    DROPDOWN_MENU_DEFAULT_LOOP_FOCUS, DROPDOWN_MENU_DEFAULT_MODAL,
    DROPDOWN_MENU_DEFAULT_OPEN_ON_HOVER, DROPDOWN_MENU_GEOMETRY_NAMESPACE,
    DropdownMenuTriggerAction,
};

pub const DROPDOWN_MENU_RADIX_COMPATIBILITY_PREFIX: &str = "radix-dropdown-menu";
pub const DROPDOWN_MENU_RADIX_ANCHOR_LABEL: &str = "trigger";

pub const DROPDOWN_MENU_PARTS: [MenuPart; 20] = [
    MenuPart::Root,
    MenuPart::Trigger,
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
