use dioxus::prelude::*;
use monoxus::foundation::date_time::{CalendarTime, TimeValue};
use monoxus::time_field::{
    TimeFieldInput, TimeFieldLabel, TimeFieldRoot, TimeFieldSegment, TimeFieldSegmentItem,
};

#[component]
pub fn TimeFieldDemo() -> Element {
    let initial = TimeValue::Time(CalendarTime {
        hour: 9,
        minute: 30,
        second: 0,
        millisecond: 0,
    });
    let mut value = use_signal(|| Some(initial));

    rsx! {
        div { class: "flex flex-col items-center gap-3",
            TimeFieldRoot {
                value: Some(value()),
                on_value_change: move |next: Option<TimeValue>| value.set(next),

                div { class: "flex flex-col gap-1.5",
                    TimeFieldLabel {
                        class: "text-xs font-medium text-foreground-alt select-none",
                        "Daily standup"
                    }
                    TimeFieldInput {
                        class: "inline-flex h-10 items-center rounded-input border border-border-input bg-background px-3 text-sm text-foreground shadow-mini focus-within:border-foreground",
                        render_children: move |segments: Vec<TimeFieldSegmentItem>| rsx! {
                            for seg in segments {
                                TimeFieldSegment {
                                    key: "{seg.index}",
                                    part: seg.kind(),
                                    index: Some(seg.index),
                                    class: "rounded px-0.5 py-0.5 tabular-nums outline-none focus:bg-dark focus:text-background data-[placeholder]:text-muted-foreground data-[segment=literal]:px-0.5 data-[segment=literal]:text-muted-foreground",
                                }
                            }
                        }
                    }
                }
            }

            p { class: "text-xs text-muted-foreground",
                "Time: "
                span { class: "font-medium text-foreground",
                    "{value().map(|v| v.time().to_iso_string()).unwrap_or_else(|| String::from(\"incomplete\"))}"
                }
            }
        }
    }
}
