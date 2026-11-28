use dioxus::prelude::*;

use super::{
    attrs::{ProgressIndicatorAttributes, ProgressRootAttributes},
    state::Progress,
};

pub use crate::foundation::compose::{
    MountedHandle as ProgressMountedHandle, compose_part_event_handlers, compose_part_refs,
    project_as_child,
};

#[derive(Clone, PartialEq)]
pub struct ProgressRuntime {
    progress: Progress,
    loaded_state_alias: bool,
}

impl ProgressRuntime {
    pub fn new(progress: Progress, loaded_state_alias: bool) -> Self {
        Self {
            progress,
            loaded_state_alias,
        }
    }

    pub fn progress(&self) -> &Progress {
        &self.progress
    }

    pub fn min(&self) -> f64 {
        self.progress.min()
    }

    pub fn percentage(&self) -> Option<f64> {
        self.progress.percentage()
    }

    pub fn root(&self, value_label: Option<String>) -> ProgressRootAttributes {
        self.progress.root(value_label, self.loaded_state_alias)
    }

    pub fn indicator(&self) -> ProgressIndicatorAttributes {
        self.progress.indicator(self.loaded_state_alias)
    }

    pub fn report_diagnostics(&self) {
        #[cfg(debug_assertions)]
        for issue in self.progress.validation_issues() {
            match issue {
                super::types::ProgressValidation::InvalidMin => {
                    crate::foundation::diagnostics::warn_invalid_prop(
                        "Progress",
                        "min",
                        "min must be finite; using 0",
                    );
                }
                super::types::ProgressValidation::InvalidMax => {
                    crate::foundation::diagnostics::warn_invalid_prop(
                        "Progress",
                        "max",
                        "max must be finite and greater than min; using fallback max",
                    );
                }
                super::types::ProgressValidation::InvalidValue => {
                    crate::foundation::diagnostics::warn_invalid_prop(
                        "Progress",
                        "value",
                        "value must be finite and between min and max; treating as indeterminate",
                    );
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct ProgressContext {
    pub(crate) runtime: Signal<ProgressRuntime>,
}

impl ProgressContext {
    pub fn runtime(&self) -> ProgressRuntime {
        self.runtime.read().clone()
    }
}

pub fn use_progress_context() -> ProgressContext {
    use_context::<ProgressContext>()
}

pub fn try_use_progress_context() -> Option<ProgressContext> {
    try_consume_context::<ProgressContext>()
}

pub fn use_progress_runtime(progress: Progress, loaded_state_alias: bool) -> ProgressRuntime {
    ProgressRuntime::new(progress, loaded_state_alias)
}
