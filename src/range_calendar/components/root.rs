use std::{borrow::Cow, cell::Cell, rc::Rc};

use dioxus::prelude::keyboard_types::{Key, NamedKey};
use dioxus::prelude::*;

use crate::{
    calendar::{
        attrs::CalendarRootAttrs,
        components::root::render_calendar_internal_children,
        runtime::{CalendarSharedContext, use_calendar_base_runtime},
        types::CalendarKind,
    },
    foundation::{
        compose::merge_global_attributes,
        date_time::{
            CalendarDate, CalendarLocale, DateRange, MonthFormat, Weekday, WeekdayFormat,
            YearFormat,
        },
        keyboard::is_ime_composing_key,
        overlay::OverlayEscapeGuardContext,
        shared::{Direction, ScopeHandle, use_direction_context, use_stable_id},
        state::sync_signal,
    },
};

use super::super::{
    relationships::RangeCalendarRelationships,
    runtime::{RangeCalendarContext, RangeCalendarRuntime},
    state::RangeCalendarDefinition,
    types::RangeCalendarSnapshot,
};

#[derive(Clone, PartialEq, Props)]
pub struct RangeCalendarRootRenderProps {
    pub attrs: CalendarRootAttrs,
    pub snapshot: RangeCalendarSnapshot,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Clone, PartialEq, Props)]
pub struct RangeCalendarRootProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub value: Option<Signal<DateRange>>,
    #[props(default)]
    pub default_value: Option<DateRange>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<DateRange>>,
    #[props(default)]
    pub start_value: Option<Signal<Option<CalendarDate>>>,
    #[props(default)]
    pub on_start_value_change: Option<EventHandler<Option<CalendarDate>>>,
    #[props(default)]
    pub end_value: Option<Signal<Option<CalendarDate>>>,
    #[props(default)]
    pub on_end_value_change: Option<EventHandler<Option<CalendarDate>>>,
    #[props(default)]
    pub on_range_select: Option<EventHandler<DateRange>>,
    #[props(default)]
    pub min_days: Option<usize>,
    #[props(default)]
    pub max_days: Option<usize>,
    #[props(default = false)]
    pub exclude_disabled: bool,
    #[props(default)]
    pub placeholder: Option<Signal<CalendarDate>>,
    #[props(default)]
    pub default_placeholder: Option<CalendarDate>,
    #[props(default)]
    pub on_placeholder_change: Option<EventHandler<CalendarDate>>,
    #[props(default)]
    pub min_value: Option<CalendarDate>,
    #[props(default)]
    pub max_value: Option<CalendarDate>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub read_only: bool,
    #[props(default = false)]
    pub prevent_deselect: bool,
    #[props(default = false)]
    pub paged_navigation: bool,
    #[props(default = false)]
    pub fixed_weeks: bool,
    #[props(default = 1)]
    pub number_of_months: usize,
    #[props(default)]
    pub week_starts_on: Option<Weekday>,
    #[props(default)]
    pub weekday_format: WeekdayFormat,
    #[props(default)]
    pub month_format: MonthFormat,
    #[props(default)]
    pub year_format: YearFormat,
    #[props(default)]
    pub locale: CalendarLocale,
    #[props(default = String::from("Event"))]
    pub calendar_label: String,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default = true)]
    pub disable_days_outside_month: bool,
    #[props(default = false)]
    pub initial_focus: bool,
    #[props(default)]
    pub is_date_disabled: Option<Callback<CalendarDate, bool>>,
    #[props(default)]
    pub is_date_unavailable: Option<Callback<CalendarDate, bool>>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<RangeCalendarRootRenderProps, Element>>,
    #[props(default)]
    pub render_children: Option<Callback<RangeCalendarSnapshot, Element>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Plain Rust render surface for `RangeCalendarRoot` and `DateRangePickerCalendar` (zero nested
/// `#[component]` taking `Option<Callback|EventHandler>` per `Dioxus #5894`).
pub(crate) fn render_range_calendar_root(props: RangeCalendarRootProps) -> Element {
    let resolved_dir = use_direction_context(props.dir);
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned)).into_owned();
    let scope = use_hook(|| ScopeHandle::root("range-calendar").child(stable_id.clone()));
    let rels = use_hook(|| RangeCalendarRelationships::new(scope.clone()));
    let dom_root_id = props.id.clone().unwrap_or_else(|| rels.root_id().to_string());

    let initial_range = props
        .value
        .map(|s| *s.peek())
        .or(props.default_value)
        .unwrap_or_default();
    let initial_start = props
        .start_value
        .and_then(|s| *s.peek())
        .or(initial_range.start);
    let initial_end = props
        .end_value
        .and_then(|s| *s.peek())
        .or(initial_range.end);

    let fallback_value = use_signal(|| DateRange {
        start: initial_start,
        end: initial_end,
    });
    let value_sig = props.value.unwrap_or(fallback_value);

    let fallback_start = use_signal(|| initial_start);
    let start_sig = props.start_value.unwrap_or(fallback_start);

    let fallback_end = use_signal(|| initial_end);
    let end_sig = props.end_value.unwrap_or(fallback_end);

    let def = RangeCalendarDefinition::new(scope.clone())
        .with_min_value(props.min_value)
        .with_max_value(props.max_value)
        .with_disabled(props.disabled)
        .with_read_only(props.read_only)
        .with_prevent_deselect(props.prevent_deselect)
        .with_paged_navigation(props.paged_navigation)
        .with_fixed_weeks(props.fixed_weeks)
        .with_number_of_months(props.number_of_months)
        .with_week_starts_on(props.week_starts_on)
        .with_weekday_format(props.weekday_format)
        .with_month_format(props.month_format)
        .with_year_format(props.year_format)
        .with_locale(props.locale.clone())
        .with_calendar_label(props.calendar_label.clone())
        .with_direction(resolved_dir)
        .with_disable_days_outside_month(props.disable_days_outside_month)
        .with_min_days(props.min_days)
        .with_max_days(props.max_days)
        .with_exclude_disabled(props.exclude_disabled);

    let base = use_calendar_base_runtime(
        CalendarKind::RangeCalendar,
        rels.clone(),
        dom_root_id,
        def.config.clone(),
        props.placeholder,
        props.default_placeholder,
        initial_start,
        props.initial_focus,
        props.is_date_disabled,
        props.is_date_unavailable,
        props.on_placeholder_change,
    );

    let mut def_sig = use_signal(|| def.clone());
    let last_pressed_sig = use_signal(|| None::<CalendarDate>);
    let focused_value_sig = use_signal(|| None::<CalendarDate>);

    let mut on_value_change = use_hook(|| CopyValue::new(props.on_value_change));
    let mut on_start_value_change = use_hook(|| CopyValue::new(props.on_start_value_change));
    let mut on_end_value_change = use_hook(|| CopyValue::new(props.on_end_value_change));
    let mut on_range_select = use_hook(|| CopyValue::new(props.on_range_select));
    on_value_change.set(props.on_value_change);
    on_start_value_change.set(props.on_start_value_change);
    on_end_value_change.set(props.on_end_value_change);
    on_range_select.set(props.on_range_select);

    use_effect(use_reactive(&def, move |next_def| {
        sync_signal(&mut def_sig, next_def);
    }));

    let runtime = RangeCalendarRuntime {
        base: base.clone(),
        def: def_sig,
        value: value_sig,
        start_value: start_sig,
        end_value: end_sig,
        last_pressed: last_pressed_sig,
        focused_value: focused_value_sig,
        on_value_change,
        on_start_value_change,
        on_end_value_change,
        on_range_select,
    };

    // Sync external value/placeholder Signal updates and reconcile exclude_disabled.
    let mut last_value_seen = use_hook(|| CopyValue::new(initial_range));
    {
        let mut rt = runtime.clone();
        use_effect(move || {
            let _ = rt.predicate_rev();
            let current = *rt.value.read();
            if *last_value_seen.read() != current {
                last_value_seen.set(current);
                if *rt.start_value.peek() != current.start {
                    rt.start_value.set(current.start);
                }
                if *rt.end_value.peek() != current.end {
                    rt.end_value.set(current.end);
                }
                if let Some(start) = current.start
                    && *rt.placeholder.peek() != start
                {
                    rt.set_placeholder(start);
                }
            }

            rt.reconcile_disabled_in_range();

            let ph = *rt.placeholder.read();
            let in_view = rt.months.peek().iter().any(|m| m.value.is_same_month(ph));
            if !in_view {
                rt.rebuild_months_from(ph);
            }
        });
    }

    use_context_provider(|| RangeCalendarContext {
        runtime: runtime.clone(),
    });
    use_context_provider(|| CalendarSharedContext { runtime: base });

    let overlay_escape_guard = try_consume_context::<OverlayEscapeGuardContext>();
    let guard_consumed_escape = use_hook(|| Rc::new(Cell::new(false)));
    {
        let guard_ctx = overlay_escape_guard.clone();
        let guard_id = rels.root_id().to_string();
        let mut rt_for_guard = runtime.clone();
        let consumed_flag = Rc::clone(&guard_consumed_escape);
        use_hook(move || {
            if let Some(ref ctx) = guard_ctx {
                ctx.register_guard(guard_id, move || {
                    let handled = rt_for_guard.handle_escape();
                    consumed_flag.set(handled);
                    handled
                });
            }
        });
    }
    {
        let guard_ctx = overlay_escape_guard;
        let guard_id = rels.root_id().to_string();
        use_drop(move || {
            if let Some(ref ctx) = guard_ctx {
                ctx.unregister_guard(guard_id);
            }
        });
    }

    let snapshot = runtime.snapshot();
    let root_attrs = runtime.root_attrs(None);
    let announcement_text = (runtime.announcement)();
    let body = if let Some(ref render_children) = props.render_children {
        render_children.call(snapshot.clone())
    } else {
        props.children
    };
    let internal_children = render_calendar_internal_children(
        rels.accessible_heading_id(),
        rels.live_region_id(),
        &snapshot.full_label,
        &announcement_text,
        body,
    );

    let mut consumer_on_key_down_slot = use_hook(|| CopyValue::new(props.on_key_down));
    consumer_on_key_down_slot.set(props.on_key_down);
    let composed_on_key_down = use_hook(|| {
        let mut rt_for_key = runtime.clone();
        let consumed_flag = Rc::clone(&guard_consumed_escape);
        EventHandler::new(move |evt: KeyboardEvent| {
            if let Some(handler) = consumer_on_key_down_slot
                .try_read()
                .ok()
                .and_then(|slot| *slot)
            {
                handler.call(evt.clone());
            }
            if evt.default_action_enabled()
                && !is_ime_composing_key(&evt)
                && evt.key() == Key::Named(NamedKey::Escape)
                && !crate::foundation::overlay::is_overlay_escape_consumed()
                && (consumed_flag.replace(false) || rt_for_key.handle_escape())
            {
                crate::foundation::overlay::mark_overlay_escape_consumed();
                evt.prevent_default();
                evt.stop_propagation();
            }
        })
    });

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(RangeCalendarRootRenderProps {
                attrs: root_attrs,
                snapshot,
                class: props.class,
                style: props.style,
                on_key_down: Some(composed_on_key_down),
                on_mounted: props.on_mounted,
                attributes: props.attributes,
                children: internal_children,
            });
        }
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::RangeCalendarRoot: as_child requires render_child; rendering the default div"
        );
    }

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );
    let on_mounted = props.on_mounted;

    rsx! {
        div {
            id: "{root_attrs.id}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            role: root_attrs.role,
            aria_label: "{root_attrs.aria_label}",
            dir: root_attrs.dir,
            "data-range-calendar-root": root_attrs.data_range_calendar_root,
            "data-invalid": root_attrs.data_invalid,
            "data-disabled": root_attrs.data_disabled,
            "data-readonly": root_attrs.data_readonly,
            onkeydown: move |evt: KeyboardEvent| composed_on_key_down.call(evt),
            onmounted: move |evt| {
                if let Some(handler) = on_mounted {
                    handler.call(evt);
                }
            },
            ..merged.rest,
            {internal_children}
        }
    }
}

#[component]
pub fn RangeCalendarRoot(props: RangeCalendarRootProps) -> Element {
    render_range_calendar_root(props)
}

pub use RangeCalendarRoot as RangeCalendar;
