use dioxus::prelude::*;
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

/// Monotonic unique identifier for an individual toast.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ToastId(pub u64);

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
    pub fn as_str(&self) -> &'static str {
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
    /// Toast has been dismissed (data-state="closed") and is awaiting unmount delay.
    Dismissing,
    /// Toast has completed exit animation and is evicted from store.
    Removed,
}

impl ToastPhase {
    pub fn as_state_str(&self) -> &'static str {
        match self {
            Self::Active => "open",
            Self::Dismissing | Self::Removed => "closed",
        }
    }
}

/// Viewport corner placement on screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ToastPosition {
    TopLeft,
    TopRight,
    TopCenter,
    BottomLeft,
    #[default]
    BottomRight,
    BottomCenter,
}

impl ToastPosition {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TopLeft => "top-left",
            Self::TopRight => "top-right",
            Self::TopCenter => "top-center",
            Self::BottomLeft => "bottom-left",
            Self::BottomRight => "bottom-right",
            Self::BottomCenter => "bottom-center",
        }
    }

    /// Horizontal position alignment ('left', 'center', or 'right').
    pub fn x_str(&self) -> &'static str {
        match self {
            Self::TopLeft | Self::BottomLeft => "left",
            Self::TopCenter | Self::BottomCenter => "center",
            Self::TopRight | Self::BottomRight => "right",
        }
    }

    /// Vertical placement anchor ('top' or 'bottom').
    pub fn y_str(&self) -> &'static str {
        match self {
            Self::TopLeft | Self::TopCenter | Self::TopRight => "top",
            Self::BottomLeft | Self::BottomCenter | Self::BottomRight => "bottom",
        }
    }

    /// Derives the default allowed pointer swipe directions from the viewport corner position.
    pub fn default_swipe_directions(&self) -> Vec<SwipeDirection> {
        match self {
            Self::TopLeft => vec![SwipeDirection::Top, SwipeDirection::Left],
            Self::TopRight => vec![SwipeDirection::Top, SwipeDirection::Right],
            Self::TopCenter => vec![SwipeDirection::Top],
            Self::BottomLeft => vec![SwipeDirection::Bottom, SwipeDirection::Left],
            Self::BottomRight => vec![SwipeDirection::Bottom, SwipeDirection::Right],
            Self::BottomCenter => vec![SwipeDirection::Bottom],
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
    pub fn as_str(&self) -> &'static str {
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
    pub fn as_str(&self) -> &'static str {
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
    pub fn uniform(value: impl Into<String>) -> Self {
        let v = value.into();
        Self {
            top: Some(v.clone()),
            right: Some(v.clone()),
            bottom: Some(v.clone()),
            left: Some(v),
        }
    }

    pub fn new() -> Self {
        Self {
            top: None,
            right: None,
            bottom: None,
            left: None,
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

/// Primary or cancel action button configuration.
#[derive(Clone)]
pub struct ToastActionData {
    pub label: String,
    pub alt_text: String,
    pub on_click: Option<Callback<ToastActionEvent>>,
}

impl ToastActionData {
    pub fn new(label: impl Into<String>, alt_text: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            alt_text: alt_text.into(),
            on_click: None,
        }
    }

    pub fn with_handler(mut self, handler: impl Into<Callback<ToastActionEvent>>) -> Self {
        self.on_click = Some(handler.into());
        self
    }
}

/// Renderer closure for custom arbitrary RSX toast notification content.
pub type ToastCustomRenderer = Rc<dyn Fn(ToastId) -> Element>;

/// Per-toast optional overrides bag.
#[derive(Clone, Default)]
pub struct ToastOptions {
    pub id: Option<ToastId>,
    pub description: Option<String>,
    pub duration: Option<Duration>,
    pub action: Option<ToastActionData>,
    pub cancel: Option<ToastActionData>,
    pub on_dismiss: Option<Callback<()>>,
    pub on_auto_close: Option<Callback<()>>,
    pub dismissible: Option<bool>,
    pub position: Option<ToastPosition>,
    pub test_id: Option<String>,
    pub custom_renderer: Option<ToastCustomRenderer>,
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
    pub pause_when_page_is_hidden: bool,
    pub container_aria_label: String,
    pub close_button_aria_label: String,
    pub dir: ToastDirection,
    pub offset: ToastOffset,
    pub mobile_offset: ToastOffset,
    pub swipe_directions: Option<Vec<SwipeDirection>>,
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
            pause_when_page_is_hidden: true,
            container_aria_label: "Notifications ({hotkey})".to_string(),
            close_button_aria_label: "Close notification".to_string(),
            dir: ToastDirection::Auto,
            offset: ToastOffset::default(),
            mobile_offset: ToastOffset::uniform("16px"),
            swipe_directions: None,
        }
    }
}

/// WASM-safe monotonic instant measuring real wall-clock durations without panicking on web targets.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct ToastInstant {
    #[cfg(target_arch = "wasm32")]
    millis: f64,
    #[cfg(not(target_arch = "wasm32"))]
    instant: std::time::Instant,
}

impl ToastInstant {
    pub fn now() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Self {
                millis: js_sys::Date::now(),
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self {
                instant: std::time::Instant::now(),
            }
        }
    }

    pub fn saturating_duration_since(&self, earlier: Self) -> Duration {
        #[cfg(target_arch = "wasm32")]
        {
            let diff = (self.millis - earlier.millis).max(0.0);
            Duration::from_secs_f64(diff / 1000.0)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.instant.saturating_duration_since(earlier.instant)
        }
    }
}

/// A queued or active toast notification record.
#[derive(Clone)]
pub struct ToastItem {
    pub id: ToastId,
    pub toast_type: ToastType,
    pub title: String,
    pub description: Option<String>,
    pub phase: ToastPhase,
    pub duration: Duration,
    pub remaining_duration: Duration,
    pub created_at: ToastInstant,
    pub last_started_at: ToastInstant,
    pub paused_at: Option<ToastInstant>,
    pub options: ToastOptions,
}

impl ToastItem {
    pub fn new(
        id: ToastId,
        toast_type: ToastType,
        title: impl Into<String>,
        duration: Duration,
        options: ToastOptions,
    ) -> Self {
        let now = ToastInstant::now();
        Self {
            id,
            toast_type,
            title: title.into(),
            description: options.description.clone(),
            phase: ToastPhase::Active,
            duration,
            remaining_duration: duration,
            created_at: now,
            last_started_at: now,
            paused_at: None,
            options,
        }
    }
}
