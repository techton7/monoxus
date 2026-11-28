use dioxus::prelude::*;
use monoxus::date_field::DateFieldSegmentItem;
use monoxus::date_range_field::RangeFieldInputKind;
use monoxus::date_range_picker::{
    DateRangePickerCalendar, DateRangePickerCell, DateRangePickerContent, DateRangePickerDay,
    DateRangePickerGrid, DateRangePickerGridBody, DateRangePickerGridHead, DateRangePickerGridRow,
    DateRangePickerHeadCell, DateRangePickerHeader, DateRangePickerHeading, DateRangePickerInput,
    DateRangePickerLabel, DateRangePickerNextButton, DateRangePickerPortal,
    DateRangePickerPrevButton, DateRangePickerRoot, DateRangePickerSegment,
    DateRangePickerTrigger,
};
use monoxus::foundation::date_time::{CalendarDate, DateValue, DateValueRange};
use monoxus::range_calendar::RangeCalendarSnapshot;

#[component]
pub fn DateRangePickerDemo() -> Element {
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
            DateRangePickerRoot {
                value: Some(range()),
                on_value_change: move |next: DateValueRange| range.set(next),
                class: "flex flex-col gap-1.5",

                DateRangePickerLabel {
                    class: "text-xs font-medium text-foreground-alt select-none",
                    "Booking dates"
                }

                div { class: "inline-flex h-10 items-center gap-1 rounded-input border border-border-input bg-background pl-3 pr-1.5 text-sm text-foreground shadow-mini focus-within:border-foreground",
                    DateRangePickerInput {
                        kind: RangeFieldInputKind::Start,
                        class: "inline-flex items-center",
                        render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segments {
                                DateRangePickerSegment {
                                    key: "s-{seg.index}",
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                    class: "rounded px-0.5 py-0.5 tabular-nums outline-none focus:bg-dark focus:text-background data-[placeholder]:text-muted-foreground data-[segment=literal]:text-muted-foreground",
                                }
                            }
                        }
                    }

                    span { class: "px-0.5 text-muted-foreground select-none", "–" }

                    DateRangePickerInput {
                        kind: RangeFieldInputKind::End,
                        class: "inline-flex items-center",
                        render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segments {
                                DateRangePickerSegment {
                                    key: "e-{seg.index}",
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                    class: "rounded px-0.5 py-0.5 tabular-nums outline-none focus:bg-dark focus:text-background data-[placeholder]:text-muted-foreground data-[segment=literal]:text-muted-foreground",
                                }
                            }
                        }
                    }

                    DateRangePickerTrigger {
                        class: "ml-1 inline-flex h-7 w-7 items-center justify-center rounded-button text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground focus:outline-none focus-visible:ring-2 focus-visible:ring-foreground",
                        "▾"
                    }
                }

                DateRangePickerPortal {
                    DateRangePickerContent {
                        side_offset: 6.0,
                        class: "z-50 w-72 rounded-card-sm border border-border-card bg-background p-3.5 shadow-popover outline-none",

                        DateRangePickerCalendar {
                            render_children: move |snap: RangeCalendarSnapshot| rsx! {
                                DateRangePickerHeader {
                                    class: "mb-3 flex items-center justify-between gap-2",
                                    DateRangePickerPrevButton {
                                        class: "inline-flex h-8 w-8 items-center justify-center rounded-button border border-border-input bg-background text-sm text-foreground transition-colors hover:bg-muted data-[disabled]:opacity-40",
                                        "‹"
                                    }
                                    DateRangePickerHeading {
                                        class: "text-sm font-medium text-foreground",
                                    }
                                    DateRangePickerNextButton {
                                        class: "inline-flex h-8 w-8 items-center justify-center rounded-button border border-border-input bg-background text-sm text-foreground transition-colors hover:bg-muted data-[disabled]:opacity-40",
                                        "›"
                                    }
                                }

                                for (month_idx, month) in snap.months.iter().enumerate() {
                                    DateRangePickerGrid {
                                        key: "{month_idx}",
                                        month_idx: Some(month_idx),
                                        class: "w-full border-collapse select-none",
                                        DateRangePickerGridHead {
                                            DateRangePickerGridRow {
                                                class: "grid grid-cols-7 mb-1",
                                                for (i, weekday) in snap.weekdays.iter().enumerate() {
                                                    DateRangePickerHeadCell {
                                                        key: "{i}",
                                                        class: "py-1 text-center text-xs font-medium text-muted-foreground",
                                                        "{weekday}"
                                                    }
                                                }
                                            }
                                        }
                                        DateRangePickerGridBody {
                                            for (w_idx, week) in month.weeks.iter().enumerate() {
                                                DateRangePickerGridRow {
                                                    key: "{w_idx}",
                                                    class: "grid grid-cols-7",
                                                    for date in week {
                                                        DateRangePickerCell {
                                                            key: "{date}",
                                                            date: *date,
                                                            month: month.value,
                                                            class: "p-0.5 text-center",
                                                            DateRangePickerDay {
                                                                class: "inline-flex h-8 w-8 cursor-pointer items-center justify-center rounded-button text-xs font-medium text-foreground outline-none transition-colors hover:bg-muted focus-visible:ring-2 focus-visible:ring-foreground data-[outside-month]:text-muted-foreground/40 data-[range-middle]:bg-muted data-[highlighted]:bg-muted data-[selection-start]:bg-dark data-[selection-start]:text-background data-[selection-end]:bg-dark data-[selection-end]:text-background data-[disabled]:pointer-events-none data-[disabled]:opacity-35",
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
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
