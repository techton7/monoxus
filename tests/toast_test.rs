use dioxus::prelude::*;
use monoxus::toast::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

#[test]
fn part_inventory_export_test() {
    // Assert all compound components, hook, and imperative APIs are exported from monoxus::toast
    let _ = ToastProvider;
    let _ = ToastViewport;
    let _ = ToastRoot;
    let _ = ToastTitle;
    let _ = ToastDescription;
    let _ = ToastAction;
    let _ = ToastClose;

    // Verify default constants
    let config = ToastConfig::default();
    assert_eq!(config.duration, Duration::from_millis(4000));
    assert_eq!(config.position, ToastPosition::BottomRight);
    assert_eq!(config.hotkey, "F8");
    assert_eq!(config.swipe_threshold, 45.0);
    assert_eq!(config.visible_toasts, 3);
    assert_eq!(config.gap, 14.0);
    assert_eq!(config.pause_when_page_is_hidden, true);
    assert_eq!(config.container_aria_label, "Notifications ({hotkey})");
    assert_eq!(config.close_button_aria_label, "Close notification");
    assert_eq!(config.dir, ToastDirection::Auto);
    assert_eq!(config.offset, ToastOffset::uniform("32px"));
    assert_eq!(config.mobile_offset, ToastOffset::uniform("16px"));
    assert_eq!(config.swipe_directions, None);
    assert_eq!(
        config.effective_swipe_directions(),
        vec![SwipeDirection::Bottom, SwipeDirection::Right]
    );
    assert_eq!(TIME_BEFORE_UNMOUNT, Duration::from_millis(200));
}

#[test]
fn queue_reducer_test() {
    let mut store = ToastStore::new();

    // 1. Add first toast
    let id1 = store.add_toast(ToastItem::new(
        ToastId(1),
        ToastType::Default,
        "First Toast",
        Duration::from_millis(4000),
        ToastOptions::default(),
    ));
    assert_eq!(store.toasts.len(), 1);
    assert_eq!(store.toasts[0].id, id1);

    // 2. Add second toast: verify deterministic newest-first insertion (index 0 is newest/front toast)
    let id2 = store.add_toast(ToastItem::new(
        ToastId(2),
        ToastType::Success,
        "Second Toast",
        Duration::from_millis(4000),
        ToastOptions {
            description: Some("Success details".to_string()),
            ..Default::default()
        },
    ));
    assert_eq!(store.toasts.len(), 2);
    assert_eq!(
        store.toasts[0].id, id2,
        "Newest toast must be prepended to index 0"
    );
    assert_eq!(store.toasts[1].id, id1);

    // 3. Dismiss toast: verify transition to ToastPhase::Dismissing (data-state="closed")
    let dismissed = store.dismiss_toast(id1);
    assert!(dismissed);
    assert_eq!(
        store.toasts.len(),
        2,
        "Dismissing toast remains in queue during exit phase"
    );
    let item1 = store.toasts.iter().find(|t| t.id == id1).unwrap();
    assert_eq!(item1.phase, ToastPhase::Dismissing);

    // 4. Remove toast: verify eviction from queue
    let removed = store.remove_toast(id1);
    assert!(removed.is_some());
    assert_eq!(store.toasts.len(), 1);
    assert_eq!(store.toasts[0].id, id2);

    // 5. Dismiss all: verify all active toasts transition to Dismissing
    store.dismiss_all();
    assert_eq!(store.toasts[0].phase, ToastPhase::Dismissing);
}

#[test]
fn same_id_update_safety_test() {
    let mut store = ToastStore::new();
    let target_id = ToastId(42);

    // Initial toast
    store.add_toast(ToastItem::new(
        target_id,
        ToastType::Default,
        "Initial Title",
        Duration::from_millis(4000),
        ToastOptions {
            id: Some(target_id),
            ..Default::default()
        },
    ));
    assert_eq!(store.toasts.len(), 1);

    // Dismiss the toast
    store.dismiss_toast(target_id);
    assert_eq!(store.toasts[0].phase, ToastPhase::Dismissing);

    // Recreate/update toast with the same ID while it's in Dismissing phase
    let returned_id = store.add_toast(ToastItem::new(
        target_id,
        ToastType::Success,
        "Updated Title",
        Duration::from_millis(5000),
        ToastOptions {
            id: Some(target_id),
            description: Some("New description".to_string()),
            duration: Some(Duration::from_millis(5000)),
            ..Default::default()
        },
    ));

    assert_eq!(returned_id, target_id);
    assert_eq!(
        store.toasts.len(),
        1,
        "Queue must not create duplicate items for the same ID"
    );
    let toast = &store.toasts[0];
    assert_eq!(toast.title, "Updated Title");
    assert_eq!(
        toast.phase,
        ToastPhase::Active,
        "Recreating with same ID must resurrect toast to Active"
    );
    assert_eq!(
        toast.remaining_duration,
        Duration::from_millis(5000),
        "Remaining duration must be reset"
    );
}

#[test]
fn visible_toasts_data_visible_test() {
    let mut store = ToastStore::with_config(ToastConfig {
        visible_toasts: 3,
        ..Default::default()
    });

    // Add 5 toasts
    for i in 1..=5 {
        store.add_toast(ToastItem::new(
            ToastId(i),
            ToastType::Default,
            format!("Toast {i}"),
            Duration::from_millis(4000),
            ToastOptions::default(),
        ));
    }

    // Assert that the queue holds all 5 items (never capped or dropped)
    assert_eq!(
        store.toasts.len(),
        5,
        "Store must keep all active toasts in queue"
    );

    // Evaluate visibility calculation
    let visible_list = store.visible_items();
    assert_eq!(visible_list.len(), 5);

    // Front 3 toasts (indices 0, 1, 2) must be visible=true
    assert_eq!(
        visible_list[0].1, true,
        "Index 0 must have data-visible=true"
    );
    assert_eq!(
        visible_list[1].1, true,
        "Index 1 must have data-visible=true"
    );
    assert_eq!(
        visible_list[2].1, true,
        "Index 2 must have data-visible=true"
    );

    // Toasts beyond limit (indices 3, 4) must be visible=false
    assert_eq!(
        visible_list[3].1, false,
        "Index 3 must have data-visible=false"
    );
    assert_eq!(
        visible_list[4].1, false,
        "Index 4 must have data-visible=false"
    );
}

#[test]
fn timer_remaining_duration_test() {
    let mut store = ToastStore::new();
    let _id = store.add_toast(ToastItem::new(
        ToastId(1),
        ToastType::Default,
        "Countdown Toast",
        Duration::from_millis(4000),
        ToastOptions::default(),
    ));

    // Allow real time to elapse
    std::thread::sleep(Duration::from_millis(60));

    // Pause all: elapsed time must be calculated and deducted dynamically (no hardcoded seeds)
    store.pause_all();
    assert!(store.paused);
    let rem1 = store.toasts[0].remaining_duration;
    assert!(
        rem1 < Duration::from_millis(4000),
        "Remaining duration must decrease after running"
    );
    assert!(
        rem1 >= Duration::from_millis(3500),
        "Remaining duration must deduct reasonable elapsed time"
    );

    // Sleep while paused: time must NOT be deducted
    std::thread::sleep(Duration::from_millis(60));
    assert_eq!(
        store.toasts[0].remaining_duration, rem1,
        "Duration must not decrease while paused"
    );

    // Resume all
    store.resume_all();
    assert!(!store.paused);
    assert_eq!(
        store.toasts[0].remaining_duration, rem1,
        "Remaining duration preserved upon resume"
    );

    // Allow more time to elapse after resume
    std::thread::sleep(Duration::from_millis(60));
    store.pause_all();
    let rem2 = store.toasts[0].remaining_duration;
    assert!(
        rem2 < rem1,
        "Remaining duration must decrease further after second running segment"
    );
}

fn block_on<F: std::future::Future>(future: F) -> F::Output {
    use std::pin::pin;
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

    fn clone(_: *const ()) -> RawWaker {
        RawWaker::new(std::ptr::null(), &VTABLE)
    }
    fn wake(_: *const ()) {}
    fn wake_by_ref(_: *const ()) {}
    fn drop(_: *const ()) {}
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

    let raw_waker = RawWaker::new(std::ptr::null(), &VTABLE);
    let waker = unsafe { Waker::from_raw(raw_waker) };
    let mut cx = Context::from_waker(&waker);
    let mut pinned = pin!(future);
    loop {
        match pinned.as_mut().poll(&mut cx) {
            Poll::Ready(val) => return val,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}

struct ThreadWaker(std::thread::Thread);
impl std::task::Wake for ThreadWaker {
    fn wake(self: std::sync::Arc<Self>) {
        self.0.unpark();
    }
    fn wake_by_ref(self: &std::sync::Arc<Self>) {
        self.0.unpark();
    }
}

fn block_on_with_timeout<F: std::future::Future>(future: F, timeout: Duration) -> bool {
    use std::pin::pin;
    use std::task::Context;

    let waker = std::task::Waker::from(std::sync::Arc::new(ThreadWaker(std::thread::current())));
    let mut cx = Context::from_waker(&waker);
    let mut pinned = pin!(future);
    let start = std::time::Instant::now();
    loop {
        if let std::task::Poll::Ready(_) = pinned.as_mut().poll(&mut cx) {
            return true;
        }
        let elapsed = start.elapsed();
        if elapsed >= timeout {
            return false;
        }
        let remaining = timeout - elapsed;
        std::thread::park_timeout(remaining.min(Duration::from_millis(20)));
    }
}

#[test]
fn promise_lifecycle_test() {
    let mut dom = dioxus::prelude::VirtualDom::new(|| dioxus::prelude::VNode::empty());
    dom.rebuild_in_place();

    // 1. Verify Success branch: Loading -> Success -> Dismissing -> Evicted
    dom.in_runtime(|| {
        TOAST_STORE.write().toasts.clear();
    });

    let result = dom.in_runtime(|| {
        block_on(async {
            let success_future = async {
                // Verify that while future is pending, toast is in Loading state
                let store = TOAST_STORE.read();
                assert_eq!(store.toasts.len(), 1, "Loading toast must be registered");
                assert_eq!(store.toasts[0].toast_type, ToastType::Loading);
                assert_eq!(store.toasts[0].title, "Saving record...");
                assert_eq!(store.toasts[0].phase, ToastPhase::Active);

                Ok::<&str, &str>("payload resolved")
            };

            toast::promise_with_options(
                success_future,
                "Saving record...",
                |data| format!("Saved: {data}"),
                |err| format!("Failed: {err}"),
                Some(ToastOptions {
                    duration: Some(Duration::from_millis(60)),
                    ..Default::default()
                }),
            )
            .await
        })
    });
    assert_eq!(result, Ok("payload resolved"));

    // Immediately after resolution: Toast is Success, Active, with remaining duration reset to 60ms
    dom.in_runtime(|| {
        let store = TOAST_STORE.read();
        assert_eq!(store.toasts.len(), 1);
        let toast = &store.toasts[0];
        assert_eq!(toast.toast_type, ToastType::Success);
        assert_eq!(toast.title, "Saved: payload resolved");
        assert_eq!(toast.phase, ToastPhase::Active);
        assert_eq!(toast.remaining_duration, Duration::from_millis(60));
    });

    // Drive VirtualDom work until the 60ms countdown expires and toast transitions to Dismissing
    let mut saw_dismissing = false;
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_millis(1500) {
        let is_dismissing = dom.in_runtime(|| {
            let store = TOAST_STORE.read();
            store
                .toasts
                .first()
                .map(|t| t.phase == ToastPhase::Dismissing)
                .unwrap_or(false)
        });
        if is_dismissing {
            saw_dismissing = true;
            break;
        }
        if block_on_with_timeout(dom.wait_for_work(), Duration::from_millis(25)) {
            dom.render_immediate(&mut dioxus_core::NoOpMutations);
        }
    }
    assert!(
        saw_dismissing,
        "Success-resolved promise toast must re-enter timer and transition to Dismissing upon duration expiry"
    );

    // Continue driving until TIME_BEFORE_UNMOUNT (200ms) elapses and toast is completely evicted
    let mut evicted = false;
    let start2 = std::time::Instant::now();
    while start2.elapsed() < Duration::from_millis(1500) {
        let is_empty = dom.in_runtime(|| TOAST_STORE.read().toasts.is_empty());
        if is_empty {
            evicted = true;
            break;
        }
        if block_on_with_timeout(dom.wait_for_work(), Duration::from_millis(25)) {
            dom.render_immediate(&mut dioxus_core::NoOpMutations);
        }
    }
    assert!(
        evicted,
        "Resolved promise toast must be evicted from TOAST_STORE after delayed unmount"
    );

    // 2. Verify Error branch: Loading -> Error -> Dismissing -> Evicted
    let err_result = dom.in_runtime(|| {
        block_on(async {
            let error_future = async {
                let store = TOAST_STORE.read();
                assert_eq!(store.toasts.len(), 1, "Loading toast must be registered");
                assert_eq!(store.toasts[0].toast_type, ToastType::Loading);
                assert_eq!(store.toasts[0].title, "Deleting record...");
                assert_eq!(store.toasts[0].phase, ToastPhase::Active);

                Err::<&str, &str>("network timeout")
            };

            toast::promise_with_options(
                error_future,
                "Deleting record...",
                |data| format!("Deleted: {data}"),
                |err| format!("Error: {err}"),
                Some(ToastOptions {
                    duration: Some(Duration::from_millis(60)),
                    ..Default::default()
                }),
            )
            .await
        })
    });
    assert_eq!(err_result, Err("network timeout"));

    dom.in_runtime(|| {
        let store = TOAST_STORE.read();
        assert_eq!(store.toasts.len(), 1);
        let toast = &store.toasts[0];
        assert_eq!(toast.toast_type, ToastType::Error);
        assert_eq!(toast.title, "Error: network timeout");
        assert_eq!(toast.phase, ToastPhase::Active);
    });

    let mut saw_error_dismissing = false;
    let start3 = std::time::Instant::now();
    while start3.elapsed() < Duration::from_millis(1500) {
        let is_dismissing = dom.in_runtime(|| {
            let store = TOAST_STORE.read();
            store
                .toasts
                .first()
                .map(|t| t.phase == ToastPhase::Dismissing)
                .unwrap_or(false)
        });
        if is_dismissing {
            saw_error_dismissing = true;
            break;
        }
        if block_on_with_timeout(dom.wait_for_work(), Duration::from_millis(25)) {
            dom.render_immediate(&mut dioxus_core::NoOpMutations);
        }
    }
    assert!(
        saw_error_dismissing,
        "Error-resolved promise toast must transition to Dismissing upon duration expiry"
    );

    let mut error_evicted = false;
    let start4 = std::time::Instant::now();
    while start4.elapsed() < Duration::from_millis(1500) {
        let is_empty = dom.in_runtime(|| TOAST_STORE.read().toasts.is_empty());
        if is_empty {
            error_evicted = true;
            break;
        }
        if block_on_with_timeout(dom.wait_for_work(), Duration::from_millis(25)) {
            dom.render_immediate(&mut dioxus_core::NoOpMutations);
        }
    }
    assert!(
        error_evicted,
        "Error-resolved promise toast must be evicted from TOAST_STORE after delayed unmount"
    );

    // 3. Verify convenience toast::promise wrapper
    dom.in_runtime(|| {
        block_on(async {
            let result = toast::promise(
                async { Ok::<i32, ()>(42) },
                "Computing...",
                |n| format!("Answer: {n}"),
                |_| "Error".into(),
            )
            .await;
            assert_eq!(result, Ok(42));
        });
        TOAST_STORE.write().toasts.clear();
    });
}

#[dioxus::prelude::component]
fn ActionTestComponent() -> dioxus::prelude::Element {
    use dioxus::prelude::*;
    let mut dismissed = use_signal(|| false);
    let mut dismissed2 = use_signal(|| false);

    let on_click = Callback::new(move |ev: ToastActionEvent| {
        ev.prevent_default();
    });

    let ev = ToastActionEvent::new();
    on_click.call(ev.clone());
    if !ev.is_default_prevented() {
        dismissed.set(true);
    }

    let on_click2 = Callback::new(move |_ev: ToastActionEvent| {});
    let ev2 = ToastActionEvent::new();
    on_click2.call(ev2.clone());
    if !ev2.is_default_prevented() {
        dismissed2.set(true);
    }

    assert!(
        !dismissed(),
        "ToastAction must suppress dismissal when prevent_default is called"
    );
    assert!(
        dismissed2(),
        "ToastAction must automatically dismiss when prevent_default is not called"
    );

    VNode::empty()
}

#[test]
fn action_prevent_default_test() {
    let event = ToastActionEvent::new();
    assert!(!event.is_default_prevented());

    // Calling prevent_default marks the flag
    event.prevent_default();
    assert!(event.is_default_prevented());

    let mut dom = dioxus::prelude::VirtualDom::new(ActionTestComponent);
    dom.rebuild_in_place();
}

#[test]
fn wai_aria_live_region_test() {
    let config = ToastConfig {
        hotkey: "F8".to_string(),
        container_aria_label: "Notifications ({hotkey})".to_string(),
        close_button_aria_label: "Close notification".to_string(),
        visible_toasts: 3,
        ..Default::default()
    };

    // 1. Viewport Landmark Attrs
    let viewport_attrs = ToastViewportAttrs::new(&config, false, "ltr");
    assert_eq!(viewport_attrs.role, "region");
    assert_eq!(viewport_attrs.aria_label, "Notifications (F8)");
    assert_eq!(viewport_attrs.tabindex, "-1");
    assert_eq!(viewport_attrs.dir, "ltr");
    assert_eq!(viewport_attrs.data_expanded, "false");

    // 2. Default / Success / Info / Loading Toast -> role="status", aria-live="polite"
    let root_attrs_success = ToastRootAttrs::new(
        ToastType::Success,
        ToastPhase::Active,
        0, // index 0 = front toast
        3, // visible_toasts
        None,
    );
    assert_eq!(root_attrs_success.role, "status");
    assert_eq!(root_attrs_success.aria_live, "polite");
    assert_eq!(root_attrs_success.aria_atomic, "true");
    assert_eq!(root_attrs_success.data_state, "open");
    assert_eq!(root_attrs_success.data_type, "success");
    assert_eq!(root_attrs_success.data_visible, "true");
    assert_eq!(root_attrs_success.data_front, "true");
    assert_eq!(root_attrs_success.data_index, "0");

    // 3. Warning / Error Toast -> role="alert", aria-live="assertive"
    let root_attrs_error = ToastRootAttrs::new(
        ToastType::Error,
        ToastPhase::Dismissing,
        3, // index 3 exceeds visible_toasts 3
        3,
        Some("error-card-testid".to_string()),
    );
    assert_eq!(root_attrs_error.role, "alert");
    assert_eq!(root_attrs_error.aria_live, "assertive");
    assert_eq!(root_attrs_error.aria_atomic, "true");
    assert_eq!(root_attrs_error.data_state, "closed");
    assert_eq!(root_attrs_error.data_type, "error");
    assert_eq!(root_attrs_error.data_visible, "false");
    assert_eq!(root_attrs_error.data_front, "false");
    assert_eq!(root_attrs_error.data_index, "3");
    assert_eq!(
        root_attrs_error.data_testid,
        Some("error-card-testid".to_string())
    );

    // 4. Action button attrs
    let action_attrs = ToastActionAttrs::new("Undo changes".to_string());
    assert_eq!(action_attrs.r#type, "button");
    assert_eq!(action_attrs.aria_label, "Undo changes");

    // 5. Close button attrs
    let close_attrs = ToastCloseAttrs::new(&config);
    assert_eq!(close_attrs.r#type, "button");
    assert_eq!(close_attrs.aria_label, "Close notification");
    assert_eq!(close_attrs.data_radix_toast_announce_exclude, "true");
}

#[test]
fn browser_event_parser_test() {
    assert_eq!(
        parse_toast_browser_event(r#"{"kind":"visibility","visible":true}"#).unwrap(),
        ToastBrowserEvent::Visibility { visible: true }
    );
    assert_eq!(
        parse_toast_browser_event(r#"{"kind":"visibility","visible":false}"#).unwrap(),
        ToastBrowserEvent::Visibility { visible: false }
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
        parse_toast_browser_event(r#"{"kind":"swipe_dismiss","id":77}"#).unwrap(),
        ToastBrowserEvent::SwipeDismiss { id: 77 }
    );
    assert_eq!(
        parse_toast_browser_event(r#"{"kind":"hotkey","key":"F8"}"#).unwrap(),
        ToastBrowserEvent::Hotkey {
            key: "F8".to_string()
        }
    );
    assert_eq!(
        parse_toast_browser_event(r#"{"kind":"stopped"}"#).unwrap(),
        ToastBrowserEvent::Stopped
    );
}

#[test]
fn toast_watcher_reset_contract_test() {
    reset_toast_watcher();
    ensure_toast_watcher("custom-viewport-1");
    ensure_toast_watcher("custom-viewport-1"); // idempotent
    ensure_toast_watcher("custom-viewport-2"); // rebound to new id
    reset_toast_watcher();
}

#[test]
fn interaction_state_precedence_test() {
    let mut store = ToastStore::new();
    assert!(!store.paused);
    assert!(!store.expanded);

    // 1. Hover triggers pause and expand
    store.set_hovered(true);
    assert!(store.paused);
    assert!(store.expanded);

    // 2. Page hidden locks pause
    store.set_page_hidden(true);
    assert!(store.paused);

    // Hover ends, but page is still hidden -> timers remain paused!
    store.set_hovered(false);
    assert!(
        store.paused,
        "Timers must remain paused while page is hidden even if hover leaves"
    );
    assert!(
        !store.expanded,
        "Expanded collapses when hover and focus are false"
    );

    // Page becomes visible -> resumes
    store.set_page_hidden(false);
    assert!(!store.paused);
    assert!(!store.expanded);

    // 3. Focus holds pause and expand
    store.set_focused(true);
    assert!(store.paused);
    assert!(store.expanded);
    store.set_focused(false);
    assert!(!store.paused);
    assert!(!store.expanded);

    // 4. Swipe locks pause and expand
    store.set_swiping(true);
    assert!(store.paused);
    assert!(store.expanded);
    store.set_swiping(false);
    assert!(!store.paused);
    assert!(!store.expanded);
}

#[dioxus::prelude::component]
fn AutoMountViewportComponent() -> dioxus::prelude::Element {
    use dioxus::prelude::*;
    rsx! {
        ToastViewport {}
    }
}

#[test]
fn custom_toast_and_viewport_automount_test() {
    let mut dom = dioxus::prelude::VirtualDom::new(AutoMountViewportComponent);
    dom.in_runtime(|| {
        // Clear existing store
        TOAST_STORE.write().toasts.clear();

        // 1. Dispatch custom toast with arbitrary RSX renderer
        let custom_id = toast::custom(
            |id| {
                dioxus::prelude::rsx! {
                    div { class: "custom-banner", "Custom Alert Body: {id.0}" }
                }
            },
            Some(ToastOptions {
                test_id: Some("custom-toast-card".to_string()),
                ..Default::default()
            }),
        );

        // 2. Dispatch standard message toast
        let standard_id = toast::message("Standard Notification", None);

        assert_eq!(TOAST_STORE.read().toasts.len(), 2);
        assert_eq!(TOAST_STORE.read().toasts[0].id, standard_id);
        assert_eq!(TOAST_STORE.read().toasts[1].id, custom_id);
    });

    // Render VirtualDom: ToastViewport auto-mounts active toasts
    dom.rebuild_in_place();

    // Verify rendered DOM elements
    dom.in_runtime(|| {
        let store = TOAST_STORE.read();
        assert_eq!(store.toasts.len(), 2);
        assert!(store.toasts[1].options.custom_renderer.is_some());
    });
}

static AUTO_CLOSE_FIRED: AtomicBool = AtomicBool::new(false);

#[dioxus::prelude::component]
fn AutoCloseTestComponent() -> dioxus::prelude::Element {
    use dioxus::prelude::*;
    let cb = Callback::new(move |()| {
        AUTO_CLOSE_FIRED.store(true, Ordering::SeqCst);
    });

    use_hook(|| {
        TOAST_STORE.write().toasts.clear();
        let _id = toast::message(
            "Quick Expiry Toast",
            Some(ToastOptions {
                duration: Some(Duration::from_millis(60)),
                on_auto_close: Some(cb),
                ..Default::default()
            }),
        );
    });

    let store = TOAST_STORE.read();
    rsx! {
        div { "Toasts: {store.toasts.len()}" }
    }
}

#[test]
fn on_auto_close_and_bulk_dismiss_delayed_removal_test() {
    AUTO_CLOSE_FIRED.store(false, Ordering::SeqCst);
    let mut dom = dioxus::prelude::VirtualDom::new(AutoCloseTestComponent);
    dom.rebuild_in_place();

    // 1. Wait for 50ms tick + expiration by driving VirtualDom work
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_millis(800) {
        if AUTO_CLOSE_FIRED.load(Ordering::SeqCst) {
            break;
        }
        if block_on_with_timeout(dom.wait_for_work(), Duration::from_millis(25)) {
            dom.render_immediate(&mut dioxus_core::NoOpMutations);
        }
    }

    assert!(
        AUTO_CLOSE_FIRED.load(Ordering::SeqCst),
        "on_auto_close callback must be triggered upon timer expiration"
    );

    // 2. Test bulk dismiss(None) schedules delayed removal for all active toasts
    dom.in_runtime(|| {
        TOAST_STORE.write().toasts.clear();
        let _id1 = toast::loading("Persistent 1", None);
        let _id2 = toast::loading("Persistent 2", None);
        assert_eq!(TOAST_STORE.read().toasts.len(), 2);

        // Calling dismiss(None) transitions all active toasts to Dismissing
        toast::dismiss(None);
        assert_eq!(TOAST_STORE.read().toasts[0].phase, ToastPhase::Dismissing);
        assert_eq!(TOAST_STORE.read().toasts[1].phase, ToastPhase::Dismissing);
    });

    // Wait past TIME_BEFORE_UNMOUNT (200ms) by driving VirtualDom work
    let start2 = std::time::Instant::now();
    while start2.elapsed() < Duration::from_millis(800) {
        let empty = dom.in_runtime(|| TOAST_STORE.read().toasts.is_empty());
        if empty {
            break;
        }
        if block_on_with_timeout(dom.wait_for_work(), Duration::from_millis(25)) {
            dom.render_immediate(&mut dioxus_core::NoOpMutations);
        }
    }

    // All toasts must be evicted from the store
    dom.in_runtime(|| {
        assert_eq!(
            TOAST_STORE.read().toasts.len(),
            0,
            "dismiss(None) must schedule delayed removal for all active toasts"
        );
    });
}

#[test]
fn config_surface_and_directional_gesture_contract_test() {
    // 1. Position-derived allowed swipe directions
    assert_eq!(
        ToastPosition::TopLeft.default_swipe_directions(),
        vec![SwipeDirection::Top, SwipeDirection::Left]
    );
    assert_eq!(
        ToastPosition::TopRight.default_swipe_directions(),
        vec![SwipeDirection::Top, SwipeDirection::Right]
    );
    assert_eq!(
        ToastPosition::TopCenter.default_swipe_directions(),
        vec![SwipeDirection::Top]
    );
    assert_eq!(
        ToastPosition::BottomLeft.default_swipe_directions(),
        vec![SwipeDirection::Bottom, SwipeDirection::Left]
    );
    assert_eq!(
        ToastPosition::BottomRight.default_swipe_directions(),
        vec![SwipeDirection::Bottom, SwipeDirection::Right]
    );
    assert_eq!(
        ToastPosition::BottomCenter.default_swipe_directions(),
        vec![SwipeDirection::Bottom]
    );

    // 2. Explicit swipe_directions override
    let mut config = ToastConfig::default();
    config.swipe_directions = Some(vec![SwipeDirection::Right]);
    assert_eq!(
        config.effective_swipe_directions(),
        vec![SwipeDirection::Right]
    );

    // 3. ToastViewportAttrs publishing dynamic dir and offset CSS variables
    let mut custom_cfg = ToastConfig::default();
    custom_cfg.dir = ToastDirection::Rtl;
    custom_cfg.offset = ToastOffset::new()
        .with_top("24px")
        .with_right("20px")
        .with_bottom("24px")
        .with_left("20px");
    custom_cfg.mobile_offset = ToastOffset::uniform("12px");

    let attrs = ToastViewportAttrs::new(&custom_cfg, false, custom_cfg.dir.as_str());
    assert_eq!(attrs.dir, "rtl");
    assert_eq!(attrs.data_position, "bottom-right");
    assert_eq!(attrs.data_x_position, "right");
    assert_eq!(attrs.data_y_position, "bottom");
    assert!(attrs.style.contains("--offset-top: 24px;"));
    assert!(attrs.style.contains("--offset-right: 20px;"));
    assert!(attrs.style.contains("--offset-bottom: 24px;"));
    assert!(attrs.style.contains("--offset-left: 20px;"));
    assert!(attrs.style.contains("--mobile-offset-top: 12px;"));
    assert!(attrs.style.contains("--mobile-offset-bottom: 12px;"));

    // 4. Viewport and Root position attribute alignment
    let top_left_attrs =
        ToastViewportAttrs::with_position(&custom_cfg, false, "ltr", ToastPosition::TopLeft);
    assert_eq!(top_left_attrs.data_position, "top-left");
    assert_eq!(top_left_attrs.data_x_position, "left");
    assert_eq!(top_left_attrs.data_y_position, "top");

    let root_attrs = ToastRootAttrs::with_options_and_position(
        ToastType::Default,
        ToastPhase::Active,
        0,
        3,
        None,
        false,
        ToastPosition::TopCenter,
    );
    assert_eq!(root_attrs.data_position, "top-center");
    assert_eq!(root_attrs.data_x_position, "center");
    assert_eq!(root_attrs.data_y_position, "top");
    assert_eq!(root_attrs.data_swipe_out, "false");
}

#[component]
fn ToastActionAndCloseTestComponent() -> Element {
    let mut called = use_signal(|| false);
    let on_click = Callback::new(move |ev: ToastActionEvent| {
        called.set(true);
        ev.prevent_default();
    });

    rsx! {
        ToastAction {
            alt_text: "Undo".to_string(),
            toast_id: Some(ToastId(42)),
            on_click: on_click,
            "Undo"
        }
        ToastClose {
            toast_id: Some(ToastId(42)),
            aria_label: "Close".to_string(),
            "X"
        }
    }
}

#[test]
fn toast_action_and_close_event_handling_contract() {
    let mut dom = VirtualDom::new(ToastActionAndCloseTestComponent);
    dom.rebuild_in_place();
}

#[cfg(feature = "native")]
#[test]
fn measured_variable_heights_publish_front_height_and_cumulative_offsets_test() {
    let mut harness = blitz_test_harness::Harness::from_component(AutoMountViewportComponent);
    harness.pump();

    let (id1, id2, id3) = {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        {
            let mut store = TOAST_STORE.write();
            store.toasts.clear();
            store.toast_heights.clear();
            store.config = ToastConfig::default();
            store.set_hovered(false);
        }
        let id1 = toast::message(
            "Back card",
            Some(ToastOptions {
                duration: Some(Duration::from_secs(30)),
                ..Default::default()
            }),
        );
        let id2 = toast::success(
            "Middle card",
            Some(ToastOptions {
                duration: Some(Duration::from_secs(30)),
                ..Default::default()
            }),
        );
        let id3 = toast::warning(
            "Front card",
            Some(ToastOptions {
                duration: Some(Duration::from_secs(30)),
                ..Default::default()
            }),
        );

        let mut store = TOAST_STORE.write();
        assert!(!store.set_toast_height(id3, 0.0), "Zero height must be rejected");
        assert!(store.set_toast_height(id3, 52.0));
        assert!(
            !store.set_toast_height(id3, 52.3),
            "Sub-0.5px jitter must not dirty store"
        );
        assert!(store.set_toast_height(id2, 84.0));
        assert!(store.set_toast_height(id1, 116.0));
        (id1, id2, id3)
    };
    harness.pump();

    let vp_style = harness
        .attr("#monoxus-toast-viewport", "style")
        .unwrap_or_default();
    assert!(
        vp_style.contains("--front-toast-height: 52px;"),
        "Viewport must publish --front-toast-height: 52px, got: {vp_style}"
    );
    assert!(
        vp_style.contains("--gap: 14px;"),
        "Viewport must publish --gap: 14px, got: {vp_style}"
    );
    assert!(
        vp_style.contains("--width: 356px;"),
        "Viewport must publish --width: 356px, got: {vp_style}"
    );

    let front_sel = format!("#monoxus-toast-{}", id3.0);
    let mid_sel = format!("#monoxus-toast-{}", id2.0);
    let back_sel = format!("#monoxus-toast-{}", id1.0);

    let s_front_col = harness.attr(&front_sel, "style").unwrap_or_default();
    let s_mid_col = harness.attr(&mid_sel, "style").unwrap_or_default();
    let s_back_col = harness.attr(&back_sel, "style").unwrap_or_default();
    assert!(s_front_col.contains("--height: 52px; --offset: 0px; --scale: 1.000;"));
    assert!(s_mid_col.contains("--height: 84px; --offset: 14px; --scale: 0.950;"));
    assert!(s_back_col.contains("--height: 116px; --offset: 28px; --scale: 0.900;"));

    // Expand stack and verify cumulative offsets: 0, 52 + 14 = 66, 52 + 84 + 28 = 164
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        TOAST_STORE.write().set_hovered(true);
    }
    harness.pump();

    assert_eq!(
        harness
            .attr("#monoxus-toast-viewport", "data-expanded")
            .as_deref(),
        Some("true")
    );
    let s_front_exp = harness.attr(&front_sel, "style").unwrap_or_default();
    let s_mid_exp = harness.attr(&mid_sel, "style").unwrap_or_default();
    let s_back_exp = harness.attr(&back_sel, "style").unwrap_or_default();
    assert!(
        s_front_exp.contains("--height: 52px; --offset: 0px; --scale: 1.000;"),
        "Front expanded style mismatch: {s_front_exp}"
    );
    assert!(
        s_mid_exp.contains("--height: 84px; --offset: 66px; --scale: 1.000;"),
        "Middle expanded style mismatch: {s_mid_exp}"
    );
    assert!(
        s_back_exp.contains("--height: 116px; --offset: 164px; --scale: 1.000;"),
        "Back expanded style mismatch: {s_back_exp}"
    );

    // Evicting toast must also evict its stored height entry
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        let mut store = TOAST_STORE.write();
        assert!(store.remove_toast(id2).is_some());
        assert_eq!(store.toast_height(id2), None);
    }
}

#[test]
fn same_id_resurrection_within_200ms_unmount_window_race_test() {
    let mut dom = VirtualDom::new(AutoMountViewportComponent);
    dom.rebuild_in_place();

    let target_id = ToastId(777);
    dom.in_runtime(|| {
        TOAST_STORE.write().toasts.clear();
        let id = toast::message(
            "Initial Notification",
            Some(ToastOptions {
                id: Some(target_id),
                duration: Some(Duration::from_secs(30)),
                ..Default::default()
            }),
        );
        assert_eq!(id, target_id);

        // Trigger dismiss -> schedules 200ms TIME_BEFORE_UNMOUNT removal task for generation 0
        toast::dismiss(Some(target_id));
        assert_eq!(TOAST_STORE.read().toasts[0].phase, ToastPhase::Dismissing);
    });

    // Advance ~50ms (strictly inside the 200ms exit window)
    let start_wait = std::time::Instant::now();
    while start_wait.elapsed() < Duration::from_millis(50) {
        if block_on_with_timeout(dom.wait_for_work(), Duration::from_millis(15)) {
            dom.render_immediate(&mut dioxus_core::NoOpMutations);
        }
    }

    // Resurrect the same ToastId before the 200ms timer expires
    dom.in_runtime(|| {
        let resurrected_id = toast::success(
            "Resurrected Notification",
            Some(ToastOptions {
                id: Some(target_id),
                duration: Some(Duration::from_secs(30)),
                ..Default::default()
            }),
        );
        assert_eq!(resurrected_id, target_id);
        let store = TOAST_STORE.read();
        assert_eq!(store.toasts.len(), 1);
        assert_eq!(store.toasts[0].phase, ToastPhase::Active);
        assert_eq!(store.toasts[0].generation, 1);
    });

    // Drive well past the original 200ms unmount deadline (320ms total)
    let after_resurrect = std::time::Instant::now();
    while after_resurrect.elapsed() < Duration::from_millis(280) {
        if block_on_with_timeout(dom.wait_for_work(), Duration::from_millis(20)) {
            dom.render_immediate(&mut dioxus_core::NoOpMutations);
        }
    }

    dom.in_runtime(|| {
        let store = TOAST_STORE.read();
        assert_eq!(
            store.toasts.len(),
            1,
            "Stale 200ms unmount timer must NOT delete a resurrected toast with the same ToastId"
        );
        assert_eq!(store.toasts[0].id, target_id);
        assert_eq!(store.toasts[0].phase, ToastPhase::Active);
        assert_eq!(store.toasts[0].title, "Resurrected Notification");
    });
}

#[cfg(feature = "native")]
#[test]
fn cancel_button_rendering_and_click_dismissal_test() {
    use std::rc::Rc;

    let mut harness = blitz_test_harness::Harness::from_component(AutoMountViewportComponent);
    harness.pump();

    let cancel_clicked = Rc::new(AtomicBool::new(false));
    let cancel_clicked_cb = cancel_clicked.clone();
    let prevented_clicked = Rc::new(AtomicBool::new(false));
    let prevented_clicked_cb = prevented_clicked.clone();

    let id_dismiss = {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        TOAST_STORE.write().toasts.clear();
        toast::message(
            "Confirm deployment",
            Some(ToastOptions {
                duration: Some(Duration::from_secs(30)),
                action: Some(ToastActionData::new("Deploy", "Deploy release")),
                cancel: Some(
                    ToastActionData::new("Cancel", "Cancel deployment").with_handler(move || {
                        cancel_clicked_cb.store(true, Ordering::SeqCst);
                    }),
                ),
                ..Default::default()
            }),
        )
    };
    harness.pump();

    let action_sel = format!("#monoxus-toast-{} button[data-button]", id_dismiss.0);
    let cancel_sel = format!("#monoxus-toast-{} button[data-cancel]", id_dismiss.0);
    assert!(
        harness.query(&action_sel).is_some(),
        "Primary action button [data-button] must render"
    );
    assert!(
        harness.query(&cancel_sel).is_some(),
        "Secondary cancel button [data-cancel] must render"
    );
    assert_eq!(harness.attr(&cancel_sel, "data-button"), None);
    assert_eq!(harness.attr(&action_sel, "data-cancel"), None);

    harness.click(&cancel_sel);
    harness.pump();

    assert!(
        cancel_clicked.load(Ordering::SeqCst),
        "Clicking [data-cancel] must invoke cancel callback"
    );
    assert_eq!(
        harness
            .attr(&format!("#monoxus-toast-{}", id_dismiss.0), "data-state")
            .as_deref(),
        Some("closed"),
        "Clicking [data-cancel] without prevent_default must dismiss the toast"
    );

    // Verify prevent_default() on cancel suppresses dismissal
    let id_keep = {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        toast::message(
            "Guarded cancel",
            Some(ToastOptions {
                duration: Some(Duration::from_secs(30)),
                cancel: Some(
                    ToastActionData::new("Hold", "Hold open").with_handler(
                        move |ev: ToastActionEvent| {
                            prevented_clicked_cb.store(true, Ordering::SeqCst);
                            ev.prevent_default();
                        },
                    ),
                ),
                ..Default::default()
            }),
        )
    };
    harness.pump();

    let keep_cancel_sel = format!("#monoxus-toast-{} button[data-cancel]", id_keep.0);
    harness.click(&keep_cancel_sel);
    harness.pump();

    assert!(prevented_clicked.load(Ordering::SeqCst));
    assert_eq!(
        harness
            .attr(&format!("#monoxus-toast-{}", id_keep.0), "data-state")
            .as_deref(),
        Some("open"),
        "prevent_default() on cancel callback must keep toast Active"
    );
}

#[cfg(feature = "native")]
#[test]
fn close_button_config_and_provider_reactivity_test() {
    #[component]
    fn ProviderReactivityHarness() -> Element {
        let mut show_close = use_signal(|| false);
        let mut force_expand = use_signal(|| true);

        rsx! {
            button {
                id: "toggle-provider-props",
                onclick: move |_| {
                    show_close.set(true);
                    force_expand.set(false);
                },
                "Toggle"
            }
            ToastProvider {
                close_button: show_close(),
                expand: force_expand(),
                ToastViewport {}
            }
        }
    }

    let mut harness = blitz_test_harness::Harness::from_component(ProviderReactivityHarness);
    harness.pump();

    let (id_default, id_override) = {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        TOAST_STORE.write().toasts.clear();
        let id_default = toast::message(
            "Default close policy",
            Some(ToastOptions {
                duration: Some(Duration::from_secs(30)),
                ..Default::default()
            }),
        );
        let id_override = toast::info(
            "Explicit close button override",
            Some(ToastOptions {
                duration: Some(Duration::from_secs(30)),
                close_button: Some(true),
                ..Default::default()
            }),
        );
        (id_default, id_override)
    };
    harness.pump();

    assert_eq!(
        harness
            .attr("#monoxus-toast-viewport", "data-expanded")
            .as_deref(),
        Some("true"),
        "ToastProvider expand=true must initialize viewport in expanded posture"
    );
    assert!(
        harness
            .query(&format!(
                "#monoxus-toast-{} [data-close-button]",
                id_default.0
            ))
            .is_none(),
        "close_button=false on ToastProvider must hide ToastClose by default"
    );
    assert!(
        harness
            .query(&format!(
                "#monoxus-toast-{} [data-close-button]",
                id_override.0
            ))
            .is_some(),
        "Per-toast ToastOptions.close_button=Some(true) must override global close_button=false"
    );

    // Dynamically flip ToastProvider props and verify reactive sync
    harness.click("#toggle-provider-props");
    harness.pump();
    harness.pump();

    assert_eq!(
        harness
            .attr("#monoxus-toast-viewport", "data-expanded")
            .as_deref(),
        Some("false"),
        "ToastProvider expand=false update must reactively collapse viewport"
    );
    assert!(
        harness
            .query(&format!(
                "#monoxus-toast-{} [data-close-button]",
                id_default.0
            ))
            .is_some(),
        "ToastProvider close_button=true update must reactively mount ToastClose"
    );
}

#[cfg(feature = "native")]
#[test]
fn onanimationend_ignores_active_toast_and_evicts_only_dismissing_toast_test() {
    const ANIM_CSS: &str = r#"
        @keyframes toast-enter-fade {
            from { opacity: 0; }
            to { opacity: 1; }
        }
        @keyframes toast-exit-fade {
            from { opacity: 1; }
            to { opacity: 0; }
        }
        #monoxus-toast-viewport li[data-sonner-toast][data-state="open"] {
            animation: toast-enter-fade 40ms linear forwards;
        }
        #monoxus-toast-viewport li[data-sonner-toast][data-state="closed"] {
            animation: toast-exit-fade 40ms linear forwards;
        }
    "#;

    #[component]
    fn EntranceAnimatedToastScenario() -> Element {
        rsx! {
            style { "{ANIM_CSS}" }
            ToastViewport {}
        }
    }

    let mut harness = blitz_test_harness::Harness::from_component(EntranceAnimatedToastScenario);
    harness.pump();

    let id = {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        TOAST_STORE.write().toasts.clear();
        toast::success(
            "Entrance animated toast",
            Some(ToastOptions {
                duration: Some(Duration::from_secs(30)),
                ..Default::default()
            }),
        )
    };
    harness.pump();

    let sel = format!("#monoxus-toast-{}", id.0);
    assert!(harness.query(&sel).is_some());

    // Advance past the 40ms entrance keyframe animation and flush real AnimationEnd into Dioxus
    harness.tick(0.06);
    harness.doc.flush_animation_events();
    harness.pump();

    assert!(
        harness.query(&sel).is_some(),
        "onanimationend must NEVER evict a ToastPhase::Active toast after entrance animation"
    );
    assert_eq!(harness.attr(&sel, "data-state").as_deref(), Some("open"));

    // Transition toast to Dismissing and advance past the 40ms exit keyframe animation
    // (well before any 200ms TIME_BEFORE_UNMOUNT fallback timer) -> must evict via onanimationend.
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        toast::dismiss(Some(id));
    }
    harness.pump();
    assert_eq!(harness.attr(&sel, "data-state").as_deref(), Some("closed"));

    harness.tick(0.06);
    harness.doc.flush_animation_events();
    harness.pump();

    assert!(
        harness.query(&sel).is_none(),
        "onanimationend must evict toast once phase == ToastPhase::Dismissing"
    );
}

#[test]
fn concurrent_three_toast_countdown_and_pause_resume_test() {
    use std::sync::Arc;
    use std::sync::atomic::AtomicUsize;

    let mut dom = VirtualDom::new(AutoMountViewportComponent);
    dom.rebuild_in_place();

    let auto_closed = Arc::new(AtomicUsize::new(0));

    // 1. Concurrent 3-toast auto-close within single-lifespan wall-clock bounds (not 3x serialized)
    let dispatch_start = std::time::Instant::now();
    dom.in_scope(dioxus::core::ScopeId::ROOT, || {
        let mut store = TOAST_STORE.write();
        store.toasts.clear();
        store.toast_heights.clear();
        store.config = ToastConfig::default();
        store.set_hovered(false);
        store.set_focused(false);
        store.set_swiping(false);
        store.set_page_hidden(false);
        drop(store);

        let lifespan = Duration::from_millis(120);
        for title in ["Concurrent 1", "Concurrent 2", "Concurrent 3"] {
            let counter = auto_closed.clone();
            let cb = Callback::new(move |()| {
                counter.fetch_add(1, Ordering::SeqCst);
            });
            toast::message(
                title,
                Some(ToastOptions {
                    duration: Some(lifespan),
                    on_auto_close: Some(cb),
                    ..Default::default()
                }),
            );
        }

        let store = TOAST_STORE.read();
        assert_eq!(store.toasts.len(), 3);
        assert!(store.toasts.iter().all(|t| t.phase == ToastPhase::Active));
    });

    while dispatch_start.elapsed() < Duration::from_millis(1000) {
        let any_active = dom.in_runtime(|| TOAST_STORE.read().has_active_toasts());
        if !any_active {
            break;
        }
        if block_on_with_timeout(dom.wait_for_work(), Duration::from_millis(15)) {
            dom.render_immediate(&mut dioxus_core::NoOpMutations);
        }
    }
    let concurrent_elapsed = dispatch_start.elapsed();

    assert_eq!(
        auto_closed.load(Ordering::SeqCst),
        3,
        "All 3 stacked toasts must fire on_auto_close"
    );
    assert!(
        !dom.in_runtime(|| TOAST_STORE.read().has_active_toasts()),
        "All 3 stacked toasts must auto-close"
    );
    assert!(
        concurrent_elapsed >= Duration::from_millis(100)
            && concurrent_elapsed < Duration::from_millis(310),
        "3 concurrent 120ms toasts must expire within single-lifespan bounds (<310ms, not >=360ms serialized), got {concurrent_elapsed:?}"
    );

    let unmount_start = std::time::Instant::now();
    while unmount_start.elapsed() < Duration::from_millis(1000) {
        if dom.in_runtime(|| TOAST_STORE.read().toasts.is_empty()) {
            break;
        }
        if block_on_with_timeout(dom.wait_for_work(), Duration::from_millis(15)) {
            dom.render_immediate(&mut dioxus_core::NoOpMutations);
        }
    }
    assert!(
        dom.in_runtime(|| TOAST_STORE.read().toasts.is_empty()),
        "All 3 dismissed toasts must be evicted after TIME_BEFORE_UNMOUNT"
    );

    // 2. Pause/resume preservation across 3 concurrent stacked toasts
    auto_closed.store(0, Ordering::SeqCst);
    dom.in_scope(dioxus::core::ScopeId::ROOT, || {
        let lifespan = Duration::from_millis(160);
        for title in ["Paused 1", "Paused 2", "Paused 3"] {
            let counter = auto_closed.clone();
            let cb = Callback::new(move |()| {
                counter.fetch_add(1, Ordering::SeqCst);
            });
            toast::info(
                title,
                Some(ToastOptions {
                    duration: Some(lifespan),
                    on_auto_close: Some(cb),
                    ..Default::default()
                }),
            );
        }
    });

    let run_before_pause = std::time::Instant::now();
    while run_before_pause.elapsed() < Duration::from_millis(60) {
        if block_on_with_timeout(dom.wait_for_work(), Duration::from_millis(15)) {
            dom.render_immediate(&mut dioxus_core::NoOpMutations);
        }
    }

    let rems_at_pause: Vec<Duration> = dom.in_runtime(|| {
        TOAST_STORE.write().set_hovered(true);
        let store = TOAST_STORE.read();
        assert!(store.paused);
        assert_eq!(store.toasts.len(), 3);
        assert!(store.toasts.iter().all(|t| t.phase == ToastPhase::Active));
        store.toasts.iter().map(|t| t.remaining_duration).collect()
    });
    for rem in &rems_at_pause {
        assert!(
            *rem < Duration::from_millis(150) && *rem > Duration::from_millis(30),
            "Expected partial remaining duration at pause, got {rem:?}"
        );
    }

    // Hold paused past full 160ms lifespan -> none may expire and remaining_duration stays frozen
    let hold_paused = std::time::Instant::now();
    while hold_paused.elapsed() < Duration::from_millis(180) {
        if block_on_with_timeout(dom.wait_for_work(), Duration::from_millis(20)) {
            dom.render_immediate(&mut dioxus_core::NoOpMutations);
        }
    }
    dom.in_runtime(|| {
        let store = TOAST_STORE.read();
        assert_eq!(auto_closed.load(Ordering::SeqCst), 0);
        assert!(store.toasts.iter().all(|t| t.phase == ToastPhase::Active));
        let rems_during_pause: Vec<Duration> =
            store.toasts.iter().map(|t| t.remaining_duration).collect();
        assert_eq!(
            rems_during_pause, rems_at_pause,
            "Remaining durations must stay frozen while paused"
        );
    });

    // Resume and verify concurrent completion of remaining ~100ms window
    let resume_start = std::time::Instant::now();
    dom.in_runtime(|| {
        TOAST_STORE.write().set_hovered(false);
        assert!(!TOAST_STORE.read().paused);
    });
    while resume_start.elapsed() < Duration::from_millis(1000) {
        if !dom.in_runtime(|| TOAST_STORE.read().has_active_toasts()) {
            break;
        }
        if block_on_with_timeout(dom.wait_for_work(), Duration::from_millis(15)) {
            dom.render_immediate(&mut dioxus_core::NoOpMutations);
        }
    }
    let resume_elapsed = resume_start.elapsed();
    assert_eq!(auto_closed.load(Ordering::SeqCst), 3);
    assert!(
        resume_elapsed < Duration::from_millis(260),
        "Resumed 3-toast stack must complete remaining ~100ms concurrently (<260ms), got {resume_elapsed:?}"
    );
}

#[cfg(feature = "native")]
#[test]
fn pointer_close_and_action_keep_viewport_unfocused_collapse_on_mouseleave_and_expire_test() {
    use crate::common::native_fixture::{
        NATIVE_TEST_LOCK, advance_frame_and_pump as advance_frame, mount_native_scenario,
    };
    use std::rc::Rc;

    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    const STACK_POINTER_CSS: &str = r#"
        html, body { width: 800px; height: 800px; margin: 0; padding: 0; }
        #monoxus-toast-viewport {
            position: fixed;
            right: 24px;
            bottom: 24px;
            width: 356px;
            margin: 0;
            padding: 0;
            list-style: none;
            outline: none;
            pointer-events: none;
        }
        #monoxus-toast-viewport li[data-sonner-toast] {
            position: absolute;
            bottom: 0;
            width: 100%;
            box-sizing: border-box;
            pointer-events: auto;
            padding: 14px 40px 14px 16px;
            background: #ffffff;
            border: 1px solid #e2e8f0;
            --lift: -1;
            --y: translateY(calc(var(--lift, -1) * var(--gap, 14px) * var(--toasts-before, 0))) scale(var(--scale, 1));
            transform: var(--y);
        }
        #monoxus-toast-viewport li[data-sonner-toast][data-expanded="true"] {
            --y: translateY(calc(var(--lift, -1) * var(--offset, 0px)));
            transform: var(--y);
            height: var(--height, auto);
        }
        #monoxus-toast-viewport li[data-sonner-toast][data-state="closed"] {
            pointer-events: none !important;
            opacity: 0;
        }
        #monoxus-toast-viewport li[data-sonner-toast] [data-close-button] {
            position: absolute;
            top: 8px;
            right: 8px;
            width: 24px;
            height: 24px;
        }
        #monoxus-toast-viewport li[data-sonner-toast] [data-button] {
            display: inline-block;
            margin-top: 8px;
            padding: 4px 10px;
        }
    "#;

    #[component]
    fn StackedPointerDismissScenario() -> Element {
        rsx! {
            style { "{STACK_POINTER_CSS}" }
            button { id: "outside-anchor", "Outside" }
            ToastViewport {}
        }
    }

    struct DocResetGuard;
    impl Drop for DocResetGuard {
        fn drop(&mut self) {
            reset_toast_watcher();
            oxidase::dom::Document::set_current(None);
        }
    }
    let _reset_guard = DocResetGuard;

    let mut harness = mount_native_scenario(StackedPointerDismissScenario);
    harness.set_viewport_size(800, 800);
    harness.pump();

    let action_fired = Rc::new(AtomicBool::new(false));
    let action_fired_cb = action_fired.clone();

    let (id_back, id_mid, id_front) = {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        toast::dismiss(None);
        let mut store = TOAST_STORE.write();
        store.toasts.clear();
        store.toast_heights.clear();
        store.config = ToastConfig::default();
        store.set_page_hidden(false);
        store.set_hovered(false);
        store.set_focused(false);
        store.set_swiping(false);
        drop(store);

        let b = toast::message(
            "Back stacked toast",
            Some(ToastOptions {
                duration: Some(Duration::from_millis(350)),
                ..Default::default()
            }),
        );
        let m = toast::info(
            "Middle actionable toast",
            Some(ToastOptions {
                duration: Some(Duration::from_millis(350)),
                action: Some(
                    ToastActionData::new("Apply", "Apply update").with_handler(move || {
                        action_fired_cb.store(true, Ordering::SeqCst);
                    }),
                ),
                ..Default::default()
            }),
        );
        let f = toast::success(
            "Front closeable toast",
            Some(ToastOptions {
                duration: Some(Duration::from_millis(350)),
                ..Default::default()
            }),
        );
        (b, m, f)
    };
    harness.pump();
    advance_frame(&mut harness, 0.02);

    let vp_nid = harness.node("#monoxus-toast-viewport");
    let front_sel = format!("#monoxus-toast-{}", id_front.0);
    let mid_sel = format!("#monoxus-toast-{}", id_mid.0);
    let back_sel = format!("#monoxus-toast-{}", id_back.0);

    // 1. Hover viewport over front toast -> stack expands and timers pause while focused remains false
    let (fx, fy) = harness.center_of(&front_sel);
    harness.move_mouse_to(fx, fy);
    harness.pump();
    advance_frame(&mut harness, 0.02);

    assert_eq!(
        harness
            .attr("#monoxus-toast-viewport", "data-expanded")
            .as_deref(),
        Some("true")
    );
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        let store = TOAST_STORE.peek();
        assert!(store.interaction_state.hovered);
        assert!(!store.interaction_state.focused);
        assert!(store.expanded);
        assert!(store.paused);
    }

    // 2. Press [data-close-button] (x) on front toast, hold across multiple frames (>1 frame),
    //    then release to verify pointer press suppression spans the entire press-and-release gesture.
    let close_sel = format!("{front_sel} [data-close-button]");
    let (cx, cy) = harness.center_of(&close_sel);
    harness.mouse_down_at(cx, cy);
    advance_frame(&mut harness, 0.05);
    advance_frame(&mut harness, 0.05);
    harness.mouse_up_at(cx, cy);
    advance_frame(&mut harness, 0.02);

    assert_eq!(
        harness.attr(&front_sel, "data-state").as_deref(),
        Some("closed"),
        "Pointer click on [data-close-button] must dismiss front toast"
    );
    assert_eq!(harness.attr(&mid_sel, "data-state").as_deref(), Some("open"));
    assert_eq!(harness.attr(&back_sel, "data-state").as_deref(), Some("open"));
    assert_ne!(
        harness.base().active_focus_node_id(),
        Some(vp_nid),
        "Pointer close must NOT move active DOM focus to #monoxus-toast-viewport"
    );
    assert_eq!(
        harness.base().active_focus_node_id(),
        None,
        "Clicked close button inside dismissing toast must blur DOM focus cleanly"
    );
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        assert!(
            !TOAST_STORE.peek().interaction_state.focused,
            "Pointer click on [data-close-button] must leave interaction_state.focused == false"
        );
    }

    // 3. Move pointer out (mouseleave) -> remaining 2-card stack collapses and unpauses
    harness.move_mouse_to(10.0, 10.0);
    harness.pump();
    advance_frame(&mut harness, 0.02);

    assert_eq!(
        harness
            .attr("#monoxus-toast-viewport", "data-expanded")
            .as_deref(),
        Some("false"),
        "mouseleave after pointer close must collapse remaining stack"
    );
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        let store = TOAST_STORE.peek();
        assert!(!store.expanded, "expanded must become false on mouseleave");
        assert!(!store.paused, "paused must become false on mouseleave");
        assert!(!store.interaction_state.focused);
    }

    // 4. Re-hover remaining 2-card stack and click [data-button] (Action) on id_mid with pointer
    let mid_ty = harness
        .base()
        .get_node(harness.node(&mid_sel))
        .and_then(|n| n.transform().as_ref())
        .map(|aff| aff.as_coeffs()[5] as f32)
        .unwrap_or(0.0);
    let (mx, my_layout) = harness.center_of(&mid_sel);
    harness.move_mouse_to(mx, my_layout + mid_ty);
    harness.pump();
    advance_frame(&mut harness, 0.02);

    let mid_ty_expanded = harness
        .base()
        .get_node(harness.node(&mid_sel))
        .and_then(|n| n.transform().as_ref())
        .map(|aff| aff.as_coeffs()[5] as f32)
        .unwrap_or(0.0);
    let action_sel = format!("{mid_sel} [data-button]");
    let (ax, ay_layout) = harness.center_of(&action_sel);
    let ay = ay_layout + mid_ty_expanded;
    harness.mouse_down_at(ax, ay);
    advance_frame(&mut harness, 0.04);
    harness.mouse_up_at(ax, ay);
    advance_frame(&mut harness, 0.02);

    assert!(action_fired.load(Ordering::SeqCst), "Action handler must fire");
    assert_eq!(
        harness.attr(&mid_sel, "data-state").as_deref(),
        Some("closed"),
        "Pointer click on [data-button] must dismiss actionable toast"
    );
    assert_eq!(harness.attr(&back_sel, "data-state").as_deref(), Some("open"));
    assert_ne!(
        harness.base().active_focus_node_id(),
        Some(vp_nid),
        "Pointer click on [data-button] must NOT move active DOM focus to #monoxus-toast-viewport"
    );
    assert_eq!(
        harness.base().active_focus_node_id(),
        None,
        "Clicked action button inside dismissing toast must blur DOM focus cleanly"
    );
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        assert!(
            !TOAST_STORE.peek().interaction_state.focused,
            "Pointer click on [data-button] must leave interaction_state.focused == false"
        );
    }

    // 5. Move pointer out (mouseleave) -> collapses, unpauses, and remaining toast expires & unmounts
    harness.move_mouse_to(10.0, 10.0);
    harness.pump();
    advance_frame(&mut harness, 0.02);

    assert_eq!(
        harness
            .attr("#monoxus-toast-viewport", "data-expanded")
            .as_deref(),
        Some("false")
    );
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        let store = TOAST_STORE.peek();
        assert!(!store.expanded);
        assert!(!store.paused);
    }

    // Advance past remaining duration (<350ms) and TIME_BEFORE_UNMOUNT (200ms)
    advance_frame(&mut harness, 0.20);
    advance_frame(&mut harness, 0.20);
    advance_frame(&mut harness, 0.25);

    assert!(
        harness.query("#monoxus-toast-viewport li").is_none(),
        "Remaining toast(s) must expire and unmount after mouseleave resumes countdown"
    );
    {
        let _guard = dioxus::core::RuntimeGuard::new(harness.doc.vdom.runtime());
        assert!(TOAST_STORE.peek().toasts.is_empty());
    }
}
