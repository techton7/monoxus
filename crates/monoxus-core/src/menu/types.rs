pub use crate::checkbox::CheckedState;
pub use crate::foundation::{
    overlay::{PlacementAlign, PlacementSide, Point2D, PortalHost, Rect2D, StickyStrategy},
    shared::{Direction, Orientation},
    state::DataState,
};

/// Default submenu hover open delay in milliseconds (`3B D-1`).
pub const MENU_DEFAULT_SUBMENU_OPEN_DELAY_MS: u64 = 100;

/// Typeahead search buffer clear timeout in milliseconds (`M-13`).
pub const MENU_TYPEAHEAD_TIMEOUT_MS: u64 = 1000;

/// Default submenu hover corridor stall fallback timeout in milliseconds (`M-11`).
pub const MENU_SUBMENU_CORRIDOR_FALLBACK_MS: u64 = 100;

/// Default submenu return corridor disengage delay in milliseconds (`M-11`).
pub const MENU_SUBMENU_RETURN_DISENGAGE_MS: u64 = 100;

/// Default horizontal/vertical corridor polygon tolerance in pixels.
pub const MENU_SUBMENU_CORRIDOR_BUFFER_PX: f64 = 2.0;

/// Default edge safety padding for submenu hover corridors in pixels.
pub const MENU_SUBMENU_EDGE_BUFFER_PX: f64 = 8.0;

/// Minimum monotonic pointer movement in pixels to extend the hover corridor timeout.
pub const MENU_SUBMENU_MIN_PROGRESS_PX: f64 = 1.0;

/// Default inset applied to a submenu trigger rect when building a safe polygon.
pub const MENU_SUBMENU_TRIGGER_INSET_PX: f64 = 4.0;

/// Default padding for convex-hull grace polygons (`create_grace_area`).
pub const MENU_SUBMENU_GRACE_PADDING_PX: f64 = 5.0;

/// Maximum pointer delta in pixels before an item release cancels click selection (scroll-under-pointer guard).
pub const MENU_POINTER_SCROLL_GUARD_PX: f64 = 4.0;

/// Default keyboard roving wrap behavior (`3B D-5`).
pub const MENU_DEFAULT_LOOP_FOCUS: bool = true;

/// Default modal policy (`3B D-1`).
pub const MENU_DEFAULT_MODAL: bool = true;

/// Default floating side offset between trigger and menu content.
pub const MENU_DEFAULT_SIDE_OFFSET: f32 = 4.0;

/// Default floating collision padding around viewport edges.
pub const MENU_DEFAULT_COLLISION_PADDING: f32 = 0.0;

/// Context menu default pointer offset (`sideOffset = 0`, `alignOffset = 0`).
pub const MENU_CONTEXT_DEFAULT_SIDE_OFFSET: f32 = 2.0;

/// Default non-mouse long-press delay in milliseconds (`CM-5`).
pub const MENU_CONTEXT_LONG_PRESS_MS: u64 = 700;

/// Host primitive variant owning a menu instance (`dropdown-menu`, `context-menu`, `menubar`, or internal `menu`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MenuVariant {
    #[default]
    Menu,
    DropdownMenu,
    ContextMenu,
    Menubar,
}

impl MenuVariant {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Menu => "menu",
            Self::DropdownMenu => "dropdown-menu",
            Self::ContextMenu => "context-menu",
            Self::Menubar => "menubar",
        }
    }

    pub const fn geometry_namespace(self) -> &'static str {
        self.as_str()
    }

    pub const fn radix_compatibility_prefix(self) -> &'static str {
        match self {
            Self::Menu => "radix-menu",
            Self::DropdownMenu => "radix-dropdown-menu",
            Self::ContextMenu => "radix-context-menu",
            Self::Menubar => "radix-menubar",
        }
    }

    pub const fn radix_anchor_label(self) -> &'static str {
        "trigger"
    }
}

/// Canonical parts exposed across `Menu` and its wrapper primitives.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MenuPart {
    Root,
    Trigger,
    Anchor,
    Portal,
    Content,
    ContentStatic,
    Group,
    GroupHeading,
    Label,
    Item,
    CheckboxItem,
    CheckboxGroup,
    RadioGroup,
    RadioItem,
    ItemIndicator,
    Separator,
    Arrow,
    Sub,
    SubTrigger,
    SubContent,
    SubContentStatic,
}

impl MenuPart {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Trigger => "trigger",
            Self::Anchor => "anchor",
            Self::Portal => "portal",
            Self::Content => "content",
            Self::ContentStatic => "content-static",
            Self::Group => "group",
            Self::GroupHeading => "group-heading",
            Self::Label => "label",
            Self::Item => "item",
            Self::CheckboxItem => "checkbox-item",
            Self::CheckboxGroup => "checkbox-group",
            Self::RadioGroup => "radio-group",
            Self::RadioItem => "radio-item",
            Self::ItemIndicator => "item-indicator",
            Self::Separator => "separator",
            Self::Arrow => "arrow",
            Self::Sub => "sub",
            Self::SubTrigger => "sub-trigger",
            Self::SubContent => "sub-content",
            Self::SubContentStatic => "sub-content-static",
        }
    }

    pub const fn static_data_attr_name(self, variant: MenuVariant) -> &'static str {
        match (variant, self) {
            (MenuVariant::Menu, Self::Root) => "data-monoxus-menu-root",
            (MenuVariant::Menu, Self::Trigger) => "data-monoxus-menu-trigger",
            (MenuVariant::Menu, Self::Anchor) => "data-monoxus-menu-anchor",
            (MenuVariant::Menu, Self::Portal) => "data-monoxus-menu-portal",
            (MenuVariant::Menu, Self::Content) => "data-monoxus-menu-content",
            (MenuVariant::Menu, Self::ContentStatic) => "data-monoxus-menu-content-static",
            (MenuVariant::Menu, Self::Group) => "data-monoxus-menu-group",
            (MenuVariant::Menu, Self::GroupHeading) => "data-monoxus-menu-group-heading",
            (MenuVariant::Menu, Self::Label) => "data-monoxus-menu-label",
            (MenuVariant::Menu, Self::Item) => "data-monoxus-menu-item",
            (MenuVariant::Menu, Self::CheckboxItem) => "data-monoxus-menu-checkbox-item",
            (MenuVariant::Menu, Self::CheckboxGroup) => "data-monoxus-menu-checkbox-group",
            (MenuVariant::Menu, Self::RadioGroup) => "data-monoxus-menu-radio-group",
            (MenuVariant::Menu, Self::RadioItem) => "data-monoxus-menu-radio-item",
            (MenuVariant::Menu, Self::ItemIndicator) => "data-monoxus-menu-item-indicator",
            (MenuVariant::Menu, Self::Separator) => "data-monoxus-menu-separator",
            (MenuVariant::Menu, Self::Arrow) => "data-monoxus-menu-arrow",
            (MenuVariant::Menu, Self::Sub) => "data-monoxus-menu-sub",
            (MenuVariant::Menu, Self::SubTrigger) => "data-monoxus-menu-sub-trigger",
            (MenuVariant::Menu, Self::SubContent) => "data-monoxus-menu-sub-content",
            (MenuVariant::Menu, Self::SubContentStatic) => "data-monoxus-menu-sub-content-static",

            (MenuVariant::DropdownMenu, Self::Root) => "data-monoxus-dropdown-menu-root",
            (MenuVariant::DropdownMenu, Self::Trigger) => "data-monoxus-dropdown-menu-trigger",
            (MenuVariant::DropdownMenu, Self::Anchor) => "data-monoxus-dropdown-menu-anchor",
            (MenuVariant::DropdownMenu, Self::Portal) => "data-monoxus-dropdown-menu-portal",
            (MenuVariant::DropdownMenu, Self::Content) => "data-monoxus-dropdown-menu-content",
            (MenuVariant::DropdownMenu, Self::ContentStatic) => "data-monoxus-dropdown-menu-content-static",
            (MenuVariant::DropdownMenu, Self::Group) => "data-monoxus-dropdown-menu-group",
            (MenuVariant::DropdownMenu, Self::GroupHeading) => "data-monoxus-dropdown-menu-group-heading",
            (MenuVariant::DropdownMenu, Self::Label) => "data-monoxus-dropdown-menu-label",
            (MenuVariant::DropdownMenu, Self::Item) => "data-monoxus-dropdown-menu-item",
            (MenuVariant::DropdownMenu, Self::CheckboxItem) => "data-monoxus-dropdown-menu-checkbox-item",
            (MenuVariant::DropdownMenu, Self::CheckboxGroup) => "data-monoxus-dropdown-menu-checkbox-group",
            (MenuVariant::DropdownMenu, Self::RadioGroup) => "data-monoxus-dropdown-menu-radio-group",
            (MenuVariant::DropdownMenu, Self::RadioItem) => "data-monoxus-dropdown-menu-radio-item",
            (MenuVariant::DropdownMenu, Self::ItemIndicator) => "data-monoxus-dropdown-menu-item-indicator",
            (MenuVariant::DropdownMenu, Self::Separator) => "data-monoxus-dropdown-menu-separator",
            (MenuVariant::DropdownMenu, Self::Arrow) => "data-monoxus-dropdown-menu-arrow",
            (MenuVariant::DropdownMenu, Self::Sub) => "data-monoxus-dropdown-menu-sub",
            (MenuVariant::DropdownMenu, Self::SubTrigger) => "data-monoxus-dropdown-menu-sub-trigger",
            (MenuVariant::DropdownMenu, Self::SubContent) => "data-monoxus-dropdown-menu-sub-content",
            (MenuVariant::DropdownMenu, Self::SubContentStatic) => "data-monoxus-dropdown-menu-sub-content-static",

            (MenuVariant::ContextMenu, Self::Root) => "data-monoxus-context-menu-root",
            (MenuVariant::ContextMenu, Self::Trigger) => "data-monoxus-context-menu-trigger",
            (MenuVariant::ContextMenu, Self::Anchor) => "data-monoxus-context-menu-anchor",
            (MenuVariant::ContextMenu, Self::Portal) => "data-monoxus-context-menu-portal",
            (MenuVariant::ContextMenu, Self::Content) => "data-monoxus-context-menu-content",
            (MenuVariant::ContextMenu, Self::ContentStatic) => "data-monoxus-context-menu-content-static",
            (MenuVariant::ContextMenu, Self::Group) => "data-monoxus-context-menu-group",
            (MenuVariant::ContextMenu, Self::GroupHeading) => "data-monoxus-context-menu-group-heading",
            (MenuVariant::ContextMenu, Self::Label) => "data-monoxus-context-menu-label",
            (MenuVariant::ContextMenu, Self::Item) => "data-monoxus-context-menu-item",
            (MenuVariant::ContextMenu, Self::CheckboxItem) => "data-monoxus-context-menu-checkbox-item",
            (MenuVariant::ContextMenu, Self::CheckboxGroup) => "data-monoxus-context-menu-checkbox-group",
            (MenuVariant::ContextMenu, Self::RadioGroup) => "data-monoxus-context-menu-radio-group",
            (MenuVariant::ContextMenu, Self::RadioItem) => "data-monoxus-context-menu-radio-item",
            (MenuVariant::ContextMenu, Self::ItemIndicator) => "data-monoxus-context-menu-item-indicator",
            (MenuVariant::ContextMenu, Self::Separator) => "data-monoxus-context-menu-separator",
            (MenuVariant::ContextMenu, Self::Arrow) => "data-monoxus-context-menu-arrow",
            (MenuVariant::ContextMenu, Self::Sub) => "data-monoxus-context-menu-sub",
            (MenuVariant::ContextMenu, Self::SubTrigger) => "data-monoxus-context-menu-sub-trigger",
            (MenuVariant::ContextMenu, Self::SubContent) => "data-monoxus-context-menu-sub-content",
            (MenuVariant::ContextMenu, Self::SubContentStatic) => "data-monoxus-context-menu-sub-content-static",

            (MenuVariant::Menubar, Self::Root) => "data-monoxus-menubar-root",
            (MenuVariant::Menubar, Self::Trigger) => "data-monoxus-menubar-trigger",
            (MenuVariant::Menubar, Self::Anchor) => "data-monoxus-menubar-anchor",
            (MenuVariant::Menubar, Self::Portal) => "data-monoxus-menubar-portal",
            (MenuVariant::Menubar, Self::Content) => "data-monoxus-menubar-content",
            (MenuVariant::Menubar, Self::ContentStatic) => "data-monoxus-menubar-content-static",
            (MenuVariant::Menubar, Self::Group) => "data-monoxus-menubar-group",
            (MenuVariant::Menubar, Self::GroupHeading) => "data-monoxus-menubar-group-heading",
            (MenuVariant::Menubar, Self::Label) => "data-monoxus-menubar-label",
            (MenuVariant::Menubar, Self::Item) => "data-monoxus-menubar-item",
            (MenuVariant::Menubar, Self::CheckboxItem) => "data-monoxus-menubar-checkbox-item",
            (MenuVariant::Menubar, Self::CheckboxGroup) => "data-monoxus-menubar-checkbox-group",
            (MenuVariant::Menubar, Self::RadioGroup) => "data-monoxus-menubar-radio-group",
            (MenuVariant::Menubar, Self::RadioItem) => "data-monoxus-menubar-radio-item",
            (MenuVariant::Menubar, Self::ItemIndicator) => "data-monoxus-menubar-item-indicator",
            (MenuVariant::Menubar, Self::Separator) => "data-monoxus-menubar-separator",
            (MenuVariant::Menubar, Self::Arrow) => "data-monoxus-menubar-arrow",
            (MenuVariant::Menubar, Self::Sub) => "data-monoxus-menubar-sub",
            (MenuVariant::Menubar, Self::SubTrigger) => "data-monoxus-menubar-sub-trigger",
            (MenuVariant::Menubar, Self::SubContent) => "data-monoxus-menubar-sub-content",
            (MenuVariant::Menubar, Self::SubContentStatic) => "data-monoxus-menubar-sub-content-static",
        }
    }

    pub fn data_attr_name(self, variant: MenuVariant) -> String {
        self.static_data_attr_name(variant).to_owned()
    }
}

/// Reason a menu or submenu opened or closed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MenuInteractionOrigin {
    #[default]
    Pointer,
    Keyboard,
    Hover,
    Programmatic,
}

/// Host-neutral keyboard intents accepted by menu content, items, and triggers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MenuKey {
    ArrowDown,
    ArrowUp,
    ArrowRight,
    ArrowLeft,
    Home,
    End,
    PageUp,
    PageDown,
    Enter,
    Space,
    Escape,
    Tab,
    Backspace,
    Char(char),
}

impl MenuKey {
    pub const fn is_selection_key(self) -> bool {
        matches!(self, Self::Enter | Self::Space)
    }

    pub const fn is_vertical_nav_key(self) -> bool {
        matches!(
            self,
            Self::ArrowDown | Self::ArrowUp | Self::Home | Self::End | Self::PageUp | Self::PageDown
        )
    }

    pub const fn is_submenu_open_key(self, direction: Direction) -> bool {
        match direction {
            Direction::Ltr => matches!(self, Self::ArrowRight),
            Direction::Rtl => matches!(self, Self::ArrowLeft),
        }
    }

    pub const fn is_submenu_close_key(self, direction: Direction) -> bool {
        match direction {
            Direction::Ltr => matches!(self, Self::ArrowLeft),
            Direction::Rtl => matches!(self, Self::ArrowRight),
        }
    }
}

/// Kind of selectable/navigable item registered inside a menu surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MenuItemKind {
    Item,
    CheckboxItem,
    RadioItem,
    SubTrigger,
}

/// Registered item metadata used for WAI-ARIA roving navigation and typeahead search.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuItemEntry {
    pub id: String,
    pub text_value: String,
    pub disabled: bool,
    pub kind: MenuItemKind,
    pub dom_order: usize,
}

impl MenuItemEntry {
    pub fn new(
        id: impl Into<String>,
        text_value: impl Into<String>,
        disabled: bool,
        kind: MenuItemKind,
        dom_order: usize,
    ) -> Self {
        Self {
            id: id.into(),
            text_value: text_value.into(),
            disabled,
            kind,
            dom_order,
        }
    }
}

/// Pointer press snapshot used to prevent accidental clicks when a menu scrolls under the pointer.
#[derive(Clone, Debug, PartialEq)]
pub struct PressedItemSnapshot {
    pub item_id: String,
    pub client_x: f64,
    pub client_y: f64,
}

impl PressedItemSnapshot {
    pub fn new(item_id: impl Into<String>, client_x: f64, client_y: f64) -> Self {
        Self {
            item_id: item_id.into(),
            client_x,
            client_y,
        }
    }

    pub fn moved_beyond(&self, client_x: f64, client_y: f64, threshold_px: f64) -> bool {
        (client_x - self.client_x).abs() >= threshold_px
            || (client_y - self.client_y).abs() >= threshold_px
    }
}

/// Configurable timeouts for menu typeahead search.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypeaheadConfig {
    pub timeout_ms: u64,
}

impl Default for TypeaheadConfig {
    fn default() -> Self {
        Self {
            timeout_ms: MENU_TYPEAHEAD_TIMEOUT_MS,
        }
    }
}

/// Parameters governing the bidirectional submenu hover grace corridor (`M-11`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SubmenuIntentConfig {
    pub corridor_buffer_px: f64,
    pub edge_buffer_px: f64,
    pub min_progress_px: f64,
    pub trigger_inset_px: f64,
    pub grace_padding_px: f64,
    pub fallback_timeout_ms: u64,
    pub return_disengage_ms: u64,
}

impl Default for SubmenuIntentConfig {
    fn default() -> Self {
        Self {
            corridor_buffer_px: MENU_SUBMENU_CORRIDOR_BUFFER_PX,
            edge_buffer_px: MENU_SUBMENU_EDGE_BUFFER_PX,
            min_progress_px: MENU_SUBMENU_MIN_PROGRESS_PX,
            trigger_inset_px: MENU_SUBMENU_TRIGGER_INSET_PX,
            grace_padding_px: MENU_SUBMENU_GRACE_PADDING_PX,
            fallback_timeout_ms: MENU_SUBMENU_CORRIDOR_FALLBACK_MS,
            return_disengage_ms: MENU_SUBMENU_RETURN_DISENGAGE_MS,
        }
    }
}

/// Active direction of a bidirectional submenu intent corridor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SubmenuCorridorDirection {
    #[default]
    Idle,
    TriggerToContent,
    ContentToTrigger,
}

/// Evaluation output from processing a pointer move inside a submenu corridor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SubmenuPointerEvaluation {
    Idle,
    ReachedTarget,
    InCorridor,
    ExitedCorridor,
    Stalled,
}

/// Navigation step resolved inside a single menu surface.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuNavigationStep {
    None,
    HighlightItem { id: String },
    OpenSubmenuAndFocusFirst { sub_trigger_id: String },
    CloseSubmenuAndFocusTrigger,
    CloseRootMenu,
    FocusContentContainer,
    TabTrap,
    TabOut { forward: bool },
}

/// Policy describing how outside interactions dismiss a menu tree.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MenuOutsideAction {
    #[default]
    Close,
    Ignore,
}

/// Focus target selected upon opening a menu surface.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MenuOpenFocusTarget {
    #[default]
    ContentContainer,
    FirstItem,
    LastItem,
    None,
}

/// Semantic request to change open state of a menu or submenu.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MenuOpenRequest {
    Open,
    Close,
}

impl MenuOpenRequest {
    pub const fn next_open(self) -> bool {
        matches!(self, Self::Open)
    }

    pub const fn data_state(self) -> DataState {
        if self.next_open() {
            DataState::Open
        } else {
            DataState::Closed
        }
    }
}
