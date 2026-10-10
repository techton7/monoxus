use crate::foundation::overlay::Point2D;
use crate::toast::attrs::ToastRootAttrs;
use crate::toast::gesture::{SwipeAction, SwipeAxis, SwipeGestureReducer, SwipeReducerConfig};
use crate::toast::layout::ToastComputedStyle;
use crate::toast::runtime::{toast, toast_now_ms, TOAST_STORE};
use crate::toast::types::{ToastId, ToastPhase, ToastPosition, ToastType};
use dioxus::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(target_arch = "wasm32")]
fn try_set_pointer_capture(evt: &PointerData) -> bool {
    use wasm_bindgen::JsCast;
    if let Some(web_event) = evt.downcast::<web_sys::PointerEvent>() {
        let pointer_id = web_event.pointer_id();
        if let Some(target) = web_event.target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) {
            return target.set_pointer_capture(pointer_id).is_ok();
        }
    }
    false
}

#[cfg(not(target_arch = "wasm32"))]
fn try_set_pointer_capture(_evt: &PointerData) -> bool {
    false
}

#[cfg(target_arch = "wasm32")]
fn try_release_pointer_capture(evt: &PointerData) {
    use wasm_bindgen::JsCast;
    if let Some(web_event) = evt.downcast::<web_sys::PointerEvent>() {
        let pointer_id = web_event.pointer_id();
        if let Some(target) = web_event.target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) {
            let _ = target.release_pointer_capture(pointer_id);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn try_release_pointer_capture(_evt: &PointerData) {}

#[cfg(target_arch = "wasm32")]
fn is_button_target(evt: &PointerData) -> bool {
    use wasm_bindgen::JsCast;
    if let Some(web_event) = evt.downcast::<web_sys::PointerEvent>() {
        if let Some(target) = web_event.target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) {
            if target.tag_name().eq_ignore_ascii_case("button")
                || target.closest("button").ok().flatten().is_some()
            {
                return true;
            }
        }
    }
    false
}

#[cfg(not(target_arch = "wasm32"))]
fn is_button_target(_evt: &PointerData) -> bool {
    false
}

#[derive(Props, Clone, PartialEq)]
pub struct ToastRootProps {
    pub id: ToastId,
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
    pub children: Element,
}

/// Headless toast item container (<li role="status" / role="alert">) with gesture-driven swipe dismissal.
#[component]
pub fn ToastRoot(props: ToastRootProps) -> Element {
    let position = props.position.unwrap_or(ToastPosition::BottomRight);
    let attrs = ToastRootAttrs::with_options_and_position(
        props.toast_type,
        props.phase,
        props.index,
        props.visible_limit,
        props.test_id.clone(),
        props.expanded,
        position,
    );

    let swipe_delta_x = use_signal(|| 0.0f64);
    let swipe_delta_y = use_signal(|| 0.0f64);
    let is_swiping_local = use_signal(|| false);
    let swipe_out = use_signal(|| false);
    let swipe_direction_name = use_signal(|| None::<&'static str>);

    let active_reducer = use_hook(|| Rc::new(RefCell::new(None::<SwipeGestureReducer>)));
    let pointer_stream_guard = use_hook(|| Rc::new(RefCell::new(None::<crate::foundation::browser::DocumentPointerStreamGuard>)));

    let onpointerdown = {
        let reducer_cell = active_reducer.clone();
        let guard_cell = pointer_stream_guard.clone();
        let toast_id = props.id;
        let effective_position = position;

        move |evt: PointerEvent| {
            if evt.trigger_button() == Some(dioxus::html::input_data::MouseButton::Secondary) {
                return;
            }
            if is_button_target(&evt.data()) {
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

            let captured = try_set_pointer_capture(&evt.data());
            if !captured {
                // Substrate resilience: activate generic document pointer stream from foundation/browser/
                let reducer_for_stream = reducer_cell.clone();
                let mut sx = swipe_delta_x;
                let mut sy = swipe_delta_y;
                let mut swiping_sig = is_swiping_local;

                let guard = crate::foundation::browser::start_document_pointer_stream(move |pt: Point2D| {
                    let mut cell = reducer_for_stream.borrow_mut();
                    if let Some(ref mut red) = *cell {
                        match red.handle_move(pt) {
                            SwipeAction::Update { delta_x, delta_y } => {
                                sx.set(delta_x);
                                sy.set(delta_y);
                                if red.is_swiping() {
                                    swiping_sig.set(true);
                                    TOAST_STORE.write().set_swiping(true);
                                }
                            }
                            _ => {}
                        }
                    }
                });
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
                match red.handle_move(pt) {
                    SwipeAction::Update { delta_x, delta_y } => {
                        sx.set(delta_x);
                        sy.set(delta_y);
                        if red.is_swiping() {
                            swiping_sig.set(true);
                            TOAST_STORE.write().set_swiping(true);
                        }
                    }
                    _ => {}
                }
            }
        }
    };

    let handle_pointer_end = {
        let reducer_cell = active_reducer.clone();
        let guard_cell = pointer_stream_guard.clone();
        let mut sx = swipe_delta_x;
        let mut sy = swipe_delta_y;
        let mut swiping_sig = is_swiping_local;
        let mut swipe_out_sig = swipe_out;
        let mut swipe_dir_sig = swipe_direction_name;

        move |evt_opt: Option<&PointerEvent>, is_cancel: bool| {
            // Drop document pointer stream guard
            guard_cell.borrow_mut().take();

            if let Some(evt) = evt_opt {
                try_release_pointer_capture(&evt.data());
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

                if was_swiping {
                    TOAST_STORE.write().set_swiping(false);
                    swiping_sig.set(false);
                }

                match action {
                    SwipeAction::Dismiss(id) => {
                        let dir_str = match reducer.axis() {
                            Some(SwipeAxis::X) => {
                                if reducer.state.delta_x() > 0.0 { "right" } else { "left" }
                            }
                            Some(SwipeAxis::Y) => {
                                if reducer.state.delta_y() > 0.0 { "down" } else { "up" }
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
    if is_swiping_val || sx_val != 0.0 || sy_val != 0.0 {
        let drag_offset = if sx_val.abs() > sy_val.abs() { sx_val } else { sy_val };
        inline_style.push_str(&format!(
            "--swipe-amount-x: {sx_val}px; --swipe-amount-y: {sy_val}px; --drag-offset: {drag_offset}px; "
        ));
    }
    if let Some(s) = props.style.as_deref() {
        inline_style.push_str(s);
    }
    let combined_style = if inline_style.trim().is_empty() {
        None
    } else {
        Some(inline_style)
    };

    rsx! {
        li {
            role: "{attrs.role}",
            aria_live: "{attrs.aria_live}",
            aria_atomic: "{attrs.aria_atomic}",
            "data-sonner-toast": "",
            "data-id": "{props.id.0}",
            "data-state": "{attrs.data_state}",
            "data-type": "{attrs.data_type}",
            "data-visible": "{attrs.data_visible}",
            "data-front": "{attrs.data_front}",
            "data-expanded": "{attrs.data_expanded}",
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
            {props.children}
        }
    }
}
