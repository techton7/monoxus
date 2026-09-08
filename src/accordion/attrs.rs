use crate::foundation::state::DataState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccordionRootAttributes {
    pub(crate) id: String,
    pub(crate) data_orientation: &'static str,
    pub(crate) disabled: bool,
}

impl AccordionRootAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccordionItemAttributes {
    pub(crate) id: String,
    pub(crate) data_state: DataState,
    pub(crate) data_orientation: &'static str,
    pub(crate) data_value: String,
    pub(crate) disabled: bool,
}

impl AccordionItemAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn data_state(&self) -> DataState {
        self.data_state.clone()
    }

    pub fn data_state_str(&self) -> &'static str {
        match self.data_state {
            DataState::Open => "open",
            _ => "closed",
        }
    }

    pub fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub fn data_value(&self) -> &str {
        &self.data_value
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccordionHeaderAttributes {
    pub(crate) id: String,
    pub(crate) role: &'static str,
    pub(crate) aria_level: u32,
    pub(crate) data_heading_level: u32,
    pub(crate) data_state: DataState,
    pub(crate) data_orientation: &'static str,
    pub(crate) data_value: String,
    pub(crate) disabled: bool,
}

impl AccordionHeaderAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn role(&self) -> &'static str {
        self.role
    }

    pub fn aria_level(&self) -> u32 {
        self.aria_level
    }

    pub fn data_heading_level(&self) -> u32 {
        self.data_heading_level
    }

    pub fn data_state(&self) -> DataState {
        self.data_state.clone()
    }

    pub fn data_state_str(&self) -> &'static str {
        match self.data_state {
            DataState::Open => "open",
            _ => "closed",
        }
    }

    pub fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub fn data_value(&self) -> &str {
        &self.data_value
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccordionTriggerAttributes {
    pub(crate) id: String,
    pub(crate) role: &'static str,
    pub(crate) is_open: bool,
    pub(crate) aria_expanded: &'static str,
    pub(crate) aria_controls: String,
    pub(crate) aria_disabled: Option<&'static str>,
    pub(crate) tabindex: i32,
    pub(crate) data_state: DataState,
    pub(crate) data_orientation: &'static str,
    pub(crate) data_value: String,
    pub(crate) disabled: bool,
}

impl AccordionTriggerAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn role(&self) -> &'static str {
        self.role
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn aria_expanded(&self) -> &'static str {
        self.aria_expanded
    }

    pub fn aria_controls(&self) -> &str {
        &self.aria_controls
    }

    pub fn aria_disabled(&self) -> Option<&'static str> {
        self.aria_disabled
    }

    pub fn tabindex(&self) -> i32 {
        self.tabindex
    }

    pub fn data_state(&self) -> DataState {
        self.data_state.clone()
    }

    pub fn data_state_str(&self) -> &'static str {
        match self.data_state {
            DataState::Open => "open",
            _ => "closed",
        }
    }

    pub fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub fn data_value(&self) -> &str {
        &self.data_value
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccordionContentAttributes {
    pub(crate) id: String,
    pub(crate) role: &'static str,
    pub(crate) aria_labelledby: String,
    pub(crate) hidden: bool,
    pub(crate) data_state: DataState,
    pub(crate) data_orientation: &'static str,
    pub(crate) data_value: String,
}

impl AccordionContentAttributes {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn role(&self) -> &'static str {
        self.role
    }

    pub fn aria_labelledby(&self) -> &str {
        &self.aria_labelledby
    }

    pub fn is_hidden(&self) -> bool {
        self.hidden
    }

    pub fn data_state(&self) -> DataState {
        self.data_state.clone()
    }

    pub fn data_state_str(&self) -> &'static str {
        match self.data_state {
            DataState::Open => "open",
            _ => "closed",
        }
    }

    pub fn data_orientation(&self) -> &'static str {
        self.data_orientation
    }

    pub fn data_value(&self) -> &str {
        &self.data_value
    }
}
