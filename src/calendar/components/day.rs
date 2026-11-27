use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::keyboard_types::Modifiers;
use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
    date_time::CalendarDate,
};

use super::{
    super::{
        runtime::{
            CalendarBaseRuntime, is_calendar_activate_key, translate_calendar_nav_key,
            use_calendar_context,
        },
        types::CalendarCellSnapshot,
    },
    cell::use_calendar_cell_context,
};

/// Shared day-ID registration + drop cleanup hook used by both `CalendarDay` and `RangeCalendarDay`.
pub(crate) fn use_register_calendar_day_id(
    base: &CalendarBaseRuntime,
    month_idx: usize,
    cell_date: CalendarDate,
    day_id: &str,
) {
    let registered_key: Rc<RefCell<Option<(usize, CalendarDate)>>> =
        use_hook(|| Rc::new(RefCell::new(None)));
    {
        let next_key = (month_idx, cell_date);
        if *registered_key.borrow() != Some(next_key) {
            if let Some((prev_idx, prev_date)) = *registered_key.borrow() {
                base.unregister_day_id(prev_idx, prev_date);
            }
            *registered_key.borrow_mut() = Some(next_key);
        }
        base.register_day_id(month_idx, cell_date, day_id.to_owned());
    }
    let cleanup_base = base.clone();
    let cleanup_key = Rc::clone(&registered_key);
    use_drop(move || {
        if let Some((idx, date)) = cleanup_key.borrow_mut().take() {
            cleanup_base.unregister_day_id(idx, date);
        }
    });
}

#[derive(Clone, PartialEq, Props)]
pub struct CalendarDayProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
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
    let CalendarDayProps {
        id,
        class,
        style,
        child,
        render_children,
        on_click,
        on_key_down,
        on_mounted,
        attributes,
        children,
    } = props;

    let ctx = use_calendar_context();
    let cell_ctx = use_calendar_cell_context();
    let runtime = ctx.runtime.clone();
    let _ = runtime.predicate_rev();

    let cell_date = (cell_ctx.date)();
    let cell_month = (cell_ctx.month)();
    let month_idx = runtime.month_index_for(cell_month);
    let cell_attrs = runtime.cell_attrs(cell_date, cell_month, None);

    let day_attrs = runtime.day_attrs(cell_date, cell_month, &cell_attrs, id);
    let snapshot = runtime.cell_snapshot(cell_date, cell_month, &cell_attrs);
    let is_disabled = day_attrs.data_disabled.is_some();

    use_register_calendar_day_id(&runtime.base, month_idx, cell_date, &day_attrs.id);

    #[derive(Clone, Copy)]
    struct DayHandlerSlot {
        date: CalendarDate,
        is_disabled: bool,
    }

    let mut slot = use_hook(|| {
        CopyValue::new(DayHandlerSlot {
            date: cell_date,
            is_disabled,
        })
    });
    slot.set(DayHandlerSlot {
        date: cell_date,
        is_disabled,
    });

    let base_on_click = use_hook(|| {
        let mut rt = runtime.clone();
        EventHandler::new(move |evt: MouseEvent| {
            let current = *slot.read();
            if current.is_disabled {
                evt.prevent_default();
                return;
            }
            if evt.default_action_enabled() {
                rt.activate_date(current.date);
            }
        })
    });

    let base_on_key_down = use_hook(|| {
        let mut rt = runtime.clone();
        EventHandler::new(move |evt: KeyboardEvent| {
            let current = *slot.read();
            let key = evt.key();
            if is_calendar_activate_key(&key) {
                if current.is_disabled {
                    evt.prevent_default();
                    return;
                }
                if evt.default_action_enabled() {
                    evt.prevent_default();
                    rt.activate_date(current.date);
                }
                return;
            }

            let shift = evt.modifiers().contains(Modifiers::SHIFT);
            if let Some(nav_key) = translate_calendar_nav_key(&key, shift)
                && evt.default_action_enabled()
                && rt.handle_nav_key(current.date, nav_key)
            {
                evt.prevent_default();
            }
        })
    });

    let rendered_children = if let Some(ref render_children) = render_children {
        Some(render_children.call(snapshot.clone()))
    } else {
        children
    };

    let mut base = Vec::with_capacity(20);
    base.push(Attribute::new("id", day_attrs.id, None, false));
    base.push(Attribute::new("role", day_attrs.role, None, false));
    base.push(Attribute::new("aria-label", day_attrs.aria_label, None, false));
    base.push(Attribute::new("aria-disabled", day_attrs.aria_disabled, None, false));
    push_attr_opt(&mut base, "tabindex", day_attrs.tabindex);
    push_attr_opt(&mut base, "data-unavailable", day_attrs.data_unavailable);
    push_attr_opt(&mut base, "data-today", day_attrs.data_today);
    push_attr_opt(&mut base, "data-outside-month", day_attrs.data_outside_month);
    push_attr_opt(
        &mut base,
        "data-outside-visible-months",
        day_attrs.data_outside_visible_months,
    );
    push_attr_opt(&mut base, "data-focused", day_attrs.data_focused);
    push_attr_opt(&mut base, "data-selected", day_attrs.data_selected);
    base.push(Attribute::new("data-value", day_attrs.data_value, None, false));
    base.push(Attribute::new("data-type", day_attrs.data_type, None, false));
    push_attr_opt(&mut base, "data-disabled", day_attrs.data_disabled);
    base.push(Attribute::new(
        "data-calendar-day",
        day_attrs.data_calendar_day,
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-bits-day",
        day_attrs.data_bits_day,
        None,
        false,
    ));
    push_event_handler(&mut base, "onclick", Some(base_on_click));
    push_event_handler(&mut base, "onkeydown", Some(base_on_key_down));

    let mut user = collect_user_attrs(class, style, attributes);
    if !is_disabled {
        push_event_handler(&mut user, "onclick", on_click);
    }
    push_event_handler(&mut user, "onkeydown", on_key_down);
    push_event_handler(&mut user, "onmounted", on_mounted);

    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            if let Some(children) = rendered_children {
                {children}
            } else {
                "{snapshot.day_text}"
            }
        }
    }
}
