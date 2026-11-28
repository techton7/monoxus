use dioxus::prelude::*;
use monoxus::foundation::date_time::{CalendarTime, TimeRange, TimeValue};
use monoxus::time_field::TimeFieldSegmentItem;
use monoxus::time_range_field::{
    RangeFieldInputKind, TimeRangeFieldInput, TimeRangeFieldLabel, TimeRangeFieldRoot,
    TimeRangeFieldSegment,
};

#[component]
pub fn TimeRangeFieldDemo() -> Element {
    let start = TimeValue::Time(CalendarTime {
        hour: 9,
        minute: 0,
        second: 0,
        millisecond: 0,
    });
    let end = TimeValue::Time(CalendarTime {
        hour: 17,
        minute: 30,
        second: 0,
        millisecond: 0,
    });
    let mut range = use_signal(|| TimeRange::new(Some(start), Some(end)));

    let summary = {
        let r = range();
        format!(
            "{} → {}",
            r.start
                .map(|v| v.time().to_iso_string())
                .unwrap_or_else(|| String::from("…")),
            r.end
                .map(|v| v.time().to_iso_string())
                .unwrap_or_else(|| String::from("…")),
        )
    };

    rsx! {
        div { class: "flex flex-col items-center gap-3",
            TimeRangeFieldRoot {
                value: Some(range()),
                on_value_change: move |next: TimeRange| range.set(next),
                class: "flex flex-col gap-1.5",

                TimeRangeFieldLabel {
                    class: "text-xs font-medium text-foreground-alt select-none",
                    "Office hours"
                }

                div { class: "inline-flex h-10 items-center gap-1.5 rounded-input border border-border-input bg-background px-3 text-sm text-foreground shadow-mini focus-within:border-foreground",
                    TimeRangeFieldInput {
                        kind: RangeFieldInputKind::Start,
                        class: "inline-flex items-center",
                        render_children: move |segments: Vec<TimeFieldSegmentItem>| rsx! {
                            for seg in segments {
                                TimeRangeFieldSegment {
                                    key: "s-{seg.index}",
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                    class: "rounded px-0.5 py-0.5 tabular-nums outline-none focus:bg-dark focus:text-background data-[placeholder]:text-muted-foreground data-[segment=literal]:text-muted-foreground",
                                }
                            }
                        }
                    }

                    span { class: "px-1 text-muted-foreground select-none", "–" }

                    TimeRangeFieldInput {
                        kind: RangeFieldInputKind::End,
                        class: "inline-flex items-center",
                        render_children: move |segments: Vec<TimeFieldSegmentItem>| rsx! {
                            for seg in segments {
                                TimeRangeFieldSegment {
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
                "Window: "
                span { class: "font-medium text-foreground", "{summary}" }
            }
        }
    }
}
