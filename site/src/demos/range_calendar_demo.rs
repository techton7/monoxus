use dioxus::prelude::*;
use monoxus::foundation::date_time::{CalendarDate, DateRange};
use monoxus::range_calendar::{
    RangeCalendarCell, RangeCalendarDay, RangeCalendarGrid, RangeCalendarGridBody,
    RangeCalendarGridHead, RangeCalendarGridRow, RangeCalendarHeadCell, RangeCalendarHeader,
    RangeCalendarHeading, RangeCalendarNextButton, RangeCalendarPrevButton, RangeCalendarRoot,
    RangeCalendarSnapshot,
};

#[component]
pub fn RangeCalendarDemo() -> Element {
    let start = CalendarDate::from_ymd_clamped(2026, 5, 10);
    let end = CalendarDate::from_ymd_clamped(2026, 5, 16);
    let mut range = use_signal(|| DateRange::new(Some(start), Some(end)));

    let summary = {
        let r = range();
        match (r.start, r.end) {
            (Some(s), Some(e)) => format!("{s} → {e}"),
            (Some(s), None) => format!("{s} → …"),
            _ => String::from("none"),
        }
    };

    rsx! {
        div { class: "flex flex-col items-center gap-3",
            RangeCalendarRoot {
                value: Some(range()),
                default_placeholder: start,
                on_value_change: move |next: DateRange| range.set(next),
                class: "w-72 rounded-card-sm border border-border-card bg-background p-3.5 shadow-mini",
                render_children: move |snap: RangeCalendarSnapshot| rsx! {
                    RangeCalendarHeader {
                        class: "mb-3 flex items-center justify-between gap-2",
                        RangeCalendarPrevButton {
                            class: "inline-flex h-8 w-8 items-center justify-center rounded-button border border-border-input bg-background text-sm text-foreground transition-colors hover:bg-muted data-[disabled]:opacity-40",
                            "‹"
                        }
                        RangeCalendarHeading {
                            class: "text-sm font-medium text-foreground",
                        }
                        RangeCalendarNextButton {
                            class: "inline-flex h-8 w-8 items-center justify-center rounded-button border border-border-input bg-background text-sm text-foreground transition-colors hover:bg-muted data-[disabled]:opacity-40",
                            "›"
                        }
                    }

                    for (month_idx, month) in snap.months.iter().enumerate() {
                        RangeCalendarGrid {
                            key: "{month_idx}",
                            month_idx: Some(month_idx),
                            class: "w-full border-collapse select-none",
                            RangeCalendarGridHead {
                                RangeCalendarGridRow {
                                    class: "grid grid-cols-7 mb-1",
                                    for (i, weekday) in snap.weekdays.iter().enumerate() {
                                        RangeCalendarHeadCell {
                                            key: "{i}",
                                            class: "py-1 text-center text-xs font-medium text-muted-foreground",
                                            "{weekday}"
                                        }
                                    }
                                }
                            }
                            RangeCalendarGridBody {
                                for (w_idx, week) in month.weeks.iter().enumerate() {
                                    RangeCalendarGridRow {
                                        key: "{w_idx}",
                                        class: "grid grid-cols-7",
                                        for date in week {
                                            RangeCalendarCell {
                                                key: "{date}",
                                                date: *date,
                                                month: month.value,
                                                class: "p-0.5 text-center",
                                                RangeCalendarDay {
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

            p { class: "text-xs text-muted-foreground",
                "Range: "
                span { class: "font-medium text-foreground", "{summary}" }
            }
        }
    }
}
