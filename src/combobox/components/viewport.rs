use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;

use crate::foundation::browser::{
    ResizeObserverBox, WatcherGuard, start_element_scroll_monitor, start_resize_monitor,
};
use crate::foundation::compose::{collect_user_attrs, merge_props, push_event_handler};

use crate::combobox::runtime::use_combobox_context;

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxViewportProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn ComboboxViewport(props: ComboboxViewportProps) -> Element {
    let ctx = use_combobox_context();
    let vp_id = props
        .id
        .clone()
        .unwrap_or_else(|| ctx.runtime.relationships().viewport_id().to_owned());
    ctx.runtime.set_viewport_id(vp_id.clone());

    let is_open = ctx.runtime.is_open();
    let input_value = ctx.runtime.input_value();
    let reg_ver = *ctx.runtime.state.registry_version.read();
    let scroll_guard: Rc<RefCell<Option<WatcherGuard>>> =
        use_hook(|| Rc::new(RefCell::new(None)));
    let resize_guard: Rc<RefCell<Option<WatcherGuard>>> =
        use_hook(|| Rc::new(RefCell::new(None)));

    {
        let rt = ctx.runtime.clone();
        let vid = vp_id.clone();
        let s_slot = Rc::clone(&scroll_guard);
        let r_slot = Rc::clone(&resize_guard);
        use_effect(use_reactive(
            (&is_open, &input_value, &reg_ver, &vid),
            move |(open, _, _, cur_vid)| {
                if let Some(prev) = s_slot.borrow_mut().take() {
                    prev.stop();
                }
                if let Some(prev) = r_slot.borrow_mut().take() {
                    prev.stop();
                }
                if !open {
                    return;
                }
                rt.refresh_scroll_buttons();
                let scroll_rt = rt.clone();
                let s_guard = start_element_scroll_monitor(&cur_vid, move || {
                    scroll_rt.refresh_scroll_buttons();
                });
                *s_slot.borrow_mut() = Some(s_guard);

                // CBX-10 / DEF-CBX-14: Re-align highlighted item on viewport resize until user scrolls.
                let resize_rt = rt.clone();
                let r_guard =
                    start_resize_monitor(&cur_vid, ResizeObserverBox::BorderBox, move |_| {
                        resize_rt.refresh_scroll_buttons();
                        if !resize_rt.user_has_scrolled_untracked() {
                            if let Some(hl) = resize_rt.highlighted_value_untracked() {
                                resize_rt.scroll_value_into_view(&hl);
                            }
                        }
                    });
                *r_slot.borrow_mut() = Some(r_guard);
            },
        ));
    }

    {
        let s_slot = Rc::clone(&scroll_guard);
        let r_slot = Rc::clone(&resize_guard);
        use_drop(move || {
            if let Some(prev) = s_slot.borrow_mut().take() {
                prev.stop();
            }
            if let Some(prev) = r_slot.borrow_mut().take() {
                prev.stop();
            }
        });
    }

    let wheel_rt = ctx.runtime.clone();
    let on_wheel = EventHandler::new(move |_evt: WheelEvent| {
        wheel_rt.set_user_has_scrolled(true);
    });

    let touch_rt = ctx.runtime.clone();
    let on_touch_move = EventHandler::new(move |_evt: TouchEvent| {
        touch_rt.set_user_has_scrolled(true);
    });

    let attrs = ctx.runtime.viewport_attributes(vp_id);
    let default_style =
        "position: relative; flex: 1 1 auto; min-height: 0; overflow-x: hidden; overflow-y: auto;";

    let id_str = attrs.id().to_owned();
    let role_str = attrs.role();

    let mut base = Vec::with_capacity(6);
    base.push(Attribute::new("id", id_str, None, false));
    base.push(Attribute::new("role", role_str, None, false));
    base.push(Attribute::new("data-combobox-viewport", "", None, false));
    base.push(Attribute::new("style", default_style, None, false));
    push_event_handler(&mut base, "onwheel", Some(on_wheel));
    push_event_handler(&mut base, "ontouchmove", Some(on_touch_move));

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
