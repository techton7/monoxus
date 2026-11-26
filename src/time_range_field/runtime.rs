use monoxus_core::foundation::date_time::field_policy::TimeFieldPolicy;

use crate::date_range_field::runtime::{
    RangeFieldContext, RangeFieldRuntime, try_use_range_field_context, use_range_field_context,
};

/// Reactive runtime for `TimeRangeField`.
pub type TimeRangeFieldRuntime = RangeFieldRuntime<TimeFieldPolicy>;

/// Dioxus context wrapping [`TimeRangeFieldRuntime`].
pub type TimeRangeFieldContext = RangeFieldContext<TimeFieldPolicy>;

/// Consumes the nearest ancestor [`TimeRangeFieldRuntime`] context (panics if outside `TimeRangeFieldRoot`).
pub fn use_time_range_field_context() -> TimeRangeFieldRuntime {
    use_range_field_context::<TimeFieldPolicy>()
}

/// Attempts to consume the nearest ancestor [`TimeRangeFieldRuntime`] context.
pub fn try_use_time_range_field_context() -> Option<TimeRangeFieldRuntime> {
    try_use_range_field_context::<TimeFieldPolicy>()
}
