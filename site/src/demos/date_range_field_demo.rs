use dioxus::prelude::*;
use monoxus::date_field::DateFieldSegmentItem;
use monoxus::date_range_field::{
    DateRangeFieldInput, DateRangeFieldLabel, DateRangeFieldRoot, DateRangeFieldSegment,
    RangeFieldInputKind,
};
use monoxus::foundation::date_time::{CalendarDate, DateValue, DateValueRange};

#[component]
pub fn DateRangeFieldDemo() -> Element {
    let start = DateValue::Date(CalendarDate::from_ymd_clamped(2026, 5, 10));
    let end = DateValue::Date(CalendarDate::from_ymd_clamped(2026, 5, 16));
    let mut range = use_signal(|| DateValueRange::new(Some(start), Some(end)));

    let summary = {
        let r = range();
        format!(
            "{} → {}",
            r.start
                .map(|v| v.to_iso_string())
                .unwrap_or_else(|| String::from("…")),
            r.end
                .map(|v| v.to_iso_string())
                .unwrap_or_else(|| String::from("…")),
        )
    };

    rsx! {
        div { class: "flex flex-col items-center gap-3",
            DateRangeFieldRoot {
                value: Some(range()),
                on_value_change: move |next: DateValueRange| range.set(next),
                class: "flex flex-col gap-1.5",

                DateRangeFieldLabel {
                    class: "text-xs font-medium text-foreground-alt select-none",
                    "Sprint window"
                }

                div { class: "inline-flex h-10 items-center gap-1.5 rounded-input border border-border-input bg-background px-3 text-sm text-foreground shadow-mini focus-within:border-foreground",
                    DateRangeFieldInput {
                        kind: RangeFieldInputKind::Start,
                        class: "inline-flex items-center",
                        render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segments {
                                DateRangeFieldSegment {
                                    key: "s-{seg.index}",
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                    class: "rounded px-0.5 py-0.5 tabular-nums outline-none focus:bg-dark focus:text-background data-[placeholder]:text-muted-foreground data-[segment=literal]:text-muted-foreground",
                                }
                            }
                        }
                    }

                    span { class: "px-1 text-muted-foreground select-none", "–" }

                    DateRangeFieldInput {
                        kind: RangeFieldInputKind::End,
                        class: "inline-flex items-center",
                        render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segments {
                                DateRangeFieldSegment {
                                    key: "e-{seg.index}",
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                    class: "rounded px-0.5 py-0.5 tabular-nums outline-none focus:bg-dark focus:text-background data-[placeholder]:text-muted-foreground data-[segment=literal]:text-muted-foreground",
                                }
                            }
                        }
                    }
                }
            }

            p { class: "text-xs text-muted-foreground",
                "Range: "
                span { class: "font-medium text-foreground", "{summary}" }
            }
        }
    }
}
