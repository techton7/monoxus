use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::keyboard_types::Modifiers;
use dioxus::prelude::*;

use crate::foundation::date_time::CalendarDate;

use super::{
    super::{
        attrs::CalendarDayAttrs,
        runtime::{
            is_calendar_activate_key, translate_calendar_nav_key, use_calendar_context,
        },
        types::CalendarCellSnapshot,
    },
    cell::use_calendar_cell_context,
};

#[derive(Clone, PartialEq, Props)]
pub struct CalendarDayRenderProps {
    pub attrs: CalendarDayAttrs,
    pub snapshot: CalendarCellSnapshot,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    pub on_click: EventHandler<MouseEvent>,
    pub on_key_down: EventHandler<KeyboardEvent>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Option<Element>,
}

#[derive(Clone, PartialEq, Props)]
pub struct CalendarDayProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<CalendarDayRenderProps, Element>>,
    #[props(default)]
    pub render_children: Option<Callback<CalendarCellSnapshot, Element>>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Option<Element>,
}

#[component]
pub fn CalendarDay(props: CalendarDayProps) -> Element {
    let ctx = use_calendar_context();
    let cell_ctx = use_calendar_cell_context();
    let runtime = ctx.runtime.clone();

    let cell_date = (cell_ctx.date)();
    let cell_month = (cell_ctx.month)();
    let month_idx = runtime.month_index_for(cell_month);
    let cell_attrs = runtime.cell_attrs(cell_date, cell_month, None);

    let day_attrs = runtime.day_attrs(
        cell_date,
        cell_month,
        &cell_attrs,
        props.id.clone(),
    );
    let snapshot = runtime.cell_snapshot(cell_date, cell_month, &cell_attrs);
    let is_disabled = day_attrs.data_disabled.is_some();
    let day_id = day_attrs.id.clone();

    // Register custom/effective ID so cross-month keyboard navigation focuses this exact element.
    let registered_key: Rc<RefCell<Option<(usize, CalendarDate)>>> =
        use_hook(|| Rc::new(RefCell::new(None)));
    {
        let next_key = (month_idx, cell_date);
        if *registered_key.borrow() != Some(next_key) {
            if let Some((prev_idx, prev_date)) = *registered_key.borrow() {
                runtime.unregister_day_id(prev_idx, prev_date);
            }
            *registered_key.borrow_mut() = Some(next_key);
        }
        runtime.register_day_id(month_idx, cell_date, day_id.clone());
    }
    let cleanup_runtime = runtime.clone();
    let cleanup_key = Rc::clone(&registered_key);
    use_drop(move || {
        if let Some((idx, date)) = cleanup_key.borrow_mut().take() {
            cleanup_runtime.unregister_day_id(idx, date);
        }
    });

    #[derive(Clone)]
    struct DayHandlerSlot {
        date: CalendarDate,
        is_disabled: bool,
        on_click: Option<EventHandler<MouseEvent>>,
        on_key_down: Option<EventHandler<KeyboardEvent>>,
    }

    let slot = use_hook(|| {
        Rc::new(RefCell::new(DayHandlerSlot {
            date: cell_date,
            is_disabled,
            on_click: props.on_click,
            on_key_down: props.on_key_down,
        }))
    });
    *slot.borrow_mut() = DayHandlerSlot {
        date: cell_date,
        is_disabled,
        on_click: props.on_click,
        on_key_down: props.on_key_down,
    };

    let on_click = use_hook(|| {
        let mut rt = runtime.clone();
        let slot = Rc::clone(&slot);
        Callback::new(move |evt: MouseEvent| {
            let current = slot.borrow().clone();
            if current.is_disabled {
                evt.prevent_default();
                return;
            }
            if let Some(ref handler) = current.on_click {
                handler.call(evt.clone());
            }
            if evt.default_action_enabled() {
                rt.activate_date(current.date);
            }
        })
    });

    let on_key_down = use_hook(|| {
        let mut rt = runtime.clone();
        let slot = Rc::clone(&slot);
        Callback::new(move |evt: KeyboardEvent| {
            let current = slot.borrow().clone();
            let key = evt.key();
            if is_calendar_activate_key(&key) {
                if current.is_disabled {
                    evt.prevent_default();
                    return;
                }
                if let Some(ref handler) = current.on_key_down {
                    handler.call(evt.clone());
                }
                if evt.default_action_enabled() {
                    evt.prevent_default();
                    rt.activate_date(current.date);
                }
                return;
            }

            let shift = evt.modifiers().contains(Modifiers::SHIFT);
            if let Some(nav_key) = translate_calendar_nav_key(&key, shift) {
                if let Some(ref handler) = current.on_key_down {
                    handler.call(evt.clone());
                }
                if evt.default_action_enabled() && rt.handle_nav_key(current.date, nav_key) {
                    evt.prevent_default();
                }
            } else if let Some(ref handler) = current.on_key_down {
                handler.call(evt);
            }
        })
    });

    let rendered_children = if let Some(ref render_children) = props.render_children {
        Some(render_children.call(snapshot.clone()))
    } else {
        props.children
    };

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(CalendarDayRenderProps {
                attrs: day_attrs,
                snapshot,
                class: props.class,
                style: props.style,
                on_click,
                on_key_down,
                on_mounted: props.on_mounted,
                attributes: props.attributes,
                children: rendered_children,
            });
        }
    }

    rsx! {
        div {
            id: "{day_attrs.id}",
            class: props.class,
            style: props.style,
            role: day_attrs.role,
            aria_label: "{day_attrs.aria_label}",
            aria_disabled: day_attrs.aria_disabled,
            tabindex: day_attrs.tabindex,
            "data-unavailable": day_attrs.data_unavailable,
            "data-today": day_attrs.data_today,
            "data-outside-month": day_attrs.data_outside_month,
            "data-outside-visible-months": day_attrs.data_outside_visible_months,
            "data-focused": day_attrs.data_focused,
            "data-selected": day_attrs.data_selected,
            "data-value": "{day_attrs.data_value}",
            "data-type": day_attrs.data_type,
            "data-disabled": day_attrs.data_disabled,
            "data-calendar-day": day_attrs.data_calendar_day,
            "data-bits-day": day_attrs.data_bits_day,
            onclick: move |evt| on_click.call(evt),
            onkeydown: move |evt| on_key_down.call(evt),
            onmounted: move |evt| {
                if let Some(ref handler) = props.on_mounted {
                    handler.call(evt);
                }
            },
            ..props.attributes,
            if let Some(children) = rendered_children {
                {children}
            } else {
                "{snapshot.day_text}"
            }
        }
    }
}
