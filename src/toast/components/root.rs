use crate::foundation::overlay::Point2D;
use crate::toast::attrs::ToastRootAttrs;
use crate::toast::gesture::{SwipeAction, SwipeAxis, SwipeGestureReducer, SwipeReducerConfig};
use crate::toast::layout::ToastComputedStyle;
use crate::toast::runtime::{TOAST_STORE, toast, toast_now_ms};
use crate::toast::types::{ToastId, ToastPhase, ToastPosition, ToastType};
use dioxus::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

fn try_set_pointer_capture(
    evt: &PointerData,
    mounted_root: Option<&oxidase::Element>,
) -> Option<i32> {
    let pointer_id = oxidase::dom::extract_pointer_id(evt);
    if let Some(root) = mounted_root
        && root.set_pointer_capture(pointer_id).is_ok()
    {
        return Some(pointer_id);
    }
    if let Some(target) = oxidase::dom::pointer_event_target(evt)
        && target.set_pointer_capture(pointer_id).is_ok()
    {
        return Some(pointer_id);
    }
    None
}

fn try_release_pointer_capture(
    evt_opt: Option<&PointerEvent>,
    mounted_root: Option<&oxidase::Element>,
    captured_pid: Option<i32>,
) {
    let pointer_id = captured_pid.or_else(|| evt_opt.map(|evt| oxidase::dom::extract_pointer_id(&evt.data())));
    let Some(pointer_id) = pointer_id else {
        return;
    };
    if let Some(root) = mounted_root {
        let _ = root.release_pointer_capture(pointer_id);
    }
    if let Some(evt) = evt_opt
        && let Some(target) = oxidase::dom::pointer_event_target(&evt.data())
    {
        let _ = target.release_pointer_capture(pointer_id);
    }
}

fn is_button_target(evt: &PointerData) -> bool {
    oxidase::dom::is_interactive_button_target(evt)
}

fn measure_toast_auto_height(dom_id: &str) -> Option<f64> {
    oxidase::dom::Document::current()
        .and_then(|doc| doc.measure_auto_height(dom_id))
        .filter(|&h| h.is_finite() && h > 0.0)
}

fn try_record_toast_height(toast_id: ToastId, height: f64) {
    if !height.is_finite() || height <= 0.0 || dioxus::core::Runtime::try_current().is_none() {
        return;
    }
    {
        let store = TOAST_STORE.peek();
        if !store.toasts.iter().any(|t| t.id == toast_id) {
            return;
        }
        if let Some(old_h) = store.toast_height(toast_id)
            && (old_h - height).abs() <= 0.5
        {
            return;
        }
    }
    TOAST_STORE.write().set_toast_height(toast_id, height);
}

fn blur_toast_card_if_active(dom_id: &str) -> bool {
    let Some(doc) = oxidase::dom::Document::current() else {
        return false;
    };
    let Some(el) = doc.element_by_id(dom_id) else {
        return false;
    };
    if doc.is_element_active(dom_id) {
        let _ = el.blur_sync();
        let vp_id = crate::toast::hotkey::resolve_enclosing_viewport_id(dom_id)
            .unwrap_or_else(|| "monoxus-toast-viewport".to_string());
        if dioxus::core::Runtime::try_current().is_some()
            && !crate::toast::hotkey::is_focus_within_element(&vp_id)
            && TOAST_STORE.peek().interaction_state.focused
        {
            TOAST_STORE.write().set_focused(false);
        }
    }
    true
}

#[derive(Props, Clone, PartialEq)]
pub struct ToastRootProps {
    pub id: ToastId,
    #[props(default)]
    pub dom_id: Option<String>,
    #[props(default)]
    pub toast_type: ToastType,
    #[props(default)]
    pub phase: ToastPhase,
    #[props(default)]
    pub index: usize,
    #[props(default = 3)]
    pub visible_limit: usize,
    #[props(default)]
    pub expanded: bool,
    #[props(default = true)]
    pub dismissible: bool,
    #[props(default)]
    pub position: Option<ToastPosition>,
    #[props(default)]
    pub test_id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub computed_style: Option<ToastComputedStyle>,
    #[props(default)]
    pub content_generation: u64,
    #[props(default)]
    pub children: Element,
}

/// Headless toast item container (<li role="status" / role="alert">) with gesture-driven swipe dismissal.
#[component]
pub fn ToastRoot(props: ToastRootProps) -> Element {
    let dom_id = props
        .dom_id
        .clone()
        .unwrap_or_else(|| format!("monoxus-toast-{}", props.id.0));
    let effective_dismissible = props.dismissible && props.toast_type != ToastType::Loading;
    let position = props.position.unwrap_or(ToastPosition::BottomRight);
    let attrs = ToastRootAttrs::with_options_and_position(
        props.toast_type,
        props.phase,
        props.index,
        props.visible_limit,
        props.test_id.clone(),
        props.expanded,
        position,
    )
    .with_dismissible(effective_dismissible);

    let mut is_mounted = use_signal(|| false);
    let swipe_delta_x = use_signal(|| 0.0f64);
    let swipe_delta_y = use_signal(|| 0.0f64);
    let is_swiping_local = use_signal(|| false);
    let swipe_out = use_signal(|| false);
    let swipe_direction_name = use_signal(|| None::<&'static str>);

    let active_reducer = use_hook(|| Rc::new(RefCell::new(None::<SwipeGestureReducer>)));
    let mounted_root = use_hook(|| Rc::new(RefCell::new(None::<oxidase::Element>)));
    let captured_pointer_id = use_hook(|| Rc::new(RefCell::new(None::<i32>)));
    let pointer_stream_guard = use_hook(|| {
        Rc::new(RefCell::new(
            None::<crate::foundation::browser::DocumentPointerStreamGuard>,
        ))
    });

    let needs_measurement = props.computed_style.is_some_and(|cs| cs.height <= 0.0);

    {
        let dom_id_for_remeasure = dom_id.clone();
        let toast_id = props.id;
        let last_unmeasured = use_hook(|| std::cell::Cell::new(false));
        let last_content_gen = use_hook(|| std::cell::Cell::new(props.content_generation));
        let was_unmeasured = last_unmeasured.replace(needs_measurement);
        let content_gen_changed =
            props.content_generation != last_content_gen.replace(props.content_generation);
        if mounted_root.borrow().is_some()
            && ((needs_measurement && !was_unmeasured) || content_gen_changed)
        {
            crate::foundation::browser::spawn_root_task(async move {
                let _ = oxidase::frame::next_frame().await;
                if let Some(h) = measure_toast_auto_height(&dom_id_for_remeasure) {
                    try_record_toast_height(toast_id, h);
                }
            });
        }
    }

    use_drop({
        let stream_guard_cell = pointer_stream_guard.clone();
        let swiping_sig = is_swiping_local;
        move || {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                stream_guard_cell.borrow_mut().take();
                if dioxus::core::Runtime::try_current().is_some() && *swiping_sig.peek() {
                    TOAST_STORE.write().set_swiping(false);
                }
            }));
        }
    });

    let onpointerdown = {
        let reducer_cell = active_reducer.clone();
        let root_cell = mounted_root.clone();
        let captured_pid_cell = captured_pointer_id.clone();
        let guard_cell = pointer_stream_guard.clone();
        let toast_id = props.id;
        let effective_position = position;
        let dom_id_for_down = dom_id.clone();

        move |evt: PointerEvent| {
            if evt.trigger_button() == Some(dioxus::html::input_data::MouseButton::Secondary) {
                return;
            }
            if is_button_target(&evt.data()) {
                return;
            }
            if !effective_dismissible {
                blur_toast_card_if_active(&dom_id_for_down);
                let dom_id_blur = dom_id_for_down.clone();
                crate::foundation::browser::spawn_root_task(async move {
                    let _ = oxidase::frame::next_frame().await;
                    blur_toast_card_if_active(&dom_id_blur);
                });
                return;
            }

            let store_config = TOAST_STORE.peek().config.clone();
            let allowed_dirs = store_config
                .swipe_directions
                .clone()
                .unwrap_or_else(|| effective_position.default_swipe_directions());
            let config = SwipeReducerConfig::new(store_config.swipe_threshold, allowed_dirs);

            let coords = evt.client_coordinates();
            let start_point = Point2D::new(coords.x, coords.y);
            let start_time = toast_now_ms();

            let reducer = SwipeGestureReducer::new(toast_id.0, start_point, start_time, config);
            *reducer_cell.borrow_mut() = Some(reducer);

            let captured_pid = {
                let root_ref = root_cell.borrow();
                try_set_pointer_capture(&evt.data(), root_ref.as_ref())
            };
            *captured_pid_cell.borrow_mut() = captured_pid;
            if captured_pid.is_none() {
                // Substrate resilience: activate generic document pointer stream from foundation/browser/
                let reducer_for_stream = reducer_cell.clone();
                let mut sx = swipe_delta_x;
                let mut sy = swipe_delta_y;
                let mut swiping_sig = is_swiping_local;

                let guard = crate::foundation::browser::start_document_pointer_stream(
                    move |pt: Point2D| {
                        let mut cell = reducer_for_stream.borrow_mut();
                        if let Some(ref mut red) = *cell {
                            if let SwipeAction::Update { delta_x, delta_y } = red.handle_move(pt) {
                                sx.set(delta_x);
                                sy.set(delta_y);
                                if red.is_swiping() {
                                    swiping_sig.set(true);
                                    TOAST_STORE.write().set_swiping(true);
                                }
                            }
                        }
                    },
                );
                *guard_cell.borrow_mut() = Some(guard);
            }
        }
    };

    let onpointermove = {
        let reducer_cell = active_reducer.clone();
        let mut sx = swipe_delta_x;
        let mut sy = swipe_delta_y;
        let mut swiping_sig = is_swiping_local;

        move |evt: PointerEvent| {
            let mut cell = reducer_cell.borrow_mut();
            if let Some(ref mut red) = *cell {
                let coords = evt.client_coordinates();
                let pt = Point2D::new(coords.x, coords.y);
                if let SwipeAction::Update { delta_x, delta_y } = red.handle_move(pt) {
                    sx.set(delta_x);
                    sy.set(delta_y);
                    if red.is_swiping() {
                        swiping_sig.set(true);
                        TOAST_STORE.write().set_swiping(true);
                    }
                }
            }
        }
    };

    let handle_pointer_end = {
        let reducer_cell = active_reducer.clone();
        let root_cell = mounted_root.clone();
        let captured_pid_cell = captured_pointer_id.clone();
        let guard_cell = pointer_stream_guard.clone();
        let mut sx = swipe_delta_x;
        let mut sy = swipe_delta_y;
        let mut swiping_sig = is_swiping_local;
        let mut swipe_out_sig = swipe_out;
        let mut swipe_dir_sig = swipe_direction_name;
        let dom_id_for_end = dom_id.clone();

        move |evt_opt: Option<&PointerEvent>, is_cancel: bool| {
            // Drop document pointer stream guard
            guard_cell.borrow_mut().take();

            let pid = captured_pid_cell.borrow_mut().take();
            {
                let root_ref = root_cell.borrow();
                try_release_pointer_capture(evt_opt, root_ref.as_ref(), pid);
            }

            let now_ms = toast_now_ms();
            let reducer_opt = reducer_cell.borrow_mut().take();

            if let Some(reducer) = reducer_opt {
                let was_swiping = reducer.is_swiping();
                let action = if is_cancel {
                    reducer.handle_cancel()
                } else {
                    reducer.handle_release(now_ms)
                };

                if was_swiping || TOAST_STORE.peek().interaction_state.swiping {
                    TOAST_STORE.write().set_swiping(false);
                }
                if *swiping_sig.peek() {
                    swiping_sig.set(false);
                }

                match action {
                    SwipeAction::Dismiss(id) => {
                        let dir_str = match reducer.axis() {
                            Some(SwipeAxis::X) => {
                                if reducer.state.delta_x() > 0.0 {
                                    "right"
                                } else {
                                    "left"
                                }
                            }
                            Some(SwipeAxis::Y) => {
                                if reducer.state.delta_y() > 0.0 {
                                    "down"
                                } else {
                                    "up"
                                }
                            }
                            None => "right",
                        };
                        swipe_out_sig.set(true);
                        swipe_dir_sig.set(Some(dir_str));
                        toast::dismiss(Some(ToastId(id)));
                    }
                    SwipeAction::SnapBack => {
                        sx.set(0.0);
                        sy.set(0.0);
                        swipe_out_sig.set(false);
                        swipe_dir_sig.set(None);
                        if !blur_toast_card_if_active(&dom_id_for_end) {
                            let dom_id_blur = dom_id_for_end.clone();
                            crate::foundation::browser::spawn_root_task(async move {
                                let _ = oxidase::frame::next_frame().await;
                                blur_toast_card_if_active(&dom_id_blur);
                            });
                        }
                    }
                    SwipeAction::Update { .. } => {}
                }
            }
        }
    };

    let sx_val = *swipe_delta_x.read();
    let sy_val = *swipe_delta_y.read();
    let is_swiping_val = *is_swiping_local.read();
    let swipe_out_val = *swipe_out.read();
    let swipe_dir_val = *swipe_direction_name.read();

    let mut inline_style = String::from("pointer-events: auto; ");
    if let Some(ref cs) = props.computed_style {
        inline_style.push_str(&cs.to_style_string());
        inline_style.push(' ');
    }
    let drag_offset = if sx_val.abs() > sy_val.abs() {
        sx_val
    } else {
        sy_val
    };
    inline_style.push_str(&format!(
        "--swipe-amount-x: {sx_val}px; --swipe-amount-y: {sy_val}px; --drag-offset: {drag_offset}px; "
    ));
    if let Some(s) = props.style.as_deref() {
        inline_style.push_str(s);
    }
    let combined_style = if inline_style.trim().is_empty() {
        None
    } else {
        Some(inline_style)
    };

    let dom_id_for_mount = dom_id.clone();

    rsx! {
        li {
            id: "{dom_id}",
            tabindex: "0",
            role: "{attrs.role}",
            aria_live: "{attrs.aria_live}",
            aria_atomic: "{attrs.aria_atomic}",
            "data-sonner-toast": "",
            "data-mounted": if *is_mounted.read() { "true" } else { "false" },
            "data-id": "{props.id.0}",
            "data-state": "{attrs.data_state}",
            "data-type": "{attrs.data_type}",
            "data-visible": "{attrs.data_visible}",
            "data-front": "{attrs.data_front}",
            "data-expanded": "{attrs.data_expanded}",
            "data-dismissible": "{attrs.data_dismissible}",
            "data-position": "{attrs.data_position}",
            "data-x-position": "{attrs.data_x_position}",
            "data-y-position": "{attrs.data_y_position}",
            "data-swipe-out": if swipe_out_val { "true" } else { attrs.data_swipe_out },
            "data-swipe-direction": swipe_dir_val,
            "data-swiping": if is_swiping_val { Some("true") } else { None },
            "data-swiped": if is_swiping_val { Some("true") } else { None },
            "data-index": "{attrs.data_index}",
            "data-testid": attrs.data_testid.as_deref().unwrap_or_default(),
            class: props.class.as_deref().unwrap_or_default(),
            style: combined_style.as_deref(),
            // NOTE(ah:01M7BTR8FNX62N461B3Q6837YJ): See ui/shadcn-dioxus/docs/shadcn_toast_styling_capability_ledger.md
            // (Section 4 Note 6). When wiring downstream <Toaster /> CSS entrance transitions ([data-mounted="false" -> "true"]),
            // reorder this block to: (1) measure_toast_auto_height while data-mounted="false", (2) await next_frame().await,
            // and (3) is_mounted.set(true) without a post-flip re-measurement pass (advancing tests/native_data_gesture_test.rs:1568-1572
            // by 1 frame). This prevents Blitz Native poll coalescing (dioxus_document.rs:254-259) and frame-1 transition: none
            // cancellation from Document::measure_auto_height (oxidase native.rs:464-481, 503-525).
            onmounted: {
                let root_cell = mounted_root.clone();
                let toast_id = props.id;
                move |evt: MountedEvent| {
                    let mounted_el = oxidase::Element::from_mounted(evt.data());
                    *root_cell.borrow_mut() = Some(mounted_el);
                    is_mounted.set(true);

                    if let Some(h) = measure_toast_auto_height(&dom_id_for_mount) {
                        try_record_toast_height(toast_id, h);
                    }

                    let dom_id_async = dom_id_for_mount.clone();
                    crate::foundation::browser::spawn_root_task(async move {
                        let _ = oxidase::frame::next_frame().await;
                        if let Some(h) = measure_toast_auto_height(&dom_id_async) {
                            try_record_toast_height(toast_id, h);
                        }
                    });
                }
            },
            onpointerdown,
            onpointermove,
            onpointerup: {
                let mut end_fn = handle_pointer_end.clone();
                move |evt: PointerEvent| end_fn(Some(&evt), false)
            },
            onpointercancel: {
                let mut end_fn = handle_pointer_end.clone();
                move |evt: PointerEvent| end_fn(Some(&evt), true)
            },
            onlostpointercapture: {
                let reducer_cell = active_reducer.clone();
                let captured_pid_cell = captured_pointer_id.clone();
                let mut end_fn = handle_pointer_end.clone();
                move |_evt: PointerEvent| {
                    *captured_pid_cell.borrow_mut() = None;
                    if reducer_cell.borrow().is_some() {
                        end_fn(None, true);
                    }
                }
            },
            onanimationend: {
                let toast_id = props.id;
                move |_evt: AnimationEvent| {
                    if TOAST_STORE
                        .peek()
                        .toasts
                        .iter()
                        .any(|t| t.id == toast_id && t.phase == ToastPhase::Dismissing)
                    {
                        TOAST_STORE.write().remove_toast_if_dismissing(toast_id);
                    }
                }
            },
            ontransitionend: {
                let toast_id = props.id;
                move |_evt: TransitionEvent| {
                    if TOAST_STORE
                        .peek()
                        .toasts
                        .iter()
                        .any(|t| t.id == toast_id && t.phase == ToastPhase::Dismissing)
                    {
                        TOAST_STORE.write().remove_toast_if_dismissing(toast_id);
                    }
                }
            },
            {props.children}
        }
    }
}
