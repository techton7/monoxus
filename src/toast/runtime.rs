use super::state::ToastStore;
use super::types::*;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::time::Duration;

pub use oxidase::WatcherGuard;

/// Options bag historically passed to browser monitor bindings.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToastBrowserOptions {
    pub hotkey: String,
    pub viewport_id: String,
    pub gap: f64,
    pub swipe_threshold: f64,
    pub position: String,
    pub dir: String,
    pub swipe_directions: Vec<String>,
    pub offset_top: String,
    pub offset_right: String,
    pub offset_bottom: String,
    pub offset_left: String,
    pub mobile_offset_top: String,
    pub mobile_offset_right: String,
    pub mobile_offset_bottom: String,
    pub mobile_offset_left: String,
}

impl ToastBrowserOptions {
    pub fn from_config(config: &ToastConfig, viewport_id: &str) -> Self {
        let default_offset = "32px";
        let default_mobile = "16px";
        Self {
            hotkey: config.hotkey.clone(),
            viewport_id: viewport_id.to_string(),
            gap: config.gap,
            swipe_threshold: config.swipe_threshold,
            position: config.position.as_str().to_string(),
            dir: config.dir.as_str().to_string(),
            swipe_directions: config
                .effective_swipe_directions()
                .iter()
                .map(|d| d.as_str().to_string())
                .collect(),
            offset_top: config
                .offset
                .top
                .clone()
                .unwrap_or_else(|| default_offset.to_string()),
            offset_right: config
                .offset
                .right
                .clone()
                .unwrap_or_else(|| default_offset.to_string()),
            offset_bottom: config
                .offset
                .bottom
                .clone()
                .unwrap_or_else(|| default_offset.to_string()),
            offset_left: config
                .offset
                .left
                .clone()
                .unwrap_or_else(|| default_offset.to_string()),
            mobile_offset_top: config
                .mobile_offset
                .top
                .clone()
                .unwrap_or_else(|| default_mobile.to_string()),
            mobile_offset_right: config
                .mobile_offset
                .right
                .clone()
                .unwrap_or_else(|| default_mobile.to_string()),
            mobile_offset_bottom: config
                .mobile_offset
                .bottom
                .clone()
                .unwrap_or_else(|| default_mobile.to_string()),
            mobile_offset_left: config
                .mobile_offset
                .left
                .clone()
                .unwrap_or_else(|| default_mobile.to_string()),
        }
    }
}

/// Browser lifecycle and window event variants observed by the toast runtime.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ToastEventPayload {
    /// Window/tab visibility state changed (`true` = visible, `false` = hidden).
    Visibility { visible: bool },
    /// Hotkey pressed (e.g. "F8") navigating focus to the landmark viewport.
    Hotkey { key: String },
    /// Viewport hover state entered or left.
    Hover { hovered: bool },
    /// Viewport focus state entered or left.
    Focus { focused: bool },
    /// Active pointer swipe gesture started or ended.
    SwipeActive { swiping: bool },
    /// Pointer swipe gesture crossed dismiss threshold for specific toast ID.
    SwipeDismiss { id: u64 },
    /// Monitor gracefully shut down.
    Stopped,
}

pub type ToastBrowserEvent = ToastEventPayload;

/// Parses the typed JSON payload emitted for toast browser events.
pub fn parse_toast_browser_event(
    json: &str,
) -> Result<ToastBrowserEvent, oxidase::serde_json::Error> {
    oxidase::serde_json::from_str(json)
}

/// Launches the browser monitor for visibility change, gestures, and landmark hotkey focus.
///
/// Under pure-Rust declarative layout, native gestures and hotkeys run directly without
/// bespoke JS bridges. Returns a no-op guard for backwards compatibility.
pub fn start_toast_browser_monitor(
    _config: &ToastConfig,
    _viewport_id: &str,
    _on_event: impl FnMut(ToastBrowserEvent) + 'static,
) -> Option<WatcherGuard> {
    Some(WatcherGuard::noop())
}

/// Gracefully signals the browser monitor to unbind all DOM listeners and terminate.
pub fn stop_toast_browser_monitor(watcher: WatcherGuard) {
    watcher.stop();
}

/// Delay duration before evicted toast is removed from store to allow exit animations.
pub const TIME_BEFORE_UNMOUNT: Duration = Duration::from_millis(200);

/// Canonical global signal store powering imperative `toast::*` calls across the application.
pub static TOAST_STORE: GlobalSignal<ToastStore> = GlobalSignal::new(ToastStore::default);

thread_local! {
    static TOAST_WATCHER: RefCell<Option<(String, WatcherGuard)>> = const { RefCell::new(None) };
}

/// Reactive runtime handle for observing and managing toast notifications.
#[derive(Clone, Copy, Debug, Default)]
pub struct ToastRuntime;

fn safe_spawn<F: std::future::Future<Output = ()> + 'static>(fut: F) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        dioxus::core::spawn_forever(fut);
    }));
}

impl ToastRuntime {
    pub fn store(&self) -> &'static GlobalSignal<ToastStore> {
        &TOAST_STORE
    }

    pub fn toasts(&self) -> Vec<ToastItem> {
        TOAST_STORE.read().toasts.clone()
    }

    pub fn is_paused(&self) -> bool {
        TOAST_STORE.read().paused
    }

    pub fn pause_all(&self) {
        TOAST_STORE.write().pause_all();
    }

    pub fn resume_all(&self) {
        TOAST_STORE.write().resume_all();
    }

    pub fn dismiss(&self, id: Option<ToastId>) {
        toast::dismiss(id);
    }

    pub fn dismiss_toast(&self, id: ToastId) {
        toast::dismiss(Some(id));
    }

    pub fn handle_swipe_action(&self, action: crate::toast::gesture::SwipeAction) {
        match action {
            crate::toast::gesture::SwipeAction::Dismiss(id) => {
                toast::dismiss(Some(ToastId(id)));
            }
            crate::toast::gesture::SwipeAction::SnapBack
            | crate::toast::gesture::SwipeAction::Update { .. } => {}
        }
    }
}

/// Returns current timestamp in milliseconds, WASM-safe.
pub fn toast_now_ms() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64() * 1000.0)
            .unwrap_or(0.0)
    }
}

/// Ensures that the global browser toast watcher is active and bound to the specified viewport ID.
///
/// If a watcher is already bound to this exact `viewport_id`, this call is an idempotent no-op.
/// If a watcher was previously bound to a different `viewport_id`, it is gracefully stopped and
/// rebound to the new `viewport_id`.
pub fn ensure_toast_watcher(viewport_id: &str) {
    if dioxus::core::Runtime::try_current().is_none() {
        return;
    }

    TOAST_WATCHER.with(|slot| {
        let mut slot_ref = slot.borrow_mut();
        if let Some((bound_id, _)) = slot_ref.as_ref() {
            if bound_id == viewport_id {
                return;
            }
            if let Some((_, watcher)) = slot_ref.take() {
                watcher.stop();
            }
        }

        let config = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            TOAST_STORE.peek().config.clone()
        }))
        .unwrap_or_default();

        if let Some(watcher) = start_toast_browser_monitor(
            &config,
            viewport_id,
            {
                let v_id = viewport_id.to_string();
                move |event| match event {
                    ToastBrowserEvent::Visibility { visible } => {
                        let should_pause = TOAST_STORE.peek().config.pause_when_page_is_hidden;
                        crate::toast::hotkey::apply_visibility_change(visible, should_pause);
                    }
                    ToastBrowserEvent::Hover { hovered } => {
                        TOAST_STORE.write().set_hovered(hovered);
                    }
                    ToastBrowserEvent::Focus { focused } => {
                        TOAST_STORE.write().set_focused(focused);
                    }
                    ToastBrowserEvent::SwipeActive { swiping } => {
                        TOAST_STORE.write().set_swiping(swiping);
                    }
                    ToastBrowserEvent::SwipeDismiss { id } => {
                        toast::dismiss(Some(ToastId(id)));
                    }
                    ToastBrowserEvent::Hotkey { key } => {
                        let cfg = TOAST_STORE.peek().config.clone();
                        crate::toast::hotkey::handle_landmark_keydown(&key, &cfg.hotkey, &v_id);
                    }
                    ToastBrowserEvent::Stopped => {}
                }
            },
        ) {
            *slot_ref = Some((viewport_id.to_string(), watcher));
        }
    });
}

/// Reactive hook subscribing to toast notifications and ensuring the viewport monitor is bound.
pub fn use_toast_runtime() -> ToastRuntime {
    ensure_toast_watcher("monoxus-toast-viewport");
    ToastRuntime
}

/// Resets the active toast watcher handle, disconnecting global browser listeners.
///
/// This is primarily designed for deterministic test isolation between test runs,
/// but is also safe for runtime reinitialization if global viewport configuration changes.
pub fn reset_toast_watcher() {
    TOAST_WATCHER.with(|slot| {
        if let Some((_, watcher)) = slot.borrow_mut().take() {
            watcher.stop();
        }
    });
}

/// Imperative toast dispatch API.
pub mod toast {
    use super::*;

    fn dispatch_typed(
        toast_type: ToastType,
        title: impl Into<String>,
        options: Option<ToastOptions>,
    ) -> ToastId {
        let options = options.unwrap_or_default();
        let mut store = TOAST_STORE.write();
        let duration = options.duration.unwrap_or(store.config.duration);
        let id_hint = options.id.unwrap_or(ToastId(0));
        let now = ToastInstant::now();
        let item = ToastItem {
            id: id_hint,
            toast_type,
            title: title.into(),
            description: options.description.clone(),
            phase: ToastPhase::Active,
            duration,
            remaining_duration: duration,
            created_at: now,
            last_started_at: now,
            paused_at: if store.paused { Some(now) } else { None },
            options,
        };
        let id = store.add_toast(item);
        drop(store);

        if toast_type != ToastType::Loading {
            spawn_toast_timer(id);
        }

        id
    }

    fn spawn_toast_timer(id: ToastId) {
        safe_spawn(async move {
            loop {
                futures_timer::Delay::new(Duration::from_millis(50)).await;
                let mut store = TOAST_STORE.write();
                let is_paused = store.paused;
                let Some(item) = store.toasts.iter_mut().find(|t| t.id == id) else {
                    break;
                };
                if item.phase != ToastPhase::Active {
                    break;
                }
                let mut expired = false;
                let mut auto_close_cb = None;
                if !is_paused {
                    let now = ToastInstant::now();
                    let elapsed = now.saturating_duration_since(item.last_started_at);
                    item.remaining_duration = item.remaining_duration.saturating_sub(elapsed);
                    item.last_started_at = now;
                    if item.remaining_duration.is_zero() {
                        expired = true;
                        auto_close_cb = item.options.on_auto_close;
                    }
                }

                if expired {
                    if let Some(cb) = auto_close_cb {
                        cb.call(());
                    }
                    store.dismiss_toast(id);
                    drop(store);
                    safe_spawn(async move {
                        futures_timer::Delay::new(TIME_BEFORE_UNMOUNT).await;
                        TOAST_STORE.write().remove_toast(id);
                    });
                    break;
                }
            }
        });
    }

    /// Dispatches a default informational toast notification.
    pub fn message(title: impl Into<String>, options: Option<ToastOptions>) -> ToastId {
        dispatch_typed(ToastType::Default, title, options)
    }

    /// Dispatches a success toast notification.
    pub fn success(title: impl Into<String>, options: Option<ToastOptions>) -> ToastId {
        dispatch_typed(ToastType::Success, title, options)
    }

    /// Dispatches an informational toast notification.
    pub fn info(title: impl Into<String>, options: Option<ToastOptions>) -> ToastId {
        dispatch_typed(ToastType::Info, title, options)
    }

    /// Dispatches a warning toast notification (`role="alert"`).
    pub fn warning(title: impl Into<String>, options: Option<ToastOptions>) -> ToastId {
        dispatch_typed(ToastType::Warning, title, options)
    }

    /// Dispatches an error toast notification (`role="alert"`, assertive live region).
    pub fn error(title: impl Into<String>, options: Option<ToastOptions>) -> ToastId {
        dispatch_typed(ToastType::Error, title, options)
    }

    /// Dispatches a persistent loading toast notification with infinite lifespan until updated.
    pub fn loading(title: impl Into<String>, options: Option<ToastOptions>) -> ToastId {
        dispatch_typed(ToastType::Loading, title, options)
    }

    /// Dispatches a custom-styled toast notification with arbitrary RSX rendering.
    pub fn custom(
        renderer: impl Fn(ToastId) -> Element + 'static,
        options: Option<ToastOptions>,
    ) -> ToastId {
        let mut opts = options.unwrap_or_default();
        opts.custom_renderer = Some(std::rc::Rc::new(renderer));
        dispatch_typed(ToastType::Default, "", Some(opts))
    }

    /// Dismisses an active toast by ID, or all toasts if ID is omitted.
    pub fn dismiss(id: Option<ToastId>) {
        let mut store = TOAST_STORE.write();
        if let Some(target_id) = id {
            if store.dismiss_toast(target_id) {
                drop(store);
                safe_spawn(async move {
                    futures_timer::Delay::new(TIME_BEFORE_UNMOUNT).await;
                    TOAST_STORE.write().remove_toast(target_id);
                });
            }
        } else {
            let active_ids: Vec<ToastId> = store
                .toasts
                .iter()
                .filter(|t| t.phase == ToastPhase::Active)
                .map(|t| t.id)
                .collect();
            store.dismiss_all();
            drop(store);
            for active_id in active_ids {
                safe_spawn(async move {
                    futures_timer::Delay::new(TIME_BEFORE_UNMOUNT).await;
                    TOAST_STORE.write().remove_toast(active_id);
                });
            }
        }
    }

    /// Automatically tracks an asynchronous future with optional toast options,
    /// showing a loading notification and transitioning to success or error state upon completion,
    /// and resuming the auto-dismiss countdown lifecycle.
    pub async fn promise_with_options<T, E, F, S, ErrMsg>(
        future: F,
        loading_msg: impl Into<String>,
        success_fn: S,
        error_fn: ErrMsg,
        options: Option<ToastOptions>,
    ) -> Result<T, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
        S: FnOnce(&T) -> String,
        ErrMsg: FnOnce(&E) -> String,
    {
        let id = loading(loading_msg, options);
        let outcome = future.await;
        let now = ToastInstant::now();
        match &outcome {
            Ok(data) => {
                let msg = success_fn(data);
                TOAST_STORE.write().update_toast(id, |item| {
                    item.toast_type = ToastType::Success;
                    item.title = msg;
                    item.phase = ToastPhase::Active;
                    item.remaining_duration = item.duration;
                    item.last_started_at = now;
                    item.paused_at = None;
                });
                spawn_toast_timer(id);
            }
            Err(err) => {
                let msg = error_fn(err);
                TOAST_STORE.write().update_toast(id, |item| {
                    item.toast_type = ToastType::Error;
                    item.title = msg;
                    item.phase = ToastPhase::Active;
                    item.remaining_duration = item.duration;
                    item.last_started_at = now;
                    item.paused_at = None;
                });
                spawn_toast_timer(id);
            }
        }
        outcome
    }

    /// Automatically tracks an asynchronous future, showing a loading notification
    /// and transitioning to success or error state upon completion with default options.
    pub async fn promise<T, E, F, S, ErrMsg>(
        future: F,
        loading_msg: impl Into<String>,
        success_fn: S,
        error_fn: ErrMsg,
    ) -> Result<T, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
        S: FnOnce(&T) -> String,
        ErrMsg: FnOnce(&E) -> String,
    {
        promise_with_options(future, loading_msg, success_fn, error_fn, None).await
    }

    /// Dynamically updates the global default toast position.
    pub fn set_position(position: ToastPosition) {
        let mut store = TOAST_STORE.write();
        store.config.position = position;
    }

    /// Dynamically updates global toast configuration settings.
    pub fn configure<F: FnOnce(&mut ToastConfig)>(updater: F) {
        let mut store = TOAST_STORE.write();
        updater(&mut store.config);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_browser_events() {
        assert_eq!(
            parse_toast_browser_event(r#"{"kind":"visibility","visible":true}"#).unwrap(),
            ToastBrowserEvent::Visibility { visible: true }
        );
        assert_eq!(
            parse_toast_browser_event(r#"{"kind":"visibility","visible":false}"#).unwrap(),
            ToastBrowserEvent::Visibility { visible: false }
        );
        assert_eq!(
            parse_toast_browser_event(r#"{"kind":"hotkey","key":"F8"}"#).unwrap(),
            ToastBrowserEvent::Hotkey {
                key: "F8".to_string()
            }
        );
        assert_eq!(
            parse_toast_browser_event(r#"{"kind":"hover","hovered":true}"#).unwrap(),
            ToastBrowserEvent::Hover { hovered: true }
        );
        assert_eq!(
            parse_toast_browser_event(r#"{"kind":"hover","hovered":false}"#).unwrap(),
            ToastBrowserEvent::Hover { hovered: false }
        );
        assert_eq!(
            parse_toast_browser_event(r#"{"kind":"focus","focused":true}"#).unwrap(),
            ToastBrowserEvent::Focus { focused: true }
        );
        assert_eq!(
            parse_toast_browser_event(r#"{"kind":"focus","focused":false}"#).unwrap(),
            ToastBrowserEvent::Focus { focused: false }
        );
        assert_eq!(
            parse_toast_browser_event(r#"{"kind":"swipe_active","swiping":true}"#).unwrap(),
            ToastBrowserEvent::SwipeActive { swiping: true }
        );
        assert_eq!(
            parse_toast_browser_event(r#"{"kind":"swipe_active","swiping":false}"#).unwrap(),
            ToastBrowserEvent::SwipeActive { swiping: false }
        );
        assert_eq!(
            parse_toast_browser_event(r#"{"kind":"swipe_dismiss","id":42}"#).unwrap(),
            ToastBrowserEvent::SwipeDismiss { id: 42 }
        );
        assert_eq!(
            parse_toast_browser_event(r#"{"kind":"stopped"}"#).unwrap(),
            ToastBrowserEvent::Stopped
        );
    }
}

