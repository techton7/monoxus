use super::{
    ComponentDoc, DataAttrDoc, KeyboardDoc, PROP_AGENT, PROP_ATTRIBUTES, PROP_CHILD, PROP_CLASS,
    PROP_ON_MOUNTED, PROP_STYLE, PartDoc, PropDoc, PropGroup, prop_id,
    shared_menu::menu_parts_slice,
};

pub(super) const DROPDOWN_MENU_DOC: ComponentDoc = ComponentDoc {
    slug: "dropdown-menu",
    title: "Dropdown Menu",
    description: "Button-triggered floating menu with roving focus, typeahead search, checkbox and radio item groups, nested submenus, and collision-aware positioning.",
    keywords: "dropdown menu trigger button submenu checkbox radio group typeahead floating",
    features: &[
        "Button-owned `DropdownMenuTrigger` with `aria-haspopup=\"menu\"`, `aria-expanded`, `aria-controls`, and return-focus management",
        "Full shared menu engine: roving focus, typeahead, standalone or grouped checkboxes, radio groups, and nested submenus",
        "Bidirectional safe-corridor pointer grace prevents accidental submenu dismissal while moving diagonally",
        "Floating (`DropdownMenuContent`) and inline static (`DropdownMenuContentStatic`) surfaces with TransitionStatus hooks",
        "Exposes --monoxus-dropdown-menu-* and --radix-dropdown-menu-* CSS custom properties",
    ],
    anatomy: "use monoxus::dropdown_menu::{\n    DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuGroup, DropdownMenuGroupHeading,\n    DropdownMenuItem, DropdownMenuItemIndicator, DropdownMenuPortal, DropdownMenuRadioGroup,\n    DropdownMenuRadioItem, DropdownMenuRoot, DropdownMenuSeparator, DropdownMenuSub,\n    DropdownMenuSubContent, DropdownMenuSubTrigger, DropdownMenuTrigger,\n};\n\nrsx! {\n    DropdownMenuRoot {\n        DropdownMenuTrigger { \"Options\" }\n        DropdownMenuPortal {\n            DropdownMenuContent {\n                DropdownMenuGroup {\n                    DropdownMenuGroupHeading { \"Actions\" }\n                    DropdownMenuItem { \"Duplicate\" }\n                }\n                DropdownMenuSeparator {}\n                DropdownMenuCheckboxItem {\n                    DropdownMenuItemIndicator { \"✓\" }\n                    \"Pinned\"\n                }\n                DropdownMenuRadioGroup { default_value: \"list\",\n                    DropdownMenuRadioItem { value: \"list\",\n                        DropdownMenuItemIndicator { \"•\" }\n                        \"List view\"\n                    }\n                }\n                DropdownMenuSub {\n                    DropdownMenuSubTrigger { \"Share\" }\n                    DropdownMenuPortal {\n                        DropdownMenuSubContent {\n                            DropdownMenuItem { \"Copy link\" }\n                        }\n                    }\n                }\n            }\n        }\n    }\n}",
    source_path: "src/dropdown_menu/mod.rs",
    aria_url: "https://www.w3.org/WAI/ARIA/apg/patterns/menu-button/",
    aria_label: "WAI-ARIA Menu Button Pattern",
    parts: menu_parts_slice!(
        "DropdownMenu",
        "dropdown-menu",
        [
            PartDoc {
                name: "DropdownMenuRoot",
                description: "Non-DOM state provider coordinating controlled/uncontrolled open state, modal policy, direction, and lifecycle.",
                props: &[
                    PropDoc { name: "open", type_simple: "bool", type_full: "Option<bool>", default: None, required: false, group: PropGroup::Core, description: "Controlled open state." },
                    PropDoc { name: "default_open", type_simple: "bool", type_full: "bool", default: Some("false"), required: false, group: PropGroup::Core, description: "Initial open state when uncontrolled." },
                    PropDoc { name: "on_open_change", type_simple: "EventHandler<bool>", type_full: "Option<EventHandler<bool>>", default: None, required: false, group: PropGroup::Core, description: "Called when open state changes." },
                    PropDoc { name: "on_open_change_details", type_simple: "EventHandler<ChangeEvent<bool>>", type_full: "Option<EventHandler<ChangeEvent<bool>>>", default: None, required: false, group: PropGroup::Core, description: "Called with full ChangeEvent details (reason, source, cancelation) when open state changes." },
                    PropDoc { name: "on_open_change_complete", type_simple: "EventHandler<bool>", type_full: "Option<EventHandler<bool>>", default: None, required: false, group: PropGroup::Core, description: "Called after open/close presence transitions complete." },
                    PropDoc { name: "modal", type_simple: "bool", type_full: "bool", default: Some("true"), required: false, group: PropGroup::Core, description: "Enables modal outside-pointer blocking while open." },
                    PropDoc { name: "dir", type_simple: "Direction", type_full: "Option<Direction>", default: None, required: false, group: PropGroup::Core, description: "Reading direction (Ltr | Rtl) inherited from DirectionProvider when omitted." },
                    PropDoc { name: "loop_focus", type_simple: "bool", type_full: "bool", default: Some("true"), required: false, group: PropGroup::Core, description: "Wraps keyboard arrow navigation from last item to first and vice versa." },
                    PropDoc { name: "submenu_open_delay_ms", type_simple: "u64", type_full: "Option<u64>", default: Some("100"), required: false, group: PropGroup::Core, description: "Default hover delay in milliseconds before nested submenus open." },
                    PROP_AGENT,
                    PropDoc { name: "children", type_simple: "Element", type_full: "dioxus::prelude::Element", default: None, required: true, group: PropGroup::Core, description: "DropdownMenuTrigger and DropdownMenuPortal children." },
                ],
                data_attrs: &[],
                css_vars: &[],
            },
            PartDoc {
                name: "DropdownMenuTrigger",
                description: "Interactive button toggling the enclosing DropdownMenu and restoring focus on close.",
                props: &[
                    prop_id("Custom trigger element ID."),
                    PROP_CLASS,
                    PROP_STYLE,
                    PropDoc { name: "disabled", type_simple: "bool", type_full: "bool", default: Some("false"), required: false, group: PropGroup::Core, description: "Disables the trigger button." },
                    PropDoc { name: "on_click", type_simple: "EventHandler<MouseEvent>", type_full: "Option<EventHandler<MouseEvent>>", default: None, required: false, group: PropGroup::Core, description: "Click handler composed with trigger toggling." },
                    PropDoc { name: "on_key_down", type_simple: "EventHandler<KeyboardEvent>", type_full: "Option<EventHandler<KeyboardEvent>>", default: None, required: false, group: PropGroup::Core, description: "Keydown handler composed with keyboard open shortcuts." },
                    PROP_ON_MOUNTED,
                    PROP_CHILD,
                    PROP_ATTRIBUTES,
                    PropDoc { name: "children", type_simple: "Element", type_full: "dioxus::prelude::Element", default: None, required: true, group: PropGroup::Core, description: "Trigger button label or icon." },
                ],
                data_attrs: &[
                    DataAttrDoc { name: "data-state", values: "\"open\" | \"closed\"", description: "Current dropdown menu open state." },
                    DataAttrDoc { name: "data-disabled", values: "\"\"", description: "Present when disabled is true." },
                    DataAttrDoc { name: "data-monoxus-dropdown-menu-trigger", values: "\"\"", description: "Part marker attribute." },
                ],
                css_vars: &[],
            },
        ]
    ),
    keyboard: &[
        KeyboardDoc { key: "Enter / Space / ArrowDown (Trigger)", description: "Opens the dropdown menu and focuses the first enabled item." },
        KeyboardDoc { key: "ArrowDown / ArrowUp", description: "Moves focus to the next or previous enabled menu item." },
        KeyboardDoc { key: "Home / End", description: "Moves focus to the first or last enabled menu item." },
        KeyboardDoc { key: "Enter / Space (Item)", description: "Activates the highlighted item or opens a highlighted submenu." },
        KeyboardDoc { key: "ArrowRight / ArrowLeft", description: "Opens a highlighted submenu or closes the active submenu back to its SubTrigger (mirrored in RTL)." },
        KeyboardDoc { key: "Printable characters", description: "Typeahead search highlights the next matching enabled item." },
        KeyboardDoc { key: "Escape", description: "Closes the menu and restores focus to DropdownMenuTrigger." },
    ],
};
