use super::{
    attrs::{ProgressIndicatorAttributes, ProgressRootAttributes},
    relationships::ProgressRelationships,
    types::{PROGRESS_PARTS, ProgressPart, ProgressState, ProgressValidation},
};
use crate::foundation::shared::ScopeHandle;

pub const PROGRESS_DEFAULT_MAX: f64 = 100.0;
pub const PROGRESS_MIN: f64 = 0.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Progress {
    relationships: ProgressRelationships,
    value: Option<f64>,
    max: f64,
    validation: Vec<ProgressValidation>,
}

impl Progress {
    pub fn new(scope: ScopeHandle, value: Option<f64>, max: f64) -> Self {
        let mut validation = Vec::new();
        let max = if max.is_finite() && max > PROGRESS_MIN {
            max
        } else {
            validation.push(ProgressValidation::InvalidMax);
            PROGRESS_DEFAULT_MAX
        };

        let value = value.and_then(|value| {
            if value.is_finite() && (PROGRESS_MIN..=max).contains(&value) {
                Some(if value == 0.0 { 0.0 } else { value })
            } else {
                validation.push(ProgressValidation::InvalidValue);
                None
            }
        });

        Self {
            relationships: ProgressRelationships::new(scope),
            value,
            max,
            validation,
        }
    }

    pub const fn parts() -> &'static [ProgressPart] {
        &PROGRESS_PARTS
    }

    pub fn with_indicator_id(mut self, indicator_id: impl Into<String>) -> Self {
        self.relationships = self.relationships.with_indicator_id(indicator_id);
        self
    }

    pub fn with_root_id(mut self, root_id: impl Into<String>) -> Self {
        self.relationships = self.relationships.with_root_id(root_id);
        self
    }

    pub fn relationships(&self) -> &ProgressRelationships {
        &self.relationships
    }

    pub const fn min(&self) -> f64 {
        PROGRESS_MIN
    }

    pub const fn max(&self) -> f64 {
        self.max
    }

    pub const fn value(&self) -> Option<f64> {
        self.value
    }

    pub const fn state(&self) -> ProgressState {
        match self.value {
            None => ProgressState::Indeterminate,
            Some(value) if value >= self.max => ProgressState::Complete,
            Some(_) => ProgressState::Loading,
        }
    }

    pub fn percentage(&self) -> Option<f64> {
        self.value.map(|value| (value / self.max) * 100.0)
    }

    pub fn default_value_label(&self) -> Option<String> {
        self.percentage()
            .map(|percentage| format!("{}%", percentage.round()))
    }

    pub fn validation_issues(&self) -> &[ProgressValidation] {
        &self.validation
    }

    pub fn root(
        &self,
        value_label: Option<String>,
        loaded_state_alias: bool,
    ) -> ProgressRootAttributes {
        ProgressRootAttributes::new(
            self.relationships.root_id().to_owned(),
            self.state(),
            self.value,
            self.max,
            value_label.or_else(|| self.default_value_label()),
            loaded_state_alias,
        )
    }

    pub fn indicator(&self, loaded_state_alias: bool) -> ProgressIndicatorAttributes {
        ProgressIndicatorAttributes::new(
            self.relationships.indicator_id().to_owned(),
            self.state(),
            self.value,
            self.max,
            loaded_state_alias,
        )
    }
}
