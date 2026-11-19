use dioxus::prelude::*;

use super::{
    attrs::{ProgressIndicatorAttributes, ProgressRootAttributes},
    state::Progress,
};

pub use crate::foundation::compose::{
    MountedHandle as ProgressMountedHandle, compose_part_event_handlers, compose_part_refs,
    project_as_child,
};

#[derive(Clone)]
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
                super::types::ProgressValidation::InvalidMax => {
                    eprintln!(
                        "monoxus::Progress: max must be finite and greater than 0; using 100"
                    );
                }
                super::types::ProgressValidation::InvalidValue => {
                    eprintln!(
                        "monoxus::Progress: value must be finite and between 0 and max; treating as indeterminate"
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
