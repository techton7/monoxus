use dioxus::prelude::keyboard_types::{Key, Modifiers, NamedKey};
use dioxus::prelude::*;

use crate::{
    calendar::{
        components::day::use_register_calendar_day_id,
        runtime::{is_calendar_activate_key, translate_calendar_nav_key},
    },
    foundation::{
        compose::merge_global_attributes, date_time::CalendarDate, keyboard::is_ime_composing_key,
    },
};

use super::{
    super::{
        attrs::RangeCalendarDayAttrs,
        runtime::use_range_calendar_context,
        types::RangeCalendarCellSnapshot,
    },
    cell::use_range_calendar_cell_context,
};

#[derive(Clone, PartialEq, Props)]
pub struct RangeCalendarDayRenderProps {
    pub attrs: RangeCalendarDayAttrs,
    pub snapshot: RangeCalendarCellSnapshot,
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
pub struct RangeCalendarDayProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<RangeCalendarDayRenderProps, Element>>,
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
    let ctx = use_range_calendar_context();
    let cell_ctx = use_range_calendar_cell_context();
    let runtime = ctx.runtime.clone();

    let cell_date = (cell_ctx.date)();
    let cell_month = (cell_ctx.month)();
    let month_idx = runtime.month_index_for(cell_month);
    let cell_attrs = runtime.cell_attrs(cell_date, cell_month, None);

    let day_attrs = runtime.day_attrs(cell_date, cell_month, &cell_attrs, props.id.clone());
    let snapshot = runtime.cell_snapshot(cell_date, cell_month, &cell_attrs);
    let is_disabled = day_attrs.data_disabled.is_some();
    let is_outside_month = !cell_date.is_same_month(cell_month);

    use_register_calendar_day_id(&runtime.base, month_idx, cell_date, &day_attrs.id);

    #[derive(Clone, Copy)]
    struct RangeDayHandlerSlot {
        date: CalendarDate,
        is_disabled: bool,
        is_outside_month: bool,
        on_click: Option<EventHandler<MouseEvent>>,
        on_key_down: Option<EventHandler<KeyboardEvent>>,
    }

    let mut slot = use_hook(|| {
        CopyValue::new(RangeDayHandlerSlot {
            date: cell_date,
            is_disabled,
            is_outside_month,
            on_click: props.on_click,
            on_key_down: props.on_key_down,
        })
    });
    slot.set(RangeDayHandlerSlot {
        date: cell_date,
        is_disabled,
        is_outside_month,
        on_click: props.on_click,
        on_key_down: props.on_key_down,
    });

    let on_click = use_hook(|| {
        let mut rt = runtime.clone();
        Callback::new(move |evt: MouseEvent| {
            let current = *slot.read();
            if current.is_disabled {
                evt.prevent_default();
                return;
            }
            if let Some(handler) = current.on_click {
                handler.call(evt.clone());
            }
            if evt.default_action_enabled() {
                rt.activate_date(current.date, current.is_outside_month);
            }
        })
    });

    let on_key_down = use_hook(|| {
        let mut rt = runtime.clone();
        Callback::new(move |evt: KeyboardEvent| {
            let current = *slot.read();
            let key = evt.key();

            if key == Key::Named(NamedKey::Escape) && !is_ime_composing_key(&evt) {
                if let Some(handler) = current.on_key_down {
                    handler.call(evt.clone());
                }
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
                if let Some(handler) = current.on_key_down {
                    handler.call(evt.clone());
                }
                if evt.default_action_enabled() {
                    evt.prevent_default();
                    rt.activate_date(current.date, current.is_outside_month);
                }
                return;
            }

            let shift = evt.modifiers().contains(Modifiers::SHIFT);
            if let Some(nav_key) = translate_calendar_nav_key(&key, shift) {
                if let Some(handler) = current.on_key_down {
                    handler.call(evt.clone());
                }
                if evt.default_action_enabled() && rt.handle_nav_key(current.date, nav_key) {
                    evt.prevent_default();
                }
            } else if let Some(handler) = current.on_key_down {
                handler.call(evt);
            }
        })
    });

    let on_hover_or_focus = {
        let mut rt = runtime.clone();
        move || {
            let current = *slot.read();
            if !current.is_disabled {
                rt.set_focused_value(Some(current.date));
            }
        }
    };

    let rendered_children = if let Some(ref render_children) = props.render_children {
        Some(render_children.call(snapshot.clone()))
    } else {
        props.children
    };

    if props.as_child
        && let Some(render_child) = props.render_child.as_ref()
    {
        return render_child.call(RangeCalendarDayRenderProps {
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

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );
    let mut on_mouse_enter = on_hover_or_focus.clone();
    let mut on_focus_in = on_hover_or_focus;
    let on_mounted = props.on_mounted;

    rsx! {
        div {
            id: "{day_attrs.id}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            role: day_attrs.role,
            aria_label: "{day_attrs.aria_label}",
            aria_disabled: day_attrs.aria_disabled,
            tabindex: day_attrs.tabindex,
            "data-unavailable": day_attrs.data_unavailable,
            "data-today": day_attrs.data_today,
            "data-outside-month": day_attrs.data_outside_month,
            "data-outside-visible-months": day_attrs.data_outside_visible_months,
            "data-focused": day_attrs.data_focused,
            "data-selection-start": day_attrs.data_selection_start,
            "data-selection-end": day_attrs.data_selection_end,
            "data-range-start": day_attrs.data_range_start,
            "data-range-end": day_attrs.data_range_end,
            "data-range-middle": day_attrs.data_range_middle,
            "data-highlighted": day_attrs.data_highlighted,
            "data-selected": day_attrs.data_selected,
            "data-value": "{day_attrs.data_value}",
            "data-type": day_attrs.data_type,
            "data-disabled": day_attrs.data_disabled,
            "data-range-calendar-day": day_attrs.data_range_calendar_day,
            "data-bits-day": day_attrs.data_bits_day,
            onclick: move |evt| on_click.call(evt),
            onkeydown: move |evt| on_key_down.call(evt),
            onmouseenter: move |_| on_mouse_enter(),
            onfocusin: move |_| on_focus_in(),
            onmounted: move |evt| {
                if let Some(handler) = on_mounted {
                    handler.call(evt);
                }
            },
            ..merged.rest,
            if let Some(children) = rendered_children {
                {children}
            } else {
                "{snapshot.day_text}"
            }
        }
    }
}
