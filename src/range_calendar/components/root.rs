use std::{borrow::Cow, cell::Cell, rc::Rc};

use dioxus::prelude::keyboard_types::{Key, NamedKey};
use dioxus::prelude::*;
use ui_sdk::kernel::ChangeEvent;

use crate::{
    agent::{
        ActionDescriptor, AgentHints, AgentSurface, ComponentSnapshot, DispatchExecResult,
        SnapshotValue, SurfaceMetaDefaults, use_agent_surface,
    },
    calendar::{
        components::root::render_calendar_internal_children,
        runtime::{
            CalendarSharedContext, calendar_locale_tag, parse_agent_range_arg,
            range_snapshot_value, use_calendar_base_runtime,
        },
        types::CalendarKind,
    },
    foundation::{
        compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
        config::use_locale_context,
        date_time::{
            CalendarDate, CalendarLocale, DateRange, MonthFormat, Weekday, WeekdayFormat,
            YearFormat,
        },
        keyboard::is_ime_composing_key,
        overlay::OverlayEscapeGuardContext,
        shared::{Direction, ScopeHandle, use_direction_context, use_stable_id},
        state::{ControllableConfig, sync_signal, use_controllable_signal_config},
    },
};

use super::super::{
    relationships::RangeCalendarRelationships,
    runtime::{RangeCalendarContext, RangeCalendarRuntime},
    state::RangeCalendarDefinition,
    types::RangeCalendarSnapshot,
};

#[derive(Clone, PartialEq, Props)]
pub struct RangeCalendarRootProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub value: Option<DateRange>,
    #[props(default)]
    pub default_value: Option<DateRange>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<DateRange>>,
    #[props(default)]
    pub on_value_change_details: Option<EventHandler<ChangeEvent<DateRange>>>,
    #[props(default)]
    pub start_value: Option<Option<CalendarDate>>,
    #[props(default)]
    pub on_start_value_change: Option<EventHandler<Option<CalendarDate>>>,
    #[props(default)]
    pub on_start_value_change_details: Option<EventHandler<ChangeEvent<Option<CalendarDate>>>>,
    #[props(default)]
    pub end_value: Option<Option<CalendarDate>>,
    #[props(default)]
    pub on_end_value_change: Option<EventHandler<Option<CalendarDate>>>,
    #[props(default)]
    pub on_end_value_change_details: Option<EventHandler<ChangeEvent<Option<CalendarDate>>>>,
    #[props(default)]
    pub on_range_select: Option<EventHandler<DateRange>>,
    #[props(default)]
    pub min_days: Option<usize>,
    #[props(default)]
    pub max_days: Option<usize>,
    #[props(default = false)]
    pub exclude_disabled: bool,
    #[props(default)]
    pub placeholder: Option<CalendarDate>,
    #[props(default)]
    pub default_placeholder: Option<CalendarDate>,
    #[props(default)]
    pub on_placeholder_change: Option<EventHandler<CalendarDate>>,
    #[props(default)]
    pub on_placeholder_change_details: Option<EventHandler<ChangeEvent<CalendarDate>>>,
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
    pub locale: Option<CalendarLocale>,
    #[props(default = String::from("Event"), into)]
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
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub render_children: Option<Callback<RangeCalendarSnapshot, Element>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(default)]
    pub agent: Option<AgentHints>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Plain Rust render surface for `RangeCalendarRoot` and `DateRangePickerCalendar` (zero nested
/// `#[component]` taking `Option<Callback|EventHandler>` per `Dioxus #5894`).
pub(crate) fn render_range_calendar_root(props: RangeCalendarRootProps) -> Element {
    let RangeCalendarRootProps {
        id,
        class,
        style,
        value,
        default_value,
        on_value_change,
        on_value_change_details,
        start_value,
        on_start_value_change,
        on_start_value_change_details,
        end_value,
        on_end_value_change,
        on_end_value_change_details,
        on_range_select,
        min_days,
        max_days,
        exclude_disabled,
        placeholder,
        default_placeholder,
        on_placeholder_change,
        on_placeholder_change_details,
        min_value,
        max_value,
        disabled,
        read_only,
        prevent_deselect,
        paged_navigation,
        fixed_weeks,
        number_of_months,
        week_starts_on,
        weekday_format,
        month_format,
        year_format,
        locale,
        calendar_label,
        dir,
        disable_days_outside_month,
        initial_focus,
        is_date_disabled,
        is_date_unavailable,
        child,
        render_children,
        on_key_down,
        on_mounted,
        agent,
        attributes,
        children,
    } = props;

    let resolved_dir = use_direction_context(dir);
    let resolved_locale = use_locale_context(locale);
    let stable_id = use_stable_id(id.clone().map(Cow::Owned)).into_owned();
    let scope = use_hook(|| ScopeHandle::root("range-calendar").child(stable_id.clone()));
    let rels = use_hook(|| RangeCalendarRelationships::new(scope.clone()));
    let dom_root_id = id.unwrap_or_else(|| rels.root_id().to_string());

    let default_range = default_value.unwrap_or_default();
    let initial_start = start_value
        .unwrap_or_else(|| value.map_or(default_range.start, |v| v.start));
    let initial_end = end_value
        .unwrap_or_else(|| value.map_or(default_range.end, |v| v.end));
    let initial_range = DateRange {
        start: initial_start,
        end: initial_end,
    };

    let value_ctl = use_controllable_signal_config(
        ControllableConfig::new(value, initial_range)
            .with_explicit_default(default_value.is_some())
            .with_on_change(on_value_change)
            .with_on_change_details(on_value_change_details)
            .with_component("RangeCalendarRoot")
            .with_prop("value"),
    );
    let mut value_sig = value_ctl.signal();

    let start_ctl = use_controllable_signal_config(
        ControllableConfig::new(start_value, initial_start)
            .with_on_change(on_start_value_change)
            .with_on_change_details(on_start_value_change_details)
            .with_component("RangeCalendarRoot")
            .with_prop("start_value"),
    );
    let mut start_sig = start_ctl.signal();

    let end_ctl = use_controllable_signal_config(
        ControllableConfig::new(end_value, initial_end)
            .with_on_change(on_end_value_change)
            .with_on_change_details(on_end_value_change_details)
            .with_component("RangeCalendarRoot")
            .with_prop("end_value"),
    );
    let mut end_sig = end_ctl.signal();

    // Synchronize uncontrolled endpoints/aggregate with controlled counterpart synchronously.
    if let Some(ext_val) = value {
        if !start_ctl.is_controlled() {
            sync_signal(&mut start_sig, ext_val.start);
        }
        if !end_ctl.is_controlled() {
            sync_signal(&mut end_sig, ext_val.end);
        }
    } else if start_ctl.is_controlled() || end_ctl.is_controlled() {
        let merged_range = DateRange {
            start: *start_sig.peek(),
            end: *end_sig.peek(),
        };
        sync_signal(&mut value_sig, merged_range);
    }

    let def = RangeCalendarDefinition::new(scope.clone())
        .with_min_value(min_value)
        .with_max_value(max_value)
        .with_disabled(disabled)
        .with_read_only(read_only)
        .with_prevent_deselect(prevent_deselect)
        .with_paged_navigation(paged_navigation)
        .with_fixed_weeks(fixed_weeks)
        .with_number_of_months(number_of_months)
        .with_week_starts_on(week_starts_on)
        .with_weekday_format(weekday_format)
        .with_month_format(month_format)
        .with_year_format(year_format)
        .with_locale(resolved_locale)
        .with_calendar_label(calendar_label.clone())
        .with_direction(resolved_dir)
        .with_disable_days_outside_month(disable_days_outside_month)
        .with_min_days(min_days)
        .with_max_days(max_days)
        .with_exclude_disabled(exclude_disabled);

    let base = use_calendar_base_runtime(
        CalendarKind::RangeCalendar,
        rels.clone(),
        dom_root_id,
        def.config.clone(),
        placeholder,
        default_placeholder,
        initial_start,
        initial_focus,
        is_date_disabled,
        is_date_unavailable,
        on_placeholder_change,
        on_placeholder_change_details,
    );

    let mut def_sig = use_signal(|| def.clone());
    let last_pressed_sig = use_signal(|| None::<CalendarDate>);
    let focused_value_sig = use_signal(|| None::<CalendarDate>);

    let mut on_value_change_slot = use_hook(|| CopyValue::new(on_value_change));
    let mut on_value_change_details_slot = use_hook(|| CopyValue::new(on_value_change_details));
    let mut on_start_value_change_slot = use_hook(|| CopyValue::new(on_start_value_change));
    let mut on_start_value_change_details_slot =
        use_hook(|| CopyValue::new(on_start_value_change_details));
    let mut on_end_value_change_slot = use_hook(|| CopyValue::new(on_end_value_change));
    let mut on_end_value_change_details_slot =
        use_hook(|| CopyValue::new(on_end_value_change_details));
    let mut on_range_select_slot = use_hook(|| CopyValue::new(on_range_select));
    on_value_change_slot.set(on_value_change);
    on_value_change_details_slot.set(on_value_change_details);
    on_start_value_change_slot.set(on_start_value_change);
    on_start_value_change_details_slot.set(on_start_value_change_details);
    on_end_value_change_slot.set(on_end_value_change);
    on_end_value_change_details_slot.set(on_end_value_change_details);
    on_range_select_slot.set(on_range_select);

    use_effect(use_reactive(&def, move |next_def| {
        sync_signal(&mut def_sig, next_def);
    }));

    let runtime = RangeCalendarRuntime {
        base: base.clone(),
        def: def_sig,
        value: value_sig,
        value_ctl,
        start_value: start_sig,
        start_ctl,
        end_value: end_sig,
        end_ctl,
        last_pressed: last_pressed_sig,
        focused_value: focused_value_sig,
        on_value_change: on_value_change_slot,
        on_value_change_details: on_value_change_details_slot,
        on_start_value_change: on_start_value_change_slot,
        on_start_value_change_details: on_start_value_change_details_slot,
        on_end_value_change: on_end_value_change_slot,
        on_end_value_change_details: on_end_value_change_details_slot,
        on_range_select: on_range_select_slot,
    };

    // Sync external value/placeholder updates and reconcile exclude_disabled.
    let mut last_value_seen = use_hook(|| CopyValue::new(initial_range));
    {
        let mut rt = runtime.clone();
        use_effect(move || {
            let _ = rt.predicate_rev();
            let current = *rt.value.read();
            if *last_value_seen.read() != current {
                last_value_seen.set(current);
                if !rt.start_ctl.is_controlled() && *rt.start_value.peek() != current.start {
                    rt.start_value.set(current.start);
                }
                if !rt.end_ctl.is_controlled() && *rt.end_value.peek() != current.end {
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

    {
        fn build_rc_snap(rt: &RangeCalendarRuntime) -> ComponentSnapshot {
            let start = rt
                .start_value
                .try_peek()
                .ok()
                .and_then(|v| *v)
                .map(|d| d.to_iso_string());
            let end = rt
                .end_value
                .try_peek()
                .ok()
                .and_then(|v| *v)
                .map(|d| d.to_iso_string());
            let ph = rt
                .placeholder
                .try_peek()
                .map(|p| p.to_iso_string())
                .unwrap_or_default();
            let loc = rt
                .config
                .try_peek()
                .map(|c| calendar_locale_tag(&c.locale))
                .unwrap_or("en-US");
            let ro = rt.config.try_peek().map(|c| c.read_only).unwrap_or(false);
            ComponentSnapshot::new()
                .with_field("value", range_snapshot_value(start, end))
                .with_field("placeholder", SnapshotValue::Text(ph))
                .with_field("locale", SnapshotValue::Text(String::from(loc)))
                .with_field("read_only", SnapshotValue::Bool(ro))
        }

        let rt_for_surface = runtime.clone();
        let rt_for_dispatch = runtime.clone();
        let rt_for_reader = runtime.clone();
        let default_label = if calendar_label.trim().is_empty() {
            String::from("Range calendar")
        } else {
            calendar_label
        };

        use_agent_surface(
            agent.as_ref(),
            SurfaceMetaDefaults::default(),
            || {
                let mut actions = Vec::new();
                if !read_only {
                    actions.push(ActionDescriptor::new(
                        "set",
                        "Set calendar date range (ISO YYYY-MM-DD start/end)",
                        true,
                    ));
                    actions.push(ActionDescriptor::new(
                        "clear",
                        "Clear selected date range",
                        false,
                    ));
                }
                AgentSurface::new("range-calendar", default_label)
                    .with_disabled(disabled)
                    .with_snapshot(build_rc_snap(&rt_for_surface))
                    .with_actions(actions)
            },
            || {
                let rt_handle = rt_for_dispatch;
                Rc::new(move |dispatch| {
                    let mut rt = rt_handle.clone();
                    match dispatch.action.as_str() {
                    "set" => {
                        match parse_agent_range_arg(
                            &dispatch.args,
                            CalendarDate::from_iso_str,
                            "date",
                        ) {
                            Ok((start, end)) => {
                                let canceled = rt.set_range_from_agent(
                                    DateRange { start, end },
                                    dispatch.ctx,
                                );
                                DispatchExecResult::ok().with_canceled(canceled)
                            }
                            Err(err) => DispatchExecResult::ok().with_host_error(err),
                        }
                    }
                    "clear" => {
                        let canceled = rt.set_range_from_agent(
                            DateRange {
                                start: None,
                                end: None,
                            },
                            dispatch.ctx,
                        );
                        DispatchExecResult::ok().with_canceled(canceled)
                    }
                    other => DispatchExecResult::ok()
                        .with_host_error(format!("unsupported range-calendar action: {other}")),
                    }
                })
            },
            || {
                let rt = rt_for_reader;
                Rc::new(move || build_rc_snap(&rt))
            },
        );
    }

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
    let body = if let Some(ref render_children) = render_children {
        render_children.call(snapshot.clone())
    } else {
        children
    };
    let internal_children = render_calendar_internal_children(
        rels.accessible_heading_id(),
        rels.live_region_id(),
        &snapshot.full_label,
        &announcement_text,
        body,
    );

    let base_on_key_down = use_hook(|| {
        let mut rt_for_key = runtime.clone();
        let consumed_flag = Rc::clone(&guard_consumed_escape);
        EventHandler::new(move |evt: KeyboardEvent| {
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

    let mut base_attrs = Vec::with_capacity(9);
    base_attrs.push(Attribute::new("id", root_attrs.id, None, false));
    base_attrs.push(Attribute::new("role", root_attrs.role, None, false));
    base_attrs.push(Attribute::new("aria-label", root_attrs.aria_label, None, false));
    base_attrs.push(Attribute::new("dir", root_attrs.dir, None, false));
    push_attr_opt(
        &mut base_attrs,
        "data-range-calendar-root",
        root_attrs.data_range_calendar_root,
    );
    push_attr_opt(&mut base_attrs, "data-invalid", root_attrs.data_invalid);
    push_attr_opt(&mut base_attrs, "data-disabled", root_attrs.data_disabled);
    push_attr_opt(&mut base_attrs, "data-readonly", root_attrs.data_readonly);
    push_event_handler(&mut base_attrs, "onkeydown", Some(base_on_key_down));

    let mut user = collect_user_attrs(class, style, attributes);
    push_event_handler(&mut user, "onkeydown", on_key_down);
    push_event_handler(&mut user, "onmounted", on_mounted);

    let merged = merge_props(base_attrs, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            {internal_children}
        }
    }
}

#[component]
pub fn RangeCalendarRoot(props: RangeCalendarRootProps) -> Element {
    render_range_calendar_root(props)
}

pub use RangeCalendarRoot as RangeCalendar;
