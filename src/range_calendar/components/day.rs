use dioxus::prelude::keyboard_types::{Key, Modifiers, NamedKey};
use dioxus::prelude::*;

use crate::{
    calendar::{
        components::day::use_register_calendar_day_id,
        runtime::{is_calendar_activate_key, translate_calendar_nav_key},
    },
    foundation::{
        compose::{
            MergePropsConfig, collect_user_attrs, merge_props_with_config, push_attr_opt,
            push_event_handler,
        },
        date_time::CalendarDate,
        keyboard::is_ime_composing_key,
    },
};

use super::{
    super::{runtime::use_range_calendar_context, types::RangeCalendarCellSnapshot},
    cell::use_range_calendar_cell_context,
};

#[derive(Clone, PartialEq, Props)]
pub struct RangeCalendarDayProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub render_children: Option<Callback<RangeCalendarCellSnapshot, Element>>,
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
pub fn RangeCalendarDay(props: RangeCalendarDayProps) -> Element {
    let RangeCalendarDayProps {
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

    let ctx = use_range_calendar_context();
    let cell_ctx = use_range_calendar_cell_context();
    let runtime = ctx.runtime.clone();
    let _ = runtime.predicate_rev();

    let cell_date = (cell_ctx.date)();
    let cell_month = (cell_ctx.month)();
    let month_idx = runtime.month_index_for(cell_month);
    let cell_attrs = runtime.cell_attrs(cell_date, cell_month, None);

    let day_attrs = runtime.day_attrs(cell_date, cell_month, &cell_attrs, id);
    let snapshot = runtime.cell_snapshot(cell_date, cell_month, &cell_attrs);
    let is_disabled = day_attrs.data_disabled.is_some();
    let is_outside_month = !cell_date.is_same_month(cell_month);

    use_register_calendar_day_id(&runtime.base, month_idx, cell_date, &day_attrs.id);

    #[derive(Clone, Copy)]
    struct RangeDayHandlerSlot {
        date: CalendarDate,
        is_disabled: bool,
        is_outside_month: bool,
    }

    let mut slot = use_hook(|| {
        CopyValue::new(RangeDayHandlerSlot {
            date: cell_date,
            is_disabled,
            is_outside_month,
        })
    });
    slot.set(RangeDayHandlerSlot {
        date: cell_date,
        is_disabled,
        is_outside_month,
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
                rt.activate_date(current.date, current.is_outside_month);
            }
        })
    });

    let base_on_key_down = use_hook(|| {
        let mut rt = runtime.clone();
        EventHandler::new(move |evt: KeyboardEvent| {
            let current = *slot.read();
            let key = evt.key();

            if key == Key::Named(NamedKey::Escape) && !is_ime_composing_key(&evt) {
                if evt.default_action_enabled() && rt.handle_escape() {
                    evt.prevent_default();
                    evt.stop_propagation();
                }
                return;
            }

            if is_calendar_activate_key(&key) {
                if current.is_disabled {
                    evt.prevent_default();
                    return;
                }
                if evt.default_action_enabled() {
                    evt.prevent_default();
                    rt.activate_date(current.date, current.is_outside_month);
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

    let base_on_mouse_enter = use_hook(|| {
        let mut rt = runtime.clone();
        EventHandler::new(move |_: MouseEvent| {
            let current = *slot.read();
            if !current.is_disabled {
                rt.set_focused_value(Some(current.date));
            }
        })
    });

    let base_on_focus_in = use_hook(|| {
        let mut rt = runtime.clone();
        EventHandler::new(move |_: FocusEvent| {
            let current = *slot.read();
            if !current.is_disabled {
                rt.set_focused_value(Some(current.date));
            }
        })
    });

    let rendered_children = if let Some(ref render_children) = render_children {
        Some(render_children.call(snapshot.clone()))
    } else {
        children
    };

    let mut base = Vec::with_capacity(28);
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
    push_attr_opt(
        &mut base,
        "data-selection-start",
        day_attrs.data_selection_start,
    );
    push_attr_opt(
        &mut base,
        "data-selection-end",
        day_attrs.data_selection_end,
    );
    push_attr_opt(&mut base, "data-range-start", day_attrs.data_range_start);
    push_attr_opt(&mut base, "data-range-end", day_attrs.data_range_end);
    push_attr_opt(&mut base, "data-range-middle", day_attrs.data_range_middle);
    push_attr_opt(&mut base, "data-highlighted", day_attrs.data_highlighted);
    push_attr_opt(&mut base, "data-selected", day_attrs.data_selected);
    base.push(Attribute::new("data-value", day_attrs.data_value, None, false));
    base.push(Attribute::new("data-type", day_attrs.data_type, None, false));
    push_attr_opt(&mut base, "data-disabled", day_attrs.data_disabled);
    base.push(Attribute::new(
        "data-range-calendar-day",
        day_attrs.data_range_calendar_day,
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
    push_event_handler(&mut base, "onmouseenter", Some(base_on_mouse_enter));
    push_event_handler(&mut base, "onfocusin", Some(base_on_focus_in));

    let mut user = collect_user_attrs(class, style, attributes);
    if !is_disabled {
        push_event_handler(&mut user, "onclick", on_click);
    }
    push_event_handler(&mut user, "onkeydown", on_key_down);
    push_event_handler(&mut user, "onmounted", on_mounted);

    let merged = merge_props_with_config(
        base,
        user,
        MergePropsConfig {
            invariant_attrs: &[],
            bookkeeping_events: &["onmouseenter", "onfocusin"],
        },
    );

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
