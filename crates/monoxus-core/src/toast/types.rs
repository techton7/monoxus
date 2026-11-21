use std::cell::Cell;
use std::fmt;
use std::rc::Rc;
use std::time::Duration;

pub const TOAST_PARTS: [ToastPart; 7] = [
    ToastPart::Provider,
    ToastPart::Viewport,
    ToastPart::Root,
    ToastPart::Title,
    ToastPart::Description,
    ToastPart::Action,
    ToastPart::Close,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ToastPart {
    Provider,
    Viewport,
    Root,
    Title,
    Description,
    Action,
    Close,
}

impl ToastPart {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Provider => "provider",
            Self::Viewport => "viewport",
            Self::Root => "root",
            Self::Title => "title",
            Self::Description => "description",
            Self::Action => "action",
            Self::Close => "close",
        }
    }
}

/// Monotonic unique identifier for an individual toast.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ToastId(pub u64);

impl fmt::Display for ToastId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Semantic type of toast influencing default live region announcements and icons.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ToastType {
    #[default]
    Default,
    Success,
    Info,
    Warning,
    Error,
    Loading,
}

impl ToastType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Success => "success",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Loading => "loading",
        }
    }
}

/// Lifecycle phase of a toast in the reactive store.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ToastPhase {
    /// Toast is visible and its auto-dismiss countdown is active.
    #[default]
    Active,
    /// Toast has been dismissed (`data-state="closed"`) and is awaiting unmount delay.
    Dismissing,
    /// Toast has completed exit animation and is evicted from store.
    Removed,
}

impl ToastPhase {
    pub const fn as_state_str(&self) -> &'static str {
        match self {
            Self::Active => "open",
            Self::Dismissing | Self::Removed => "closed",
        }
    }
}

/// Viewport corner placement on screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum ToastPosition {
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    BottomCenter,
    #[default]
    BottomRight,
}

impl ToastPosition {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::TopLeft => "top-left",
            Self::TopCenter => "top-center",
            Self::TopRight => "top-right",
            Self::BottomLeft => "bottom-left",
            Self::BottomCenter => "bottom-center",
            Self::BottomRight => "bottom-right",
        }
    }

    /// Horizontal position alignment (`left`, `center`, or `right`).
    pub const fn x_str(&self) -> &'static str {
        match self {
            Self::TopLeft | Self::BottomLeft => "left",
            Self::TopCenter | Self::BottomCenter => "center",
            Self::TopRight | Self::BottomRight => "right",
        }
    }

    /// Vertical placement anchor (`top` or `bottom`).
    pub const fn y_str(&self) -> &'static str {
        match self {
            Self::TopLeft | Self::TopCenter | Self::TopRight => "top",
            Self::BottomLeft | Self::BottomCenter | Self::BottomRight => "bottom",
        }
    }

    pub const fn is_top(&self) -> bool {
        matches!(self, Self::TopLeft | Self::TopCenter | Self::TopRight)
    }

    pub const fn lift_sign(&self) -> f64 {
        if self.is_top() { 1.0 } else { -1.0 }
    }

    /// Derives the default allowed pointer swipe directions from the viewport corner position.
    pub fn default_swipe_directions(&self) -> Vec<SwipeDirection> {
        match self {
            Self::TopLeft => vec![SwipeDirection::Top, SwipeDirection::Left],
            Self::TopCenter => vec![SwipeDirection::Top],
            Self::TopRight => vec![SwipeDirection::Top, SwipeDirection::Right],
            Self::BottomLeft => vec![SwipeDirection::Bottom, SwipeDirection::Left],
            Self::BottomCenter => vec![SwipeDirection::Bottom],
            Self::BottomRight => vec![SwipeDirection::Bottom, SwipeDirection::Right],
        }
    }
}

/// Reading and writing direction for layout and gesture alignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ToastDirection {
    #[default]
    Auto,
    Ltr,
    Rtl,
}

impl ToastDirection {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Ltr => "ltr",
            Self::Rtl => "rtl",
        }
    }
}

/// Allowed pointer swipe directions for dismissing toasts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SwipeDirection {
    Top,
    Right,
    Bottom,
    Left,
}

impl SwipeDirection {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Right => "right",
            Self::Bottom => "bottom",
            Self::Left => "left",
        }
    }
}

/// Viewport edge offsets applied to landmark positioning and responsive mobile margins.
#[derive(Clone, Debug, PartialEq)]
pub struct ToastOffset {
    pub top: Option<String>,
    pub right: Option<String>,
    pub bottom: Option<String>,
    pub left: Option<String>,
}

impl ToastOffset {
    pub fn new() -> Self {
        Self {
            top: None,
            right: None,
            bottom: None,
            left: None,
        }
    }

    pub fn uniform(value: impl Into<String>) -> Self {
        let v = value.into();
        Self {
            top: Some(v.clone()),
            right: Some(v.clone()),
            bottom: Some(v.clone()),
            left: Some(v),
        }
    }

    pub fn with_top(mut self, val: impl Into<String>) -> Self {
        self.top = Some(val.into());
        self
    }

    pub fn with_right(mut self, val: impl Into<String>) -> Self {
        self.right = Some(val.into());
        self
    }

    pub fn with_bottom(mut self, val: impl Into<String>) -> Self {
        self.bottom = Some(val.into());
        self
    }

    pub fn with_left(mut self, val: impl Into<String>) -> Self {
        self.left = Some(val.into());
        self
    }
}

impl Default for ToastOffset {
    fn default() -> Self {
        Self::uniform("32px")
    }
}

/// Event contract passed to action button callbacks allowing dismissal cancellation.
#[derive(Clone, Default)]
pub struct ToastActionEvent {
    default_prevented: Rc<Cell<bool>>,
}

impl ToastActionEvent {
    pub fn new() -> Self {
        Self {
            default_prevented: Rc::new(Cell::new(false)),
        }
    }

    /// Prevents the default automatic dismissal of the toast on action click.
    pub fn prevent_default(&self) {
        self.default_prevented.set(true);
    }

    /// Checks whether the action callback suppressed toast dismissal.
    pub fn is_default_prevented(&self) -> bool {
        self.default_prevented.get()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToastClassNames {
    pub toast: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub action_button: Option<String>,
    pub cancel_button: Option<String>,
    pub close_button: Option<String>,
    pub default: Option<String>,
    pub success: Option<String>,
    pub info: Option<String>,
    pub warning: Option<String>,
    pub error: Option<String>,
    pub loading: Option<String>,
    pub expanded: Option<String>,
    pub collapsed_behind: Option<String>,
}

impl Default for ToastClassNames {
    fn default() -> Self {
        Self {
            toast: Some(
                "relative flex flex-col gap-2 rounded-xl border border-zinc-800 bg-zinc-950 p-4 text-zinc-50 shadow-xl"
                    .to_string(),
            ),
            title: Some("text-sm font-semibold text-zinc-50".to_string()),
            description: Some("text-sm leading-5 text-zinc-400".to_string()),
            action_button: Some(
                "mt-1 rounded-md bg-zinc-50 px-3 py-1.5 text-xs font-semibold text-zinc-950"
                    .to_string(),
            ),
            cancel_button: Some(
                "mt-1 rounded-md border border-zinc-700 px-3 py-1.5 text-xs font-medium text-zinc-300"
                    .to_string(),
            ),
            close_button: Some(
                "absolute right-2 top-2 rounded-md px-2 py-1 text-zinc-400 hover:bg-zinc-900 hover:text-zinc-50"
                    .to_string(),
            ),
            default: Some("border-zinc-800".to_string()),
            success: Some("border-emerald-800".to_string()),
            info: Some("border-sky-800".to_string()),
            warning: Some("border-amber-800".to_string()),
            error: Some("border-red-800".to_string()),
            loading: Some("border-violet-800".to_string()),
            expanded: None,
            collapsed_behind: None,
        }
    }
}

/// Global toaster and viewport configuration defaults.
#[derive(Clone, Debug, PartialEq)]
pub struct ToastConfig {
    pub duration: Duration,
    pub position: ToastPosition,
    pub hotkey: String,
    pub swipe_threshold: f64,
    pub visible_toasts: usize,
    pub gap: f64,
    pub width: f64,
    pub pause_when_page_is_hidden: bool,
    pub close_button: bool,
    pub expand: bool,
    pub container_aria_label: String,
    pub close_button_aria_label: String,
    pub dir: ToastDirection,
    pub offset: ToastOffset,
    pub mobile_offset: ToastOffset,
    pub swipe_directions: Option<Vec<SwipeDirection>>,
    pub class_names: ToastClassNames,
}

impl ToastConfig {
    pub fn effective_swipe_directions(&self) -> Vec<SwipeDirection> {
        self.swipe_directions
            .clone()
            .unwrap_or_else(|| self.position.default_swipe_directions())
    }
}

impl Default for ToastConfig {
    fn default() -> Self {
        Self {
            duration: Duration::from_millis(4000),
            position: ToastPosition::BottomRight,
            hotkey: "F8".to_string(),
            swipe_threshold: 45.0,
            visible_toasts: 3,
            gap: 14.0,
            width: 356.0,
            pause_when_page_is_hidden: true,
            close_button: true,
            expand: false,
            container_aria_label: "Notifications ({hotkey})".to_string(),
            close_button_aria_label: "Close notification".to_string(),
            dir: ToastDirection::Auto,
            offset: ToastOffset::default(),
            mobile_offset: ToastOffset::uniform("16px"),
            swipe_directions: None,
            class_names: ToastClassNames::default(),
        }
    }
}
