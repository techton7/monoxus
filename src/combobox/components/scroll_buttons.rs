use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use dioxus::prelude::*;
use futures_timer::Delay;

use crate::foundation::browser::{get_element_scroll_metrics, set_element_scroll_position};
use crate::foundation::compose::{collect_user_attrs, merge_props, push_event_handler};

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

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxScrollButtonProps {
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
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

    let mut base = Vec::with_capacity(8);
    base.push(Attribute::new("aria-hidden", attrs.aria_hidden(), None, false));
    base.push(Attribute::new(data_attr, "", None, false));
    base.push(Attribute::new("style", "flex-shrink: 0;", None, false));
    push_event_handler(&mut base, "onmousedown", Some(on_mouse_down));
    push_event_handler(&mut base, "onpointerdown", Some(on_pointer_down));
    push_event_handler(&mut base, "onpointermove", Some(on_pointer_move));
    push_event_handler(&mut base, "onpointerleave", Some(on_pointer_leave));
    push_event_handler(&mut base, "onpointerup", Some(on_pointer_up));

    let user = collect_user_attrs(props.class, props.style, props.attributes);
    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            {props.children}
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
