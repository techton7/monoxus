use dioxus::prelude::*;
use std::rc::Rc;
use std::time::Duration;

pub use monoxus_core::toast::types::*;

/// Primary or cancel action button configuration.
/// Helper conversion trait allowing closures or Callbacks to be used as toast action handlers.
pub trait IntoToastActionHandler<Marker = ()> {
    fn into_handler(self) -> Callback<ToastActionEvent>;
}

impl IntoToastActionHandler<()> for Callback<ToastActionEvent> {
    fn into_handler(self) -> Callback<ToastActionEvent> {
        self
    }
}

fn make_root_action_callback<F: FnMut(ToastActionEvent) + 'static>(
    f: F,
) -> Callback<ToastActionEvent> {
    if let Some(rt) = dioxus::core::Runtime::try_current() {
        rt.in_scope(dioxus::core::ScopeId::ROOT, || Callback::new(f))
    } else {
        Callback::new(f)
    }
}

pub struct ActionEventClosureMarker;
impl<F: FnMut(ToastActionEvent) + 'static> IntoToastActionHandler<ActionEventClosureMarker> for F {
    fn into_handler(self) -> Callback<ToastActionEvent> {
        make_root_action_callback(self)
    }
}

pub struct UnitClosureMarker;
impl<F: FnMut() + 'static> IntoToastActionHandler<UnitClosureMarker> for F {
    fn into_handler(mut self) -> Callback<ToastActionEvent> {
        make_root_action_callback(move |_| self())
    }
}

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

    pub fn with_handler<Marker>(
        mut self,
        handler: impl IntoToastActionHandler<Marker>,
    ) -> Self {
        self.on_click = Some(handler.into_handler());
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
    pub close_button: Option<bool>,
    pub position: Option<ToastPosition>,
    pub test_id: Option<String>,
    pub custom_renderer: Option<ToastCustomRenderer>,
}

/// WASM-safe monotonic instant measuring real wall-clock durations without panicking on web targets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ToastInstant(oxidase::time::MonotonicInstant);

impl ToastInstant {
    pub fn now() -> Self {
        Self(oxidase::time::MonotonicInstant::now())
    }

    pub fn saturating_duration_since(&self, earlier: Self) -> Duration {
        self.0.saturating_duration_since(earlier.0)
    }
}

impl monoxus_core::toast::state::ToastClockInstant for ToastInstant {
    #[inline]
    fn saturating_duration_since(&self, earlier: Self) -> Duration {
        self.0.saturating_duration_since(earlier.0)
    }
}

/// A queued or active toast notification record.
#[derive(Clone)]
pub struct ToastItem {
    pub id: ToastId,
    pub generation: u64,
    pub content_generation: u64,
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
            generation: 0,
            content_generation: 0,
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

impl From<ToastItem> for monoxus_core::toast::state::ToastQueueItem<ToastOptions, ToastInstant> {
    fn from(item: ToastItem) -> Self {
        Self {
            id: item.id,
            generation: item.generation,
            content_generation: item.content_generation,
            toast_type: item.toast_type,
            title: item.title,
            description: item.description,
            phase: item.phase,
            duration: item.duration,
            remaining_duration: item.remaining_duration,
            created_at: item.created_at,
            last_started_at: item.last_started_at,
            paused_at: item.paused_at,
            meta: item.options,
        }
    }
}

impl From<monoxus_core::toast::state::ToastQueueItem<ToastOptions, ToastInstant>> for ToastItem {
    fn from(item: monoxus_core::toast::state::ToastQueueItem<ToastOptions, ToastInstant>) -> Self {
        Self {
            id: item.id,
            generation: item.generation,
            content_generation: item.content_generation,
            toast_type: item.toast_type,
            title: item.title,
            description: item.description,
            phase: item.phase,
            duration: item.duration,
            remaining_duration: item.remaining_duration,
            created_at: item.created_at,
            last_started_at: item.last_started_at,
            paused_at: item.paused_at,
            options: item.meta,
        }
    }
}
