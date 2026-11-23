use crate::menu::MenuPart;

pub const DROPDOWN_MENU_GEOMETRY_NAMESPACE: &str = "dropdown-menu";
pub const DROPDOWN_MENU_RADIX_COMPATIBILITY_PREFIX: &str = "radix-dropdown-menu";
pub const DROPDOWN_MENU_RADIX_ANCHOR_LABEL: &str = "trigger";
pub const DROPDOWN_MENU_DEFAULT_MODAL: bool = true;
pub const DROPDOWN_MENU_DEFAULT_LOOP_FOCUS: bool = true;

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

/// Keyboard action evaluated on `DropdownMenu.Trigger`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DropdownMenuTriggerAction {
    ToggleOpen,
    OpenAndFocusFirst,
    Ignore,
}
