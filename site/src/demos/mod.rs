use dioxus::prelude::*;

macro_rules! register_demos {
    ($( $slug:literal => $module:ident :: $component:ident ),* $(,)?) => {
        $( pub mod $module; )*

        pub fn render_demo(slug: &str) -> Element {
            match slug {
                $( $slug => rsx! { $module::$component {} }, )*
                _ => rsx! {
                    div { class: "text-sm text-muted-foreground",
                        "Interactive demo not found for `{slug}`."
                    }
                },
            }
        }

        pub fn demo_source(slug: &str) -> &'static str {
            match slug {
                $( $slug => include_str!(concat!(stringify!($module), ".rs")), )*
                _ => "",
            }
        }

        pub fn demo_filename(slug: &str) -> &'static str {
            match slug {
                $( $slug => concat!(stringify!($module), ".rs"), )*
                _ => "demo.rs",
            }
        }
    };
}

register_demos! {
    "accessible-icon" => accessible_icon_demo::AccessibleIconDemo,
    "accordion" => accordion_demo::AccordionDemo,
    "alert-dialog" => alert_dialog_demo::AlertDialogDemo,
    "aspect-ratio" => aspect_ratio_demo::AspectRatioDemo,
    "avatar" => avatar_demo::AvatarDemo,
    "button" => button_demo::ButtonDemo,
    "calendar" => calendar_demo::CalendarDemo,
    "checkbox" => checkbox_demo::CheckboxDemo,
    "collapsible" => collapsible_demo::CollapsibleDemo,
    "combobox" => combobox_demo::ComboboxDemo,
    "command" => command_demo::CommandDemo,
    "context-menu" => context_menu_demo::ContextMenuDemo,
    "date-field" => date_field_demo::DateFieldDemo,
    "date-picker" => date_picker_demo::DatePickerDemo,
    "date-range-field" => date_range_field_demo::DateRangeFieldDemo,
    "date-range-picker" => date_range_picker_demo::DateRangePickerDemo,
    "dialog" => dialog_demo::DialogDemo,
    "dropdown-menu" => dropdown_menu_demo::DropdownMenuDemo,
    "field" => field_demo::FieldDemo,
    "hover-card" => hover_card_demo::HoverCardDemo,
    "label" => label_demo::LabelDemo,
    "menubar" => menubar_demo::MenubarDemo,
    "meter" => meter_demo::MeterDemo,
    "navigation-menu" => navigation_menu_demo::NavigationMenuDemo,
    "pagination" => pagination_demo::PaginationDemo,
    "pin-input" => pin_input_demo::PinInputDemo,
    "popover" => popover_demo::PopoverDemo,
    "progress" => progress_demo::ProgressDemo,
    "radio-group" => radio_group_demo::RadioGroupDemo,
    "range-calendar" => range_calendar_demo::RangeCalendarDemo,
    "rating-group" => rating_group_demo::RatingGroupDemo,
    "scroll-area" => scroll_area_demo::ScrollAreaDemo,
    "select" => select_demo::SelectDemo,
    "separator" => separator_demo::SeparatorDemo,
    "slider" => slider_demo::SliderDemo,
    "switch" => switch_demo::SwitchDemo,
    "tabs" => tabs_demo::TabsDemo,
    "time-field" => time_field_demo::TimeFieldDemo,
    "time-range-field" => time_range_field_demo::TimeRangeFieldDemo,
    "toast" => toast_demo::ToastDemo,
    "toggle" => toggle_demo::ToggleDemo,
    "toggle-group" => toggle_group_demo::ToggleGroupDemo,
    "toolbar" => toolbar_demo::ToolbarDemo,
    "tooltip" => tooltip_demo::TooltipDemo,
    "visually-hidden" => visually_hidden_demo::VisuallyHiddenDemo,
}
