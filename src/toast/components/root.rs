use std::cell::RefCell;
use std::rc::Rc;

use dioxus::core::{AttributeValue, Event, ListenerCallback};
use dioxus::html::PlatformEventData;
use dioxus::prelude::*;

use crate::foundation::{
    browser::{try_release_pointer_capture, try_set_pointer_capture},
    compose::{collect_user_attrs, merge_props, push_attr_opt},
    overlay::Point2D,
};
use crate::toast::attrs::ToastRootAttrs;
use crate::toast::gesture::{SwipeAction, SwipeAxis, SwipeGestureReducer, SwipeReducerConfig};
use crate::toast::layout::ToastComputedStyle;
use crate::toast::runtime::{TOAST_STORE, toast, toast_now_ms};
use crate::toast::types::{ToastId, ToastPhase, ToastPosition, ToastType};

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
    #[props(default, into)]
    pub dom_id: Option<String>,
    #[props(default)]
    pub toast_type: ToastType,
    #[props(default)]
    pub phase: ToastPhase,
    #[props(default = true)]
    pub dismissible: bool,
    #[props(default)]
    pub position: Option<ToastPosition>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

pub(crate) struct ToastRootSpec {
    pub id: ToastId,
    pub dom_id: Option<String>,
    pub toast_type: ToastType,
    pub phase: ToastPhase,
    pub index: usize,
    pub visible_limit: usize,
    pub expanded: bool,
    pub dismissible: bool,
    pub position: Option<ToastPosition>,
    pub test_id: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub computed_style: Option<ToastComputedStyle>,
    pub content_generation: u64,
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Props, Clone, PartialEq)]
pub(crate) struct ManagedToastRootProps {
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
    #[props(default = true)]
    pub dismissible: bool,
    #[props(default)]
    pub position: Option<ToastPosition>,
    #[props(default)]
    pub test_id: Option<String>,
    #[props(default)]
    pub computed_style: Option<ToastComputedStyle>,
    #[props(default)]
    pub content_generation: u64,
    #[props(default)]
    pub children: Element,
}

#[component]
pub(crate) fn ManagedToastRoot(props: ManagedToastRootProps) -> Element {
    render_toast_root(ToastRootSpec {
        id: props.id,
        dom_id: None,
        toast_type: props.toast_type,
        phase: props.phase,
        index: props.index,
        visible_limit: props.visible_limit,
        expanded: props.expanded,
        dismissible: props.dismissible,
        position: props.position,
        test_id: props.test_id,
        class: None,
        style: None,
        computed_style: props.computed_style,
        content_generation: props.content_generation,
        child: None,
        attributes: Vec::new(),
        children: props.children,
    })
}

/// Headless toast item container (<li role="status" / role="alert">) with gesture-driven swipe dismissal.
#[component]
pub fn ToastRoot(props: ToastRootProps) -> Element {
    render_toast_root(ToastRootSpec {
        id: props.id,
        dom_id: props.dom_id,
        toast_type: props.toast_type,
        phase: props.phase,
        index: 0,
        visible_limit: 3,
        expanded: false,
        dismissible: props.dismissible,
        position: props.position,
        test_id: None,
        class: props.class,
        style: props.style,
        computed_style: None,
        content_generation: 0,
        child: props.child,
        attributes: props.attributes,
        children: props.children,
    })
}

pub(crate) fn render_toast_root(spec: ToastRootSpec) -> Element {
    let dom_id = spec
        .dom_id
        .clone()
        .unwrap_or_else(|| format!("monoxus-toast-{}", spec.id.0));
    let effective_dismissible = spec.dismissible && spec.toast_type != ToastType::Loading;
    let position = spec.position.unwrap_or(ToastPosition::BottomRight);
    let attrs = ToastRootAttrs::with_options_and_position(
        spec.toast_type,
        spec.phase,
        spec.index,
        spec.visible_limit,
        spec.test_id,
        spec.expanded,
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

    let needs_measurement = spec.computed_style.is_some_and(|cs| cs.height <= 0.0);

    {
        let dom_id_for_remeasure = dom_id.clone();
        let toast_id = spec.id;
        let last_unmeasured = use_hook(|| std::cell::Cell::new(false));
        let last_content_gen = use_hook(|| std::cell::Cell::new(spec.content_generation));
        let was_unmeasured = last_unmeasured.replace(needs_measurement);
        let content_gen_changed =
            spec.content_generation != last_content_gen.replace(spec.content_generation);
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
        let toast_id = spec.id;
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

    let mut base_style = String::from("pointer-events: auto; ");
    if let Some(ref cs) = spec.computed_style {
        base_style.push_str(&cs.to_style_string());
        base_style.push(' ');
    }
    let drag_offset = if sx_val.abs() > sy_val.abs() {
        sx_val
    } else {
        sy_val
    };
    base_style.push_str(&format!(
        "--swipe-amount-x: {sx_val}px; --swipe-amount-y: {sy_val}px; --drag-offset: {drag_offset}px;"
    ));

    let dom_id_for_mount = dom_id.clone();
    let toast_id = spec.id;
    let mounted_flag = if *is_mounted.read() { "true" } else { "false" };
    let data_swipe_out = if swipe_out_val {
        "true"
    } else {
        attrs.data_swipe_out
    };
    let data_testid = attrs.data_testid.filter(|tid| !tid.is_empty());

    let on_pointer_down = onpointerdown;
    let mut on_pointer_move = onpointermove;
    let mut end_for_up = handle_pointer_end.clone();
    let mut end_for_cancel = handle_pointer_end.clone();
    let mut end_for_lost = handle_pointer_end;
    let reducer_for_lost = active_reducer.clone();
    let captured_for_lost = captured_pointer_id.clone();
    let root_for_mount = mounted_root.clone();

    let mut base = Vec::with_capacity(32);
    base.push(Attribute::new("id", dom_id, None, false));
    base.push(Attribute::new("tabindex", "0", None, false));
    base.push(Attribute::new("role", attrs.role, None, false));
    base.push(Attribute::new("aria-live", attrs.aria_live, None, false));
    base.push(Attribute::new("aria-atomic", attrs.aria_atomic, None, false));
    base.push(Attribute::new("data-sonner-toast", "", None, false));
    base.push(Attribute::new("data-mounted", mounted_flag, None, false));
    base.push(Attribute::new("data-id", toast_id.0.to_string(), None, false));
    base.push(Attribute::new("data-state", attrs.data_state, None, false));
    base.push(Attribute::new("data-type", attrs.data_type, None, false));
    base.push(Attribute::new("data-visible", attrs.data_visible, None, false));
    base.push(Attribute::new("data-front", attrs.data_front, None, false));
    base.push(Attribute::new("data-expanded", attrs.data_expanded, None, false));
    base.push(Attribute::new(
        "data-dismissible",
        attrs.data_dismissible,
        None,
        false,
    ));
    base.push(Attribute::new("data-position", attrs.data_position, None, false));
    base.push(Attribute::new(
        "data-x-position",
        attrs.data_x_position,
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-y-position",
        attrs.data_y_position,
        None,
        false,
    ));
    base.push(Attribute::new("data-swipe-out", data_swipe_out, None, false));
    push_attr_opt(&mut base, "data-swipe-direction", swipe_dir_val);
    push_attr_opt(
        &mut base,
        "data-swiping",
        is_swiping_val.then_some("true"),
    );
    push_attr_opt(&mut base, "data-swiped", is_swiping_val.then_some("true"));
    base.push(Attribute::new("data-index", attrs.data_index, None, false));
    push_attr_opt(&mut base, "data-testid", data_testid);
    base.push(Attribute::new("style", base_style, None, false));
    // NOTE(ah:01M7BTR8FNX62N461B3Q6837YJ): See ui/shadcn-dioxus/docs/shadcn_toast_styling_capability_ledger.md
    // (Section 4 Note 6) for co-applying measure_toast_auto_height -> next_frame -> is_mounted.set(true).
    base.push(Attribute::new(
        "onmounted",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                let mounted_evt = evt.map(|d| MountedData::from(d));
                let mounted_el = oxidase::Element::from_mounted(mounted_evt.data());
                *root_for_mount.borrow_mut() = Some(mounted_el);
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
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onpointerdown",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                on_pointer_down(evt.map(|d| PointerData::from(d)));
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onpointermove",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                on_pointer_move(evt.map(|d| PointerData::from(d)));
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onpointerup",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                let ptr_evt = evt.map(|d| PointerData::from(d));
                end_for_up(Some(&ptr_evt), false);
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onpointercancel",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |evt: Event<PlatformEventData>| {
                let ptr_evt = evt.map(|d| PointerData::from(d));
                end_for_cancel(Some(&ptr_evt), true);
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onlostpointercapture",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |_evt: Event<PlatformEventData>| {
                *captured_for_lost.borrow_mut() = None;
                if reducer_for_lost.borrow().is_some() {
                    end_for_lost(None, true);
                }
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onanimationend",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |_evt: Event<PlatformEventData>| {
                if TOAST_STORE
                    .peek()
                    .toasts
                    .iter()
                    .any(|t| t.id == toast_id && t.phase == ToastPhase::Dismissing)
                {
                    TOAST_STORE.write().remove_toast_if_dismissing(toast_id);
                }
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "ontransitionend",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |_evt: Event<PlatformEventData>| {
                if TOAST_STORE
                    .peek()
                    .toasts
                    .iter()
                    .any(|t| t.id == toast_id && t.phase == ToastPhase::Dismissing)
                {
                    TOAST_STORE.write().remove_toast_if_dismissing(toast_id);
                }
            })
            .erase(),
        ),
        None,
        false,
    ));

    let user = collect_user_attrs(spec.class, spec.style, spec.attributes);
    let merged = merge_props(base, user);

    if let Some(child) = spec.child {
        return child.call(merged);
    }

    rsx! {
        li {
            ..merged,
            {spec.children}
        }
    }
}
