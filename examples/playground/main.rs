#![allow(non_snake_case)]

pub mod accessible_icon;
pub mod accordion;
pub mod alert_dialog;
pub mod aspect_ratio;
pub mod avatar;
pub mod button;
pub mod calendar;
pub mod checkbox;
pub mod collapsible;
pub mod combobox;
pub mod command;
pub mod context_menu;
pub mod date_field;
pub mod date_picker;
pub mod date_range_field;
pub mod date_range_picker;
pub mod dialog;
pub mod dropdown_menu;
pub mod form_1b;
pub mod hover_card;
pub mod label;
pub mod menubar;
pub mod meter;
pub mod navigation_menu;
pub mod pages;
pub mod pagination;
pub mod pin_input;
pub mod popover;
pub mod progress;
pub mod radio_group;
pub mod range_calendar;
pub mod rating_group;
pub mod scenarios;
pub mod scroll_area;
pub mod separator;
pub mod slider;
pub mod switch;
pub mod tabs;
pub mod time_field;
pub mod time_range_field;
pub mod toggle;
pub mod toggle_group;
pub mod toolbar;
pub mod tooltip;
pub mod visually_hidden;

use dioxus::prelude::*;
use pages::{
    AccessibleIconPage, AccordionPage, AlertDialogPage, AspectRatioPage, AvatarPage, ButtonPage,
    CalendarPage, CheckboxPage, CollapsiblePage, ComboboxPage, CommandPage, ContextMenuPage,
    DateFieldPage, DatePickerPage, DateRangeFieldPage, DateRangePickerPage, DialogPage,
    DropdownMenuPage, Form1bPage, HoverCardPage, LabelPage, MenubarPage, MeterPage,
    NavigationMenuPage, PaginationPage, PinInputPage, PopoverPage, ProgressPage, RadioGroupPage,
    RangeCalendarPage, RatingGroupPage, ScrollAreaPage, SelectPage, SeparatorPage, SliderPage,
    SwitchPage, TabsPage, TimeFieldPage, TimeRangeFieldPage, ToastPage, ToggleGroupPage,
    TogglePage, ToolbarPage, TooltipPage, VisuallyHiddenPage,
};

#[component]
fn RootApp() -> Element {
    #[cfg(not(target_arch = "wasm32"))]
    {
        rsx! {
            blitz_host::BlitzHost {
                App {}
            }
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        rsx! {
            App {}
        }
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
#[component]
fn NativeHostedApp() -> Element {
    oxidase::launch::ensure_document_context();

    let window = dioxus_native::use_window();
    use_hook(|| {
        let w = window.clone();
        std::rc::Rc::new(oxidase::launch::bind_host_redraw_requester(move || {
            w.request_redraw();
        }))
    });

    dioxus_native::use_window_event(move |event, _target| match event {
        dioxus_native::winit::event::WindowEvent::RedrawRequested => {
            oxidase::launch::step_hosted_frame();
        }
        dioxus_native::winit::event::WindowEvent::Occluded(occluded) => {
            oxidase::dom::document().set_document_hidden(*occluded);
        }
        dioxus_native::winit::event::WindowEvent::Focused(false) => {
            oxidase::dom::document().dispatch_window_blur();
        }
        _ => {}
    });

    rsx! {
        oxidase::launch::HostedRootWrapper {
            RootApp {}
        }
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
fn main() {
    let mut window_attributes = dioxus_native::WindowAttributes::default();
    let hidden = std::env::var("BLITZ_HOST_HIDDEN").as_deref() == Ok("1")
        || std::env::var("BLITZ_HEADLESS").as_deref() == Ok("1");
    if hidden {
        window_attributes = window_attributes.with_visible(false).with_active(false);
    } else if std::env::var("BLITZ_HOST_NO_ACTIVATE").as_deref() == Ok("1") {
        window_attributes = window_attributes.with_active(false);
    }

    let config = dioxus_native::Config::default().with_window_attributes(window_attributes);

    dioxus_native::launch_cfg(NativeHostedApp, vec![], vec![Box::new(config)]);
}

#[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
#[oxidase::main]
fn main() {
    dioxus::launch(RootApp);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ComponentPage {
    Button,
    Separator,
    Label,
    Toggle,
    ToggleGroup,
    Checkbox,
    Switch,
    RadioGroup,
    Progress,
    Meter,
    AspectRatio,
    Avatar,
    Slider,
    ScrollArea,
    HoverCard,
    Toolbar,
    DropdownMenu,
    ContextMenu,
    Menubar,
    NavigationMenu,
    Pagination,
    PinInput,
    Command,
    Combobox,
    VisuallyHidden,
    AccessibleIcon,
    RatingGroup,
    Calendar,
    RangeCalendar,
    DateField,
    TimeField,
    DateRangeField,
    TimeRangeField,
    DatePicker,
    DateRangePicker,
    Form1B,
    Select,
    Collapsible,
    Accordion,
    Tabs,
    Dialog,
    AlertDialog,
    Popover,
    Tooltip,
    Toast,
}

impl ComponentPage {
    pub fn title(&self) -> &'static str {
        match self {
            Self::Button => "Button",
            Self::Separator => "Separator",
            Self::Label => "Label",
            Self::Toggle => "Toggle",
            Self::ToggleGroup => "Toggle Group",
            Self::Checkbox => "Checkbox",
            Self::Switch => "Switch",
            Self::RadioGroup => "Radio Group",
            Self::Progress => "Progress",
            Self::Meter => "Meter",
            Self::AspectRatio => "Aspect Ratio",
            Self::Avatar => "Avatar",
            Self::Slider => "Slider",
            Self::ScrollArea => "Scroll Area",
            Self::HoverCard => "Hover Card",
            Self::Toolbar => "Toolbar",
            Self::DropdownMenu => "Dropdown Menu",
            Self::ContextMenu => "Context Menu",
            Self::Menubar => "Menubar",
            Self::NavigationMenu => "Navigation Menu",
            Self::Pagination => "Pagination",
            Self::PinInput => "Pin Input",
            Self::Command => "Command",
            Self::Combobox => "Combobox",
            Self::VisuallyHidden => "Visually Hidden",
            Self::AccessibleIcon => "Accessible Icon",
            Self::RatingGroup => "Rating Group",
            Self::Calendar => "Calendar",
            Self::RangeCalendar => "Range Calendar",
            Self::DateField => "Date Field",
            Self::TimeField => "Time Field",
            Self::DateRangeField => "Date Range Field",
            Self::TimeRangeField => "Time Range Field",
            Self::DatePicker => "Date Picker",
            Self::DateRangePicker => "Date Range Picker",
            Self::Form1B => "Form (1B)",
            Self::Select => "Select",
            Self::Collapsible => "Collapsible",
            Self::Accordion => "Accordion",
            Self::Tabs => "Tabs",
            Self::Dialog => "Dialog",
            Self::AlertDialog => "Alert Dialog",
            Self::Popover => "Popover",
            Self::Tooltip => "Tooltip",
            Self::Toast => "Toast",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Button => "▣",
            Self::Separator => "―",
            Self::Label => "¶",
            Self::Toggle => "◉",
            Self::ToggleGroup => "⊞",
            Self::Checkbox => "☑",
            Self::Switch => "⊸",
            Self::RadioGroup => "◉",
            Self::Progress => "▰",
            Self::Meter => "▱",
            Self::AspectRatio => "▭",
            Self::Avatar => "◉",
            Self::Slider => "⊶",
            Self::ScrollArea => "⇳",
            Self::HoverCard => "◱",
            Self::Toolbar => "☰",
            Self::DropdownMenu => "▾",
            Self::ContextMenu => "▤",
            Self::Menubar => "⊟",
            Self::NavigationMenu => "⇥",
            Self::Pagination => "⋯",
            Self::PinInput => "⌸",
            Self::Command => "⌘",
            Self::Combobox => "⌕",
            Self::VisuallyHidden => "◌",
            Self::AccessibleIcon => "★",
            Self::RatingGroup => "☆",
            Self::Calendar => "▦",
            Self::RangeCalendar => "◫",
            Self::DateField => "📅",
            Self::TimeField => "⏱",
            Self::DateRangeField => "⇄",
            Self::TimeRangeField => "⏲",
            Self::DatePicker => "📆",
            Self::DateRangePicker => "🗓",
            Self::Form1B => "📝",
            Self::Select => "▾",
            Self::Collapsible => "◰",
            Self::Accordion => "≡",
            Self::Tabs => "◫",
            Self::Dialog => "◻",
            Self::AlertDialog => "⚠",
            Self::Popover => "⌖",
            Self::Tooltip => "ℹ",
            Self::Toast => "💬",
        }
    }

    pub fn badge(&self) -> &'static str {
        match self {
            Self::Button => "Polymorphic",
            Self::Separator => "Divider",
            Self::Label => "Association",
            Self::Toggle => "2-State",
            Self::ToggleGroup => "Roving",
            Self::Checkbox => "Tri-state",
            Self::Switch => "Track/Thumb",
            Self::RadioGroup => "Roving APG",
            Self::Progress => "WAI-ARIA",
            Self::Meter => "ARIA Meter",
            Self::AspectRatio => "Live Ratio",
            Self::Avatar => "Image Status",
            Self::Slider => "Range/Drag",
            Self::ScrollArea => "Custom Scroll",
            Self::HoverCard => "Hover/Grace",
            Self::Toolbar => "Roving Bar",
            Self::DropdownMenu => "Menu/Sub",
            Self::ContextMenu => "Virtual Rect",
            Self::Menubar => "Bar/Swap",
            Self::NavigationMenu => "Viewport/Nav",
            Self::Pagination => "Page Range",
            Self::PinInput => "OTP Overlay",
            Self::Command => "Palette/Score",
            Self::Combobox => "Filter List",
            Self::VisuallyHidden => "SR-Only",
            Self::AccessibleIcon => "Icon A11y",
            Self::RatingGroup => "Half-Star",
            Self::Calendar => "2D Grid",
            Self::RangeCalendar => "Range Grid",
            Self::DateField => "Spinbutton",
            Self::TimeField => "12h/24h",
            Self::DateRangeField => "Start/End",
            Self::TimeRangeField => "Time Range",
            Self::DatePicker => "Popover Cal",
            Self::DateRangePicker => "Range Picker",
            Self::Form1B => "FormData Reset",
            Self::Select => "7 Scenarios",
            Self::Collapsible => "Disclosure",
            Self::Accordion => "Interactive",
            Self::Tabs => "WAI-ARIA",
            Self::Dialog => "Modal",
            Self::AlertDialog => "Destructive",
            Self::Popover => "Anchored",
            Self::Tooltip => "Hover/Focus",
            Self::Toast => "Sonner Stacking",
        }
    }
}

#[component]
pub fn App() -> Element {
    let mut current_page = use_signal(|| ComponentPage::Select);

    let pages = [
        ComponentPage::Button,
        ComponentPage::Separator,
        ComponentPage::Label,
        ComponentPage::Toggle,
        ComponentPage::ToggleGroup,
        ComponentPage::Checkbox,
        ComponentPage::Switch,
        ComponentPage::RadioGroup,
        ComponentPage::Progress,
        ComponentPage::Meter,
        ComponentPage::AspectRatio,
        ComponentPage::Avatar,
        ComponentPage::Slider,
        ComponentPage::ScrollArea,
        ComponentPage::HoverCard,
        ComponentPage::Toolbar,
        ComponentPage::DropdownMenu,
        ComponentPage::ContextMenu,
        ComponentPage::Menubar,
        ComponentPage::NavigationMenu,
        ComponentPage::Pagination,
        ComponentPage::PinInput,
        ComponentPage::Command,
        ComponentPage::Combobox,
        ComponentPage::VisuallyHidden,
        ComponentPage::AccessibleIcon,
        ComponentPage::RatingGroup,
        ComponentPage::Calendar,
        ComponentPage::RangeCalendar,
        ComponentPage::DateField,
        ComponentPage::TimeField,
        ComponentPage::DateRangeField,
        ComponentPage::TimeRangeField,
        ComponentPage::DatePicker,
        ComponentPage::DateRangePicker,
        ComponentPage::Form1B,
        ComponentPage::Select,
        ComponentPage::Collapsible,
        ComponentPage::Accordion,
        ComponentPage::Tabs,
        ComponentPage::Dialog,
        ComponentPage::AlertDialog,
        ComponentPage::Popover,
        ComponentPage::Tooltip,
        ComponentPage::Toast,
    ];

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/tailwind.css") }
        style { {include_str!("../../assets/pretendard.css")} }
        style { "{SW1A_PLAYGROUND_CSS}\n{SW1B_PLAYGROUND_CSS}\n{SW2A_PLAYGROUND_CSS}\n{SW2B_PLAYGROUND_CSS}" }
        div {
            style: "min-height: 100vh; background-color: #f8fafc; display: flex; flex-direction: row; color: #0f172a; font-family: 'Pretendard', -apple-system, BlinkMacSystemFont, system-ui, Roboto, 'Helvetica Neue', 'Segoe UI', 'Apple SD Gothic Neo', 'Noto Sans KR', 'Malgun Gothic', sans-serif;",

            // Left Navigation Sidebar
            aside {
                style: "width: 260px; min-width: 260px; background-color: #ffffff; border-right: 1px solid #e2e8f0; display: flex; flex-direction: column; padding: 1.5rem 1rem; gap: 1.5rem;",

                // Header / Branding
                div {
                    style: "display: flex; flex-direction: column; gap: 0.35rem;",
                    div {
                        style: "display: flex; align-items: center; justify-content: space-between;",
                        div {
                            style: "display: flex; align-items: center; gap: 0.5rem;",
                            MonoxusIcon {}
                            h1 {
                                style: "margin: 0; font-size: 1.25rem; font-weight: 700; color: #581c87; letter-spacing: -0.025em;",
                                "monoxus"
                            }
                        }
                        span {
                            style: "font-size: 0.6875rem; font-weight: 600; padding: 0.125rem 0.375rem; border-radius: 9999px; background-color: #f3e8ff; color: #7e22ce;",
                            "alpha"
                        }
                    }
                    p {
                        style: "margin: 0; font-size: 0.75rem; color: #64748b;",
                        "Headless WAI-ARIA Primitives"
                    }
                }

                // Nav Links
                nav {
                    "aria-label": "Component Navigation",
                    style: "display: flex; flex-direction: column; gap: 0.375rem;",
                    div {
                        style: "font-size: 0.6875rem; font-weight: 700; text-transform: uppercase; color: #94a3b8; letter-spacing: 0.05em; padding-left: 0.5rem; margin-bottom: 0.25rem;",
                        "Components"
                    }
                    for page in pages {
                        {
                            let is_active = current_page() == page;
                            let active_style = if is_active {
                                "background-color: #f3e8ff; color: #7e22ce; font-weight: 600; border: 1px solid #d8b4fe;"
                            } else {
                                "background-color: transparent; color: #475569; font-weight: 500; border: 1px solid transparent;"
                            };
                            let page_id = match page {
                                ComponentPage::Checkbox => "nav-checkbox",
                                ComponentPage::Switch => "nav-switch",
                                ComponentPage::RadioGroup => "nav-radio-group",
                                ComponentPage::Form1B => "nav-form-1b",
                                _ => "",
                            };
                            let legacy_id = format!("nav-link-{}", page.title().to_lowercase().replace(' ', "-").replace('(', "").replace(')', ""));
                            let primary_id = if !page_id.is_empty() {
                                page_id.to_string()
                            } else {
                                legacy_id.clone()
                            };
                            rsx! {
                                button {
                                    id: "{primary_id}",
                                    style: "display: flex; align-items: center; justify-content: space-between; width: 100%; padding: 0.5rem 0.75rem; border-radius: 0.5rem; font-size: 0.875rem; cursor: pointer; text-align: left; transition: all 0.15s ease; outline: none; {active_style}",
                                    onclick: move |_| current_page.set(page),
                                    div {
                                        style: "display: flex; align-items: center; gap: 0.625rem;",
                                        span {
                                            style: "font-family: monospace; font-size: 0.9375rem; width: 1rem; text-align: center; color: #9333ea;",
                                            "{page.icon()}"
                                        }
                                        span { "{page.title()}" }
                                    }
                                    span {
                                        style: "font-size: 0.6875rem; padding: 0.125rem 0.375rem; border-radius: 0.25rem; background-color: rgba(147, 51, 234, 0.08); color: #7e22ce;",
                                        "{page.badge()}"
                                    }
                                }
                                if page == ComponentPage::Form1B {
                                    button {
                                        id: "nav-sw1b-form",
                                        style: "display: none;",
                                        onclick: move |_| current_page.set(ComponentPage::Form1B),
                                    }
                                }
                                if primary_id != legacy_id {
                                    button {
                                        id: "{legacy_id}",
                                        style: "display: none;",
                                        onclick: move |_| current_page.set(page),
                                    }
                                }
                            }
                        }
                    }
                }

                // Footer Information
                div {
                    style: "margin-top: auto; padding-top: 1rem; border-top: 1px solid #f1f5f9; font-size: 0.75rem; color: #94a3b8;",
                    p { style: "margin: 0 0 0.25rem 0;", "Run with:" }
                    code {
                        style: "display: block; padding: 0.375rem 0.5rem; background-color: #f1f5f9; border-radius: 0.25rem; font-size: 0.6875rem; color: #334155; overflow-x: auto;",
                        "dx serve --example playground"
                    }
                }
            }

            // Main Content Body
            main {
                id: "playground-main-content",
                style: "flex: 1; min-width: 0; padding: 2rem 2.5rem; max-width: 64rem; overflow-y: auto;",

                // Active Page Rendered in Complete Isolation
                match current_page() {
                    ComponentPage::Button => rsx! { ButtonPage {} },
                    ComponentPage::Separator => rsx! { SeparatorPage {} },
                    ComponentPage::Label => rsx! { LabelPage {} },
                    ComponentPage::Toggle => rsx! { TogglePage {} },
                    ComponentPage::ToggleGroup => rsx! { ToggleGroupPage {} },
                    ComponentPage::Checkbox => rsx! { CheckboxPage {} },
                    ComponentPage::Switch => rsx! { SwitchPage {} },
                    ComponentPage::RadioGroup => rsx! { RadioGroupPage {} },
                    ComponentPage::Progress => rsx! { ProgressPage {} },
                    ComponentPage::Meter => rsx! { MeterPage {} },
                    ComponentPage::AspectRatio => rsx! { AspectRatioPage {} },
                    ComponentPage::Avatar => rsx! { AvatarPage {} },
                    ComponentPage::Slider => rsx! { SliderPage {} },
                    ComponentPage::ScrollArea => rsx! { ScrollAreaPage {} },
                    ComponentPage::HoverCard => rsx! { HoverCardPage {} },
                    ComponentPage::Toolbar => rsx! { ToolbarPage {} },
                    ComponentPage::DropdownMenu => rsx! { DropdownMenuPage {} },
                    ComponentPage::ContextMenu => rsx! { ContextMenuPage {} },
                    ComponentPage::Menubar => rsx! { MenubarPage {} },
                    ComponentPage::NavigationMenu => rsx! { NavigationMenuPage {} },
                    ComponentPage::Pagination => rsx! { PaginationPage {} },
                    ComponentPage::PinInput => rsx! { PinInputPage {} },
                    ComponentPage::Command => rsx! { CommandPage {} },
                    ComponentPage::Combobox => rsx! { ComboboxPage {} },
                    ComponentPage::VisuallyHidden => rsx! { VisuallyHiddenPage {} },
                    ComponentPage::AccessibleIcon => rsx! { AccessibleIconPage {} },
                    ComponentPage::RatingGroup => rsx! { RatingGroupPage {} },
                    ComponentPage::Calendar => rsx! { CalendarPage {} },
                    ComponentPage::RangeCalendar => rsx! { RangeCalendarPage {} },
                    ComponentPage::DateField => rsx! { DateFieldPage {} },
                    ComponentPage::TimeField => rsx! { TimeFieldPage {} },
                    ComponentPage::DateRangeField => rsx! { DateRangeFieldPage {} },
                    ComponentPage::TimeRangeField => rsx! { TimeRangeFieldPage {} },
                    ComponentPage::DatePicker => rsx! { DatePickerPage {} },
                    ComponentPage::DateRangePicker => rsx! { DateRangePickerPage {} },
                    ComponentPage::Form1B => rsx! { Form1bPage {} },
                    ComponentPage::Select => rsx! { SelectPage {} },
                    ComponentPage::Collapsible => rsx! { CollapsiblePage {} },
                    ComponentPage::Accordion => rsx! { AccordionPage {} },
                    ComponentPage::Tabs => rsx! { TabsPage {} },
                    ComponentPage::Dialog => rsx! { DialogPage {} },
                    ComponentPage::AlertDialog => rsx! { AlertDialogPage {} },
                    ComponentPage::Popover => rsx! { PopoverPage {} },
                    ComponentPage::Tooltip => rsx! { TooltipPage {} },
                    ComponentPage::Toast => rsx! { ToastPage {} },
                }
            }
        }
    }
}

#[component]
fn MonoxusIcon() -> Element {
    let svg_content = include_str!("../../assets/icon.svg");
    rsx! {
        div {
            style: "width: 24px; height: 24px; flex-shrink: 0; display: flex; align-items: center; justify-content: center;",
            dangerous_inner_html: "{svg_content}",
        }
    }
}

pub const SW1A_PLAYGROUND_CSS: &str = r#"
.sw1a-btn-primary {
    background-color: #7c3aed;
    color: #ffffff;
    border: 1px solid #6d28d9;
    box-shadow: 0 1px 2px rgba(15, 23, 42, 0.12);
    border-radius: 0.5rem;
    padding: 0.5rem 1rem;
    font-size: 0.875rem;
    font-weight: 600;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    text-decoration: none;
    transition: background-color 0.15s ease, border-color 0.15s ease;
}
.sw1a-btn-primary:hover:not(:disabled):not([data-disabled]) {
    background-color: #6d28d9;
}
.sw1a-btn-primary:active:not(:disabled):not([data-disabled]) {
    background-color: #5b21b6;
}
.sw1a-btn-primary:focus-visible {
    outline: 2px solid #9333ea;
    outline-offset: 2px;
}

.sw1a-btn-dark {
    background-color: #0f172a;
    color: #ffffff;
    border: 1px solid #0f172a;
    box-shadow: 0 1px 2px rgba(15, 23, 42, 0.12);
    border-radius: 0.5rem;
    padding: 0.375rem 0.75rem;
    font-size: 0.75rem;
    font-weight: 600;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    text-decoration: none;
    transition: background-color 0.15s ease, border-color 0.15s ease;
}
.sw1a-btn-dark:hover:not(:disabled):not([data-disabled]) {
    background-color: #1e293b;
}
.sw1a-btn-dark:active:not(:disabled):not([data-disabled]) {
    background-color: #334155;
}
.sw1a-btn-dark:focus-visible {
    outline: 2px solid #9333ea;
    outline-offset: 2px;
}

.sw1a-btn-secondary {
    background-color: #ffffff;
    color: #334155;
    border: 1px solid #cbd5e1;
    box-shadow: 0 1px 2px rgba(15, 23, 42, 0.05);
    border-radius: 0.5rem;
    padding: 0.5rem 1rem;
    font-size: 0.875rem;
    font-weight: 500;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    text-decoration: none;
    transition: background-color 0.15s ease, border-color 0.15s ease;
}
.sw1a-btn-secondary:hover:not(:disabled):not([data-disabled]) {
    background-color: #f8fafc;
    border-color: #94a3b8;
}
.sw1a-btn-secondary:focus-visible {
    outline: 2px solid #9333ea;
    outline-offset: 2px;
}

[data-button-root][data-disabled],
[data-button-root]:disabled {
    background-color: #f1f5f9 !important;
    color: #94a3b8 !important;
    border: 1px solid #e2e8f0 !important;
    opacity: 0.65;
    cursor: not-allowed !important;
    box-shadow: none !important;
}

[data-toggle-root],
[data-toggle-group-item] {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 0.375rem;
    font-size: 0.8125rem;
    font-weight: 600;
    cursor: pointer;
    text-decoration: none;
    user-select: none;
    outline: none;
    padding: 0.375rem 0.75rem;
}

[data-toggle-root],
[data-toggle-group-item],
[data-toggle-root][data-state="off"],
[data-toggle-group-item][data-state="off"] {
    background-color: #ffffff;
    color: #334155;
    border: 1px solid #cbd5e1;
    transition: background-color 0.15s ease, color 0.15s ease, border-color 0.15s ease;
}

[data-toggle-root]:hover:not([data-disabled]),
[data-toggle-group-item]:hover:not([data-disabled]) {
    background-color: #f5f3ff;
    border-color: #c4b5fd;
    color: #5b21b6;
}

[data-toggle-root][data-state="on"],
[data-toggle-group-item][data-state="on"] {
    background-color: #7c3aed !important;
    color: #ffffff !important;
    border-color: #6d28d9 !important;
    box-shadow: 0 1px 3px rgba(124, 58, 237, 0.35);
}

[data-toggle-root][data-state="on"]:hover:not([data-disabled]),
[data-toggle-group-item][data-state="on"]:hover:not([data-disabled]) {
    background-color: #6d28d9 !important;
    border-color: #5b21b6 !important;
    color: #ffffff !important;
}

[data-toggle-root][data-disabled],
[data-toggle-group-item][data-disabled] {
    background-color: #f1f5f9 !important;
    color: #94a3b8 !important;
    border-color: #e2e8f0 !important;
    opacity: 0.55;
    cursor: not-allowed !important;
    box-shadow: none !important;
}

[data-toggle-root][data-state="on"][data-disabled],
[data-toggle-group-item][data-state="on"][data-disabled] {
    background-color: #ede9fe !important;
    color: #6d28d9 !important;
    border-color: #c4b5fd !important;
    opacity: 0.65;
    cursor: not-allowed !important;
    box-shadow: none !important;
}

/* Pills and Badges */
.sw1a-pill-purple {
    display: inline-flex;
    align-items: center;
    border-radius: 0.375rem;
    background-color: #f5f3ff;
    color: #6d28d9;
    border: 1px solid #ddd6fe;
    padding: 0.125rem 0.5rem;
    font-size: 0.75rem;
    font-family: monospace;
    font-weight: 500;
}

.sw1a-pill-amber {
    display: inline-flex;
    align-items: center;
    border-radius: 0.375rem;
    background-color: #fffbeb;
    color: #b45309;
    border: 1px solid #fde68a;
    padding: 0.125rem 0.5rem;
    font-size: 0.75rem;
    font-family: monospace;
    font-weight: 500;
}

.sw1a-pill-slate {
    display: inline-flex;
    align-items: center;
    border-radius: 0.375rem;
    background-color: #f8fafc;
    color: #475569;
    border: 1px solid #e2e8f0;
    padding: 0.125rem 0.5rem;
    font-size: 0.75rem;
    font-family: monospace;
    font-weight: 500;
}

.sw1a-badge-on {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 0.25rem;
    background-color: #ede9fe;
    color: #6d28d9;
    border: 1px solid #c4b5fd;
    padding: 0.0625rem 0.375rem;
    font-size: 0.625rem;
    font-weight: 700;
    line-height: 1;
}

.sw1a-badge-off {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 0.25rem;
    background-color: #f1f5f9;
    color: #64748b;
    border: 1px solid #e2e8f0;
    padding: 0.0625rem 0.375rem;
    font-size: 0.625rem;
    font-weight: 600;
    line-height: 1;
}

.sw1a-tg-vertical {
    width: 11rem;
}

[data-toggle-root]:focus-visible,
[data-toggle-group-item]:focus-visible {
    outline: 2px solid #9333ea;
    outline-offset: 2px;
}

[data-toggle-group-item][tabindex="0"]:not([data-disabled]) {
    box-shadow: inset 0 -2px 0 #a855f7;
}

[data-toggle-group-item][data-state="on"][tabindex="0"]:not([data-disabled]) {
    box-shadow: 0 1px 3px rgba(124, 58, 237, 0.35), inset 0 -2px 0 #ede9fe;
}

[data-separator-root][data-orientation="horizontal"] {
    height: 2px;
    width: 100%;
    background-color: #cbd5e1;
}

[data-separator-root][data-orientation="vertical"] {
    width: 2px;
    height: 22px;
    background-color: #cbd5e1;
    flex-shrink: 0;
    align-self: center;
}

#demo-input:focus,
#nested-input:focus,
#label-nested-input:focus {
    border-color: #7c3aed;
    outline: 2px solid rgba(124, 58, 237, 0.25);
    outline-offset: 0px;
}
"#;

pub const SW1B_PLAYGROUND_CSS: &str = r#"
/* Checkbox Styles */
[data-checkbox-root] {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border-radius: 4px;
    border: 1px solid #cbd5e1;
    background-color: #ffffff;
    cursor: pointer;
    padding: 0;
    margin: 0;
    user-select: none;
    outline: none;
    box-sizing: border-box;
    transition: background-color 0.15s ease, border-color 0.15s ease;
}

[data-checkbox-root]:hover:not([data-disabled]):not([data-readonly]) {
    border-color: #7c3aed;
    background-color: #f5f3ff;
}

[data-checkbox-root][data-state="checked"],
[data-checkbox-root][data-state="indeterminate"] {
    background-color: #7c3aed;
    border-color: #6d28d9;
    color: #ffffff;
}

[data-checkbox-root]:focus-visible {
    outline: 2px solid #9333ea;
    outline-offset: 2px;
}

[data-checkbox-root][data-disabled] {
    background-color: #f1f5f9 !important;
    border-color: #e2e8f0 !important;
    color: #94a3b8 !important;
    cursor: not-allowed !important;
    opacity: 0.55;
}

[data-checkbox-root][data-readonly] {
    border-color: #cbd5e1;
    background-color: #f8fafc;
    cursor: default;
}

[data-checkbox-indicator] {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    font-size: 13px;
    font-weight: 700;
    line-height: 1;
    color: #ffffff;
    pointer-events: none;
}

/* Switch Styles - Dual-host Taffy + Web flex track alignment */
[data-switch-root] {
    display: inline-flex;
    align-items: center;
    width: 38px;
    height: 22px;
    border-radius: 9999px;
    border: 2px solid transparent;
    background-color: #cbd5e1;
    padding: 1px;
    cursor: pointer;
    box-sizing: border-box;
    outline: none;
    transition: background-color 0.15s ease;
}

[data-switch-root][data-state="unchecked"] {
    justify-content: flex-start;
    background-color: #cbd5e1;
}

[data-switch-root][data-state="checked"] {
    justify-content: flex-end;
    background-color: #7c3aed;
}

[data-switch-root]:focus-visible {
    outline: 2px solid #9333ea;
    outline-offset: 2px;
}

[data-switch-root][data-disabled] {
    opacity: 0.55;
    cursor: not-allowed !important;
    background-color: #e2e8f0 !important;
}

[data-switch-root][data-readonly] {
    cursor: default;
}

[data-switch-thumb] {
    display: block;
    width: 16px;
    height: 16px;
    border-radius: 9999px;
    background-color: #ffffff;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
    pointer-events: none;
    flex-shrink: 0;
}

/* Radio Group Styles */
[data-radio-group-root] {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
}

[data-radio-group-root][data-orientation="horizontal"] {
    flex-direction: row;
    align-items: center;
}

[data-radio-group-item] {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border-radius: 9999px;
    border: 1px solid #cbd5e1;
    background-color: #ffffff;
    padding: 0;
    margin: 0;
    cursor: pointer;
    box-sizing: border-box;
    outline: none;
    user-select: none;
    transition: border-color 0.15s ease, background-color 0.15s ease;
}

[data-radio-group-item]:hover:not([data-disabled]):not([data-readonly]) {
    border-color: #7c3aed;
    background-color: #f5f3ff;
}

[data-radio-group-item][data-state="checked"] {
    border-color: #7c3aed;
    background-color: #ffffff;
}

[data-radio-group-item]:focus-visible {
    outline: 2px solid #9333ea;
    outline-offset: 2px;
}

[data-radio-group-item][data-disabled] {
    background-color: #f1f5f9 !important;
    border-color: #e2e8f0 !important;
    cursor: not-allowed !important;
    opacity: 0.55;
}

[data-radio-group-item][data-readonly] {
    cursor: default;
}

[data-radio-group-indicator] {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 10px;
    height: 10px;
    border-radius: 9999px;
    background-color: #7c3aed;
    pointer-events: none;
}
"#;

pub const SW2A_PLAYGROUND_CSS: &str = r#"
[data-progress-root] {
    height: 0.75rem;
    overflow: hidden;
    border-radius: 9999px;
    background-color: #e2e8f0;
}

[data-progress-indicator] {
    height: 100%;
    border-radius: inherit;
    transition: width 0.3s ease, background-color 0.3s ease;
}

[data-progress-indicator][data-state="loading"] {
    background-color: #7c3aed;
}

[data-progress-indicator][data-state="complete"] {
    background-color: #16a34a;
}

[data-progress-indicator][data-state="indeterminate"] {
    background-color: #a78bfa;
}
"#;

pub const SW2B_PLAYGROUND_CSS: &str = r#"
@keyframes sw2bSlideDown {
    from {
        height: 0;
    }
    to {
        height: var(--radix-collapsible-content-height);
    }
}

@keyframes sw2bSlideUp {
    from {
        height: var(--radix-collapsible-content-height);
    }
    to {
        height: 0;
    }
}

.sw2b-collapsible-animated {
    overflow: hidden;
}

.sw2b-collapsible-animated[data-state="open"] {
    animation: sw2bSlideDown 250ms ease-out;
}

.sw2b-collapsible-animated[data-state="closed"] {
    animation: sw2bSlideUp 350ms ease-out forwards;
}

.sw2b-slider-track {
    position: relative;
    flex-grow: 1;
    overflow: hidden;
    border-radius: 9999px;
    background-color: #e2e8f0;
    user-select: none;
}

.sw2b-slider-track[data-orientation="vertical"] {
    flex-grow: 0;
}

.sw2b-slider-range {
    position: absolute;
    background-color: #7c3aed;
    border-radius: 9999px;
}

.sw2b-slider-range[data-orientation="horizontal"] {
    top: 0;
    bottom: 0;
    height: 100%;
}

.sw2b-slider-range[data-orientation="vertical"] {
    left: 0;
    right: 0;
    width: 100%;
}

.sw2b-slider-thumb {
    display: block;
    width: 20px;
    height: 20px;
    border-radius: 9999px;
    background-color: #ffffff;
    border: 2px solid #7c3aed;
    box-shadow: 0 1px 3px rgba(15, 23, 42, 0.25);
    cursor: grab;
    outline: none;
}

.sw2b-slider-thumb:focus-visible {
    outline: 2px solid #9333ea;
    outline-offset: 2px;
}

.sw2b-slider-thumb[data-active] {
    cursor: grabbing;
    box-shadow: 0 0 0 4px rgba(124, 58, 237, 0.25);
}

.sw2b-slider-thumb[data-disabled] {
    cursor: not-allowed;
    border-color: #cbd5e1;
}

.sw2b-sa-viewport {
    width: 100%;
    height: 100%;
    border-radius: inherit;
    outline: none;
}

.sw2b-sa-viewport:focus-visible {
    outline: 2px solid #9333ea;
    outline-offset: 2px;
}

.sw2b-sa-scrollbar {
    display: flex;
    user-select: none;
    touch-action: none;
    padding: 2px;
    box-sizing: border-box;
    background-color: rgba(15, 23, 42, 0.06);
    transition: background-color 150ms ease-out;
}

.sw2b-sa-scrollbar:hover {
    background-color: rgba(15, 23, 42, 0.12);
}

.sw2b-sa-scrollbar[data-orientation="vertical"] {
    width: 10px;
}

.sw2b-sa-scrollbar[data-orientation="horizontal"] {
    height: 10px;
    flex-direction: column;
}

.sw2b-sa-thumb {
    flex: 1;
    background-color: rgba(100, 116, 139, 0.65);
    border-radius: 9999px;
    position: relative;
}

.sw2b-sa-thumb:hover {
    background-color: rgba(71, 85, 105, 0.85);
}

.sw2b-sa-corner {
    background-color: rgba(148, 163, 184, 0.35);
}
"#;
