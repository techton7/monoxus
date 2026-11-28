use super::{
    ComponentDoc, DataAttrDoc, KeyboardDoc, PROP_ATTRIBUTES, PROP_CHILD, PROP_CLASS,
    PROP_ON_MOUNTED, PROP_STYLE, PartDoc, PropDoc, PropGroup, prop_id,
    shared_menu::menu_parts_slice,
};

pub(super) const CONTEXT_MENU_DOC: ComponentDoc = ComponentDoc {
    slug: "context-menu",
    title: "Context Menu",
    description: "Virtual-point context menu opened via right-click or 700ms touch long-press, with nested submenus, checkbox/radio items, typeahead, and floating collision avoidance.",
    keywords: "context menu right click long press submenu checkbox radio typeahead floating",
    features: &[
        "Opens anchored to cursor coordinates on right-click (`contextmenu`) or 700ms non-mouse long-press",
        "Full shared menu engine: roving focus, typeahead search, checkbox groups, radio groups, and nested submenus with safe-corridor grace",
        "Supports modal and non-modal policies, 4-way Escape/outside dismiss behavior, and prevent_scroll",
        "Floating (`ContextMenuContent`) and inline static (`ContextMenuContentStatic`) surfaces with TransitionStatus hooks",
        "Exposes --monoxus-context-menu-* and --radix-context-menu-* CSS custom properties",
    ],
    anatomy: "use monoxus::context_menu::{\n    ContextMenuCheckboxItem, ContextMenuContent, ContextMenuItem, ContextMenuItemIndicator,\n    ContextMenuPortal, ContextMenuRadioGroup, ContextMenuRadioItem, ContextMenuRoot,\n    ContextMenuSeparator, ContextMenuSub, ContextMenuSubContent, ContextMenuSubTrigger,\n    ContextMenuTrigger,\n};\n\nrsx! {\n    ContextMenuRoot {\n        ContextMenuTrigger { \"Right-click here\" }\n        ContextMenuPortal {\n            ContextMenuContent {\n                ContextMenuItem { \"Back\" }\n                ContextMenuSeparator {}\n                ContextMenuCheckboxItem {\n                    ContextMenuItemIndicator { \"✓\" }\n                    \"Show Grid\"\n                }\n                ContextMenuRadioGroup { default_value: \"compact\",\n                    ContextMenuRadioItem { value: \"compact\",\n                        ContextMenuItemIndicator { \"•\" }\n                        \"Compact\"\n                    }\n                }\n                ContextMenuSub {\n                    ContextMenuSubTrigger { \"More Tools\" }\n                    ContextMenuPortal {\n                        ContextMenuSubContent {\n                            ContextMenuItem { \"Inspect\" }\n                        }\n                    }\n                }\n            }\n        }\n    }\n}",
    source_path: "src/context_menu/mod.rs",
    aria_url: "https://www.w3.org/WAI/ARIA/apg/patterns/menu-button/",
    aria_label: "WAI-ARIA Menu Pattern",
    parts: menu_parts_slice!(
        "ContextMenu",
        "context-menu",
        [
            PartDoc {
                name: "ContextMenuRoot",
                description: "Non-DOM state provider coordinating virtual-point context menu open state, modal policy, direction, and lifecycle.",
                props: &[
                    PropDoc { name: "open", type_simple: "bool", type_full: "Option<bool>", default: None, required: false, group: PropGroup::Core, description: "Controlled open state." },
                    PropDoc { name: "default_open", type_simple: "bool", type_full: "bool", default: Some("false"), required: false, group: PropGroup::Core, description: "Initial open state when uncontrolled." },
                    PropDoc { name: "on_open_change", type_simple: "EventHandler<bool>", type_full: "Option<EventHandler<bool>>", default: None, required: false, group: PropGroup::Core, description: "Called when open state changes." },
                    PropDoc { name: "on_open_change_complete", type_simple: "EventHandler<bool>", type_full: "Option<EventHandler<bool>>", default: None, required: false, group: PropGroup::Core, description: "Called after open/close presence transitions complete." },
                    PropDoc { name: "modal", type_simple: "bool", type_full: "bool", default: Some("true"), required: false, group: PropGroup::Core, description: "Enables modal outside-pointer blocking while open." },
                    PropDoc { name: "dir", type_simple: "Direction", type_full: "Option<Direction>", default: None, required: false, group: PropGroup::Core, description: "Reading direction (Ltr | Rtl) inherited from DirectionProvider when omitted." },
                    PropDoc { name: "loop_focus", type_simple: "bool", type_full: "bool", default: Some("true"), required: false, group: PropGroup::Core, description: "Wraps keyboard arrow navigation from last item to first and vice versa." },
                    PropDoc { name: "submenu_open_delay_ms", type_simple: "u64", type_full: "Option<u64>", default: Some("100"), required: false, group: PropGroup::Core, description: "Default hover delay in milliseconds before nested submenus open." },
                    PropDoc { name: "children", type_simple: "Element", type_full: "dioxus::prelude::Element", default: None, required: true, group: PropGroup::Core, description: "ContextMenuTrigger and ContextMenuPortal children." },
                ],
                data_attrs: &[],
                css_vars: &[],
            },
            PartDoc {
                name: "ContextMenuTrigger",
                description: "Interactive target region capturing right-click `contextmenu` events and 700ms non-mouse long-press gestures.",
                props: &[
                    prop_id("Custom trigger element ID."),
                    PROP_CLASS,
                    PROP_STYLE,
                    PropDoc { name: "disabled", type_simple: "bool", type_full: "bool", default: Some("false"), required: false, group: PropGroup::Core, description: "Disables opening the context menu from this trigger." },
                    PropDoc { name: "on_click", type_simple: "EventHandler<MouseEvent>", type_full: "Option<EventHandler<MouseEvent>>", default: None, required: false, group: PropGroup::Core, description: "Click handler forwarded to the trigger element." },
                    PropDoc { name: "on_key_down", type_simple: "EventHandler<KeyboardEvent>", type_full: "Option<EventHandler<KeyboardEvent>>", default: None, required: false, group: PropGroup::Core, description: "Keydown handler forwarded to the trigger element." },
                    PROP_ON_MOUNTED,
                    PROP_CHILD,
                    PROP_ATTRIBUTES,
                    PropDoc { name: "children", type_simple: "Element", type_full: "dioxus::prelude::Element", default: None, required: true, group: PropGroup::Core, description: "Trigger target region content." },
                ],
                data_attrs: &[
                    DataAttrDoc { name: "data-state", values: "\"open\" | \"closed\"", description: "Current context menu open state." },
                    DataAttrDoc { name: "data-disabled", values: "\"\"", description: "Present when disabled is true." },
                    DataAttrDoc { name: "data-monoxus-context-menu-trigger", values: "\"\"", description: "Part marker attribute." },
                ],
                css_vars: &[],
            },
        ]
    ),
    keyboard: &[
        KeyboardDoc { key: "ArrowDown / ArrowUp", description: "Moves focus to the next or previous enabled menu item." },
        KeyboardDoc { key: "Home / End", description: "Moves focus to the first or last enabled menu item." },
        KeyboardDoc { key: "Enter / Space", description: "Activates the highlighted item or opens a highlighted submenu." },
        KeyboardDoc { key: "ArrowRight / ArrowLeft", description: "Opens a highlighted submenu or closes the active submenu back to its SubTrigger (mirrored in RTL)." },
        KeyboardDoc { key: "Printable characters", description: "Typeahead search highlights the next matching enabled item." },
        KeyboardDoc { key: "Escape", description: "Closes the context menu tree." },
    ],
};
