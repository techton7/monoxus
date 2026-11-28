use std::{borrow::Cow, rc::Rc};

use dioxus::prelude::*;
use ui_sdk::kernel::ChangeEvent;

use crate::{
    agent::{
        ActionDescriptor, AgentHints, AgentSurface, ComponentSnapshot, DispatchExecResult,
        SnapshotValue, SurfaceMetaDefaults, use_agent_surface,
    },
    foundation::{
        compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
        config::use_locale_context,
        date_time::{
            CalendarDate, CalendarLocale, MonthFormat, Weekday, WeekdayFormat, YearFormat,
        },
        shared::{Direction, ScopeHandle, use_direction_context, use_stable_id},
        state::{ControllableConfig, sync_signal, use_controllable_signal_config},
    },
    visually_hidden::VisuallyHidden,
};

use super::super::{
    relationships::CalendarRelationships,
    runtime::{
        CalendarContext, CalendarRuntime, CalendarSharedContext, calendar_locale_tag,
        use_calendar_base_runtime,
    },
    state::CalendarDefinition,
    types::{CalendarKind, CalendarMode, CalendarSnapshot},
};

#[derive(Clone, PartialEq, Props)]
pub struct CalendarRootProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub mode: CalendarMode,
    #[props(default)]
    pub value: Option<Option<CalendarDate>>,
    #[props(default)]
    pub default_value: Option<CalendarDate>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<Option<CalendarDate>>>,
    #[props(default)]
    pub on_value_change_details: Option<EventHandler<ChangeEvent<Option<CalendarDate>>>>,
    #[props(default)]
    pub values: Option<Vec<CalendarDate>>,
    #[props(default)]
    pub default_values: Option<Vec<CalendarDate>>,
    #[props(default)]
    pub on_values_change: Option<EventHandler<Vec<CalendarDate>>>,
    #[props(default)]
    pub on_values_change_details: Option<EventHandler<ChangeEvent<Vec<CalendarDate>>>>,
    #[props(default)]
    pub on_date_select: Option<EventHandler<CalendarDate>>,
    #[props(default)]
    pub max_days: Option<usize>,
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
    pub render_children: Option<Callback<CalendarSnapshot, Element>>,
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

/// Shared accessible heading + live region wrapper for `CalendarRoot` and `RangeCalendarRoot`.
pub(crate) fn render_calendar_internal_children(
    accessible_heading_id: &str,
    live_region_id: &str,
    full_label: &str,
    announcement_text: &str,
    body: Element,
) -> Element {
    rsx! {
        VisuallyHidden {
            id: "{accessible_heading_id}",
            role: "heading",
            "aria-level": "2",
            "{full_label}"
        }
        VisuallyHidden {
            id: "{live_region_id}",
            role: "status",
            "aria-live": "polite",
            "aria-atomic": "true",
            "{announcement_text}"
        }
        {body}
    }
}

/// Plain Rust render surface for `CalendarRoot` and `DatePickerCalendar` (zero nested
/// `#[component]` taking `Option<Callback|EventHandler>` per `Dioxus #5894`).
pub(crate) fn render_calendar_root(props: CalendarRootProps) -> Element {
    let CalendarRootProps {
        id,
        class,
        style,
        mode,
        value,
        default_value,
        on_value_change,
        on_value_change_details,
        values,
        default_values,
        on_values_change,
        on_values_change_details,
        on_date_select,
        max_days,
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
    let scope = use_hook(|| ScopeHandle::root("calendar").child(stable_id.clone()));
    let rels = use_hook(|| CalendarRelationships::new(scope.clone()));
    let dom_root_id = id.unwrap_or_else(|| rels.root_id().to_string());

    let value_ctl = use_controllable_signal_config(
        ControllableConfig::new(value, default_value)
            .with_explicit_default(default_value.is_some())
            .with_on_change(on_value_change)
            .with_on_change_details(on_value_change_details)
            .with_component("CalendarRoot")
            .with_prop("value"),
    );
    let value_sig = value_ctl.signal();

    let has_explicit_default_values = default_values.is_some();
    let values_ctl = use_controllable_signal_config(
        ControllableConfig::new(values, default_values.unwrap_or_default())
            .with_explicit_default(has_explicit_default_values)
            .with_on_change(on_values_change)
            .with_on_change_details(on_values_change_details)
            .with_component("CalendarRoot")
            .with_prop("values"),
    );
    let values_sig = values_ctl.signal();

    let initial_single = *value_sig.peek();
    let initial_multiple = values_sig.peek().clone();
    let initial_default_from_value = match mode {
        CalendarMode::Single => initial_single,
        CalendarMode::Multiple => initial_multiple.last().copied(),
    };

    let def = CalendarDefinition::new(scope.clone())
        .with_mode(mode)
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
        .with_max_days(max_days);

    let base = use_calendar_base_runtime(
        CalendarKind::Calendar,
        rels.clone(),
        dom_root_id,
        def.config.clone(),
        placeholder,
        default_placeholder,
        initial_default_from_value,
        initial_focus,
        is_date_disabled,
        is_date_unavailable,
        on_placeholder_change,
        on_placeholder_change_details,
    );

    let mut def_sig = use_signal(|| def.clone());
    let mut on_date_select_slot = use_hook(|| CopyValue::new(on_date_select));
    on_date_select_slot.set(on_date_select);

    use_effect(use_reactive(&def, move |next_def| {
        sync_signal(&mut def_sig, next_def);
    }));

    let runtime = CalendarRuntime {
        base: base.clone(),
        def: def_sig,
        value: value_sig,
        value_ctl,
        values: values_sig,
        values_ctl,
        on_date_select: on_date_select_slot,
    };

    // Sync external value/placeholder updates.
    let mut last_single_seen = use_hook(|| CopyValue::new(initial_single));
    let mut last_multiple_seen = use_hook(|| CopyValue::new(initial_multiple));
    {
        let mut rt = runtime.clone();
        use_effect(move || {
            let mode = rt.def.read().mode;
            match mode {
                CalendarMode::Single => {
                    let current = *rt.value.read();
                    if *last_single_seen.read() != current {
                        last_single_seen.set(current);
                        if let Some(d) = current
                            && *rt.placeholder.peek() != d
                        {
                            rt.set_placeholder(d);
                        }
                    }
                }
                CalendarMode::Multiple => {
                    let current = rt.values.read().clone();
                    if *last_multiple_seen.read() != current {
                        last_multiple_seen.set(current.clone());
                        if let Some(last) = current.last().copied()
                            && *rt.placeholder.peek() != last
                        {
                            rt.set_placeholder(last);
                        }
                    }
                }
            }
            let ph = *rt.placeholder.read();
            let in_view = rt.months.peek().iter().any(|m| m.value.is_same_month(ph));
            if !in_view {
                rt.rebuild_months_from(ph);
            }
        });
    }

    use_context_provider(|| CalendarContext {
        runtime: runtime.clone(),
    });
    use_context_provider(|| CalendarSharedContext { runtime: base });

    {
        fn build_cal_snap(rt: &CalendarRuntime) -> ComponentSnapshot {
            let cur_mode = rt.def.try_peek().map(|d| d.mode).unwrap_or(CalendarMode::Single);
            let cur_ph = rt
                .placeholder
                .try_peek()
                .map(|p| p.to_iso_string())
                .unwrap_or_default();
            let cur_locale = rt
                .config
                .try_peek()
                .map(|c| calendar_locale_tag(&c.locale))
                .unwrap_or("en-US");
            let cur_read_only = rt.config.try_peek().map(|c| c.read_only).unwrap_or(false);
            let mut snap = ComponentSnapshot::new()
                .with_field("placeholder", SnapshotValue::Text(cur_ph))
                .with_field("locale", SnapshotValue::Text(String::from(cur_locale)))
                .with_field("read_only", SnapshotValue::Bool(cur_read_only));
            match cur_mode {
                CalendarMode::Single => {
                    let val = rt
                        .value
                        .try_peek()
                        .ok()
                        .and_then(|v| *v)
                        .map(|d| SnapshotValue::Text(d.to_iso_string()))
                        .unwrap_or(SnapshotValue::Null);
                    snap = snap.with_field("value", val);
                }
                CalendarMode::Multiple => {
                    let vals = rt
                        .values
                        .try_peek()
                        .map(|v| {
                            v.iter()
                                .map(|d| SnapshotValue::Text(d.to_iso_string()))
                                .collect()
                        })
                        .unwrap_or_default();
                    snap = snap.with_field("values", SnapshotValue::List(vals));
                }
            }
            snap
        }

        let rt_for_surface = runtime.clone();
        let rt_for_dispatch = runtime.clone();
        let rt_for_reader = runtime.clone();
        let default_label = if calendar_label.trim().is_empty() {
            String::from("Calendar")
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
                        "Set calendar date (ISO YYYY-MM-DD)",
                        true,
                    ));
                    actions.push(ActionDescriptor::new(
                        "clear",
                        "Clear selected calendar date(s)",
                        false,
                    ));
                }
                AgentSurface::new("calendar", default_label)
                    .with_disabled(disabled)
                    .with_snapshot(build_cal_snap(&rt_for_surface))
                    .with_actions(actions)
            },
            || {
                let rt_handle = rt_for_dispatch;
                Rc::new(move |dispatch| {
                    let mut rt = rt_handle.clone();
                    match dispatch.action.as_str() {
                    "set" => {
                        let cur_mode = rt
                            .def
                            .try_peek()
                            .map(|d| d.mode)
                            .unwrap_or(CalendarMode::Single);
                        match cur_mode {
                            CalendarMode::Single => {
                                let arg = dispatch
                                    .args
                                    .get("value")
                                    .or_else(|| dispatch.args.get("date"));
                                match arg {
                                    Some(SnapshotValue::Null) => {
                                        let outcome =
                                            rt.value_ctl.set_with_context(None, dispatch.ctx);
                                        DispatchExecResult::ok()
                                            .with_canceled(outcome.is_canceled())
                                    }
                                    Some(SnapshotValue::Text(raw)) => {
                                        let Some(date) = CalendarDate::from_iso_str(raw.trim())
                                        else {
                                            return DispatchExecResult::ok().with_host_error(
                                                format!("invalid ISO date: {raw}"),
                                            );
                                        };
                                        let outcome = rt
                                            .value_ctl
                                            .set_with_context(Some(date), dispatch.ctx);
                                        if outcome.is_canceled() {
                                            return DispatchExecResult::ok().with_canceled(true);
                                        }
                                        if outcome.is_committed() {
                                            rt.set_placeholder(date);
                                        }
                                        DispatchExecResult::ok()
                                    }
                                    _ => DispatchExecResult::ok()
                                        .with_host_error("missing ISO date argument `value`"),
                                }
                            }
                            CalendarMode::Multiple => {
                                let parsed: Result<Vec<CalendarDate>, String> =
                                    if let Some(SnapshotValue::List(items)) = dispatch
                                        .args
                                        .get("values")
                                        .or_else(|| dispatch.args.get("value"))
                                    {
                                        items
                                            .iter()
                                            .map(|item| match item {
                                                SnapshotValue::Text(raw) => {
                                                    CalendarDate::from_iso_str(raw.trim())
                                                        .ok_or_else(|| {
                                                            format!("invalid ISO date: {raw}")
                                                        })
                                                }
                                                _ => Err(String::from(
                                                    "expected ISO date strings in `values`",
                                                )),
                                            })
                                            .collect()
                                    } else if let Some(SnapshotValue::Text(raw)) = dispatch
                                        .args
                                        .get("value")
                                        .or_else(|| dispatch.args.get("date"))
                                    {
                                        CalendarDate::from_iso_str(raw.trim())
                                            .map(|d| vec![d])
                                            .ok_or_else(|| format!("invalid ISO date: {raw}"))
                                    } else {
                                        Err(String::from(
                                            "missing ISO date argument `value` or `values`",
                                        ))
                                    };
                                match parsed {
                                    Ok(dates) => {
                                        let last = dates.last().copied();
                                        let outcome =
                                            rt.values_ctl.set_with_context(dates, dispatch.ctx);
                                        if outcome.is_canceled() {
                                            return DispatchExecResult::ok().with_canceled(true);
                                        }
                                        if outcome.is_committed()
                                            && let Some(d) = last
                                        {
                                            rt.set_placeholder(d);
                                        }
                                        DispatchExecResult::ok()
                                    }
                                    Err(err) => DispatchExecResult::ok().with_host_error(err),
                                }
                            }
                        }
                    }
                    "clear" => {
                        let cur_mode = rt
                            .def
                            .try_peek()
                            .map(|d| d.mode)
                            .unwrap_or(CalendarMode::Single);
                        let outcome = match cur_mode {
                            CalendarMode::Single => {
                                rt.value_ctl.set_with_context(None, dispatch.ctx)
                            }
                            CalendarMode::Multiple => {
                                rt.values_ctl.set_with_context(Vec::new(), dispatch.ctx)
                            }
                        };
                        DispatchExecResult::ok().with_canceled(outcome.is_canceled())
                    }
                    other => DispatchExecResult::ok()
                        .with_host_error(format!("unsupported calendar action: {other}")),
                    }
                })
            },
            || {
                let rt = rt_for_reader;
                Rc::new(move || build_cal_snap(&rt))
            },
        );
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

    let mut base_attrs = Vec::with_capacity(8);
    base_attrs.push(Attribute::new("id", root_attrs.id, None, false));
    base_attrs.push(Attribute::new("role", root_attrs.role, None, false));
    base_attrs.push(Attribute::new("aria-label", root_attrs.aria_label, None, false));
    base_attrs.push(Attribute::new("dir", root_attrs.dir, None, false));
    push_attr_opt(
        &mut base_attrs,
        "data-calendar-root",
        root_attrs.data_calendar_root,
    );
    push_attr_opt(&mut base_attrs, "data-invalid", root_attrs.data_invalid);
    push_attr_opt(&mut base_attrs, "data-disabled", root_attrs.data_disabled);
    push_attr_opt(&mut base_attrs, "data-readonly", root_attrs.data_readonly);

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
pub fn CalendarRoot(props: CalendarRootProps) -> Element {
    render_calendar_root(props)
}

pub use CalendarRoot as Calendar;
