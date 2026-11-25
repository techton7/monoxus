use dioxus::prelude::*;
use monoxus::{
    foundation::date_time::{
        CalendarLocale, FieldInvalidEvent, HourCycle, TimeGranularity, TimeValue,
    },
    time_field::*,
};

const TIME_FIELD_PLAYGROUND_CSS: &str = r#"
[data-time-field-input] {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    min-height: 38px;
    padding: 0 12px;
    border-radius: 0.5rem;
    border: 1px solid #cbd5e1;
    background-color: #ffffff;
    color: #0f172a;
    font-size: 0.875rem;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
}
[data-time-field-input][data-invalid] {
    border-color: #ef4444;
    background-color: #fef2f2;
}
[data-time-field-input][data-disabled] {
    background-color: #f1f5f9;
    color: #94a3b8;
    opacity: 0.7;
}
[data-time-field-label] {
    font-size: 0.8125rem;
    font-weight: 600;
    color: #334155;
    cursor: pointer;
    user-select: none;
}
[data-time-field-label][data-invalid] {
    color: #dc2626;
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

fn tf_time(iso: &str) -> TimeValue {
    TimeValue::from_iso_str(iso).expect("valid playground time")
}

#[component]
pub fn TimeFieldShowcase() -> Element {
    rsx! {
        style { "{TIME_FIELD_PLAYGROUND_CSS}" }
        div {
            class: "max-w-4xl mx-auto space-y-8",
            style: "max-width: 56rem; margin: 0 auto; display: flex; flex-direction: column; gap: 2rem;",

            div {
                class: "space-y-1.5",
                style: "display: flex; flex-direction: column; gap: 0.375rem;",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    style: "margin: 0; font-size: 1.5rem; font-weight: 700; color: #0f172a;",
                    "TimeField"
                }
                p {
                    class: "text-sm text-slate-500",
                    style: "margin: 0; font-size: 0.875rem; color: #64748b;",
                    "Segmented wall-clock time spinbutton field supporting 12h/24h cycles, dayPeriod a/p toggle, ArrowUp/Down wrap-without-carry, Backspace clear to None, min/max bounds, and hidden form input."
                }
            }

            TwelveHourTimeFieldScenario {}
            TwentyFourHourSecondsScenario {}
        }
    }
}

#[component]
fn TwelveHourTimeFieldScenario() -> Element {
    let value = use_signal(|| Some(tf_time("09:30:00")));

    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            style: "{CARD_STYLE}",

            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "1. 12-Hour TimeField (dayPeriod a/p Toggle, ArrowUp/Down, Backspace Clear) (SW5C-1, SW5C-2)"
                }
                p {
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Use digits, ArrowUp/Down, 'a'/'p' on AM/PM segment, or Backspace to clear a segment to None."
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; align-items: center; gap: 1.5rem;",
                TimeFieldRoot {
                    id: "sw5c-tf-12h",
                    value: Some(value),
                    default_placeholder: Some(tf_time("09:00:00")),
                    granularity: TimeGranularity::Minute,
                    hour_cycle: Some(HourCycle::H12),
                    locale: CalendarLocale::EnUs,
                    div {
                        style: "display: flex; flex-direction: column; gap: 0.375rem;",
                        TimeFieldLabel {
                            id: "sw5c-tf-12h-label",
                            "Meeting Time (12h)"
                        }
                        TimeFieldInput {
                            id: "sw5c-tf-12h-input",
                            name: "meeting_time_12h",
                            render_children: move |segments: Vec<TimeFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    TimeFieldSegment {
                                        key: "{seg.index}",
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        id: format!("sw5c-tf-12h-seg-{}", seg.index),
                                    }
                                }
                            }
                        }
                    }
                }

                div {
                    style: "display: flex; align-items: center; gap: 0.5rem; font-size: 0.75rem;",
                    span { style: "color: #64748b; font-weight: 600;", "ISO Value:" }
                    span {
                        id: "sw5c-tf-12h-value",
                        style: "{BADGE_STYLE}",
                        {value().map_or_else(|| String::from("none"), |v| v.to_iso_string())}
                    }
                }
            }
        }
    }
}

#[component]
fn TwentyFourHourSecondsScenario() -> Element {
    let value = use_signal(|| Some(tf_time("14:15:30")));
    let mut invalid_reason = use_signal(|| String::from("none"));

    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            style: "{CARD_STYLE}",

            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "2. 24-Hour TimeField with TimeGranularity::Second, min/max & Hidden Input (SW5C-2)"
                }
                p {
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Bounded to 08:00:00..=18:00:00 with hidden form input synchronization."
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; align-items: center; gap: 1.5rem;",
                TimeFieldRoot {
                    id: "sw5c-tf-24h",
                    value: Some(value),
                    default_placeholder: Some(tf_time("09:00:00")),
                    granularity: TimeGranularity::Second,
                    hour_cycle: Some(HourCycle::H24),
                    min_value: Some(tf_time("08:00:00")),
                    max_value: Some(tf_time("18:00:00")),
                    on_value_change: move |next: Option<TimeValue>| {
                        if let Some(v) = next {
                            let min = tf_time("08:00:00");
                            let max = tf_time("18:00:00");
                            if v >= min && v <= max {
                                invalid_reason.set(String::from("none"));
                            }
                        } else {
                            invalid_reason.set(String::from("none"));
                        }
                    },
                    on_invalid: move |evt: FieldInvalidEvent| {
                        invalid_reason.set(format!("{:?}", evt.reason));
                    },
                    div {
                        style: "display: flex; flex-direction: column; gap: 0.375rem;",
                        TimeFieldLabel {
                            id: "sw5c-tf-24h-label",
                            "Shift Window (24h HH:MM:SS)"
                        }
                        TimeFieldInput {
                            id: "sw5c-tf-24h-input",
                            name: "shift_time_24h",
                            render_children: move |segments: Vec<TimeFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    TimeFieldSegment {
                                        key: "{seg.index}",
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        id: format!("sw5c-tf-24h-seg-{}", seg.index),
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
                        span { style: "color: #64748b; font-weight: 600;", "ISO Value:" }
                        span {
                            id: "sw5c-tf-24h-value",
                            style: "{BADGE_STYLE}",
                            {value().map_or_else(|| String::from("none"), |v| v.to_iso_string())}
                        }
                    }
                    div {
                        style: "display: flex; align-items: center; gap: 0.5rem;",
                        span { style: "color: #64748b; font-weight: 600;", "Invalid Reason:" }
                        span {
                            id: "sw5c-tf-24h-invalid",
                            style: "{BADGE_STYLE}",
                            "{invalid_reason}"
                        }
                    }
                }
            }
        }
    }
}
