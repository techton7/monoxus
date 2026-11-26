use std::borrow::Cow;

use dioxus::prelude::*;

use crate::{
    foundation::{
        compose::merge_global_attributes,
        date_time::{
            CalendarDate, CalendarLocale, MonthFormat, Weekday, WeekdayFormat, YearFormat,
        },
        shared::{Direction, ScopeHandle, use_direction_context, use_stable_id},
        state::sync_signal,
    },
    visually_hidden::VisuallyHidden,
};

use super::super::{
    attrs::CalendarRootAttrs,
    relationships::CalendarRelationships,
    runtime::{
        CalendarContext, CalendarRuntime, CalendarSharedContext, use_calendar_base_runtime,
    },
    state::CalendarDefinition,
    types::{CalendarKind, CalendarMode, CalendarSnapshot},
};

#[derive(Clone, PartialEq, Props)]
pub struct CalendarRootRenderProps {
    pub attrs: CalendarRootAttrs,
    pub snapshot: CalendarSnapshot,
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
pub struct CalendarRootProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub mode: CalendarMode,
    #[props(default)]
    pub value: Option<Signal<Option<CalendarDate>>>,
    #[props(default)]
    pub default_value: Option<CalendarDate>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<Option<CalendarDate>>>,
    #[props(default)]
    pub values: Option<Signal<Vec<CalendarDate>>>,
    #[props(default)]
    pub default_values: Option<Vec<CalendarDate>>,
    #[props(default)]
    pub on_values_change: Option<EventHandler<Vec<CalendarDate>>>,
    #[props(default)]
    pub on_date_select: Option<EventHandler<CalendarDate>>,
    #[props(default)]
    pub max_days: Option<usize>,
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
    pub render_child: Option<Callback<CalendarRootRenderProps, Element>>,
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
    let resolved_dir = use_direction_context(props.dir);
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned)).into_owned();
    let scope = use_hook(|| ScopeHandle::root("calendar").child(stable_id.clone()));
    let rels = use_hook(|| CalendarRelationships::new(scope.clone()));
    let dom_root_id = props.id.clone().unwrap_or_else(|| rels.root_id().to_string());

    let initial_single = props
        .value
        .and_then(|s| *s.peek())
        .or(props.default_value);
    let initial_multiple = props
        .values
        .map(|s| s.peek().clone())
        .or_else(|| props.default_values.clone())
        .unwrap_or_default();

    let fallback_value = use_signal(|| initial_single);
    let value_sig = props.value.unwrap_or(fallback_value);

    let fallback_values = use_signal(|| initial_multiple.clone());
    let values_sig = props.values.unwrap_or(fallback_values);

    let initial_default_from_value = match props.mode {
        CalendarMode::Single => initial_single,
        CalendarMode::Multiple => initial_multiple.last().copied(),
    };

    let def = CalendarDefinition::new(scope.clone())
        .with_mode(props.mode)
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
        .with_max_days(props.max_days);

    let base = use_calendar_base_runtime(
        CalendarKind::Calendar,
        rels.clone(),
        dom_root_id,
        def.config.clone(),
        props.placeholder,
        props.default_placeholder,
        initial_default_from_value,
        props.initial_focus,
        props.is_date_disabled,
        props.is_date_unavailable,
        props.on_placeholder_change,
    );

    let mut def_sig = use_signal(|| def.clone());
    let mut on_value_change = use_hook(|| CopyValue::new(props.on_value_change));
    let mut on_values_change = use_hook(|| CopyValue::new(props.on_values_change));
    let mut on_date_select = use_hook(|| CopyValue::new(props.on_date_select));
    on_value_change.set(props.on_value_change);
    on_values_change.set(props.on_values_change);
    on_date_select.set(props.on_date_select);

    use_effect(use_reactive(&def, move |next_def| {
        sync_signal(&mut def_sig, next_def);
    }));

    let runtime = CalendarRuntime {
        base: base.clone(),
        def: def_sig,
        value: value_sig,
        values: values_sig,
        on_value_change,
        on_values_change,
        on_date_select,
    };

    // Sync external value/placeholder Signal updates.
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

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(CalendarRootRenderProps {
                attrs: root_attrs,
                snapshot,
                class: props.class,
                style: props.style,
                on_key_down: props.on_key_down,
                on_mounted: props.on_mounted,
                attributes: props.attributes,
                children: internal_children,
            });
        }
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::CalendarRoot: as_child requires render_child; rendering the default div"
        );
    }

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );
    let on_key_down = props.on_key_down;
    let on_mounted = props.on_mounted;

    rsx! {
        div {
            id: "{root_attrs.id}",
            class: merged.class.as_deref(),
            style: merged.style.as_deref(),
            role: root_attrs.role,
            aria_label: "{root_attrs.aria_label}",
            dir: root_attrs.dir,
            "data-calendar-root": root_attrs.data_calendar_root,
            "data-invalid": root_attrs.data_invalid,
            "data-disabled": root_attrs.data_disabled,
            "data-readonly": root_attrs.data_readonly,
            onkeydown: move |evt| {
                if let Some(handler) = on_key_down {
                    handler.call(evt);
                }
            },
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
pub fn CalendarRoot(props: CalendarRootProps) -> Element {
    render_calendar_root(props)
}

pub use CalendarRoot as Calendar;
