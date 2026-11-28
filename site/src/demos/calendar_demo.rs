use dioxus::prelude::*;
use monoxus::calendar::{
    CalendarCell, CalendarDay, CalendarGrid, CalendarGridBody, CalendarGridHead, CalendarGridRow,
    CalendarHeadCell, CalendarHeader, CalendarHeading, CalendarNextButton, CalendarPrevButton,
    CalendarRoot, CalendarSnapshot,
};
use monoxus::foundation::date_time::CalendarDate;

#[component]
pub fn CalendarDemo() -> Element {
    let initial = CalendarDate::from_ymd_clamped(2026, 5, 14);
    let mut selected = use_signal(|| Some(initial));

    rsx! {
        div { class: "flex flex-col items-center gap-3",
            CalendarRoot {
                value: Some(selected()),
                default_placeholder: initial,
                on_value_change: move |next: Option<CalendarDate>| selected.set(next),
                class: "w-72 rounded-card-sm border border-border-card bg-background p-3.5 shadow-mini",
                render_children: move |snap: CalendarSnapshot| rsx! {
                    CalendarHeader {
                        class: "mb-3 flex items-center justify-between gap-2",
                        CalendarPrevButton {
                            class: "inline-flex h-8 w-8 items-center justify-center rounded-button border border-border-input bg-background text-sm text-foreground transition-colors hover:bg-muted data-[disabled]:opacity-40",
                            "‹"
                        }
                        CalendarHeading {
                            class: "text-sm font-medium text-foreground",
                        }
                        CalendarNextButton {
                            class: "inline-flex h-8 w-8 items-center justify-center rounded-button border border-border-input bg-background text-sm text-foreground transition-colors hover:bg-muted data-[disabled]:opacity-40",
                            "›"
                        }
                    }

                    for (month_idx, month) in snap.months.iter().enumerate() {
                        CalendarGrid {
                            key: "{month_idx}",
                            month_idx: Some(month_idx),
                            class: "w-full border-collapse select-none",
                            CalendarGridHead {
                                CalendarGridRow {
                                    class: "grid grid-cols-7 mb-1",
                                    for (i, weekday) in snap.weekdays.iter().enumerate() {
                                        CalendarHeadCell {
                                            key: "{i}",
                                            class: "py-1 text-center text-xs font-medium text-muted-foreground",
                                            "{weekday}"
                                        }
                                    }
                                }
                            }
                            CalendarGridBody {
                                for (w_idx, week) in month.weeks.iter().enumerate() {
                                    CalendarGridRow {
                                        key: "{w_idx}",
                                        class: "grid grid-cols-7",
                                        for date in week {
                                            CalendarCell {
                                                key: "{date}",
                                                date: *date,
                                                month: month.value,
                                                class: "p-0.5 text-center",
                                                CalendarDay {
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

            p { class: "text-xs text-muted-foreground",
                "Selected: "
                span { class: "font-medium text-foreground",
                    "{selected().map(|d| d.to_iso_string()).unwrap_or_else(|| String::from(\"none\"))}"
                }
            }
        }
    }
}
