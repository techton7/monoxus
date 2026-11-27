// [KEEP-MONOXUS-BRAND] Legacy Radix compat constants and `MenuPart` slice alongside `ui_sdk::menubar::types`.
use crate::menu::MenuPart;

pub use ui_sdk::menubar::types::{
    MENUBAR_DEFAULT_LOOP_FOCUS, MENUBAR_DEFAULT_MODAL, MENUBAR_GEOMETRY_NAMESPACE,
    MenubarCrossMenuResult, MenubarMenuEntry, MenubarNavigationStep,
};

pub const MENUBAR_RADIX_COMPATIBILITY_PREFIX: &str = "radix-menubar";
pub const MENUBAR_RADIX_ANCHOR_LABEL: &str = "trigger";

pub const MENUBAR_PARTS: [MenuPart; 20] = [
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
