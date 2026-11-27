use std::borrow::Cow;

use dioxus::prelude::*;

use crate::{
    foundation::{
        compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
        date_time::{
            CalendarDate, CalendarLocale, MonthFormat, Weekday, WeekdayFormat, YearFormat,
        },
        shared::{Direction, ScopeHandle, use_direction_context, use_stable_id},
        state::{sync_signal, use_controllable_signal},
    },
    visually_hidden::VisuallyHidden,
};

use super::super::{
    relationships::CalendarRelationships,
    runtime::{
        CalendarContext, CalendarRuntime, CalendarSharedContext, use_calendar_base_runtime,
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
    pub values: Option<Vec<CalendarDate>>,
    #[props(default)]
    pub default_values: Option<Vec<CalendarDate>>,
    #[props(default)]
    pub on_values_change: Option<EventHandler<Vec<CalendarDate>>>,
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
    #[props(default, into)]
    pub locale: CalendarLocale,
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
        values,
        default_values,
        on_values_change,
        on_date_select,
        max_days,
        placeholder,
        default_placeholder,
        on_placeholder_change,
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
        attributes,
        children,
    } = props;

    let resolved_dir = use_direction_context(dir);
    let stable_id = use_stable_id(id.clone().map(Cow::Owned)).into_owned();
    let scope = use_hook(|| ScopeHandle::root("calendar").child(stable_id.clone()));
    let rels = use_hook(|| CalendarRelationships::new(scope.clone()));
    let dom_root_id = id.unwrap_or_else(|| rels.root_id().to_string());

    let value_ctl = use_controllable_signal(value, default_value, on_value_change);
    let value_sig = value_ctl.signal();

    let values_ctl =
        use_controllable_signal(values, default_values.unwrap_or_default(), on_values_change);
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
        .with_locale(locale)
        .with_calendar_label(calendar_label)
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
