use dioxus::prelude::*;

#[component]
pub fn MonoxusMark(#[props(default = "h-5 w-5".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            "aria-hidden": "true",
            rect {
                x: "2",
                y: "2",
                width: "20",
                height: "20",
                rx: "5",
                fill: "currentColor",
            }
            path {
                d: "M7 16.5v-9h2.2l2.8 4.6 2.8-4.6H17v9h-2v-5.2l-2.3 3.7h-1.4L9 11.3v5.2H7Z",
                fill: "var(--background)",
            }
        }
    }
}

#[component]
pub fn SearchIcon(#[props(default = "h-4 w-4".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.75",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            circle { cx: "11", cy: "11", r: "7" }
            path { d: "m20 20-3.5-3.5" }
        }
    }
}

#[component]
pub fn SunIcon(#[props(default = "h-4 w-4".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.75",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            circle { cx: "12", cy: "12", r: "4" }
            path { d: "M12 2v2" }
            path { d: "M12 20v2" }
            path { d: "m4.93 4.93 1.41 1.41" }
            path { d: "m17.66 17.66 1.41 1.41" }
            path { d: "M2 12h2" }
            path { d: "M20 12h2" }
            path { d: "m6.34 17.66-1.41 1.41" }
            path { d: "m19.07 4.93-1.41 1.41" }
        }
    }
}

#[component]
pub fn MoonIcon(#[props(default = "h-4 w-4".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.75",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" }
        }
    }
}

#[component]
pub fn GithubIcon(#[props(default = "h-4 w-4".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "currentColor",
            "aria-hidden": "true",
            path {
                d: "M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0 1 12 6.844a9.59 9.59 0 0 1 2.504.337c1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.02 10.02 0 0 0 22 12.017C22 6.484 17.522 2 12 2Z",
            }
        }
    }
}

#[component]
pub fn MenuIcon(#[props(default = "h-5 w-5".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.75",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            line { x1: "4", x2: "20", y1: "7", y2: "7" }
            line { x1: "4", x2: "20", y1: "12", y2: "12" }
            line { x1: "4", x2: "20", y1: "17", y2: "17" }
        }
    }
}

#[component]
pub fn CloseIcon(#[props(default = "h-4 w-4".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.75",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M18 6 6 18" }
            path { d: "m6 6 12 12" }
        }
    }
}

#[component]
pub fn CopyIcon(#[props(default = "h-3.5 w-3.5".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.75",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            rect { width: "14", height: "14", x: "8", y: "8", rx: "2", ry: "2" }
            path { d: "M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" }
        }
    }
}

#[component]
pub fn CheckIcon(#[props(default = "h-3.5 w-3.5".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M20 6 9 17l-5-5" }
        }
    }
}

#[component]
pub fn ChevronDownIcon(#[props(default = "h-4 w-4".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.75",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "m6 9 6 6 6-6" }
        }
    }
}

#[component]
pub fn ChevronRightIcon(#[props(default = "h-4 w-4".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.75",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "m9 18 6-6-6-6" }
        }
    }
}

#[component]
pub fn ChevronLeftIcon(#[props(default = "h-4 w-4".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.75",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "m15 18-6-6 6-6" }
        }
    }
}

#[component]
pub fn InfoIcon(#[props(default = "h-3.5 w-3.5".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.75",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            circle { cx: "12", cy: "12", r: "10" }
            path { d: "M12 16v-4" }
            path { d: "M12 8h.01" }
        }
    }
}

#[component]
pub fn ExternalLinkIcon(#[props(default = "h-3.5 w-3.5".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.75",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M15 3h6v6" }
            path { d: "M10 14 21 3" }
            path { d: "M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" }
        }
    }
}

#[component]
pub fn CodeIcon(#[props(default = "h-3.5 w-3.5".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.75",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            polyline { points: "16 18 22 12 16 6" }
            polyline { points: "8 6 2 12 8 18" }
        }
    }
}

#[component]
pub fn FileTextIcon(#[props(default = "h-4 w-4".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.75",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z" }
            path { d: "M14 2v4a2 2 0 0 0 2 2h4" }
            path { d: "M10 9H8" }
            path { d: "M16 13H8" }
            path { d: "M16 17H8" }
        }
    }
}
