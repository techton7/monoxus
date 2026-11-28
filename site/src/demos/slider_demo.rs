use dioxus::prelude::*;
use monoxus::slider::{
    SliderRange, SliderRoot, SliderThumb, SliderTick, SliderTrack, SliderValue,
};

#[component]
pub fn SliderDemo() -> Element {
    let mut range = use_signal(|| vec![25.0_f64, 75.0_f64]);

    let vals = range();
    let low = vals.first().copied().unwrap_or(0.0);
    let high = vals.get(1).copied().unwrap_or(100.0);

    rsx! {
        div { class: "w-full max-w-sm space-y-4",
            div { class: "flex items-center justify-between text-xs",
                span { class: "font-medium text-foreground-alt", "Budget range" }
                span { class: "font-mono font-medium text-foreground", "${low:.0} – ${high:.0}" }
            }

            SliderRoot {
                value: Some(SliderValue::Multiple(vals)),
                on_value_change: move |next: SliderValue| range.set(next.to_vec()),
                min: 0.0,
                max: 100.0,
                step: 5.0,
                min_steps_between_thumbs: 2,
                class: "relative flex h-6 w-full touch-none select-none items-center",

                SliderTrack {
                    class: "relative h-1.5 w-full grow overflow-hidden rounded-full bg-muted",
                    SliderRange {
                        class: "absolute h-full rounded-full bg-dark",
                    }
                    for tick in [0.0, 25.0, 50.0, 75.0, 100.0] {
                        SliderTick {
                            key: "{tick}",
                            value: tick,
                            class: "h-1.5 w-0.5 bg-background/60",
                        }
                    }
                }

                SliderThumb {
                    index: 0,
                    class: "block h-5 w-5 rounded-full border border-border-input bg-background shadow-btn transition-transform focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-foreground data-[active]:scale-110",
                }
                SliderThumb {
                    index: 1,
                    class: "block h-5 w-5 rounded-full border border-border-input bg-background shadow-btn transition-transform focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-foreground data-[active]:scale-110",
                }
            }
        }
    }
}
