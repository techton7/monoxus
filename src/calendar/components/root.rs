use std::{borrow::Cow, cell::RefCell, collections::HashMap, rc::Rc};

use dioxus::prelude::*;

use crate::{
    foundation::{
        browser::{FocusOptions, focus_element_by_id, focus_element_sync_or_defer},
        date_time::{
            CalendarDate, CalendarLocale, MonthFormat, Weekday, WeekdayFormat, YearFormat,
            build_visible_months, default_placeholder_date,
        },
        shared::{Direction, ScopeHandle, use_direction_context, use_stable_id},
    },
    visually_hidden::VisuallyHidden,
};

use super::super::{
    attrs::CalendarRootAttrs,
    relationships::CalendarRelationships,
    runtime::{
        CalendarContext, CalendarRuntime, CalendarSharedContext, CalendarSharedRuntime,
        CalendarViewConfig, local_today,
    },
    state::CalendarDefinition,
    types::{CalendarMode, CalendarSnapshot},
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

#[component]
pub fn CalendarRoot(props: CalendarRootProps) -> Element {
    let resolved_dir = use_direction_context(props.dir);
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned)).into_owned();
    let scope = use_hook(|| ScopeHandle::root("calendar").child(stable_id.clone()));
    let rels = use_hook(|| CalendarRelationships::new(scope.clone()));
    let dom_root_id = props.id.clone().unwrap_or_else(|| rels.root_id().to_string());

    let today = use_hook(local_today);
    let today_sig = use_signal(|| today);

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
    let initial_placeholder = use_hook(|| {
        if let Some(ext_ph) = props.placeholder {
            *ext_ph.peek()
        } else if let Some(def_ph) = props.default_placeholder {
            def_ph.clamp(props.min_value, props.max_value)
        } else {
            default_placeholder_date(
                initial_default_from_value,
                props.min_value,
                props.max_value,
                today,
            )
        }
    });
    let fallback_placeholder = use_signal(|| initial_placeholder);
    let placeholder_sig = props.placeholder.unwrap_or(fallback_placeholder);
    let initial_placeholder_year = use_hook(|| initial_placeholder.year);

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

    let effective_week_start = def.effective_week_starts_on();
    let view_config = CalendarViewConfig {
        week_starts_on: effective_week_start,
        fixed_weeks: def.fixed_weeks,
        number_of_months: def.number_of_months,
    };

    let def_sig = use_signal(|| def.clone());
    let root_dom_id_sig = use_signal(|| dom_root_id.clone());
    let months_sig = use_signal(|| {
        build_visible_months(
            initial_placeholder,
            def.number_of_months,
            effective_week_start,
            def.fixed_weeks,
        )
    });
    let announcement_sig = use_signal(String::new);

    let is_date_disabled_cb = use_hook(|| Rc::new(RefCell::new(props.is_date_disabled)));
    let is_date_unavailable_cb = use_hook(|| Rc::new(RefCell::new(props.is_date_unavailable)));
    let on_value_change = use_hook(|| Rc::new(RefCell::new(props.on_value_change)));
    let on_values_change = use_hook(|| Rc::new(RefCell::new(props.on_values_change)));
    let on_placeholder_change = use_hook(|| Rc::new(RefCell::new(props.on_placeholder_change)));
    let on_date_select = use_hook(|| Rc::new(RefCell::new(props.on_date_select)));
    let day_id_overrides = use_hook(|| Rc::new(RefCell::new(HashMap::new())));
    let pending_focus_target = use_hook(|| Rc::new(RefCell::new(None)));
    let pending_focus_rev = use_signal(|| 0_u64);
    let last_view_config = use_hook(|| Rc::new(RefCell::new(view_config.clone())));

    let mut runtime = CalendarRuntime {
        def: def_sig,
        rels: rels.clone(),
        root_dom_id: root_dom_id_sig,
        value: value_sig,
        values: values_sig,
        placeholder: placeholder_sig,
        months: months_sig,
        announcement: announcement_sig,
        today_date: today_sig,
        initial_placeholder_year,
        is_date_disabled_cb,
        is_date_unavailable_cb,
        on_value_change,
        on_values_change,
        on_placeholder_change,
        on_date_select,
        day_id_overrides,
        pending_focus_target,
        pending_focus_rev,
        last_view_config,
    };

    // Keep reactive configuration & callback slots synchronized on re-render.
    {
        *runtime.is_date_disabled_cb.borrow_mut() = props.is_date_disabled;
        *runtime.is_date_unavailable_cb.borrow_mut() = props.is_date_unavailable;
        *runtime.on_value_change.borrow_mut() = props.on_value_change;
        *runtime.on_values_change.borrow_mut() = props.on_values_change;
        *runtime.on_placeholder_change.borrow_mut() = props.on_placeholder_change;
        *runtime.on_date_select.borrow_mut() = props.on_date_select;

        if *runtime.def.peek() != def {
            runtime.def.set(def);
        }
        if *runtime.root_dom_id.peek() != dom_root_id {
            runtime.root_dom_id.set(dom_root_id);
        }
        if *runtime.last_view_config.borrow() != view_config {
            *runtime.last_view_config.borrow_mut() = view_config;
            let anchor = runtime
                .months
                .peek()
                .first()
                .map_or(*runtime.placeholder.peek(), |m| m.value);
            runtime.rebuild_months_from(anchor);
        }
    }

    // Ensure initial placeholder is non-disabled when mount renders.
    let ensured_once = use_hook(|| Rc::new(RefCell::new(false)));
    if !*ensured_once.borrow() {
        *ensured_once.borrow_mut() = true;
        if props.placeholder.is_none() {
            runtime.ensure_valid_initial_placeholder();
        }
    }

    // Sync external value/placeholder Signal updates.
    let last_single_seen = use_hook(|| Rc::new(RefCell::new(initial_single)));
    let last_multiple_seen = use_hook(|| Rc::new(RefCell::new(initial_multiple)));
    {
        let mut rt = runtime.clone();
        let last_single = Rc::clone(&last_single_seen);
        let last_multiple = Rc::clone(&last_multiple_seen);
        use_effect(move || {
            let mode = rt.def.read().mode;
            match mode {
                CalendarMode::Single => {
                    let current = *rt.value.read();
                    if *last_single.borrow() != current {
                        *last_single.borrow_mut() = current;
                        if let Some(d) = current
                            && *rt.placeholder.peek() != d
                        {
                            rt.set_placeholder(d);
                        }
                    }
                }
                CalendarMode::Multiple => {
                    let current = rt.values.read().clone();
                    if *last_multiple.borrow() != current {
                        *last_multiple.borrow_mut() = current.clone();
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

    // Focus tabbable day cell on initial mount when `initial_focus = true`.
    let did_initial_focus = use_hook(|| Rc::new(RefCell::new(false)));
    {
        let rt = runtime.clone();
        let initial_focus = props.initial_focus;
        let disabled = props.disabled;
        let did_focus = Rc::clone(&did_initial_focus);
        use_effect(move || {
            if initial_focus && !disabled && !*did_focus.borrow() {
                *did_focus.borrow_mut() = true;
                let target_date = rt.tabbable_date();
                let month_idx = rt.month_index_for(target_date);
                let day_id = rt.resolve_day_id(month_idx, target_date);
                focus_element_by_id(&day_id);
            }
        });
    }

    // Focus target day cell after cross-month keyboard paging mounts the new month's cells.
    {
        let rt = runtime.clone();
        use_effect(move || {
            let _ = (rt.pending_focus_rev)();
            if let Some((month_idx, target_date)) = rt.pending_focus_target.borrow_mut().take() {
                let target_id = rt.resolve_day_id(month_idx, target_date);
                focus_element_sync_or_defer(&target_id, FocusOptions::new(true));
            }
        });
    }

    use_context_provider(|| CalendarContext {
        runtime: runtime.clone(),
    });
    use_context_provider(|| CalendarSharedContext {
        runtime: CalendarSharedRuntime::Calendar(runtime.clone()),
    });

    let snapshot = runtime.snapshot();
    let root_attrs = runtime.root_attrs(None);
    let accessible_heading_id = rels.accessible_heading_id();
    let live_region_id = rels.live_region_id();
    let announcement_text = (runtime.announcement)();
    let full_label = snapshot.full_label.clone();

    let body = if let Some(ref render_children) = props.render_children {
        render_children.call(snapshot.clone())
    } else {
        props.children
    };

    let internal_children = rsx! {
        VisuallyHidden {
            id: accessible_heading_id,
            role: "heading",
            "aria-level": "2",
            "{full_label}"
        }
        VisuallyHidden {
            id: live_region_id,
            role: "status",
            "aria-live": "polite",
            "aria-atomic": "true",
            "{announcement_text}"
        }
        {body}
    };

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

    rsx! {
        div {
            id: "{root_attrs.id}",
            class: props.class,
            style: props.style,
            role: root_attrs.role,
            aria_label: "{root_attrs.aria_label}",
            dir: root_attrs.dir,
            "data-calendar-root": root_attrs.data_calendar_root,
            "data-invalid": root_attrs.data_invalid,
            "data-disabled": root_attrs.data_disabled,
            "data-readonly": root_attrs.data_readonly,
            onkeydown: move |evt| {
                if let Some(ref handler) = props.on_key_down {
                    handler.call(evt);
                }
            },
            onmounted: move |evt| {
                if let Some(ref handler) = props.on_mounted {
                    handler.call(evt);
                }
            },
            ..props.attributes,
            {internal_children}
        }
    }
}

pub use CalendarRoot as Calendar;
