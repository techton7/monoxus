use super::*;
use crate::foundation::{compose::Slottable, shared::ScopeHandle};

fn progress(value: Option<f64>, max: f64) -> Progress {
    Progress::new(ScopeHandle::root("progress-test"), value, max)
}

#[test]
fn defaults_are_radix_indeterminate() {
    let progress = progress(None, 100.0);
    let root = progress.root(None, false);
    assert_eq!(progress.state(), ProgressState::Indeterminate);
    assert_eq!(root.role, "progressbar");
    assert_eq!(root.aria_valuemin, "0");
    assert_eq!(root.aria_valuemax, "100");
    assert_eq!(root.aria_valuenow, None);
    assert_eq!(root.aria_valuetext, None);
    assert_eq!(root.data_state, "indeterminate");
    assert_eq!(root.data_value, None);
    assert_eq!(root.data_indeterminate, Some(""));
}

#[test]
fn normalizes_invalid_max_before_revalidating_value() {
    let progress = progress(Some(150.0), f64::NAN);
    assert_eq!(progress.max(), 100.0);
    assert_eq!(progress.value(), None);
    assert_eq!(
        progress.validation_issues(),
        &[
            ProgressValidation::InvalidMax,
            ProgressValidation::InvalidValue
        ]
    );
}

#[test]
fn rejects_non_finite_negative_and_out_of_range_values() {
    for value in [f64::NAN, f64::INFINITY, -1.0, 101.0] {
        let progress = progress(Some(value), 100.0);
        assert_eq!(progress.value(), None);
        assert_eq!(progress.state(), ProgressState::Indeterminate);
    }
}

#[test]
fn reports_loading_complete_and_compatibility_state() {
    let loading = progress(Some(25.0), 100.0).root(None, false);
    assert_eq!(loading.data_state, "loading");
    assert_eq!(loading.aria_valuetext.as_deref(), Some("25%"));

    let state = progress(Some(100.0), 100.0);
    assert_eq!(state.state(), ProgressState::Complete);
    assert_eq!(state.root(None, false).data_state, "complete");
    assert_eq!(state.root(None, true).data_state, "loaded");
}

#[test]
fn root_and_indicator_publish_synchronized_data_without_indicator_aria() {
    let progress = progress(Some(20.0), 80.0);
    let root = progress.root(Some("one quarter".to_string()), false);
    let indicator = progress.indicator(false);
    assert_eq!(root.data_state, indicator.data_state);
    assert_eq!(root.data_value, indicator.data_value);
    assert_eq!(root.data_max, indicator.data_max);
    assert_eq!(root.aria_valuetext.as_deref(), Some("one quarter"));
    assert_eq!(indicator.data_progress_indicator, "");
}

#[test]
fn as_child_projection_preserves_target_and_attributes() {
    let root = progress(Some(50.0), 100.0).root(None, false);
    let (target, attrs, content) =
        project_progress_root_as_child(&root, "article", Slottable::new("content"));
    assert_eq!(target, "article");
    assert_eq!(attrs.role, "progressbar");
    assert_eq!(attrs.data_state, "loading");
    assert_eq!(content, "content");

    let indicator = progress(Some(50.0), 100.0).indicator(false);
    let (target, attrs, content) =
        project_progress_indicator_as_child(&indicator, "span", Slottable::new("fill"));
    assert_eq!(target, "span");
    assert_eq!(attrs.data_progress_indicator, "");
    assert_eq!(content, "fill");
}
