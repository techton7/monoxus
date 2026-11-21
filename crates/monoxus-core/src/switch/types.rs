use std::rc::Rc;

pub const SWITCH_PARTS: [SwitchPart; 2] = [SwitchPart::Root, SwitchPart::Thumb];

/// Structural parts comprising the headless switch primitive family.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SwitchPart {
    Root,
    Thumb,
}

impl SwitchPart {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Thumb => "thumb",
        }
    }
}

pub type SwitchCheckedChangeHandler = Rc<dyn Fn(bool)>;
