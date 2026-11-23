use crate::menu::MenuPart;

pub const MENUBAR_GEOMETRY_NAMESPACE: &str = "menubar";
pub const MENUBAR_RADIX_COMPATIBILITY_PREFIX: &str = "radix-menubar";
pub const MENUBAR_RADIX_ANCHOR_LABEL: &str = "trigger";
pub const MENUBAR_DEFAULT_LOOP_FOCUS: bool = true;
pub const MENUBAR_DEFAULT_MODAL: bool = false;

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

/// Registered top-level menu slot owned by a `Menubar` (`MB-1`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenubarMenuEntry {
    pub value: String,
    pub trigger_id: String,
    pub content_id: String,
    pub disabled: bool,
    pub dom_order: usize,
}

impl MenubarMenuEntry {
    pub fn new(
        value: impl Into<String>,
        trigger_id: impl Into<String>,
        content_id: impl Into<String>,
        disabled: bool,
        dom_order: usize,
    ) -> Self {
        Self {
            value: value.into(),
            trigger_id: trigger_id.into(),
            content_id: content_id.into(),
            disabled,
            dom_order,
        }
    }
}

/// Roving navigation / open intent resolved from a `Menubar.Trigger` key press (`MB-2`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenubarNavigationStep {
    None,
    FocusTrigger { value: String, trigger_id: String },
    FocusAndOpenMenu { value: String, trigger_id: String },
    OpenCurrentAndFocusFirst { value: String },
    OpenCurrentAndFocusLast { value: String },
    CloseAll,
}

/// Cross-menu switch resolved when horizontal arrow keys are pressed inside open menu content (`MB-6`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenubarCrossMenuResult {
    None,
    SwitchToMenu { value: String, trigger_id: String },
}
