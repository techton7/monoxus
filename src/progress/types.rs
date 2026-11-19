pub const PROGRESS_PARTS: [ProgressPart; 2] = [ProgressPart::Root, ProgressPart::Indicator];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProgressPart {
    Root,
    Indicator,
}

impl ProgressPart {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Indicator => "indicator",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProgressState {
    Indeterminate,
    Loading,
    Complete,
}

impl ProgressState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Indeterminate => "indeterminate",
            Self::Loading => "loading",
            Self::Complete => "complete",
        }
    }

    pub const fn compatibility_str(self, loaded_state_alias: bool) -> &'static str {
        match self {
            Self::Complete if loaded_state_alias => "loaded",
            _ => self.as_str(),
        }
    }

    pub const fn is_complete(self) -> bool {
        matches!(self, Self::Complete)
    }

    pub const fn is_indeterminate(self) -> bool {
        matches!(self, Self::Indeterminate)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgressValidation {
    InvalidMax,
    InvalidValue,
}
