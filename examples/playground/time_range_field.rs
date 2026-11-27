use dioxus::prelude::*;
use monoxus::{
    foundation::date_time::{
        CalendarLocale, FieldInvalidEvent, HourCycle, TimeGranularity, TimeRange, TimeValue,
    },
    time_field::TimeFieldSegmentItem,
    time_range_field::*,
};

const TIME_RANGE_FIELD_PLAYGROUND_CSS: &str = r#"
[data-time-range-field-root] {
    display: inline-flex;
    flex-direction: column;
    gap: 0.375rem;
}
[data-time-range-field-label] {
    font-size: 0.8125rem;
    font-weight: 600;
    color: #334155;
    cursor: pointer;
    user-select: none;
}
[data-time-range-field-label][data-invalid] {
    color: #dc2626;
}
.trf-inputs-row {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    min-height: 38px;
    padding: 0 12px;
    border-radius: 0.5rem;
    border: 1px solid #cbd5e1;
    background-color: #ffffff;
    color: #0f172a;
    font-size: 0.875rem;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
}
[data-time-range-field-root][data-invalid] .trf-inputs-row {
    border-color: #ef4444;
    background-color: #fef2f2;
}
[data-time-field-input] {
    display: inline-flex;
    align-items: center;
    gap: 2px;
}
[data-time-field-segment] {
    padding: 2px 3px;
    border-radius: 0.25rem;
    outline: none;
    user-select: none;
}
[data-time-field-segment][data-placeholder] {
    color: #94a3b8;
}
[data-time-field-segment]:focus {
    background-color: #0f172a;
    color: #ffffff;
}
[data-time-field-segment][data-invalid]:not(:focus) {
    color: #dc2626;
}
"#;

const CARD_STYLE: &str = "border-radius: 1rem; border: 1px solid #e2e8f0; background-color: #ffffff; padding: 1.5rem; display: flex; flex-direction: column; gap: 1rem;";
const BADGE_STYLE: &str = "display: inline-flex; align-items: center; border-radius: 0.25rem; padding: 0.125rem 0.5rem; background-color: #f1f5f9; color: #334155; font-size: 0.75rem; font-weight: 700; font-family: monospace;";
const BTN_STYLE: &str = "display: inline-flex; align-items: center; height: 32px; padding: 0 12px; border-radius: 0.5rem; border: 1px solid #cbd5e1; background-color: #f8fafc; color: #334155; font-size: 0.75rem; font-weight: 600; cursor: pointer;";

fn trf_time(iso: &str) -> TimeValue {
    TimeValue::from_iso_str(iso).expect("valid playground time")
}

#[component]
pub fn TimeRangeFieldShowcase() -> Element {
    rsx! {
        style { "{TIME_RANGE_FIELD_PLAYGROUND_CSS}" }
        div {
            class: "max-w-4xl mx-auto space-y-8",
            style: "max-width: 56rem; margin: 0 auto; display: flex; flex-direction: column; gap: 2rem;",

            div {
                class: "space-y-1.5",
                style: "display: flex; flex-direction: column; gap: 0.375rem;",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    style: "margin: 0; font-size: 1.5rem; font-weight: 700; color: #0f172a;",
                    "TimeRangeField"
                }
                p {
                    class: "text-sm text-slate-500",
                    style: "margin: 0; font-size: 0.875rem; color: #64748b;",
                    "Paired Start & End segmented time inputs (12h and 24h) with cross-field traversal, overnight range support (20:00–05:00 valid by default), optional custom validate / min-max bounds, and dual hidden form inputs."
                }
            }

            TwelveHourTimeRangeScenario {}
            TwentyFourHourTimeRangeScenario {}
        }
    }
}

#[component]
fn TwelveHourTimeRangeScenario() -> Element {
    let mut range = use_signal(|| TimeRange::new(Some(trf_time("09:00:00")), Some(trf_time("17:00:00"))));
    let mut invalid_reason = use_signal(|| String::from("none"));

    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            style: "{CARD_STYLE}",

            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "1. 12-Hour TimeRangeField with Cross-Field Traversal & Optional Same-Day Validation (SW5C-3)"
                }
                p {
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "ArrowRight crosses from Start dayPeriod into End hour. Overnight ranges are valid by default; this scenario opts into same-day ordering via the validate callback (TimeRange::is_inverted)."
                }
            }

            form {
                id: "sw5c-trf-form",
                style: "display: flex; flex-wrap: wrap; align-items: flex-end; gap: 1rem;",
                TimeRangeFieldRoot {
                    id: "sw5c-trf-12h",
                    value: Some(range()),
                    default_value: TimeRange::new(
                        Some(trf_time("09:00:00")),
                        Some(trf_time("17:00:00")),
                    ),
                    default_placeholder: Some(trf_time("09:00:00")),
                    granularity: TimeGranularity::Minute,
                    hour_cycle: Some(HourCycle::H12),
                    locale: CalendarLocale::EnUs,
                    validate: Callback::new(|next: TimeRange| {
                        next.is_inverted()
                            .then(|| vec!["End time must be on or after start time".to_string()])
                    }),
                    on_value_change: move |next: TimeRange| {
                        range.set(next);
                        if !next.is_inverted() {
                            invalid_reason.set(String::from("none"));
                        }
                    },
                    on_invalid: move |evt: FieldInvalidEvent| {
                        invalid_reason.set(format!("{:?}", evt.reason));
                    },
                    TimeRangeFieldLabel {
                        id: "sw5c-trf-12h-label",
                        "Business Hours (12h)"
                    }
                    div {
                        class: "trf-inputs-row",
                        TimeRangeFieldInput {
                            kind: RangeFieldInputKind::Start,
                            id: "sw5c-trf-12h-start-input",
                            name: "hours_start",
                            render_children: move |segments: Vec<TimeFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    TimeRangeFieldSegment {
                                        key: "{seg.index}",
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        id: format!("sw5c-trf-12h-start-seg-{}", seg.index),
                                    }
                                }
                            }
                        }
                        span {
                            style: "color: #64748b; user-select: none;",
                            "–"
                        }
                        TimeRangeFieldInput {
                            kind: RangeFieldInputKind::End,
                            id: "sw5c-trf-12h-end-input",
                            name: "hours_end",
                            render_children: move |segments: Vec<TimeFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    TimeRangeFieldSegment {
                                        key: "{seg.index}",
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        id: format!("sw5c-trf-12h-end-seg-{}", seg.index),
                                    }
                                }
                            }
                        }
                    }
                }

                button {
                    id: "sw5c-trf-reset-btn",
                    r#type: "reset",
                    style: "{BTN_STYLE}",
                    "Reset Form"
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; gap: 1rem; font-size: 0.75rem;",
                div {
                    style: "display: flex; align-items: center; gap: 0.5rem;",
                    span { style: "color: #64748b; font-weight: 600;", "Start:" }
                    span {
                        id: "sw5c-trf-12h-start-value",
                        style: "{BADGE_STYLE}",
                        {range().start.map_or_else(|| String::from("none"), |v| v.to_iso_string())}
                    }
                }
                div {
                    style: "display: flex; align-items: center; gap: 0.5rem;",
                    span { style: "color: #64748b; font-weight: 600;", "End:" }
                    span {
                        id: "sw5c-trf-12h-end-value",
                        style: "{BADGE_STYLE}",
                        {range().end.map_or_else(|| String::from("none"), |v| v.to_iso_string())}
                    }
                }
                div {
                    style: "display: flex; align-items: center; gap: 0.5rem;",
                    span { style: "color: #64748b; font-weight: 600;", "Invalid Reason:" }
                    span {
                        id: "sw5c-trf-12h-invalid",
                        style: "{BADGE_STYLE}",
                        "{invalid_reason}"
                    }
                }
            }
        }
    }
}

#[component]
fn TwentyFourHourTimeRangeScenario() -> Element {
    let mut range = use_signal(|| TimeRange::new(Some(trf_time("20:00:00")), Some(trf_time("05:00:00"))));

    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            style: "{CARD_STYLE}",

            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "2. 24-Hour Overnight TimeRangeField (20:00 – 05:00 Valid by Default)"
                }
                p {
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Overnight time ranges (End < Start, e.g. 20:00 – 05:00) are valid by default, matching bits-ui. Add min_value/max_value or a custom validate callback when same-day ordering is needed."
                }
            }

            TimeRangeFieldRoot {
                id: "sw5c-trf-24h",
                value: Some(range()),
                on_value_change: move |next: TimeRange| range.set(next),
                default_placeholder: Some(trf_time("20:00:00")),
                granularity: TimeGranularity::Minute,
                hour_cycle: Some(HourCycle::H24),
                TimeRangeFieldLabel {
                    id: "sw5c-trf-24h-label",
                    "Night Shift Window (24h)"
                }
                div {
                    class: "trf-inputs-row",
                    TimeRangeFieldInput {
                        kind: RangeFieldInputKind::Start,
                        id: "sw5c-trf-24h-start-input",
                        render_children: move |segments: Vec<TimeFieldSegmentItem>| rsx! {
                            for seg in segments {
                                TimeRangeFieldSegment {
                                    key: "{seg.index}",
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                    id: format!("sw5c-trf-24h-start-seg-{}", seg.index),
                                }
                            }
                        }
                    }
                    span {
                        style: "color: #64748b; user-select: none;",
                        "–"
                    }
                    TimeRangeFieldInput {
                        kind: RangeFieldInputKind::End,
                        id: "sw5c-trf-24h-end-input",
                        render_children: move |segments: Vec<TimeFieldSegmentItem>| rsx! {
                            for seg in segments {
                                TimeRangeFieldSegment {
                                    key: "{seg.index}",
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                    id: format!("sw5c-trf-24h-end-seg-{}", seg.index),
                                }
                            }
                        }
                    }
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; gap: 1rem; font-size: 0.75rem;",
                div {
                    style: "display: flex; align-items: center; gap: 0.5rem;",
                    span { style: "color: #64748b; font-weight: 600;", "Start:" }
                    span {
                        id: "sw5c-trf-24h-start-value",
                        style: "{BADGE_STYLE}",
                        {range().start.map_or_else(|| String::from("none"), |v| v.to_iso_string())}
                    }
                }
                div {
                    style: "display: flex; align-items: center; gap: 0.5rem;",
                    span { style: "color: #64748b; font-weight: 600;", "End:" }
                    span {
                        id: "sw5c-trf-24h-end-value",
                        style: "{BADGE_STYLE}",
                        {range().end.map_or_else(|| String::from("none"), |v| v.to_iso_string())}
                    }
                }
            }
        }
    }
}
