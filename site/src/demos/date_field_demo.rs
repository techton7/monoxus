use dioxus::prelude::*;
use monoxus::date_field::{
    DateFieldInput, DateFieldLabel, DateFieldRoot, DateFieldSegment, DateFieldSegmentItem,
};
use monoxus::foundation::date_time::{CalendarDate, DateValue};

#[component]
pub fn DateFieldDemo() -> Element {
    let initial = DateValue::Date(CalendarDate::from_ymd_clamped(2026, 5, 14));
    let mut value = use_signal(|| Some(initial));

    rsx! {
        div { class: "flex flex-col items-center gap-3",
            DateFieldRoot {
                value: Some(value()),
                on_value_change: move |next: Option<DateValue>| value.set(next),

                div { class: "flex flex-col gap-1.5",
                    DateFieldLabel {
                        class: "text-xs font-medium text-foreground-alt select-none",
                        "Release date"
                    }
                    DateFieldInput {
                        class: "inline-flex h-10 items-center rounded-input border border-border-input bg-background px-3 text-sm text-foreground shadow-mini focus-within:border-foreground data-[invalid]:border-destructive",
                        render_children: move |segments: Vec<DateFieldSegmentItem>| rsx! {
                            for seg in segments {
                                DateFieldSegment {
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
                "ISO value: "
                span { class: "font-medium text-foreground",
                    "{value().map(|v| v.to_iso_string()).unwrap_or_else(|| String::from(\"incomplete\"))}"
                }
            }
        }
    }
}
