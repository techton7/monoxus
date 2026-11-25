#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PropDoc {
    pub name: &'static str,
    pub type_simple: &'static str,
    pub type_full: &'static str,
    pub default: Option<&'static str>,
    pub required: bool,
    pub description: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataAttrDoc {
    pub name: &'static str,
    pub values: &'static str,
    pub description: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CssVarDoc {
    pub name: &'static str,
    pub description: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyboardDoc {
    pub key: &'static str,
    pub description: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PartDoc {
    pub name: &'static str,
    pub description: &'static str,
    pub props: &'static [PropDoc],
    pub data_attrs: &'static [DataAttrDoc],
    pub css_vars: &'static [CssVarDoc],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComponentDoc {
    pub slug: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub features: &'static [&'static str],
    pub anatomy: &'static str,
    pub source_path: &'static str,
    pub aria_url: &'static str,
    pub aria_label: &'static str,
    pub parts: &'static [PartDoc],
    pub keyboard: &'static [KeyboardDoc],
}

static BUTTON_DOC: ComponentDoc = ComponentDoc {
    slug: "button",
    title: "Button",
    description: "Polymorphic headless button rendering a native <button> or <a> element with disabled link semantics.",
    features: &[
        "Renders <button> by default or <a> when href is provided",
        "Defaults button type to ButtonType::Button to prevent accidental form submission",
        "Suppresses click and Enter/Space activation when disabled",
        "Omits href and sets role=\"link\", aria-disabled=\"true\", tabindex=\"-1\" on disabled links",
        "Forwards ARIA attributes and GlobalAttributes",
    ],
    anatomy: "use monoxus::button::{Button, ButtonType};\n\nrsx! {\n    Button { r#type: ButtonType::Button, \"Click me\" }\n    Button { href: \"/docs\", \"Anchor link\" }\n}",
    source_path: "src/button.rs",
    aria_url: "https://www.w3.org/WAI/ARIA/apg/patterns/button/",
    aria_label: "WAI-ARIA Button Pattern",
    parts: &[PartDoc {
        name: "Button",
        description: "Polymorphic button/anchor root element (also aliased as ButtonRoot).",
        props: &[
            PropDoc {
                name: "id",
                type_simple: "Option<String>",
                type_full: "Option<String>",
                default: None,
                required: false,
                description: "Custom DOM element ID.",
            },
            PropDoc {
                name: "class",
                type_simple: "Option<String>",
                type_full: "Option<String>",
                default: None,
                required: false,
                description: "CSS class names applied to the rendered element.",
            },
            PropDoc {
                name: "style",
                type_simple: "Option<String>",
                type_full: "Option<String>",
                default: None,
                required: false,
                description: "Inline CSS styles applied to the rendered element.",
            },
            PropDoc {
                name: "type",
                type_simple: "ButtonType",
                type_full: "monoxus::button::ButtonType (Button | Submit | Reset)",
                default: Some("ButtonType::Button"),
                required: false,
                description: "Native HTML button type when rendered as <button>.",
            },
            PropDoc {
                name: "form",
                type_simple: "Option<String>",
                type_full: "Option<String>",
                default: None,
                required: false,
                description: "Associates the button with a <form> element by ID.",
            },
            PropDoc {
                name: "href",
                type_simple: "Option<String>",
                type_full: "Option<String>",
                default: None,
                required: false,
                description: "When set, renders an <a> element instead of <button>.",
            },
            PropDoc {
                name: "target",
                type_simple: "Option<String>",
                type_full: "Option<String>",
                default: None,
                required: false,
                description: "Browsing context target when rendered as an anchor (omitted when disabled).",
            },
            PropDoc {
                name: "rel",
                type_simple: "Option<String>",
                type_full: "Option<String>",
                default: None,
                required: false,
                description: "Link relationship attribute when rendered as an anchor (omitted when disabled).",
            },
            PropDoc {
                name: "download",
                type_simple: "Option<String>",
                type_full: "Option<String>",
                default: None,
                required: false,
                description: "Prompts the browser to save the linked URL instead of navigating.",
            },
            PropDoc {
                name: "disabled",
                type_simple: "bool",
                type_full: "bool",
                default: Some("false"),
                required: false,
                description: "Disables interaction, prevents click/keyboard activation, and sets data-disabled.",
            },
            PropDoc {
                name: "tabindex",
                type_simple: "Option<i32>",
                type_full: "Option<i32>",
                default: None,
                required: false,
                description: "Overrides the element's tab order.",
            },
            PropDoc {
                name: "role",
                type_simple: "Option<String>",
                type_full: "Option<String>",
                default: None,
                required: false,
                description: "Overrides the computed ARIA role.",
            },
            PropDoc {
                name: "aria_label",
                type_simple: "Option<String>",
                type_full: "Option<String>",
                default: None,
                required: false,
                description: "Accessible label string for assistive technologies.",
            },
            PropDoc {
                name: "aria_describedby",
                type_simple: "Option<String>",
                type_full: "Option<String>",
                default: None,
                required: false,
                description: "ID reference to an element describing the button.",
            },
            PropDoc {
                name: "aria_expanded",
                type_simple: "Option<bool>",
                type_full: "Option<bool>",
                default: None,
                required: false,
                description: "Indicates whether a controlled element is expanded.",
            },
            PropDoc {
                name: "aria_controls",
                type_simple: "Option<String>",
                type_full: "Option<String>",
                default: None,
                required: false,
                description: "ID of the element controlled by this button.",
            },
            PropDoc {
                name: "aria_haspopup",
                type_simple: "Option<String>",
                type_full: "Option<String>",
                default: None,
                required: false,
                description: "Indicates the availability and type of an interactive popup.",
            },
            PropDoc {
                name: "aria_invalid",
                type_simple: "Option<String>",
                type_full: "Option<String>",
                default: None,
                required: false,
                description: "Indicates validation error state.",
            },
            PropDoc {
                name: "on_click",
                type_simple: "Option<EventHandler<MouseEvent>>",
                type_full: "Option<EventHandler<MouseEvent>>",
                default: None,
                required: false,
                description: "Invoked when clicked while enabled.",
            },
            PropDoc {
                name: "on_key_down",
                type_simple: "Option<EventHandler<KeyboardEvent>>",
                type_full: "Option<EventHandler<KeyboardEvent>>",
                default: None,
                required: false,
                description: "Invoked on keydown while enabled (Enter/Space suppressed when disabled).",
            },
            PropDoc {
                name: "on_mounted",
                type_simple: "Option<EventHandler<MountedEvent>>",
                type_full: "Option<EventHandler<MountedEvent>>",
                default: None,
                required: false,
                description: "Invoked when the underlying DOM element mounts.",
            },
            PropDoc {
                name: "attributes",
                type_simple: "Vec<Attribute>",
                type_full: "Vec<Attribute> (#[props(extends = GlobalAttributes)])",
                default: None,
                required: false,
                description: "Additional global HTML/ARIA attributes forwarded to the element.",
            },
            PropDoc {
                name: "children",
                type_simple: "Element",
                type_full: "dioxus::prelude::Element",
                default: None,
                required: true,
                description: "Button label or child content.",
            },
        ],
        data_attrs: &[
            DataAttrDoc {
                name: "data-button-root",
                values: "\"\"",
                description: "Present on the rendered button or anchor element.",
            },
            DataAttrDoc {
                name: "data-disabled",
                values: "\"\"",
                description: "Present when disabled is true; omitted when enabled.",
            },
        ],
        css_vars: &[],
    }],
    keyboard: &[
        KeyboardDoc {
            key: "Space",
            description: "Activates the button (suppressed when disabled).",
        },
        KeyboardDoc {
            key: "Enter",
            description: "Activates the button or follows the anchor link (suppressed when disabled).",
        },
    ],
};

static DIALOG_DOC: ComponentDoc = ComponentDoc {
    slug: "dialog",
    title: "Dialog",
    description: "Modal and non-modal dialog overlay with focus trapping, scroll lock, escape dismissal, and ARIA relationships.",
    features: &[
        "Supports controlled (open Signal) and uncontrolled (default_open) state",
        "Modal scroll lock, outside aria-hidden/inert shielding, and focus trapping",
        "Restores focus to the trigger when closed",
        "Dismisses on Escape key or outside pointer interaction (configurable)",
        "Automatic aria-labelledby and aria-describedby wiring when Title/Description mount",
    ],
    anatomy: "use monoxus::dialog::{\n    DialogClose, DialogContent, DialogDescription, DialogOverlay,\n    DialogPortal, DialogRoot, DialogTitle, DialogTrigger,\n};\n\nrsx! {\n    DialogRoot {\n        DialogTrigger { \"Open\" }\n        DialogPortal {\n            DialogOverlay {}\n            DialogContent {\n                DialogTitle { \"Title\" }\n                DialogDescription { \"Description\" }\n                DialogClose { \"Close\" }\n            }\n        }\n    }\n}",
    source_path: "src/dialog/components.rs",
    aria_url: "https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/",
    aria_label: "WAI-ARIA Dialog (Modal) Pattern",
    parts: &[
        PartDoc {
            name: "DialogRoot",
            description: "Provides dialog state, lifecycle policies, and context to all child parts.",
            props: &[
                PropDoc { name: "id", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Custom scope identifier." },
                PropDoc { name: "open", type_simple: "Option<Signal<bool>>", type_full: "Option<Signal<bool>>", default: None, required: false, description: "Controlled open state signal." },
                PropDoc { name: "default_open", type_simple: "Option<bool>", type_full: "Option<bool>", default: Some("false"), required: false, description: "Initial open state when uncontrolled." },
                PropDoc { name: "on_open_change", type_simple: "Option<EventHandler<bool>>", type_full: "Option<EventHandler<bool>>", default: None, required: false, description: "Callback invoked when open state changes." },
                PropDoc { name: "mode", type_simple: "Option<DialogMode>", type_full: "Option<DialogMode>", default: None, required: false, description: "Modal or non-modal dialog behavior." },
                PropDoc { name: "open_focus", type_simple: "Option<DialogOpenFocusPolicy>", type_full: "Option<DialogOpenFocusPolicy>", default: None, required: false, description: "Focus policy applied when the dialog opens." },
                PropDoc { name: "close_focus", type_simple: "Option<DialogCloseFocusPolicy>", type_full: "Option<DialogCloseFocusPolicy>", default: None, required: false, description: "Focus restoration policy when the dialog closes." },
                PropDoc { name: "scroll_lock", type_simple: "Option<DialogScrollLockPolicy>", type_full: "Option<DialogScrollLockPolicy>", default: None, required: false, description: "Document scroll lock policy while open." },
                PropDoc { name: "outside_interaction", type_simple: "Option<DialogOutsideInteractionPolicy>", type_full: "Option<DialogOutsideInteractionPolicy>", default: None, required: false, description: "Controls whether outside clicks or focus dismiss the dialog." },
                PropDoc { name: "portal_host", type_simple: "Option<PortalHost>", type_full: "Option<PortalHost>", default: None, required: false, description: "Default portal host for DialogPortal." },
                PropDoc { name: "prevent_body_pointer_events", type_simple: "Option<bool>", type_full: "Option<bool>", default: None, required: false, description: "Blocks pointer events on body while modal dialog is open." },
                PropDoc { name: "children", type_simple: "Element", type_full: "dioxus::prelude::Element", default: None, required: true, description: "Dialog trigger and portal content." },
            ],
            data_attrs: &[DataAttrDoc { name: "data-state", values: "\"open\" | \"closed\"", description: "Current dialog open state." }],
            css_vars: &[],
        },
        PartDoc {
            name: "DialogTrigger",
            description: "Button that opens the dialog.",
            props: &[
                PropDoc { name: "id", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Custom trigger element ID." },
                PropDoc { name: "class", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "CSS class names." },
                PropDoc { name: "style", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Inline CSS styles." },
                PropDoc { name: "disabled", type_simple: "bool", type_full: "bool", default: Some("false"), required: false, description: "Prevents opening the dialog when true." },
                PropDoc { name: "children", type_simple: "Element", type_full: "dioxus::prelude::Element", default: None, required: true, description: "Trigger label." },
            ],
            data_attrs: &[
                DataAttrDoc { name: "data-state", values: "\"open\" | \"closed\"", description: "Current dialog open state." },
                DataAttrDoc { name: "data-disabled", values: "\"\"", description: "Present when disabled is true; omitted when enabled." },
            ],
            css_vars: &[],
        },
        PartDoc {
            name: "DialogPortal",
            description: "Teleports dialog overlay and content to a PortalHost.",
            props: &[
                PropDoc { name: "host", type_simple: "Option<PortalHost>", type_full: "Option<PortalHost>", default: None, required: false, description: "Target portal host override." },
                PropDoc { name: "force_mount", type_simple: "bool", type_full: "bool", default: Some("false"), required: false, description: "Keeps the portal mounted even when closed." },
                PropDoc { name: "children", type_simple: "Element", type_full: "dioxus::prelude::Element", default: None, required: true, description: "Overlay and content elements." },
            ],
            data_attrs: &[DataAttrDoc { name: "data-portal-host", values: "\"inline\" | \"default\" | <id>", description: "Resolved portal host target." }],
            css_vars: &[],
        },
        PartDoc {
            name: "DialogOverlay",
            description: "Backdrop layer rendered behind DialogContent.",
            props: &[
                PropDoc { name: "id", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Custom overlay element ID." },
                PropDoc { name: "class", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "CSS class names." },
                PropDoc { name: "style", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Inline CSS styles." },
                PropDoc { name: "force_mount", type_simple: "bool", type_full: "bool", default: Some("false"), required: false, description: "Keeps the overlay mounted when closed." },
                PropDoc { name: "children", type_simple: "Option<Element>", type_full: "Option<Element>", default: None, required: false, description: "Optional overlay children." },
            ],
            data_attrs: &[
                DataAttrDoc { name: "data-state", values: "\"open\" | \"closed\"", description: "Current dialog open state." },
                DataAttrDoc { name: "data-overlay", values: "\"true\"", description: "Identifies the modal backdrop element." },
            ],
            css_vars: &[],
        },
        PartDoc {
            name: "DialogContent",
            description: "Main dialog container with role=\"dialog\", focus management, and dismiss handling.",
            props: &[
                PropDoc { name: "id", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Custom content element ID." },
                PropDoc { name: "class", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "CSS class names." },
                PropDoc { name: "style", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Inline CSS styles." },
                PropDoc { name: "aria_labelledby", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Overrides automatic DialogTitle ID association." },
                PropDoc { name: "aria_describedby", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Overrides automatic DialogDescription ID association." },
                PropDoc { name: "force_mount", type_simple: "bool", type_full: "bool", default: Some("false"), required: false, description: "Keeps the content mounted when closed." },
                PropDoc { name: "prevent_body_pointer_events", type_simple: "Option<bool>", type_full: "Option<bool>", default: None, required: false, description: "Overrides body pointer-event blocking while open." },
                PropDoc { name: "children", type_simple: "Element", type_full: "dioxus::prelude::Element", default: None, required: true, description: "Dialog body content." },
            ],
            data_attrs: &[DataAttrDoc { name: "data-state", values: "\"open\" | \"closed\"", description: "Current dialog open state." }],
            css_vars: &[],
        },
        PartDoc {
            name: "DialogTitle",
            description: "Accessible heading (<h2>) automatically linked via aria-labelledby.",
            props: &[
                PropDoc { name: "id", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Custom title element ID." },
                PropDoc { name: "class", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "CSS class names." },
                PropDoc { name: "style", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Inline CSS styles." },
                PropDoc { name: "children", type_simple: "Element", type_full: "dioxus::prelude::Element", default: None, required: true, description: "Heading text." },
            ],
            data_attrs: &[],
            css_vars: &[],
        },
        PartDoc {
            name: "DialogDescription",
            description: "Accessible description (<p>) automatically linked via aria-describedby.",
            props: &[
                PropDoc { name: "id", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Custom description element ID." },
                PropDoc { name: "class", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "CSS class names." },
                PropDoc { name: "style", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Inline CSS styles." },
                PropDoc { name: "children", type_simple: "Element", type_full: "dioxus::prelude::Element", default: None, required: true, description: "Description text." },
            ],
            data_attrs: &[],
            css_vars: &[],
        },
        PartDoc {
            name: "DialogClose",
            description: "Button that closes the dialog and restores focus to the trigger.",
            props: &[
                PropDoc { name: "id", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Custom close button ID." },
                PropDoc { name: "class", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "CSS class names." },
                PropDoc { name: "style", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Inline CSS styles." },
                PropDoc { name: "disabled", type_simple: "bool", type_full: "bool", default: Some("false"), required: false, description: "Disables the close button when true." },
                PropDoc { name: "children", type_simple: "Element", type_full: "dioxus::prelude::Element", default: None, required: true, description: "Close button label." },
            ],
            data_attrs: &[
                DataAttrDoc { name: "data-state", values: "\"open\" | \"closed\"", description: "Current dialog open state." },
                DataAttrDoc { name: "data-disabled", values: "\"\"", description: "Present when disabled is true; omitted when enabled." },
            ],
            css_vars: &[],
        },
    ],
    keyboard: &[
        KeyboardDoc { key: "Space / Enter", description: "Opens the dialog when focus is on DialogTrigger, or closes it on DialogClose." },
        KeyboardDoc { key: "Tab / Shift+Tab", description: "Cycles focus within DialogContent when modal." },
        KeyboardDoc { key: "Escape", description: "Closes the dialog and restores focus to DialogTrigger." },
    ],
};

static SELECT_DOC: ComponentDoc = ComponentDoc {
    slug: "select",
    title: "Select",
    description: "Single and multiple selection listbox with floating positioning, typeahead search, keyboard navigation, and form input.",
    features: &[
        "Single and multiple selection modes with controllable value/values",
        "Floating placement with collision avoidance, same_width, and CSS custom properties",
        "Keyboard navigation (ArrowUp/Down, Home/End, PageUp/Down) and character typeahead",
        "Grouped items with SelectGroup and SelectLabel",
        "Hidden form <input> synchronization when name is provided",
    ],
    anatomy: "use monoxus::select::{\n    SelectContent, SelectItem, SelectItemIndicator, SelectItemText,\n    SelectPortal, SelectRoot, SelectTrigger, SelectValue, SelectViewport,\n};\n\nrsx! {\n    SelectRoot { default_value: Some(\"apple\".into()),\n        SelectTrigger {\n            SelectValue { placeholder: \"Choose a fruit\" }\n        }\n        SelectPortal {\n            SelectContent {\n                SelectViewport {\n                    SelectItem { value: \"apple\".to_string(),\n                        SelectItemText { \"Apple\" }\n                        SelectItemIndicator {}\n                    }\n                }\n            }\n        }\n    }\n}",
    source_path: "src/select/mod.rs",
    aria_url: "https://www.w3.org/WAI/ARIA/apg/patterns/listbox/",
    aria_label: "WAI-ARIA Listbox Pattern",
    parts: &[
        PartDoc {
            name: "SelectRoot",
            description: "Root state provider managing selection, open state, and optional hidden form input.",
            props: &[
                PropDoc { name: "runtime", type_simple: "Option<SelectRuntime>", type_full: "Option<SelectRuntime>", default: None, required: false, description: "Optional externally managed SelectRuntime." },
                PropDoc { name: "id", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Custom root element ID." },
                PropDoc { name: "class", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "CSS class names." },
                PropDoc { name: "style", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Inline CSS styles." },
                PropDoc { name: "mode", type_simple: "Option<SelectMode>", type_full: "Option<SelectMode>", default: None, required: false, description: "Selection mode (Single or Multiple)." },
                PropDoc { name: "multiple", type_simple: "bool", type_full: "bool", default: Some("false"), required: false, description: "Enables multiple selection when mode is not specified." },
                PropDoc { name: "value", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Controlled single selected value." },
                PropDoc { name: "default_value", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Initial single selected value." },
                PropDoc { name: "values", type_simple: "Option<Vec<String>>", type_full: "Option<Vec<String>>", default: None, required: false, description: "Controlled multiple selected values." },
                PropDoc { name: "default_values", type_simple: "Option<Vec<String>>", type_full: "Option<Vec<String>>", default: None, required: false, description: "Initial multiple selected values." },
                PropDoc { name: "open", type_simple: "bool", type_full: "bool", default: Some("false"), required: false, description: "Initial open state." },
                PropDoc { name: "on_value_change", type_simple: "Option<EventHandler<Option<String>>>", type_full: "Option<EventHandler<Option<String>>>", default: None, required: false, description: "Called when the single selected value changes." },
                PropDoc { name: "on_values_change", type_simple: "Option<EventHandler<Vec<String>>>", type_full: "Option<EventHandler<Vec<String>>>", default: None, required: false, description: "Called when multiple selected values change." },
                PropDoc { name: "on_open_change", type_simple: "Option<EventHandler<bool>>", type_full: "Option<EventHandler<bool>>", default: None, required: false, description: "Called when the dropdown opens or closes." },
                PropDoc { name: "on_open_change_complete", type_simple: "Option<EventHandler<bool>>", type_full: "Option<EventHandler<bool>>", default: None, required: false, description: "Called after open/close presence transition finishes." },
                PropDoc { name: "allow_deselect", type_simple: "bool", type_full: "bool", default: Some("false"), required: false, description: "Allows clicking a selected item in single mode to clear selection." },
                PropDoc { name: "disabled", type_simple: "bool", type_full: "bool", default: Some("false"), required: false, description: "Disables the entire select." },
                PropDoc { name: "required", type_simple: "bool", type_full: "bool", default: Some("false"), required: false, description: "Marks the hidden form input as required." },
                PropDoc { name: "name", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Form field name; renders SelectHiddenInput when set." },
                PropDoc { name: "form", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Associates the hidden input with a form ID." },
                PropDoc { name: "autocomplete", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Autocomplete hint forwarded to the hidden input." },
                PropDoc { name: "items", type_simple: "Option<Vec<SelectItemData>>", type_full: "Option<Vec<SelectItemData>>", default: None, required: false, description: "Optional pre-registered item metadata for closed-state label lookup." },
                PropDoc { name: "loop_selection", type_simple: "bool", type_full: "bool", default: Some("false"), required: false, description: "Wraps keyboard highlight navigation from last to first item." },
                PropDoc { name: "dir", type_simple: "Option<Direction>", type_full: "Option<monoxus::foundation::shared::Direction>", default: None, required: false, description: "Reading direction override (Ltr or Rtl)." },
                PropDoc { name: "children", type_simple: "Element", type_full: "dioxus::prelude::Element", default: None, required: true, description: "SelectTrigger and SelectPortal children." },
            ],
            data_attrs: &[
                DataAttrDoc { name: "data-state", values: "\"open\" | \"closed\"", description: "Current dropdown open state." },
                DataAttrDoc { name: "data-disabled", values: "\"\"", description: "Present when disabled is true; omitted when enabled." },
            ],
            css_vars: &[],
        },
        PartDoc {
            name: "SelectTrigger",
            description: "Combobox trigger button that toggles the listbox popup.",
            props: &[
                PropDoc { name: "class", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "CSS class names." },
                PropDoc { name: "style", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Inline CSS styles." },
                PropDoc { name: "children", type_simple: "Element", type_full: "dioxus::prelude::Element", default: None, required: true, description: "Typically SelectValue and SelectIcon." },
            ],
            data_attrs: &[
                DataAttrDoc { name: "data-state", values: "\"open\" | \"closed\"", description: "Current dropdown open state." },
                DataAttrDoc { name: "data-disabled", values: "\"\"", description: "Present when disabled." },
                DataAttrDoc { name: "data-placeholder", values: "\"true\" | \"false\"", description: "Whether no value is currently selected." },
            ],
            css_vars: &[],
        },
        PartDoc {
            name: "SelectValue",
            description: "Displays the selected item label(s) or a placeholder.",
            props: &[
                PropDoc { name: "placeholder", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Text displayed when no value is selected." },
                PropDoc { name: "class", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "CSS class names." },
                PropDoc { name: "style", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Inline CSS styles." },
                PropDoc { name: "children", type_simple: "Option<Element>", type_full: "Option<Element>", default: None, required: false, description: "Optional custom value display." },
            ],
            data_attrs: &[DataAttrDoc { name: "data-placeholder", values: "\"true\" | \"false\"", description: "True when showing placeholder text." }],
            css_vars: &[],
        },
        PartDoc {
            name: "SelectIcon",
            description: "Decorative indicator icon inside SelectTrigger.",
            props: &[
                PropDoc { name: "class", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "CSS class names." },
                PropDoc { name: "style", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Inline CSS styles." },
                PropDoc { name: "children", type_simple: "Option<Element>", type_full: "Option<Element>", default: None, required: false, description: "Custom icon element; defaults to \"▼\"." },
            ],
            data_attrs: &[],
            css_vars: &[],
        },
        PartDoc {
            name: "SelectPortal",
            description: "Portals SelectContent to a PortalHost.",
            props: &[
                PropDoc { name: "host", type_simple: "Option<PortalHost>", type_full: "Option<PortalHost>", default: None, required: false, description: "Target portal host override." },
                PropDoc { name: "disabled", type_simple: "bool", type_full: "bool", default: Some("false"), required: false, description: "Forces inline rendering without teleporting." },
                PropDoc { name: "force_mount", type_simple: "bool", type_full: "bool", default: Some("false"), required: false, description: "Keeps the portal mounted when closed." },
                PropDoc { name: "children", type_simple: "Element", type_full: "dioxus::prelude::Element", default: None, required: true, description: "SelectContent element." },
            ],
            data_attrs: &[
                DataAttrDoc { name: "data-portal-host", values: "\"inline\" | \"default\" | <id>", description: "Resolved portal host." },
                DataAttrDoc { name: "data-portal-disabled", values: "\"true\" | \"false\"", description: "Whether teleportation is disabled." },
                DataAttrDoc { name: "data-portal-teleported", values: "\"true\" | \"false\"", description: "Whether content was teleported." },
            ],
            css_vars: &[],
        },
        PartDoc {
            name: "SelectContent",
            description: "Floating listbox container with collision-aware placement, typeahead, and dismiss handling.",
            props: &[
                PropDoc { name: "class", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "CSS class names." },
                PropDoc { name: "style", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Inline CSS styles." },
                PropDoc { name: "side", type_simple: "PlacementSide", type_full: "PlacementSide", default: Some("PlacementSide::Bottom"), required: false, description: "Preferred side relative to trigger." },
                PropDoc { name: "side_offset", type_simple: "f32", type_full: "f32", default: Some("4.0"), required: false, description: "Distance in pixels from the trigger." },
                PropDoc { name: "align", type_simple: "PlacementAlign", type_full: "PlacementAlign", default: Some("PlacementAlign::Start"), required: false, description: "Preferred alignment along the trigger axis." },
                PropDoc { name: "align_offset", type_simple: "f32", type_full: "f32", default: Some("0.0"), required: false, description: "Offset in pixels along the alignment axis." },
                PropDoc { name: "avoid_collisions", type_simple: "bool", type_full: "bool", default: Some("true"), required: false, description: "Flips/shifts placement to stay within viewport bounds." },
                PropDoc { name: "collision_boundary", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Optional boundary element ID for collision detection." },
                PropDoc { name: "collision_padding", type_simple: "f32", type_full: "f32", default: Some("0.0"), required: false, description: "Minimum padding from collision boundary edges." },
                PropDoc { name: "arrow_padding", type_simple: "f32", type_full: "f32", default: Some("0.0"), required: false, description: "Minimum margin between arrow and content edges." },
                PropDoc { name: "sticky", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Sticky alignment strategy." },
                PropDoc { name: "hide_when_detached", type_simple: "bool", type_full: "bool", default: Some("false"), required: false, description: "Hides content when the trigger is scrolled out of view." },
                PropDoc { name: "same_width", type_simple: "bool", type_full: "bool", default: Some("false"), required: false, description: "Matches content width to the trigger width." },
                PropDoc { name: "custom_anchor", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Custom anchor element ID." },
                PropDoc { name: "prevent_scroll", type_simple: "bool", type_full: "bool", default: Some("false"), required: false, description: "Locks document scroll while open." },
                PropDoc { name: "prevent_overflow_text_selection", type_simple: "bool", type_full: "bool", default: Some("false"), required: false, description: "Disables body text selection while open." },
                PropDoc { name: "force_mount", type_simple: "bool", type_full: "bool", default: Some("false"), required: false, description: "Keeps content mounted when closed." },
                PropDoc { name: "on_pointer_down_outside", type_simple: "Option<EventHandler<PointerDownOutsideEvent>>", type_full: "Option<EventHandler<PointerDownOutsideEvent>>", default: None, required: false, description: "Callback fired on outside pointer down." },
                PropDoc { name: "on_escape_keydown", type_simple: "Option<EventHandler<KeyboardEvent>>", type_full: "Option<EventHandler<KeyboardEvent>>", default: None, required: false, description: "Callback fired when Escape is pressed." },
                PropDoc { name: "on_close_auto_focus", type_simple: "Option<EventHandler<()>>", type_full: "Option<EventHandler<()>>", default: None, required: false, description: "Callback fired before restoring focus on close." },
                PropDoc { name: "children", type_simple: "Element", type_full: "dioxus::prelude::Element", default: None, required: true, description: "SelectViewport and SelectItem elements." },
            ],
            data_attrs: &[
                DataAttrDoc { name: "data-state", values: "\"open\" | \"closed\"", description: "Current dropdown open state." },
                DataAttrDoc { name: "data-side", values: "\"top\" | \"right\" | \"bottom\" | \"left\"", description: "Resolved placement side." },
                DataAttrDoc { name: "data-align", values: "\"start\" | \"center\" | \"end\"", description: "Resolved alignment." },
                DataAttrDoc { name: "data-positioning-state", values: "\"measuring\" | \"ready\"", description: "Floating measurement readiness." },
            ],
            css_vars: &[
                CssVarDoc { name: "--monoxus-select-floating-x", description: "Resolved X coordinate in pixels." },
                CssVarDoc { name: "--monoxus-select-floating-y", description: "Resolved Y coordinate in pixels." },
                CssVarDoc { name: "--monoxus-select-available-width", description: "Available horizontal space in pixels." },
                CssVarDoc { name: "--monoxus-select-available-height", description: "Available vertical space in pixels." },
                CssVarDoc { name: "--monoxus-select-anchor-width", description: "Trigger width in pixels." },
                CssVarDoc { name: "--monoxus-select-anchor-height", description: "Trigger height in pixels." },
                CssVarDoc { name: "--radix-select-content-transform-origin", description: "Radix-compatible transform-origin value." },
                CssVarDoc { name: "--radix-select-content-available-width", description: "Radix-compatible available width." },
                CssVarDoc { name: "--radix-select-content-available-height", description: "Radix-compatible available height." },
                CssVarDoc { name: "--radix-select-trigger-width", description: "Radix-compatible trigger width." },
                CssVarDoc { name: "--radix-select-trigger-height", description: "Radix-compatible trigger height." },
                CssVarDoc { name: "--bits-select-arrow-padding", description: "Configured arrow padding in pixels." },
            ],
        },
        PartDoc {
            name: "SelectViewport",
            description: "Scrollable container for SelectItem and SelectGroup children.",
            props: &[
                PropDoc { name: "id", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Custom viewport element ID." },
                PropDoc { name: "class", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "CSS class names." },
                PropDoc { name: "style", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Inline CSS styles." },
                PropDoc { name: "children", type_simple: "Element", type_full: "dioxus::prelude::Element", default: None, required: true, description: "Items and groups." },
            ],
            data_attrs: &[],
            css_vars: &[],
        },
        PartDoc {
            name: "SelectGroup",
            description: "Groups related options with role=\"group\" and aria-labelledby.",
            props: &[
                PropDoc { name: "group_key", type_simple: "String", type_full: "String", default: None, required: true, description: "Unique key linking SelectGroup and SelectLabel." },
                PropDoc { name: "class", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "CSS class names." },
                PropDoc { name: "style", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Inline CSS styles." },
                PropDoc { name: "children", type_simple: "Element", type_full: "dioxus::prelude::Element", default: None, required: true, description: "SelectLabel and SelectItem elements." },
            ],
            data_attrs: &[],
            css_vars: &[],
        },
        PartDoc {
            name: "SelectLabel",
            description: "Non-selectable label for a SelectGroup.",
            props: &[
                PropDoc { name: "group_key", type_simple: "String", type_full: "String", default: None, required: true, description: "Matching group_key of the parent SelectGroup." },
                PropDoc { name: "class", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "CSS class names." },
                PropDoc { name: "style", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Inline CSS styles." },
                PropDoc { name: "children", type_simple: "Element", type_full: "dioxus::prelude::Element", default: None, required: true, description: "Group heading text." },
            ],
            data_attrs: &[],
            css_vars: &[],
        },
        PartDoc {
            name: "SelectItem",
            description: "Selectable option item with highlight and typeahead registration.",
            props: &[
                PropDoc { name: "value", type_simple: "String", type_full: "String", default: None, required: true, description: "Unique option value." },
                PropDoc { name: "text", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Explicit label text for SelectValue and typeahead." },
                PropDoc { name: "disabled", type_simple: "bool", type_full: "bool", default: Some("false"), required: false, description: "Prevents selection and skips during keyboard navigation." },
                PropDoc { name: "class", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "CSS class names." },
                PropDoc { name: "style", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Inline CSS styles." },
                PropDoc { name: "on_highlight", type_simple: "Option<EventHandler<()>>", type_full: "Option<EventHandler<()>>", default: None, required: false, description: "Called when the item becomes highlighted." },
                PropDoc { name: "on_unhighlight", type_simple: "Option<EventHandler<()>>", type_full: "Option<EventHandler<()>>", default: None, required: false, description: "Called when the item loses highlight." },
                PropDoc { name: "children", type_simple: "Element", type_full: "dioxus::prelude::Element", default: None, required: true, description: "SelectItemText and SelectItemIndicator." },
            ],
            data_attrs: &[
                DataAttrDoc { name: "data-state", values: "\"checked\" | \"unchecked\"", description: "Selection state of this item." },
                DataAttrDoc { name: "data-highlighted", values: "\"true\" | \"false\"", description: "Whether this item is currently highlighted." },
                DataAttrDoc { name: "data-disabled", values: "\"\"", description: "Present when disabled is true; omitted when enabled." },
                DataAttrDoc { name: "data-value", values: "<value>", description: "The item's value string." },
                DataAttrDoc { name: "data-label", values: "<label>", description: "The resolved item label." },
                DataAttrDoc { name: "data-selected", values: "\"true\" | \"false\"", description: "Boolean selection indicator." },
            ],
            css_vars: &[],
        },
        PartDoc {
            name: "SelectItemText",
            description: "Text span for a SelectItem.",
            props: &[
                PropDoc { name: "class", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "CSS class names." },
                PropDoc { name: "style", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Inline CSS styles." },
                PropDoc { name: "children", type_simple: "Element", type_full: "dioxus::prelude::Element", default: None, required: true, description: "Option text." },
            ],
            data_attrs: &[],
            css_vars: &[],
        },
        PartDoc {
            name: "SelectItemIndicator",
            description: "Visual checkmark indicator for a selected item.",
            props: &[
                PropDoc { name: "class", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "CSS class names." },
                PropDoc { name: "style", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Inline CSS styles." },
                PropDoc { name: "children", type_simple: "Option<Element>", type_full: "Option<Element>", default: None, required: false, description: "Custom indicator; defaults to \"✓\"." },
            ],
            data_attrs: &[],
            css_vars: &[],
        },
        PartDoc {
            name: "SelectSeparator",
            description: "Visual divider between option groups.",
            props: &[
                PropDoc { name: "class", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "CSS class names." },
                PropDoc { name: "style", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Inline CSS styles." },
            ],
            data_attrs: &[],
            css_vars: &[],
        },
        PartDoc {
            name: "SelectArrow",
            description: "Positioned arrow pointing toward SelectTrigger.",
            props: &[
                PropDoc { name: "class", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "CSS class names." },
                PropDoc { name: "style", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Inline CSS styles." },
                PropDoc { name: "width", type_simple: "f32", type_full: "f32", default: Some("10.0"), required: false, description: "SVG arrow width in pixels." },
                PropDoc { name: "height", type_simple: "f32", type_full: "f32", default: Some("5.0"), required: false, description: "SVG arrow height in pixels." },
                PropDoc { name: "children", type_simple: "Option<Element>", type_full: "Option<Element>", default: None, required: false, description: "Optional custom arrow element." },
            ],
            data_attrs: &[],
            css_vars: &[],
        },
    ],
    keyboard: &[
        KeyboardDoc { key: "Space / Enter / ArrowDown", description: "Opens the listbox when focus is on SelectTrigger." },
        KeyboardDoc { key: "ArrowDown / ArrowUp", description: "Moves highlight to the next or previous enabled option." },
        KeyboardDoc { key: "Home / End", description: "Jumps highlight to the first or last enabled option." },
        KeyboardDoc { key: "Enter / Space", description: "Selects the currently highlighted option." },
        KeyboardDoc { key: "Escape", description: "Closes the listbox and restores focus to SelectTrigger." },
        KeyboardDoc { key: "A-Z / 0-9", description: "Typeahead search jumps to the matching option." },
    ],
};

static TABS_DOC: ComponentDoc = ComponentDoc {
    slug: "tabs",
    title: "Tabs",
    description: "Accessible tabbed interface with roving focus, horizontal/vertical orientation, RTL support, and automatic or manual activation.",
    features: &[
        "Controlled (value Signal) and uncontrolled (default_value) active tab state",
        "Horizontal and vertical orientation with RTL-aware arrow-key navigation",
        "Automatic (activate on focus) or Manual (activate on Enter/Space/click) modes",
        "Optional focus wrapping via loop_focus on TabsList",
        "Automatic aria-controls and aria-labelledby wiring across triggers and panels",
    ],
    anatomy: "use monoxus::tabs::{TabsContent, TabsList, TabsRoot, TabsTrigger};\n\nrsx! {\n    TabsRoot { default_value: Some(\"account\".into()),\n        TabsList {\n            TabsTrigger { value: \"account\".to_string(), \"Account\" }\n            TabsTrigger { value: \"password\".to_string(), \"Password\" }\n        }\n        TabsContent { value: \"account\".to_string(), \"Account settings\" }\n        TabsContent { value: \"password\".to_string(), \"Password settings\" }\n    }\n}",
    source_path: "src/tabs/components.rs",
    aria_url: "https://www.w3.org/WAI/ARIA/apg/patterns/tabs/",
    aria_label: "WAI-ARIA Tabs Pattern",
    parts: &[
        PartDoc {
            name: "TabsRoot",
            description: "Root container managing active tab value, orientation, direction, and activation mode.",
            props: &[
                PropDoc { name: "id", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Custom root element ID." },
                PropDoc { name: "class", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "CSS class names." },
                PropDoc { name: "style", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Inline CSS styles." },
                PropDoc { name: "value", type_simple: "Option<Signal<String>>", type_full: "Option<Signal<String>>", default: None, required: false, description: "Controlled active tab value signal." },
                PropDoc { name: "default_value", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Initial active tab value when uncontrolled." },
                PropDoc { name: "on_value_change", type_simple: "Option<EventHandler<String>>", type_full: "Option<EventHandler<String>>", default: None, required: false, description: "Called when the active tab changes." },
                PropDoc { name: "orientation", type_simple: "Option<TabsOrientation>", type_full: "Option<TabsOrientation>", default: Some("Horizontal"), required: false, description: "Layout and keyboard navigation orientation." },
                PropDoc { name: "dir", type_simple: "Option<TabsDirection>", type_full: "Option<TabsDirection>", default: None, required: false, description: "Reading direction override (Ltr or Rtl)." },
                PropDoc { name: "activation_mode", type_simple: "Option<TabsActivationMode>", type_full: "Option<TabsActivationMode>", default: Some("Automatic"), required: false, description: "Automatic activates on focus; Manual requires Enter/Space/click." },
                PropDoc { name: "children", type_simple: "Element", type_full: "dioxus::prelude::Element", default: None, required: true, description: "TabsList and TabsContent children." },
            ],
            data_attrs: &[DataAttrDoc { name: "data-orientation", values: "\"horizontal\" | \"vertical\"", description: "Current tabs orientation." }],
            css_vars: &[],
        },
        PartDoc {
            name: "TabsList",
            description: "Tablist container (role=\"tablist\") coordinating roving focus across TabsTrigger children.",
            props: &[
                PropDoc { name: "id", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Custom tablist element ID." },
                PropDoc { name: "class", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "CSS class names." },
                PropDoc { name: "style", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Inline CSS styles." },
                PropDoc { name: "loop_focus", type_simple: "bool", type_full: "bool", default: Some("true"), required: false, description: "Wraps keyboard focus from last trigger to first." },
                PropDoc { name: "children", type_simple: "Element", type_full: "dioxus::prelude::Element", default: None, required: true, description: "TabsTrigger elements." },
            ],
            data_attrs: &[
                DataAttrDoc { name: "data-orientation", values: "\"horizontal\" | \"vertical\"", description: "Current tabs orientation." },
                DataAttrDoc { name: "data-disabled", values: "\"\"", description: "Present when tabs root is disabled." },
            ],
            css_vars: &[],
        },
        PartDoc {
            name: "TabsTrigger",
            description: "Tab button (role=\"tab\") that activates its matching TabsContent.",
            props: &[
                PropDoc { name: "value", type_simple: "String", type_full: "String", default: None, required: true, description: "Unique value matching a TabsContent." },
                PropDoc { name: "disabled", type_simple: "bool", type_full: "bool", default: Some("false"), required: false, description: "Disables this tab and skips it in roving focus." },
                PropDoc { name: "id", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Custom trigger element ID." },
                PropDoc { name: "class", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "CSS class names." },
                PropDoc { name: "style", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Inline CSS styles." },
                PropDoc { name: "children", type_simple: "Element", type_full: "dioxus::prelude::Element", default: None, required: true, description: "Tab button label." },
            ],
            data_attrs: &[
                DataAttrDoc { name: "data-state", values: "\"active\" | \"inactive\"", description: "Whether this tab is currently active." },
                DataAttrDoc { name: "data-orientation", values: "\"horizontal\" | \"vertical\"", description: "Current tabs orientation." },
                DataAttrDoc { name: "data-disabled", values: "\"\"", description: "Present when disabled is true; omitted when enabled." },
            ],
            css_vars: &[],
        },
        PartDoc {
            name: "TabsContent",
            description: "Tab panel (role=\"tabpanel\") rendered when its matching tab is active.",
            props: &[
                PropDoc { name: "value", type_simple: "String", type_full: "String", default: None, required: true, description: "Matching value of the controlling TabsTrigger." },
                PropDoc { name: "force_mount", type_simple: "bool", type_full: "bool", default: Some("false"), required: false, description: "Keeps the panel mounted in the DOM when inactive." },
                PropDoc { name: "id", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Custom panel element ID." },
                PropDoc { name: "class", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "CSS class names." },
                PropDoc { name: "style", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Inline CSS styles." },
                PropDoc { name: "tabindex", type_simple: "Option<i32>", type_full: "Option<i32>", default: Some("0"), required: false, description: "Tabindex applied to the tabpanel." },
                PropDoc { name: "children", type_simple: "Element", type_full: "dioxus::prelude::Element", default: None, required: true, description: "Panel content." },
            ],
            data_attrs: &[
                DataAttrDoc { name: "data-state", values: "\"active\" | \"inactive\"", description: "Whether this panel is currently active." },
                DataAttrDoc { name: "data-orientation", values: "\"horizontal\" | \"vertical\"", description: "Current tabs orientation." },
            ],
            css_vars: &[],
        },
    ],
    keyboard: &[
        KeyboardDoc { key: "Tab", description: "Moves focus into the active TabsTrigger or into the active TabsContent." },
        KeyboardDoc { key: "ArrowRight / ArrowLeft", description: "Moves focus between horizontal tabs (RTL-aware)." },
        KeyboardDoc { key: "ArrowDown / ArrowUp", description: "Moves focus between vertical tabs." },
        KeyboardDoc { key: "Home / End", description: "Moves focus to the first or last enabled tab." },
        KeyboardDoc { key: "Space / Enter", description: "Activates the focused tab (required in Manual activation mode)." },
    ],
};

static TOOLTIP_DOC: ComponentDoc = ComponentDoc {
    slug: "tooltip",
    title: "Tooltip",
    description: "Hover and focus-triggered floating tooltip with shared provider delay timers, collision avoidance, and arrow positioning.",
    features: &[
        "Shared TooltipProvider coordinates open/skip delay timers across multiple tooltips",
        "Opens on pointer hover and keyboard focus-visible; closes on Escape or pointer leave",
        "Floating placement with side, align, collision avoidance, and TooltipArrow",
        "Optional aria_label override for screen-reader-only tooltip text",
        "Exposes --monoxus-tooltip-* and --radix-tooltip-* CSS custom properties",
    ],
    anatomy: "use monoxus::tooltip::{\n    TooltipArrow, TooltipContent, TooltipPortal, TooltipProvider,\n    TooltipRoot, TooltipTrigger,\n};\n\nrsx! {\n    TooltipProvider {\n        TooltipRoot {\n            TooltipTrigger { \"Hover me\" }\n            TooltipPortal {\n                TooltipContent {\n                    TooltipArrow {}\n                    \"Tooltip text\"\n                }\n            }\n        }\n    }\n}",
    source_path: "src/tooltip/mod.rs",
    aria_url: "https://www.w3.org/WAI/ARIA/apg/patterns/tooltip/",
    aria_label: "WAI-ARIA Tooltip Pattern",
    parts: &[
        PartDoc {
            name: "TooltipProvider",
            description: "Coordinates shared open delay and skip-delay timers across descendant tooltips.",
            props: &[
                PropDoc { name: "id", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Custom provider scope ID." },
                PropDoc { name: "delay_duration", type_simple: "u64", type_full: "u64", default: Some("700"), required: false, description: "Hover delay in milliseconds before opening." },
                PropDoc { name: "skip_delay_duration", type_simple: "u64", type_full: "u64", default: Some("300"), required: false, description: "Grace window in milliseconds to skip delay when moving between tooltips." },
                PropDoc { name: "disable_hoverable_content", type_simple: "bool", type_full: "bool", default: Some("false"), required: false, description: "Closes tooltip immediately when pointer leaves trigger." },
                PropDoc { name: "close_on_trigger_click", type_simple: "bool", type_full: "bool", default: Some("true"), required: false, description: "Closes tooltip when the trigger is clicked." },
                PropDoc { name: "ignore_non_keyboard_focus", type_simple: "bool", type_full: "bool", default: Some("true"), required: false, description: "Prevents opening on pointer-triggered focus." },
                PropDoc { name: "children", type_simple: "Element", type_full: "dioxus::prelude::Element", default: None, required: true, description: "Application subtree containing TooltipRoot instances." },
            ],
            data_attrs: &[],
            css_vars: &[],
        },
        PartDoc {
            name: "TooltipRoot",
            description: "Root state and placement configuration for a single tooltip.",
            props: &[
                PropDoc { name: "id", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Custom root element ID." },
                PropDoc { name: "class", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "CSS class names." },
                PropDoc { name: "style", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Inline CSS styles." },
                PropDoc { name: "open", type_simple: "Option<Signal<bool>>", type_full: "Option<Signal<bool>>", default: None, required: false, description: "Controlled open state signal." },
                PropDoc { name: "default_open", type_simple: "Option<bool>", type_full: "Option<bool>", default: Some("false"), required: false, description: "Initial open state when uncontrolled." },
                PropDoc { name: "on_open_change", type_simple: "Option<EventHandler<bool>>", type_full: "Option<EventHandler<bool>>", default: None, required: false, description: "Called when open state changes." },
                PropDoc { name: "delay_duration", type_simple: "Option<u64>", type_full: "Option<u64>", default: None, required: false, description: "Overrides TooltipProvider delay_duration." },
                PropDoc { name: "skip_delay_duration", type_simple: "Option<u64>", type_full: "Option<u64>", default: None, required: false, description: "Overrides TooltipProvider skip_delay_duration." },
                PropDoc { name: "disable_hoverable_content", type_simple: "Option<bool>", type_full: "Option<bool>", default: None, required: false, description: "Overrides TooltipProvider disable_hoverable_content." },
                PropDoc { name: "close_on_trigger_click", type_simple: "Option<bool>", type_full: "Option<bool>", default: None, required: false, description: "Overrides TooltipProvider close_on_trigger_click." },
                PropDoc { name: "ignore_non_keyboard_focus", type_simple: "Option<bool>", type_full: "Option<bool>", default: None, required: false, description: "Overrides TooltipProvider ignore_non_keyboard_focus." },
                PropDoc { name: "portal_host", type_simple: "Option<PortalHost>", type_full: "Option<PortalHost>", default: None, required: false, description: "Default portal host for TooltipPortal." },
                PropDoc { name: "side", type_simple: "Option<PlacementSide>", type_full: "Option<PlacementSide>", default: Some("PlacementSide::Top"), required: false, description: "Preferred placement side." },
                PropDoc { name: "side_offset", type_simple: "Option<f32>", type_full: "Option<f32>", default: Some("8.0"), required: false, description: "Distance in pixels from the trigger." },
                PropDoc { name: "align", type_simple: "Option<PlacementAlign>", type_full: "Option<PlacementAlign>", default: None, required: false, description: "Preferred alignment along the trigger axis." },
                PropDoc { name: "align_offset", type_simple: "Option<f32>", type_full: "Option<f32>", default: None, required: false, description: "Offset in pixels along the alignment axis." },
                PropDoc { name: "children", type_simple: "Element", type_full: "dioxus::prelude::Element", default: None, required: true, description: "TooltipTrigger and TooltipPortal children." },
            ],
            data_attrs: &[DataAttrDoc { name: "data-state", values: "\"open\" | \"closed\"", description: "Current tooltip open state." }],
            css_vars: &[],
        },
        PartDoc {
            name: "TooltipTrigger",
            description: "Interactive trigger button wired with aria-describedby when open.",
            props: &[
                PropDoc { name: "id", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Custom trigger element ID." },
                PropDoc { name: "class", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "CSS class names." },
                PropDoc { name: "style", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Inline CSS styles." },
                PropDoc { name: "disabled", type_simple: "bool", type_full: "bool", default: Some("false"), required: false, description: "Disables the trigger button." },
                PropDoc { name: "children", type_simple: "Element", type_full: "dioxus::prelude::Element", default: None, required: true, description: "Trigger content." },
            ],
            data_attrs: &[
                DataAttrDoc { name: "data-state", values: "\"open\" | \"closed\"", description: "Current tooltip open state." },
                DataAttrDoc { name: "data-disabled", values: "\"\"", description: "Present when disabled is true; omitted when enabled." },
            ],
            css_vars: &[],
        },
        PartDoc {
            name: "TooltipPortal",
            description: "Teleports TooltipContent to a PortalHost.",
            props: &[
                PropDoc { name: "host", type_simple: "Option<PortalHost>", type_full: "Option<PortalHost>", default: None, required: false, description: "Target portal host override." },
                PropDoc { name: "force_mount", type_simple: "bool", type_full: "bool", default: Some("false"), required: false, description: "Keeps the portal mounted when closed." },
                PropDoc { name: "children", type_simple: "Element", type_full: "dioxus::prelude::Element", default: None, required: true, description: "TooltipContent element." },
            ],
            data_attrs: &[DataAttrDoc { name: "data-portal-host", values: "\"inline\" | \"default\" | <id>", description: "Resolved portal host." }],
            css_vars: &[],
        },
        PartDoc {
            name: "TooltipContent",
            description: "Floating tooltip container with collision detection and CSS custom properties.",
            props: &[
                PropDoc { name: "id", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Custom content element ID." },
                PropDoc { name: "class", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "CSS class names." },
                PropDoc { name: "style", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Inline CSS styles." },
                PropDoc { name: "force_mount", type_simple: "bool", type_full: "bool", default: Some("false"), required: false, description: "Keeps content mounted when closed." },
                PropDoc { name: "collision_boundary", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Optional boundary element ID for collision detection." },
                PropDoc { name: "collision_padding", type_simple: "Option<f32>", type_full: "Option<f32>", default: None, required: false, description: "Minimum padding from collision boundary edges." },
                PropDoc { name: "aria_label", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Alternative accessible tooltip text rendered in a visually hidden span." },
                PropDoc { name: "children", type_simple: "Element", type_full: "dioxus::prelude::Element", default: None, required: true, description: "Tooltip text and optional TooltipArrow." },
            ],
            data_attrs: &[
                DataAttrDoc { name: "data-state", values: "\"open\" | \"closed\"", description: "Current tooltip open state." },
                DataAttrDoc { name: "data-side", values: "\"top\" | \"right\" | \"bottom\" | \"left\"", description: "Resolved placement side." },
                DataAttrDoc { name: "data-align", values: "\"start\" | \"center\" | \"end\"", description: "Resolved alignment." },
            ],
            css_vars: &[
                CssVarDoc { name: "--monoxus-tooltip-floating-x", description: "Resolved X coordinate in pixels." },
                CssVarDoc { name: "--monoxus-tooltip-floating-y", description: "Resolved Y coordinate in pixels." },
                CssVarDoc { name: "--monoxus-tooltip-transform-origin-x", description: "Transform origin X in pixels." },
                CssVarDoc { name: "--monoxus-tooltip-transform-origin-y", description: "Transform origin Y in pixels." },
                CssVarDoc { name: "--monoxus-tooltip-available-width", description: "Available horizontal space in pixels." },
                CssVarDoc { name: "--monoxus-tooltip-available-height", description: "Available vertical space in pixels." },
                CssVarDoc { name: "--monoxus-tooltip-anchor-width", description: "Trigger width in pixels." },
                CssVarDoc { name: "--monoxus-tooltip-anchor-height", description: "Trigger height in pixels." },
                CssVarDoc { name: "--monoxus-tooltip-content-width", description: "Content width in pixels." },
                CssVarDoc { name: "--monoxus-tooltip-content-height", description: "Content height in pixels." },
                CssVarDoc { name: "--radix-tooltip-content-transform-origin", description: "Radix-compatible transform-origin value." },
                CssVarDoc { name: "--radix-tooltip-content-available-width", description: "Radix-compatible available width." },
                CssVarDoc { name: "--radix-tooltip-content-available-height", description: "Radix-compatible available height." },
                CssVarDoc { name: "--radix-tooltip-trigger-width", description: "Radix-compatible trigger width." },
                CssVarDoc { name: "--radix-tooltip-trigger-height", description: "Radix-compatible trigger height." },
            ],
        },
        PartDoc {
            name: "TooltipArrow",
            description: "Positioned arrow pointing toward TooltipTrigger.",
            props: &[
                PropDoc { name: "id", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Custom arrow element ID." },
                PropDoc { name: "class", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "CSS class names." },
                PropDoc { name: "style", type_simple: "Option<String>", type_full: "Option<String>", default: None, required: false, description: "Inline CSS styles." },
                PropDoc { name: "width", type_simple: "f32", type_full: "f32", default: Some("10.0"), required: false, description: "SVG arrow width in pixels." },
                PropDoc { name: "height", type_simple: "f32", type_full: "f32", default: Some("5.0"), required: false, description: "SVG arrow height in pixels." },
                PropDoc { name: "children", type_simple: "Option<Element>", type_full: "Option<Element>", default: None, required: false, description: "Optional custom arrow element." },
            ],
            data_attrs: &[
                DataAttrDoc { name: "data-state", values: "\"open\" | \"closed\"", description: "Current tooltip open state." },
                DataAttrDoc { name: "data-side", values: "\"top\" | \"right\" | \"bottom\" | \"left\"", description: "Resolved placement side." },
                DataAttrDoc { name: "data-align", values: "\"start\" | \"center\" | \"end\"", description: "Resolved alignment." },
            ],
            css_vars: &[],
        },
    ],
    keyboard: &[
        KeyboardDoc { key: "Tab / Shift+Tab", description: "Opens the tooltip when keyboard focus lands on TooltipTrigger." },
        KeyboardDoc { key: "Escape", description: "Closes the tooltip immediately." },
    ],
};

static ALL_COMPONENT_DOCS: &[ComponentDoc] = &[
    BUTTON_DOC,
    DIALOG_DOC,
    SELECT_DOC,
    TABS_DOC,
    TOOLTIP_DOC,
];

pub fn all_component_docs() -> &'static [ComponentDoc] {
    ALL_COMPONENT_DOCS
}

pub fn find_component_doc(slug: &str) -> Option<&'static ComponentDoc> {
    ALL_COMPONENT_DOCS.iter().find(|doc| doc.slug == slug)
}

pub fn component_to_markdown(doc: &ComponentDoc, demo_source: &str) -> String {
    let mut md = String::new();
    md.push_str(&format!("# {}\n\n", doc.title));
    md.push_str(&format!("{}\n\n", doc.description));
    md.push_str(&format!(
        "- Source: `{}`\n- Accessibility: [{}]({})\n\n",
        doc.source_path, doc.aria_label, doc.aria_url
    ));

    if !doc.features.is_empty() {
        md.push_str("## Features\n\n");
        for feat in doc.features {
            md.push_str(&format!("- {feat}\n"));
        }
        md.push('\n');
    }

    if !demo_source.trim().is_empty() {
        md.push_str("## Example\n\n```rust\n");
        md.push_str(demo_source.trim());
        md.push_str("\n```\n\n");
    }

    md.push_str("## Anatomy\n\n```rust\n");
    md.push_str(doc.anatomy.trim());
    md.push_str("\n```\n\n");

    md.push_str("## API Reference\n\n");
    for part in doc.parts {
        md.push_str(&format!("### {}\n\n{}\n\n", part.name, part.description));
        if !part.props.is_empty() {
            md.push_str("| Prop | Type | Default | Description |\n|---|---|---|---|\n");
            for prop in part.props {
                let def = if prop.required {
                    "Required".to_string()
                } else {
                    prop.default.unwrap_or("—").to_string()
                };
                md.push_str(&format!(
                    "| `{}` | `{}` | `{}` | {} |\n",
                    prop.name, prop.type_simple, def, prop.description
                ));
            }
            md.push('\n');
        }
        if !part.data_attrs.is_empty() {
            md.push_str("| Data Attribute | Values | Description |\n|---|---|---|\n");
            for attr in part.data_attrs {
                md.push_str(&format!(
                    "| `{}` | `{}` | {} |\n",
                    attr.name, attr.values, attr.description
                ));
            }
            md.push('\n');
        }
        if !part.css_vars.is_empty() {
            md.push_str("| CSS Custom Property | Description |\n|---|---|\n");
            for var in part.css_vars {
                md.push_str(&format!("| `{}` | {} |\n", var.name, var.description));
            }
            md.push('\n');
        }
    }

    if !doc.keyboard.is_empty() {
        md.push_str("## Keyboard Interactions\n\n| Key | Description |\n|---|---|\n");
        for kb in doc.keyboard {
            md.push_str(&format!("| `{}` | {} |\n", kb.key, kb.description));
        }
    }

    md
}
