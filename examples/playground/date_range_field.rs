use dioxus::prelude::*;
use monoxus::{
    date_field::DateFieldSegmentItem,
    date_range_field::*,
    foundation::{
        date_time::{
            CalendarDate, CalendarLocale, DateValue, DateValueRange, FieldInvalidEvent,
        },
        shared::Direction,
    },
};

const DATE_RANGE_FIELD_PLAYGROUND_CSS: &str = r#"
[data-date-range-field-root] {
    display: inline-flex;
    flex-direction: column;
    gap: 0.375rem;
}
[data-date-range-field-label] {
    font-size: 0.8125rem;
    font-weight: 600;
    color: #334155;
    cursor: pointer;
    user-select: none;
}
[data-date-range-field-label][data-invalid] {
    color: #dc2626;
}
.drf-inputs-row {
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
[data-date-range-field-root][data-invalid] .drf-inputs-row {
    border-color: #ef4444;
    background-color: #fef2f2;
}
[data-date-field-input] {
    display: inline-flex;
    align-items: center;
    gap: 2px;
}
[data-date-field-segment] {
    padding: 2px 3px;
    border-radius: 0.25rem;
    outline: none;
    user-select: none;
}
[data-date-field-segment][data-placeholder] {
    color: #94a3b8;
}
[data-date-field-segment]:focus {
    background-color: #0f172a;
    color: #ffffff;
}
[data-date-field-segment][data-invalid]:not(:focus) {
    color: #dc2626;
}
"#;

const CARD_STYLE: &str = "border-radius: 1rem; border: 1px solid #e2e8f0; background-color: #ffffff; padding: 1.5rem; display: flex; flex-direction: column; gap: 1rem;";
const BADGE_STYLE: &str = "display: inline-flex; align-items: center; border-radius: 0.25rem; padding: 0.125rem 0.5rem; background-color: #f1f5f9; color: #334155; font-size: 0.75rem; font-weight: 700; font-family: monospace;";
const BTN_STYLE: &str = "display: inline-flex; align-items: center; height: 32px; padding: 0 12px; border-radius: 0.5rem; border: 1px solid #cbd5e1; background-color: #f8fafc; color: #334155; font-size: 0.75rem; font-weight: 600; cursor: pointer;";

fn drf_date(year: i32, month: u8, day: u8) -> DateValue {
    DateValue::Date(CalendarDate::new(year, month, day).expect("valid playground range date"))
}

#[component]
pub fn DateRangeFieldShowcase() -> Element {
    let range = use_signal(|| {
        DateValueRange::new(Some(drf_date(2024, 6, 10)), Some(drf_date(2024, 6, 15)))
    });
    let mut invalid_reason = use_signal(|| String::from("none"));

    rsx! {
        style { "{DATE_RANGE_FIELD_PLAYGROUND_CSS}" }
        div {
            class: "max-w-4xl mx-auto space-y-8",
            style: "max-width: 56rem; margin: 0 auto; display: flex; flex-direction: column; gap: 2rem;",

            div {
                class: "space-y-1.5",
                style: "display: flex; flex-direction: column; gap: 0.375rem;",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    style: "margin: 0; font-size: 1.5rem; font-weight: 700; color: #0f172a;",
                    "DateRangeField"
                }
                p {
                    class: "text-sm text-slate-500",
                    style: "margin: 0; font-size: 0.875rem; color: #64748b;",
                    "Paired Start & End segmented date inputs with seamless cross-field ArrowRight/ArrowLeft traversal, built-in end < start RangeOrder validation, and dual hidden form inputs."
                }
            }

            div {
                class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
                style: "{CARD_STYLE}",

                div {
                    style: "display: flex; flex-direction: column; gap: 0.25rem;",
                    h3 {
                        style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                        "1. Start & End Cross-Field Traversal, RangeOrder Validation & Dual Hidden Inputs (SW5C-3)"
                    }
                    p {
                        style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                        "Press ArrowRight from the last Start segment to jump to the first End segment (ArrowLeft reverses). Set End < Start to trigger FieldInvalidReason::RangeOrder and data-invalid."
                    }
                }

                form {
                    id: "sw5c-drf-form",
                    style: "display: flex; flex-wrap: wrap; align-items: flex-end; gap: 1rem;",
                    DateRangeFieldRoot {
                        id: "sw5c-drf-root",
                        value: Some(range),
                        default_value: DateValueRange::new(
                            Some(drf_date(2024, 6, 10)),
                            Some(drf_date(2024, 6, 15)),
                        ),
                        default_placeholder: Some(drf_date(2024, 6, 10)),
                        locale: CalendarLocale::EnUs,
                        on_value_change: move |next: DateValueRange| {
                            if !next.is_inverted() {
                                invalid_reason.set(String::from("none"));
                            }
                        },
                        on_invalid: move |evt: FieldInvalidEvent| {
                            invalid_reason.set(format!("{:?}", evt.reason));
                        },
                        DateRangeFieldLabel {
                            id: "sw5c-drf-label",
                            "Booking Dates"
                        }
                        div {
                            class: "drf-inputs-row",
                            DateRangeFieldInput {
                                kind: RangeFieldInputKind::Start,
                                id: "sw5c-drf-start-input",
                                name: "booking_start",
                                render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                    for seg in segments {
                                        DateRangeFieldSegment {
                                            key: "{seg.index}",
                                            part: seg.kind(),
                                            index: Some(seg.index),
                                            id: format!("sw5c-drf-start-seg-{}", seg.index),
                                        }
                                    }
                                }
                            }
                            span {
                                style: "color: #64748b; user-select: none;",
                                "–"
                            }
                            DateRangeFieldInput {
                                kind: RangeFieldInputKind::End,
                                id: "sw5c-drf-end-input",
                                name: "booking_end",
                                render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                    for seg in segments {
                                        DateRangeFieldSegment {
                                            key: "{seg.index}",
                                            part: seg.kind(),
                                            index: Some(seg.index),
                                            id: format!("sw5c-drf-end-seg-{}", seg.index),
                                        }
                                    }
                                }
                            }
                        }
                    }

                    button {
                        id: "sw5c-drf-reset-btn",
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
                            id: "sw5c-drf-start-value",
                            style: "{BADGE_STYLE}",
                            {range().start.map_or_else(|| String::from("none"), |v| v.to_iso_string())}
                        }
                    }
                    div {
                        style: "display: flex; align-items: center; gap: 0.5rem;",
                        span { style: "color: #64748b; font-weight: 600;", "End:" }
                        span {
                            id: "sw5c-drf-end-value",
                            style: "{BADGE_STYLE}",
                            {range().end.map_or_else(|| String::from("none"), |v| v.to_iso_string())}
                        }
                    }
                    div {
                        style: "display: flex; align-items: center; gap: 0.5rem;",
                        span { style: "color: #64748b; font-weight: 600;", "Invalid Reason:" }
                        span {
                            id: "sw5c-drf-invalid",
                            style: "{BADGE_STYLE}",
                            "{invalid_reason}"
                        }
                    }
                }
            }

            RtlDateRangeFieldScenario {}
        }
    }
}

#[component]
fn RtlDateRangeFieldScenario() -> Element {
    let rtl_range = use_signal(|| {
        DateValueRange::new(Some(drf_date(2025, 4, 10)), Some(drf_date(2025, 4, 20)))
    });

    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            style: "{CARD_STYLE}",

            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "2. RTL DateRangeField Cross-Field Traversal"
                }
                p {
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Direction::Rtl reverses ArrowLeft (Next) and ArrowRight (Prev) across Start and End."
                }
            }

            DateRangeFieldRoot {
                id: "sw5c-drf-rtl",
                dir: Some(Direction::Rtl),
                value: Some(rtl_range),
                default_value: DateValueRange::new(
                    Some(drf_date(2025, 4, 10)),
                    Some(drf_date(2025, 4, 20)),
                ),
                locale: CalendarLocale::EnUs,
                DateRangeFieldLabel {
                    id: "sw5c-drf-rtl-label",
                    "RTL Booking Range"
                }
                div {
                    class: "drf-inputs-row",
                    DateRangeFieldInput {
                        kind: RangeFieldInputKind::Start,
                        id: "sw5c-drf-rtl-start-input",
                        render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segments {
                                DateRangeFieldSegment {
                                    key: "{seg.index}",
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                    id: format!("sw5c-drf-rtl-start-seg-{}", seg.index),
                                }
                            }
                        }
                    }
                    span { style: "color: #64748b; user-select: none;", "–" }
                    DateRangeFieldInput {
                        kind: RangeFieldInputKind::End,
                        id: "sw5c-drf-rtl-end-input",
                        render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segments {
                                DateRangeFieldSegment {
                                    key: "{seg.index}",
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                    id: format!("sw5c-drf-rtl-end-seg-{}", seg.index),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
