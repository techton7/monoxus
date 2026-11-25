use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use dioxus::prelude::*;
use futures_timer::Delay;

use crate::foundation::browser::{get_element_scroll_metrics, set_element_scroll_position};

use crate::combobox::{
    attrs::ComboboxScrollButtonAttrs,
    runtime::{ComboboxRuntime, use_combobox_context},
};

const SCROLL_BUTTON_STEP_PX: f64 = 24.0;
const SCROLL_BUTTON_INTERVAL_MS: u64 = 50;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScrollDirection {
    Up,
    Down,
}

fn step_viewport_scroll(rt: &ComboboxRuntime, direction: ScrollDirection) -> bool {
    let vp_id = rt.current_viewport_id_untracked();
    let Some(m) = get_element_scroll_metrics(&vp_id) else {
        return false;
    };
    let max_y = (m.scroll_height - m.client_height).max(0.0);
    let next_y = match direction {
        ScrollDirection::Up => (m.scroll_top - SCROLL_BUTTON_STEP_PX).max(0.0),
        ScrollDirection::Down => (m.scroll_top + SCROLL_BUTTON_STEP_PX).min(max_y),
    };
    let moved = (next_y - m.scroll_top).abs() > 0.25;
    if moved {
        set_element_scroll_position(&vp_id, m.scroll_left, next_y);
    }
    rt.refresh_scroll_buttons();
    moved
}

#[derive(Clone, PartialEq)]
pub struct ComboboxScrollButtonRenderProps {
    pub attrs: ComboboxScrollButtonAttrs,
    pub class: Option<String>,
    pub style: String,
    pub on_mouse_down: EventHandler<MouseEvent>,
    pub on_pointer_down: EventHandler<PointerEvent>,
    pub on_pointer_move: EventHandler<PointerEvent>,
    pub on_pointer_leave: EventHandler<PointerEvent>,
    pub on_pointer_up: EventHandler<PointerEvent>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxScrollButtonProps {
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<ComboboxScrollButtonRenderProps, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

fn render_scroll_button(
    props: ComboboxScrollButtonProps,
    direction: ScrollDirection,
    data_attr: &'static str,
) -> Element {
    let ctx = use_combobox_context();
    let is_open = ctx.runtime.is_open();
    let can_scroll = match direction {
        ScrollDirection::Up => ctx.runtime.can_scroll_up(),
        ScrollDirection::Down => ctx.runtime.can_scroll_down(),
    };

    let auto_scroll_task: Rc<RefCell<Option<dioxus::core::Task>>> =
        use_hook(|| Rc::new(RefCell::new(None)));

    // Cancel auto-scroll timer whenever menu closes or button can no longer scroll.
    {
        let task_slot = Rc::clone(&auto_scroll_task);
        let active = is_open && (can_scroll || props.force_mount);
        use_effect(use_reactive((&active,), move |(still_active,)| {
            if !still_active {
                if let Some(task) = task_slot.borrow_mut().take() {
                    task.cancel();
                }
            }
        }));
    }

    {
        let task_slot = Rc::clone(&auto_scroll_task);
        use_drop(move || {
            if let Some(task) = task_slot.borrow_mut().take() {
                task.cancel();
            }
        });
    }

    if !can_scroll && !props.force_mount {
        return rsx! {};
    }

    let attrs = ComboboxScrollButtonAttrs::new();
    let combined_style = match props.style {
        Some(ref s) if !s.is_empty() => format!("flex-shrink: 0; {s}"),
        _ => "flex-shrink: 0;".to_string(),
    };

    let start_auto_scroll = {
        let rt = ctx.runtime.clone();
        let task_slot = Rc::clone(&auto_scroll_task);
        move || {
            rt.set_user_has_scrolled(true);
            if task_slot.borrow().is_some() {
                return;
            }
            step_viewport_scroll(&rt, direction);
            let loop_rt = rt.clone();
            let loop_slot = Rc::clone(&task_slot);
            let task = spawn(async move {
                loop {
                    Delay::new(Duration::from_millis(SCROLL_BUTTON_INTERVAL_MS)).await;
                    if !loop_rt.is_open_untracked() || !step_viewport_scroll(&loop_rt, direction) {
                        break;
                    }
                }
                *loop_slot.borrow_mut() = None;
            });
            *task_slot.borrow_mut() = Some(task);
        }
    };

    let stop_auto_scroll = {
        let task_slot = Rc::clone(&auto_scroll_task);
        move || {
            if let Some(task) = task_slot.borrow_mut().take() {
                task.cancel();
            }
        }
    };

    let on_mouse_down = EventHandler::new(move |evt: MouseEvent| {
        evt.prevent_default();
    });

    let pd_start = start_auto_scroll.clone();
    let on_pointer_down = EventHandler::new(move |evt: PointerEvent| {
        evt.prevent_default();
        pd_start();
    });

    let pm_start = start_auto_scroll;
    let on_pointer_move = EventHandler::new(move |_evt: PointerEvent| {
        pm_start();
    });

    let pl_stop = stop_auto_scroll.clone();
    let on_pointer_leave = EventHandler::new(move |_evt: PointerEvent| {
        pl_stop();
    });

    let pu_stop = stop_auto_scroll;
    let on_pointer_up = EventHandler::new(move |_evt: PointerEvent| {
        pu_stop();
    });

    if props.as_child {
        if let Some(ref render_child) = props.render_child {
            return render_child.call(ComboboxScrollButtonRenderProps {
                attrs,
                class: props.class,
                style: combined_style,
                on_mouse_down,
                on_pointer_down,
                on_pointer_move,
                on_pointer_leave,
                on_pointer_up,
                attributes: props.attributes,
                children: props.children,
            });
        }
    }

    if data_attr == "data-combobox-scroll-up-button" {
        rsx! {
            div {
                "aria-hidden": "{attrs.aria_hidden()}",
                "data-combobox-scroll-up-button": "",
                class: props.class.as_deref().unwrap_or_default(),
                style: "{combined_style}",
                onmousedown: move |evt| on_mouse_down.call(evt),
                onpointerdown: move |evt| on_pointer_down.call(evt),
                onpointermove: move |evt| on_pointer_move.call(evt),
                onpointerleave: move |evt| on_pointer_leave.call(evt),
                onpointerup: move |evt| on_pointer_up.call(evt),
                ..props.attributes,
                {props.children}
            }
        }
    } else {
        rsx! {
            div {
                "aria-hidden": "{attrs.aria_hidden()}",
                "data-combobox-scroll-down-button": "",
                class: props.class.as_deref().unwrap_or_default(),
                style: "{combined_style}",
                onmousedown: move |evt| on_mouse_down.call(evt),
                onpointerdown: move |evt| on_pointer_down.call(evt),
                onpointermove: move |evt| on_pointer_move.call(evt),
                onpointerleave: move |evt| on_pointer_leave.call(evt),
                onpointerup: move |evt| on_pointer_up.call(evt),
                ..props.attributes,
                {props.children}
            }
        }
    }
}

#[component]
pub fn ComboboxScrollUpButton(props: ComboboxScrollButtonProps) -> Element {
    render_scroll_button(props, ScrollDirection::Up, "data-combobox-scroll-up-button")
}

#[component]
pub fn ComboboxScrollDownButton(props: ComboboxScrollButtonProps) -> Element {
    render_scroll_button(
        props,
        ScrollDirection::Down,
        "data-combobox-scroll-down-button",
    )
}
