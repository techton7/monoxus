use super::state::ToastStore;
use super::types::*;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::BTreeMap;
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

/// Launches the browser monitor for visibility change, focus tracking, `Escape` dismissal,
/// and landmark hotkey (`F8`) focus.
///
/// Uses `oxidase::dom::observer::{observe_capture_events, observe_visibility_change}` across
/// Web and Blitz Native substrates without custom JS files or target-conditional branches.
pub fn start_toast_browser_monitor(
    config: &ToastConfig,
    viewport_id: &str,
    on_event: impl FnMut(ToastBrowserEvent) + 'static,
) -> Option<WatcherGuard> {
    use oxidase::dom::observer::{
        CaptureEvent, CaptureEventKind, observe_capture_events, observe_visibility_change,
    };
    use std::rc::Rc;

    let runtime_weak = dioxus::core::Runtime::try_current().map(|rt| std::rc::Rc::downgrade(&rt));
    let scope_id = dioxus::core::Runtime::try_current()
        .as_ref()
        .and_then(|rt| rt.try_current_scope_id());
    let vp_id = viewport_id.to_string();
    let fallback_hotkey = config.hotkey.clone();
    let fallback_pause = config.pause_when_page_is_hidden;

    let on_event_cell: Rc<RefCell<Box<dyn FnMut(ToastBrowserEvent)>>> =
        Rc::new(RefCell::new(Box::new(on_event)));

    let capture_guard = {
        let vp_id = vp_id.clone();
        let runtime_weak = runtime_weak.clone();
        let on_event_cell = on_event_cell.clone();
        observe_capture_events(move |event: CaptureEvent| {
            let handle = || match event.kind {
                CaptureEventKind::KeyDown { key, is_composing } => {
                    if is_composing {
                        return;
                    }
                    if dioxus::core::Runtime::try_current().is_some()
                        && TOAST_STORE.peek().pointer_down_in_viewport
                    {
                        TOAST_STORE.write().clear_viewport_pointer_press();
                    }

                    let active_hotkey = if dioxus::core::Runtime::try_current().is_some() {
                        TOAST_STORE.peek().config.hotkey.clone()
                    } else {
                        fallback_hotkey.clone()
                    };

                    let inside_viewport = event.path_ids.iter().any(|id| id == &vp_id);
                    if crate::toast::hotkey::matches_landmark_hotkey(&key, &active_hotkey) {
                        crate::toast::hotkey::handle_landmark_keydown_with_origin(
                            &key,
                            &active_hotkey,
                            &vp_id,
                            event.target_id.as_deref(),
                            inside_viewport,
                        );
                        if let Ok(mut cb) = on_event_cell.try_borrow_mut() {
                            cb(ToastBrowserEvent::Hotkey { key });
                        }
                    } else if key == "Escape" && dioxus::core::Runtime::try_current().is_some() {
                        if !inside_viewport {
                            return;
                        }

                        let focused_toast_id = event.path_ids.iter().find_map(|id| {
                            id.strip_prefix("monoxus-toast-")
                                .and_then(|raw| raw.parse::<u64>().ok())
                                .map(ToastId)
                        });

                        if let Some(tid) = focused_toast_id {
                            let can_dismiss = TOAST_STORE.peek().toasts.iter().any(|t| {
                                t.id == tid
                                    && t.phase == ToastPhase::Active
                                    && t.options.dismissible != Some(false)
                                    && t.toast_type != ToastType::Loading
                            });
                            if can_dismiss {
                                toast::dismiss(Some(tid));
                                crate::toast::hotkey::handle_post_dismiss_focus(Some(tid), &vp_id);
                            }
                        } else {
                            let front_dismissible = TOAST_STORE
                                .peek()
                                .toasts
                                .iter()
                                .find(|t| {
                                    t.phase == ToastPhase::Active
                                        && t.options.dismissible != Some(false)
                                        && t.toast_type != ToastType::Loading
                                })
                                .map(|t| t.id);
                            if let Some(tid) = front_dismissible {
                                toast::dismiss(Some(tid));
                                crate::toast::hotkey::handle_post_dismiss_focus(None, &vp_id);
                            }
                        }
                    }
                }
                CaptureEventKind::FocusIn => {
                    if dioxus::core::Runtime::try_current().is_none() {
                        return;
                    }
                    let inside_viewport = event.path_ids.iter().any(|id| id == &vp_id);
                    let currently_focused = TOAST_STORE.peek().interaction_state.focused;
                    if !currently_focused && inside_viewport {
                        if TOAST_STORE.peek().has_active_toasts()
                            && !TOAST_STORE.peek().pointer_down_in_viewport
                        {
                            TOAST_STORE.write().set_focused(true);
                            if let Ok(mut cb) = on_event_cell.try_borrow_mut() {
                                cb(ToastBrowserEvent::Focus { focused: true });
                            }
                        }
                    } else if !inside_viewport && currently_focused {
                        {
                            let mut store = TOAST_STORE.write();
                            store.last_focused_element_id = None;
                            store.set_focused(false);
                        }
                        if let Ok(mut cb) = on_event_cell.try_borrow_mut() {
                            cb(ToastBrowserEvent::Focus { focused: false });
                        }
                    }
                }
                CaptureEventKind::PointerDown { .. } => {
                    if dioxus::core::Runtime::try_current().is_none() {
                        return;
                    }
                    let inside_viewport = event.path_ids.iter().any(|id| id == &vp_id);
                    if inside_viewport {
                        TOAST_STORE.write().begin_viewport_pointer_press();
                    } else {
                        if TOAST_STORE.peek().pointer_down_in_viewport {
                            TOAST_STORE.write().clear_viewport_pointer_press();
                        }
                        if TOAST_STORE.peek().interaction_state.focused {
                            let vp_check = vp_id.clone();
                            let on_event_check = on_event_cell.clone();
                            spawn_root_task(async move {
                                let _ = oxidase::frame::next_frame().await;
                                if !crate::toast::hotkey::is_focus_within_element(&vp_check)
                                    && TOAST_STORE.peek().interaction_state.focused
                                {
                                    {
                                        let mut store = TOAST_STORE.write();
                                        store.last_focused_element_id = None;
                                        store.set_focused(false);
                                    }
                                    if let Ok(mut cb) = on_event_check.try_borrow_mut() {
                                        cb(ToastBrowserEvent::Focus { focused: false });
                                    }
                                }
                            });
                        }
                    }
                }
                CaptureEventKind::Click { .. } => {}
            };

            if let Some(rt) = runtime_weak.as_ref().and_then(|w| w.upgrade()) {
                let scope = scope_id.unwrap_or(dioxus::core::ScopeId::ROOT);
                rt.in_scope(scope, handle);
            } else if runtime_weak.is_none() {
                handle();
            }
        })
        .unwrap_or_else(|_| WatcherGuard::noop())
    };

    let visibility_guard = {
        let runtime_for_vis = runtime_weak.clone();
        let on_event_for_vis = on_event_cell.clone();
        observe_visibility_change(move |visibility| {
            let visible = visibility.is_visible();
            let run = || {
                let should_pause = if dioxus::core::Runtime::try_current().is_some() {
                    TOAST_STORE.peek().config.pause_when_page_is_hidden
                } else {
                    fallback_pause
                };
                crate::toast::hotkey::apply_visibility_change(visible, should_pause);
                if let Ok(mut cb) = on_event_for_vis.try_borrow_mut() {
                    cb(ToastBrowserEvent::Visibility { visible });
                }
            };

            if let Some(rt) = runtime_for_vis.as_ref().and_then(|w| w.upgrade()) {
                let scope = scope_id.unwrap_or(dioxus::core::ScopeId::ROOT);
                rt.in_scope(scope, run);
            } else if runtime_for_vis.is_none() {
                run();
            }
        })
        .unwrap_or_else(|_| WatcherGuard::noop())
    };

    Some(WatcherGuard::with_cleanup(
        "toast_browser_monitor",
        0,
        None,
        move || {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                capture_guard.stop();
                visibility_guard.stop();
                if let Ok(mut cb) = on_event_cell.try_borrow_mut() {
                    cb(ToastBrowserEvent::Stopped);
                }
            }));
        },
    ))
}

/// Gracefully signals the browser monitor to unbind all DOM listeners and terminate.
pub fn stop_toast_browser_monitor(watcher: WatcherGuard) {
    watcher.stop();
}

/// Delay duration before evicted toast is removed from store to allow exit animations.
pub const TIME_BEFORE_UNMOUNT: Duration = Duration::from_millis(200);

/// Canonical global signal store powering imperative `toast::*` calls across the application.
pub static TOAST_STORE: GlobalSignal<ToastStore> = GlobalSignal::new(ToastStore::default);

struct ToastTimerNotifyState {
    seq: u64,
    wakers: BTreeMap<u64, std::task::Waker>,
    next_waiter_id: u64,
}

impl ToastTimerNotifyState {
    const fn new() -> Self {
        Self {
            seq: 0,
            wakers: BTreeMap::new(),
            next_waiter_id: 0,
        }
    }
}

thread_local! {
    static TOAST_WATCHER: RefCell<Option<(String, WatcherGuard)>> = const { RefCell::new(None) };
    static TOAST_TIMER_NOTIFY: RefCell<ToastTimerNotifyState> =
        const { RefCell::new(ToastTimerNotifyState::new()) };
}

pub(crate) fn current_toast_timer_seq() -> u64 {
    TOAST_TIMER_NOTIFY
        .try_with(|cell| cell.borrow().seq)
        .unwrap_or(0)
}

pub(crate) fn notify_toast_timers() {
    let wakers: Vec<std::task::Waker> = TOAST_TIMER_NOTIFY
        .try_with(|cell| {
            if let Ok(mut state) = cell.try_borrow_mut() {
                state.seq = state.seq.wrapping_add(1);
                std::mem::take(&mut state.wakers).into_values().collect()
            } else {
                Vec::new()
            }
        })
        .unwrap_or_default();
    for waker in wakers {
        waker.wake();
    }
}

pub(crate) struct ToastTimerChangeFuture {
    seen_seq: u64,
    waiter_id: Option<u64>,
}

pub(crate) fn wait_toast_timer_change(seen_seq: u64) -> ToastTimerChangeFuture {
    ToastTimerChangeFuture {
        seen_seq,
        waiter_id: None,
    }
}

impl std::future::Future for ToastTimerChangeFuture {
    type Output = u64;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.as_mut().get_mut();
        TOAST_TIMER_NOTIFY
            .try_with(|cell| {
                let mut state = cell.borrow_mut();
                if state.seq != this.seen_seq {
                    if let Some(id) = this.waiter_id.take() {
                        state.wakers.remove(&id);
                    }
                    std::task::Poll::Ready(state.seq)
                } else {
                    let id = *this.waiter_id.get_or_insert_with(|| {
                        state.next_waiter_id = state.next_waiter_id.wrapping_add(1);
                        state.next_waiter_id
                    });
                    match state.wakers.get_mut(&id) {
                        Some(existing) => existing.clone_from(cx.waker()),
                        None => {
                            state.wakers.insert(id, cx.waker().clone());
                        }
                    }
                    std::task::Poll::Pending
                }
            })
            .unwrap_or(std::task::Poll::Ready(this.seen_seq))
    }
}

impl Drop for ToastTimerChangeFuture {
    fn drop(&mut self) {
        if let Some(id) = self.waiter_id.take() {
            let _ = TOAST_TIMER_NOTIFY.try_with(|cell| {
                if let Ok(mut state) = cell.try_borrow_mut() {
                    state.wakers.remove(&id);
                }
            });
        }
    }
}

/// Reactive runtime handle for observing and managing toast notifications.
#[derive(Clone, Copy, Debug, Default)]
pub struct ToastRuntime;

use crate::foundation::browser::spawn_root_task;

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

/// Returns current monotonic timestamp in milliseconds across Web and Native hosts.
pub fn toast_now_ms() -> f64 {
    oxidase::time::monotonic_now_ms()
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

        let config = TOAST_STORE.peek().config.clone();

        if let Some(watcher) = start_toast_browser_monitor(&config, viewport_id, {
            move |event| match event {
                ToastBrowserEvent::Hover { hovered } => {
                    TOAST_STORE.write().set_hovered(hovered);
                }
                ToastBrowserEvent::SwipeActive { swiping } => {
                    TOAST_STORE.write().set_swiping(swiping);
                }
                ToastBrowserEvent::SwipeDismiss { id } => {
                    toast::dismiss(Some(ToastId(id)));
                }
                ToastBrowserEvent::Visibility { .. }
                | ToastBrowserEvent::Focus { .. }
                | ToastBrowserEvent::Hotkey { .. }
                | ToastBrowserEvent::Stopped => {}
            }
        }) {
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
    let _ = TOAST_WATCHER.try_with(|slot| {
        if let Ok(mut slot_ref) = slot.try_borrow_mut()
            && let Some((_, watcher)) = slot_ref.take()
        {
            watcher.stop();
        }
    });
}

/// Imperative toast dispatch API.
pub mod toast {
    use super::*;

    async fn wait_for_unmount_delay() {
        if oxidase::frame::is_host_driven() {
            futures_timer::Delay::new(TIME_BEFORE_UNMOUNT).await;
            return;
        }

        let mut delay = Box::pin(futures_timer::Delay::new(TIME_BEFORE_UNMOUNT));
        let mut accumulated = Duration::ZERO;
        while accumulated < TIME_BEFORE_UNMOUNT {
            match futures_util::future::select(
                Box::pin(oxidase::frame::next_frame()),
                delay.as_mut(),
            )
            .await
            {
                futures_util::future::Either::Left((frame, _)) => {
                    accumulated = accumulated.saturating_add(frame.delta);
                }
                futures_util::future::Either::Right(((), _)) => {
                    break;
                }
            }
        }
    }

    fn dispatch_typed(
        toast_type: ToastType,
        title: impl Into<String>,
        options: Option<ToastOptions>,
    ) -> ToastId {
        let options = options.unwrap_or_default();
        let mut store = TOAST_STORE.write();
        let duration = options.duration.unwrap_or(store.config.duration);
        let id_hint = options.id.unwrap_or(ToastId(0));
        let mut item = ToastItem::new(id_hint, toast_type, title, duration, options);
        if store.paused {
            item.paused_at = Some(item.created_at);
        }
        let id = store.add_toast(item);
        let generation = store
            .toasts
            .iter()
            .find(|t| t.id == id)
            .map(|t| t.generation)
            .unwrap_or(0);
        drop(store);

        if toast_type != ToastType::Loading {
            spawn_toast_timer(id, generation);
        }

        id
    }

    enum TimerPollStep {
        Exit,
        WaitPaused,
        Expire,
        WaitRemaining(Duration),
    }

    fn inspect_toast_timer(id: ToastId, generation: u64) -> TimerPollStep {
        let store = TOAST_STORE.peek();
        let Some(item) = store.toasts.iter().find(|t| t.id == id) else {
            return TimerPollStep::Exit;
        };
        if item.generation != generation
            || item.phase != ToastPhase::Active
            || item.toast_type == ToastType::Loading
        {
            return TimerPollStep::Exit;
        }
        if store.paused {
            return TimerPollStep::WaitPaused;
        }
        let now = ToastInstant::now();
        let elapsed = now.saturating_duration_since(item.last_started_at);
        let rem = item.remaining_duration.saturating_sub(elapsed);
        if rem.is_zero() || item.duration.is_zero() {
            TimerPollStep::Expire
        } else {
            TimerPollStep::WaitRemaining(rem)
        }
    }

    fn expire_in_store(
        store: &mut ToastStore,
        id: ToastId,
        auto_close_cb: Option<Callback<()>>,
    ) -> bool {
        if let Some(cb) = auto_close_cb {
            cb.call(());
        }
        store.dismiss_toast(id);
        let emptied = !store.has_active_toasts();
        if emptied {
            store.clear_interaction_if_queue_empty();
        }
        emptied
    }

    fn finalize_expired_toast(id: ToastId, generation: u64, emptied: bool) {
        if emptied {
            crate::toast::hotkey::handle_post_dismiss_focus(
                Some(id),
                "monoxus-toast-viewport",
            );
        }
        spawn_root_task(async move {
            wait_for_unmount_delay().await;
            TOAST_STORE
                .write()
                .remove_toast_if_dismissing_generation(id, generation);
        });
    }

    enum ExpireAttempt {
        Expired,
        Retry,
        Abort,
    }

    fn try_expire_toast(id: ToastId, generation: u64) -> ExpireAttempt {
        let mut store = TOAST_STORE.write();
        if store.paused {
            return ExpireAttempt::Retry;
        }
        let now = ToastInstant::now();
        let Some(item) = store.toasts.iter_mut().find(|t| t.id == id) else {
            return ExpireAttempt::Abort;
        };
        if item.generation != generation
            || item.phase != ToastPhase::Active
            || item.toast_type == ToastType::Loading
        {
            return ExpireAttempt::Abort;
        }
        let elapsed = now.saturating_duration_since(item.last_started_at);
        let remaining_duration = item.remaining_duration.saturating_sub(elapsed);
        if !remaining_duration.is_zero() && !item.duration.is_zero() {
            return ExpireAttempt::Retry;
        }
        item.remaining_duration = Duration::ZERO;
        item.last_started_at = now;
        let auto_close_cb = item.options.on_auto_close;
        let emptied = expire_in_store(&mut store, id, auto_close_cb);
        drop(store);
        finalize_expired_toast(id, generation, emptied);
        ExpireAttempt::Expired
    }

    fn spawn_toast_timer(id: ToastId, generation: u64) {
        spawn_root_task(async move {
            loop {
                let seen_seq = current_toast_timer_seq();
                let step = inspect_toast_timer(id, generation);

                if oxidase::frame::is_host_driven() {
                    match step {
                        TimerPollStep::Exit => break,
                        TimerPollStep::WaitPaused => {
                            wait_toast_timer_change(seen_seq).await;
                        }
                        TimerPollStep::Expire => match try_expire_toast(id, generation) {
                            ExpireAttempt::Expired | ExpireAttempt::Abort => break,
                            ExpireAttempt::Retry => continue,
                        },
                        TimerPollStep::WaitRemaining(rem) => {
                            futures_util::future::select(
                                Box::pin(futures_timer::Delay::new(rem)),
                                Box::pin(wait_toast_timer_change(seen_seq)),
                            )
                            .await;
                        }
                    }
                } else {
                    match step {
                        TimerPollStep::Exit => break,
                        TimerPollStep::WaitPaused => {
                            futures_util::future::select(
                                Box::pin(oxidase::frame::next_frame()),
                                Box::pin(wait_toast_timer_change(seen_seq)),
                            )
                            .await;
                        }
                        TimerPollStep::Expire => match try_expire_toast(id, generation) {
                            ExpireAttempt::Expired | ExpireAttempt::Abort => break,
                            ExpireAttempt::Retry => continue,
                        },
                        TimerPollStep::WaitRemaining(rem) => {
                            let timer_or_change = futures_util::future::select(
                                Box::pin(futures_timer::Delay::new(rem)),
                                Box::pin(wait_toast_timer_change(seen_seq)),
                            );
                            match futures_util::future::select(
                                Box::pin(oxidase::frame::next_frame()),
                                timer_or_change,
                            )
                            .await
                            {
                                futures_util::future::Either::Left((frame, _)) => {
                                    let mut store = TOAST_STORE.write();
                                    if store.paused {
                                        continue;
                                    }
                                    let now = ToastInstant::now();
                                    let Some(item) = store.toasts.iter_mut().find(|t| t.id == id)
                                    else {
                                        break;
                                    };
                                    if item.generation != generation
                                        || item.phase != ToastPhase::Active
                                        || item.toast_type == ToastType::Loading
                                    {
                                        break;
                                    }
                                    item.last_started_at = now;
                                    item.remaining_duration =
                                        item.remaining_duration.saturating_sub(frame.delta);
                                    if item.remaining_duration.is_zero()
                                        || item.duration.is_zero()
                                    {
                                        let auto_close_cb = item.options.on_auto_close;
                                        let emptied =
                                            expire_in_store(&mut store, id, auto_close_cb);
                                        drop(store);
                                        finalize_expired_toast(id, generation, emptied);
                                        break;
                                    }
                                }
                                futures_util::future::Either::Right(_) => {
                                    continue;
                                }
                            }
                        }
                    }
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
            let generation = store
                .toasts
                .iter()
                .find(|t| t.id == target_id && t.phase == ToastPhase::Active)
                .map(|t| t.generation);
            if let Some(generation) = generation
                && store.dismiss_toast(target_id)
            {
                let emptied = !store.has_active_toasts();
                if emptied {
                    store.clear_interaction_if_queue_empty();
                }
                drop(store);
                if emptied {
                    crate::toast::hotkey::handle_post_dismiss_focus(
                        id,
                        "monoxus-toast-viewport",
                    );
                }
                spawn_root_task(async move {
                    wait_for_unmount_delay().await;
                    TOAST_STORE
                        .write()
                        .remove_toast_if_dismissing_generation(target_id, generation);
                });
            }
        } else {
            let active_targets: Vec<(ToastId, u64)> = store
                .toasts
                .iter()
                .filter(|t| t.phase == ToastPhase::Active)
                .map(|t| (t.id, t.generation))
                .collect();
            store.dismiss_all();
            let emptied = !store.has_active_toasts();
            if emptied {
                store.clear_interaction_if_queue_empty();
            }
            drop(store);
            if emptied {
                crate::toast::hotkey::handle_post_dismiss_focus(
                    None,
                    "monoxus-toast-viewport",
                );
            }
            for (active_id, generation) in active_targets {
                spawn_root_task(async move {
                    wait_for_unmount_delay().await;
                    TOAST_STORE
                        .write()
                        .remove_toast_if_dismissing_generation(active_id, generation);
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
        let next_type_and_msg = match &outcome {
            Ok(data) => (ToastType::Success, success_fn(data)),
            Err(err) => (ToastType::Error, error_fn(err)),
        };

        let mut next_gen = None;
        {
            let mut store = TOAST_STORE.write();
            let is_paused = store.paused;
            if let Some(item) = store.toasts.iter_mut().find(|t| t.id == id) {
                item.generation = item.generation.wrapping_add(1);
                item.content_generation = item.content_generation.wrapping_add(1);
                item.toast_type = next_type_and_msg.0;
                item.title = next_type_and_msg.1;
                item.phase = ToastPhase::Active;
                item.remaining_duration = item.duration;
                item.last_started_at = now;
                item.paused_at = if is_paused { Some(now) } else { None };
                next_gen = Some(item.generation);
            }
        }

        if let Some(generation) = next_gen {
            notify_toast_timers();
            spawn_toast_timer(id, generation);
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

    #[test]
    fn timer_notify_wakes_and_drop_removes_cancelled_waiter() {
        use std::future::Future;
        use std::pin::pin;
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::task::{Context, Poll, Wake, Waker};

        struct CountWaker(AtomicUsize);
        impl Wake for CountWaker {
            fn wake(self: Arc<Self>) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
            fn wake_by_ref(self: &Arc<Self>) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }

        let initial_seq = current_toast_timer_seq();
        let counter = Arc::new(CountWaker(AtomicUsize::new(0)));
        let waker = Waker::from(counter.clone());
        let mut cx = Context::from_waker(&waker);

        // 1. Cancelled select waiter must remove its waker on Drop (zero leak)
        {
            let mut cancelled_fut = pin!(wait_toast_timer_change(initial_seq));
            assert_eq!(cancelled_fut.as_mut().poll(&mut cx), Poll::Pending);
            let waker_count = TOAST_TIMER_NOTIFY.with(|c| c.borrow().wakers.len());
            assert_eq!(waker_count, 1);
        }
        let after_drop_count = TOAST_TIMER_NOTIFY.with(|c| c.borrow().wakers.len());
        assert_eq!(
            after_drop_count, 0,
            "Dropping ToastTimerChangeFuture must remove registered waker"
        );

        // 2. Active waiter wakes upon notify_toast_timers() and resolves next sequence
        let mut active_fut = pin!(wait_toast_timer_change(initial_seq));
        assert_eq!(active_fut.as_mut().poll(&mut cx), Poll::Pending);
        notify_toast_timers();
        assert_eq!(counter.0.load(Ordering::SeqCst), 1);
        assert_eq!(
            active_fut.as_mut().poll(&mut cx),
            Poll::Ready(initial_seq.wrapping_add(1))
        );
        assert_eq!(TOAST_TIMER_NOTIFY.with(|c| c.borrow().wakers.len()), 0);
    }
}
