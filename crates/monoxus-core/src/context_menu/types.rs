// [KEEP-MONOXUS-BRAND] Legacy Radix compat constants and `MenuPart` slice alongside `ui_sdk::context_menu::types`.
use crate::menu::MenuPart;

pub use ui_sdk::context_menu::types::{
    CONTEXT_MENU_DEFAULT_LOOP_FOCUS, CONTEXT_MENU_DEFAULT_MODAL, CONTEXT_MENU_GEOMETRY_NAMESPACE,
    CONTEXT_MENU_LONG_PRESS_DELAY_MS, CONTEXT_MENU_LONG_PRESS_MOVE_THRESHOLD_PX,
    CONTEXT_MENU_TOUCH_ANCHOR_SIZE_PX, ContextMenuTriggerAction, VirtualAnchorPoint,
};

pub const CONTEXT_MENU_RADIX_COMPATIBILITY_PREFIX: &str = "radix-context-menu";
pub const CONTEXT_MENU_RADIX_ANCHOR_LABEL: &str = "trigger";

pub const CONTEXT_MENU_PARTS: [MenuPart; 20] = [
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
