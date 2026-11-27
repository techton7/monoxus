use dioxus::prelude::*;
use monoxus::{
    date_field::*,
    foundation::date_time::{
        CalendarDate, CalendarLocale, DateValue, FieldInvalidEvent, Granularity, HourCycle,
    },
};

const DATE_FIELD_PLAYGROUND_CSS: &str = r#"
[data-date-field-input] {
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
[data-date-field-input][data-invalid] {
    border-color: #ef4444;
    background-color: #fef2f2;
}
[data-date-field-input][data-disabled] {
    background-color: #f1f5f9;
    color: #94a3b8;
    opacity: 0.7;
}
[data-date-field-label] {
    font-size: 0.8125rem;
    font-weight: 600;
    color: #334155;
    cursor: pointer;
    user-select: none;
}
[data-date-field-label][data-invalid] {
    color: #dc2626;
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

fn df_date(year: i32, month: u8, day: u8) -> DateValue {
    DateValue::Date(CalendarDate::new(year, month, day).expect("valid playground date"))
}

#[component]
pub fn DateFieldShowcase() -> Element {
    rsx! {
        style { "{DATE_FIELD_PLAYGROUND_CSS}" }
        div {
            class: "max-w-4xl mx-auto space-y-8",
            style: "max-width: 56rem; margin: 0 auto; display: flex; flex-direction: column; gap: 2rem;",

            div {
                class: "space-y-1.5",
                style: "display: flex; flex-direction: column; gap: 0.375rem;",
                h2 {
                    class: "text-2xl font-bold tracking-tight text-slate-900",
                    style: "margin: 0; font-size: 1.5rem; font-weight: 700; color: #0f172a;",
                    "DateField"
                }
                p {
                    class: "text-sm text-slate-500",
                    style: "margin: 0; font-size: 0.875rem; color: #64748b;",
                    "Segmented date & date-time spinbutton field with digit-entry auto-advance, ArrowUp/Down cycling, Backspace clearing, min/max validation, pure-Rust locales, and hidden form input sync."
                }
            }

            EnUsFormDateFieldScenario {}
            KoKrDateFieldScenario {}
            DateTimeMinuteFieldScenario {}
            Empty12hDateTimeFieldScenario {}
        }
    }
}

#[component]
fn EnUsFormDateFieldScenario() -> Element {
    let mut value = use_signal(|| Some(df_date(2024, 6, 15)));
    let mut invalid_reason = use_signal(|| String::from("none"));

    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            style: "{CARD_STYLE}",

            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "1. EnUs DateField with min/max Bounds, HiddenFormInput & Reset (SW5C-1, SW5C-2)"
                }
                p {
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Bounded to 2024-01-01..=2026-12-31. Type digits to test leading-zero wait & auto-advance, ArrowUp/Down, Backspace, or Reset form."
                }
            }

            form {
                id: "sw5c-df-form",
                style: "display: flex; flex-wrap: wrap; align-items: flex-end; gap: 1rem;",
                DateFieldRoot {
                    id: "sw5c-df-enus",
                    value: Some(value()),
                    default_value: Some(df_date(2024, 6, 15)),
                    default_placeholder: Some(df_date(2024, 6, 15)),
                    min_value: Some(df_date(2024, 1, 1)),
                    max_value: Some(df_date(2026, 12, 31)),
                    locale: CalendarLocale::EnUs,
                    on_value_change: move |next: Option<DateValue>| {
                        value.set(next);
                        if let Some(v) = next {
                            let min = df_date(2024, 1, 1);
                            let max = df_date(2026, 12, 31);
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
                        DateFieldLabel {
                            id: "sw5c-df-enus-label",
                            "Appointment Date (en-US)"
                        }
                        DateFieldInput {
                            id: "sw5c-df-enus-input",
                            name: "appointment_date",
                            render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    DateFieldSegment {
                                        key: "{seg.index}",
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        id: format!("sw5c-df-enus-seg-{}", seg.index),
                                    }
                                }
                            }
                        }
                    }
                }

                button {
                    id: "sw5c-df-reset-btn",
                    r#type: "reset",
                    style: "{BTN_STYLE}",
                    "Reset Form"
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; gap: 1rem; font-size: 0.75rem;",
                div {
                    style: "display: flex; align-items: center; gap: 0.5rem;",
                    span { style: "color: #64748b; font-weight: 600;", "ISO Value:" }
                    span {
                        id: "sw5c-df-enus-value",
                        style: "{BADGE_STYLE}",
                        {value().map_or_else(|| String::from("none"), |v| v.to_iso_string())}
                    }
                }
                div {
                    style: "display: flex; align-items: center; gap: 0.5rem;",
                    span { style: "color: #64748b; font-weight: 600;", "Invalid Reason:" }
                    span {
                        id: "sw5c-df-enus-invalid",
                        style: "{BADGE_STYLE}",
                        "{invalid_reason}"
                    }
                }
            }
        }
    }
}

#[component]
fn KoKrDateFieldScenario() -> Element {
    let mut value = use_signal(|| Some(df_date(2025, 3, 20)));

    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            style: "{CARD_STYLE}",

            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "2. KoKr Locale DateField (YYYY. MM. DD.)"
                }
                p {
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Pure-Rust CalendarLocale::KoKr segment ordering and localized placeholders."
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; align-items: center; gap: 1.5rem;",
                DateFieldRoot {
                    id: "sw5c-df-kokr",
                    value: Some(value()),
                    on_value_change: move |next: Option<DateValue>| value.set(next),
                    default_placeholder: Some(df_date(2025, 3, 20)),
                    locale: CalendarLocale::KoKr,
                    div {
                        style: "display: flex; flex-direction: column; gap: 0.375rem;",
                        DateFieldLabel {
                            id: "sw5c-df-kokr-label",
                            "예약 날짜 (ko-KR)"
                        }
                        DateFieldInput {
                            id: "sw5c-df-kokr-input",
                            render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    DateFieldSegment {
                                        key: "{seg.index}",
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        id: format!("sw5c-df-kokr-seg-{}", seg.index),
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
                        id: "sw5c-df-kokr-value",
                        style: "{BADGE_STYLE}",
                        {value().map_or_else(|| String::from("none"), |v| v.to_iso_string())}
                    }
                }
            }
        }
    }
}

#[component]
fn DateTimeMinuteFieldScenario() -> Element {
    let initial = DateValue::from_iso_str("2025-08-15T14:30:00").expect("valid datetime");
    let mut value = use_signal(|| Some(initial));

    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            style: "{CARD_STYLE}",

            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "3. Granularity::Minute DateTime Field"
                }
                p {
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Includes date segments plus hour, minute, and AM/PM dayPeriod segments."
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; align-items: center; gap: 1.5rem;",
                DateFieldRoot {
                    id: "sw5c-df-datetime",
                    value: Some(value()),
                    on_value_change: move |next: Option<DateValue>| value.set(next),
                    default_placeholder: Some(initial),
                    granularity: Some(Granularity::Minute),
                    locale: CalendarLocale::EnUs,
                    div {
                        style: "display: flex; flex-direction: column; gap: 0.375rem;",
                        DateFieldLabel {
                            id: "sw5c-df-datetime-label",
                            "Event Start (Minute Granularity)"
                        }
                        DateFieldInput {
                            id: "sw5c-df-datetime-input",
                            render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    DateFieldSegment {
                                        key: "{seg.index}",
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        id: format!("sw5c-df-datetime-seg-{}", seg.index),
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
                        id: "sw5c-df-datetime-value",
                        style: "{BADGE_STYLE}",
                        {value().map_or_else(|| String::from("none"), |v| v.to_iso_string())}
                    }
                }
            }
        }
    }
}

#[component]
fn Empty12hDateTimeFieldScenario() -> Element {
    let mut value = use_signal(|| None::<DateValue>);

    rsx! {
        div {
            class: "rounded-2xl border border-slate-200 bg-white p-6 shadow-sm space-y-4",
            style: "{CARD_STYLE}",

            div {
                style: "display: flex; flex-direction: column; gap: 0.25rem;",
                h3 {
                    style: "margin: 0; font-size: 1rem; font-weight: 600; color: #0f172a;",
                    "4. Empty 12-Hour DateTime Field with Required & Partial-Buffer Reset (DEF-5C-01, DEF-5C-03)"
                }
                p {
                    style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                    "Starts empty with default AM meridiem. Typing date + time commits immediately; Reset clears partial buffers."
                }
            }

            form {
                id: "sw5c-df-empty-form",
                style: "display: flex; flex-wrap: wrap; align-items: flex-end; gap: 1rem;",
                DateFieldRoot {
                    id: "sw5c-df-empty-12h",
                    value: Some(value()),
                    on_value_change: move |next: Option<DateValue>| value.set(next),
                    granularity: Some(Granularity::Minute),
                    hour_cycle: Some(HourCycle::H12),
                    locale: CalendarLocale::EnUs,
                    required: true,
                    div {
                        style: "display: flex; flex-direction: column; gap: 0.375rem;",
                        DateFieldLabel {
                            id: "sw5c-df-empty-12h-label",
                            "Schedule DateTime (Empty 12h)"
                        }
                        DateFieldInput {
                            id: "sw5c-df-empty-12h-input",
                            name: "schedule_dt",
                            render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    DateFieldSegment {
                                        key: "{seg.index}",
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        id: format!("sw5c-df-empty-12h-seg-{}", seg.index),
                                    }
                                }
                            }
                        }
                    }
                }

                button {
                    id: "sw5c-df-empty-reset-btn",
                    r#type: "reset",
                    style: "{BTN_STYLE}",
                    "Reset Empty"
                }

                div {
                    style: "display: flex; align-items: center; gap: 0.5rem; font-size: 0.75rem;",
                    span { style: "color: #64748b; font-weight: 600;", "ISO Value:" }
                    span {
                        id: "sw5c-df-empty-12h-value",
                        style: "{BADGE_STYLE}",
                        {value().map_or_else(|| String::from("none"), |v| v.to_iso_string())}
                    }
                }
            }
        }
    }
}
