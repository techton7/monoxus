use monoxus_core::foundation::date_time::field_policy::TimeFieldPolicy;

use crate::date_field::runtime::{
    SegmentFieldContext, SegmentFieldRuntime, try_use_segment_field_context,
    use_segment_field_context,
};

/// Reactive runtime for `TimeField`.
pub type TimeFieldRuntime = SegmentFieldRuntime<TimeFieldPolicy>;

/// Dioxus context wrapping [`TimeFieldRuntime`].
pub type TimeFieldContext = SegmentFieldContext<TimeFieldPolicy>;

/// Consumes the nearest ancestor [`TimeFieldRuntime`] context (panics if outside `TimeFieldRoot`).
pub fn use_time_field_context() -> TimeFieldRuntime {
    use_segment_field_context::<TimeFieldPolicy>()
}

/// Attempts to consume the nearest ancestor [`TimeFieldRuntime`] context.
pub fn try_use_time_field_context() -> Option<TimeFieldRuntime> {
    try_use_segment_field_context::<TimeFieldPolicy>()
}
