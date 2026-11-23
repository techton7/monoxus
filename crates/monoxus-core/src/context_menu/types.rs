use crate::menu::{MENU_CONTEXT_LONG_PRESS_MS, MenuPart};

pub const CONTEXT_MENU_GEOMETRY_NAMESPACE: &str = "context-menu";
pub const CONTEXT_MENU_RADIX_COMPATIBILITY_PREFIX: &str = "radix-context-menu";
pub const CONTEXT_MENU_RADIX_ANCHOR_LABEL: &str = "trigger";
pub const CONTEXT_MENU_DEFAULT_MODAL: bool = true;
pub const CONTEXT_MENU_DEFAULT_LOOP_FOCUS: bool = true;
pub const CONTEXT_MENU_LONG_PRESS_DELAY_MS: u64 = MENU_CONTEXT_LONG_PRESS_MS;

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

/// Screen-space virtual anchor coordinate recorded on `contextmenu` / long-press open (`CM-3`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VirtualAnchorPoint {
    pub x: f64,
    pub y: f64,
}

impl VirtualAnchorPoint {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// Outcome of evaluating a context-menu trigger gesture (`contextmenu` or long-press).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ContextMenuTriggerAction {
    OpenAtPoint { point: VirtualAnchorPoint },
    Ignore,
}
