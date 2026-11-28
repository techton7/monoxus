use dioxus::prelude::*;
use monoxus::calendar::CalendarSnapshot;
use monoxus::date_field::DateFieldSegmentItem;
use monoxus::date_picker::{
    DatePickerCalendar, DatePickerCell, DatePickerContent, DatePickerDay, DatePickerGrid,
    DatePickerGridBody, DatePickerGridHead, DatePickerGridRow, DatePickerHeadCell,
    DatePickerHeader, DatePickerHeading, DatePickerInput, DatePickerLabel, DatePickerNextButton,
    DatePickerPortal, DatePickerPrevButton, DatePickerRoot, DatePickerSegment, DatePickerTrigger,
};
use monoxus::foundation::date_time::{CalendarDate, DateValue};

#[component]
pub fn DatePickerDemo() -> Element {
    let initial = DateValue::Date(CalendarDate::from_ymd_clamped(2026, 5, 14));
    let mut value = use_signal(|| Some(initial));

    rsx! {
        div { class: "flex flex-col items-center gap-3",
            DatePickerRoot {
                value: Some(value()),
                on_value_change: move |next: Option<DateValue>| value.set(next),

                div { class: "flex flex-col gap-1.5",
                    DatePickerLabel {
                        class: "text-xs font-medium text-foreground-alt select-none",
                        "Launch date"
                    }

                    div { class: "inline-flex h-10 items-center gap-1 rounded-input border border-border-input bg-background pl-3 pr-1.5 text-sm text-foreground shadow-mini focus-within:border-foreground",
                        DatePickerInput {
                            class: "inline-flex items-center",
                            render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                                for seg in segments {
                                    DatePickerSegment {
                                        key: "{seg.index}",
                                        part: seg.kind(),
                                        index: Some(seg.index),
                                        class: "rounded px-0.5 py-0.5 tabular-nums outline-none focus:bg-dark focus:text-background data-[placeholder]:text-muted-foreground data-[segment=literal]:text-muted-foreground",
                                    }
                                }
                            }
                        }

                        DatePickerTrigger {
                            class: "ml-1 inline-flex h-7 w-7 items-center justify-center rounded-button text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground focus:outline-none focus-visible:ring-2 focus-visible:ring-foreground",
                            "▾"
                        }
                    }
                }

                DatePickerPortal {
                    DatePickerContent {
                        side_offset: 6.0,
                        class: "z-50 w-72 rounded-card-sm border border-border-card bg-background p-3.5 shadow-popover outline-none",

                        DatePickerCalendar {
                            render_children: move |snap: CalendarSnapshot| rsx! {
                                DatePickerHeader {
                                    class: "mb-3 flex items-center justify-between gap-2",
                                    DatePickerPrevButton {
                                        class: "inline-flex h-8 w-8 items-center justify-center rounded-button border border-border-input bg-background text-sm text-foreground transition-colors hover:bg-muted data-[disabled]:opacity-40",
                                        "‹"
                                    }
                                    DatePickerHeading {
                                        class: "text-sm font-medium text-foreground",
                                    }
                                    DatePickerNextButton {
                                        class: "inline-flex h-8 w-8 items-center justify-center rounded-button border border-border-input bg-background text-sm text-foreground transition-colors hover:bg-muted data-[disabled]:opacity-40",
                                        "›"
                                    }
                                }

                                for (month_idx, month) in snap.months.iter().enumerate() {
                                    DatePickerGrid {
                                        key: "{month_idx}",
                                        month_idx: Some(month_idx),
                                        class: "w-full border-collapse select-none",
                                        DatePickerGridHead {
                                            DatePickerGridRow {
                                                class: "grid grid-cols-7 mb-1",
                                                for (i, weekday) in snap.weekdays.iter().enumerate() {
                                                    DatePickerHeadCell {
                                                        key: "{i}",
                                                        class: "py-1 text-center text-xs font-medium text-muted-foreground",
                                                        "{weekday}"
                                                    }
                                                }
                                            }
                                        }
                                        DatePickerGridBody {
                                            for (w_idx, week) in month.weeks.iter().enumerate() {
                                                DatePickerGridRow {
                                                    key: "{w_idx}",
                                                    class: "grid grid-cols-7",
                                                    for date in week {
                                                        DatePickerCell {
                                                            key: "{date}",
                                                            date: *date,
                                                            month: month.value,
                                                            class: "p-0.5 text-center",
                                                            DatePickerDay {
                                                                class: "inline-flex h-8 w-8 cursor-pointer items-center justify-center rounded-button text-xs font-medium text-foreground outline-none transition-colors hover:bg-muted focus-visible:ring-2 focus-visible:ring-foreground data-[outside-month]:text-muted-foreground/40 data-[today]:border data-[today]:border-border-input data-[selected]:bg-dark data-[selected]:text-background data-[disabled]:pointer-events-none data-[disabled]:opacity-35",
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
                "Selected: "
                span { class: "font-medium text-foreground",
                    "{value().map(|v| v.to_iso_string()).unwrap_or_else(|| String::from(\"none\"))}"
                }
            }
        }
    }
}
