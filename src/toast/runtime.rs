use super::state::ToastStore;
use super::types::*;
use dioxus::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// Delay duration before evicted toast is removed from store to allow exit animations.
pub const TIME_BEFORE_UNMOUNT: Duration = Duration::from_millis(200);

/// Canonical global signal store powering imperative `toast::*` calls across the application.
pub static TOAST_STORE: GlobalSignal<ToastStore> = GlobalSignal::new(ToastStore::default);

/// Singleton flag ensuring only one long-lived browser event monitor task runs concurrently.
static MONITOR_RUNNING: AtomicBool = AtomicBool::new(false);

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
}

/// Reactive hook subscribing to toast notifications and binding window visibility listeners.
pub fn use_toast_runtime() -> ToastRuntime {
    use_hook(move || {
        if MONITOR_RUNNING
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return;
        }

        let config = TOAST_STORE.peek().config.clone();
        safe_spawn(async move {
            let Some(mut monitor) = crate::toast::browser::start_toast_browser_monitor(
                &config,
                "monoxus-toast-viewport",
            ) else {
                MONITOR_RUNNING.store(false, Ordering::SeqCst);
                return;
            };

            while let Ok(event) =
                crate::toast::browser::recv_toast_browser_event(&mut monitor).await
            {
                match event {
                    crate::toast::browser::ToastBrowserEvent::Visibility(visible) => {
                        let should_pause = TOAST_STORE.peek().config.pause_when_page_is_hidden;
                        if should_pause {
                            TOAST_STORE.write().set_page_hidden(!visible);
                        }
                    }
                    crate::toast::browser::ToastBrowserEvent::Hover(hovered) => {
                        TOAST_STORE.write().set_hovered(hovered);
                    }
                    crate::toast::browser::ToastBrowserEvent::Focus(focused) => {
                        TOAST_STORE.write().set_focused(focused);
                    }
                    crate::toast::browser::ToastBrowserEvent::SwipeActive(swiping) => {
                        TOAST_STORE.write().set_swiping(swiping);
                    }
                    crate::toast::browser::ToastBrowserEvent::SwipeDismiss(id) => {
                        toast::dismiss(Some(id));
                    }
                    crate::toast::browser::ToastBrowserEvent::Hotkey(_) => {
                        // Landmark focus handled directly in browser.js
                    }
                    crate::toast::browser::ToastBrowserEvent::Stopped => break,
                    crate::toast::browser::ToastBrowserEvent::Unknown(_) => {}
                }
            }
            MONITOR_RUNNING.store(false, Ordering::SeqCst);
        });
    });

    ToastRuntime
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
