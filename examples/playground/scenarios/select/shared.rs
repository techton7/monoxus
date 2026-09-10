use dioxus::prelude::*;
use monoxus::select::{SelectItemIndicator, SelectItemText};

pub const CARD_STYLE: &str = "display: grid; gap: 1rem; padding: 1.25rem; border-radius: 0.75rem; border: 1px solid #c084fc; background-color: white; box-shadow: 0 10px 30px rgba(147, 51, 234, 0.08);";
pub const MUTED_STYLE: &str = "margin: 0; color: #6b21a8;";
pub const BADGE_STYLE: &str = "display: inline-block; padding: 0.2rem 0.6rem; border-radius: 9999px; font-size: 0.75rem; font-weight: 600; background-color: #f3e8ff; color: #7e22ce;";

pub const SELECT_PLAYGROUND_CSS: &str = r#"
.select-item, [role="option"] {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.375rem 0.5rem;
    border-radius: 0.25rem;
    font-size: 0.875rem;
    cursor: pointer;
    user-select: none;
}
.select-item[data-highlighted="true"], [role="option"][data-highlighted="true"] {
    background-color: #f3e8ff !important;
    color: #7e22ce !important;
}
.select-item[data-disabled="true"], [role="option"][data-disabled="true"] {
    opacity: 0.45 !important;
    cursor: not-allowed !important;
    pointer-events: none !important;
}

@keyframes monoxus-select-scale-in {
    from {
        transform: scale(0.95);
        opacity: 0;
    }
    to {
        transform: scale(1);
        opacity: 1;
    }
}

@keyframes monoxus-select-scale-out {
    from {
        transform: scale(1);
        opacity: 1;
    }
    to {
        transform: scale(0.95);
        opacity: 0;
    }
}

@keyframes monoxus-select-enter-bottom {
    from {
        transform: translateY(-0.25rem) scale(0.95);
        opacity: 0;
    }
    to {
        transform: translateY(0) scale(1);
        opacity: 1;
    }
}

@keyframes monoxus-select-exit-bottom {
    from {
        transform: translateY(0) scale(1);
        opacity: 1;
    }
    to {
        transform: translateY(-0.25rem) scale(0.95);
        opacity: 0;
    }
}

@keyframes monoxus-select-enter-top {
    from {
        transform: translateY(0.25rem) scale(0.95);
        opacity: 0;
    }
    to {
        transform: translateY(0) scale(1);
        opacity: 1;
    }
}

@keyframes monoxus-select-exit-top {
    from {
        transform: translateY(0) scale(1);
        opacity: 1;
    }
    to {
        transform: translateY(0.25rem) scale(0.95);
        opacity: 0;
    }
}

[role="listbox"][data-side="bottom"] {
    transform-origin: top center;
}

[role="listbox"][data-side="top"] {
    transform-origin: bottom center;
}

[role="listbox"][data-state="open"] {
    animation: monoxus-select-scale-in 200ms cubic-bezier(0.16, 1, 0.3, 1) forwards;
}

[role="listbox"][data-state="closed"] {
    animation: monoxus-select-scale-out 200ms ease-in forwards;
}

[role="listbox"][data-side="bottom"][data-state="open"] {
    animation: monoxus-select-enter-bottom 200ms cubic-bezier(0.16, 1, 0.3, 1) forwards;
}

[role="listbox"][data-side="bottom"][data-state="closed"] {
    animation: monoxus-select-exit-bottom 200ms ease-in forwards;
}

[role="listbox"][data-side="top"][data-state="open"] {
    animation: monoxus-select-enter-top 200ms cubic-bezier(0.16, 1, 0.3, 1) forwards;
}

[role="listbox"][data-side="top"][data-state="closed"] {
    animation: monoxus-select-exit-top 200ms ease-in forwards;
}
"#;

#[component]
pub fn ItemRow(text: &'static str, is_selected: bool) -> Element {
    rsx! {
        div {
            style: "display: flex; justify-content: space-between; align-items: center; width: 100%;",
            SelectItemText {
                span { "{text}" }
            }
            SelectItemIndicator {
                if is_selected {
                    span { style: "color: #9333ea; font-weight: bold; margin-left: 0.5rem;", "✓" }
                }
            }
        }
    }
}
