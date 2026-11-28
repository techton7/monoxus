use std::{borrow::Cow, rc::Rc};

use dioxus::prelude::*;
use ui_sdk::foundation::date_time::field_policy::DateFieldPolicy;
use ui_sdk::kernel::ChangeEvent;

use crate::{
    agent::{
        ActionDescriptor, AgentHints, AgentSurface, ComponentSnapshot, DispatchExecResult,
        SnapshotValue, SurfaceMetaDefaults, use_agent_surface,
    },
    calendar::runtime::{parse_agent_range_arg, range_snapshot_value},
    date_field::default_date_field_placeholder,
    date_picker::{
        components::{
            PickerContentSurfaceSpec, render_picker_content_surface, render_picker_trigger_surface,
        },
        runtime::PickerRuntimeCore,
    },
    date_range_field::{
        components::{
            RangeFieldRootSpec, render_range_field_root_with_runtime, use_range_field_runtime,
        },
        try_use_date_range_field_context,
    },
    field::try_use_field_context,
    foundation::{
        browser::{focus::use_composite_focus_out_tracked, local_today},
        compose::FloatingContentChildProps,
        config::use_locale_context,
        date_time::{
            CalendarConfig, CalendarDate, CalendarLocale, DateValue, DateValueRange,
            EditableSegmentKind, FieldInvalidEvent, Granularity, HourCycle, MonthFormat, Weekday,
            WeekdayFormat, YearFormat,
        },
        overlay::{DismissBehavior, PlacementAlign, PlacementSide, StickyStrategy},
        shared::{Direction, ScopeHandle, use_direction_context, use_stable_id},
        state::{ControllableConfig, sync_signal, use_controllable_signal_config},
    },
    popover::{
        CloseAutoFocusEvent, EscapeKeyDownEvent, InteractOutsideEvent, OpenAutoFocusEvent,
        PointerDownOutsideEvent, PopoverCloseFocusPolicy, PopoverOpenFocusPolicy,
        PopoverOutsideInteractionPolicy, PopoverRoot, PopoverScrollLockPolicy,
    },
    range_calendar::{
        RangeCalendarRootProps, RangeCalendarSnapshot, components::render_range_calendar_root,
    },
};

use super::{
    relationships::{DATE_RANGE_PICKER_OPEN_FOCUS_SELECTOR, DateRangePickerRelationships},
    runtime::{
        DateRangePickerCalendarConfig, DateRangePickerContext, DateRangePickerRuntime,
        use_date_range_picker_context,
    },
    state::{DateRangePickerDefinition, DateRangePickerState},
};

/// Props for [`DateRangePickerRoot`].
#[derive(Clone, PartialEq, Props)]
pub struct DateRangePickerRootProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub value: Option<DateValueRange>,
    #[props(default)]
    pub default_value: Option<DateValueRange>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<DateValueRange>>,
    #[props(default)]
    pub on_value_change_details: Option<EventHandler<ChangeEvent<DateValueRange>>>,
    #[props(default)]
    pub start_value: Option<Option<DateValue>>,
    #[props(default)]
    pub on_start_value_change: Option<EventHandler<Option<DateValue>>>,
    #[props(default)]
    pub on_start_value_change_details: Option<EventHandler<ChangeEvent<Option<DateValue>>>>,
    #[props(default)]
    pub end_value: Option<Option<DateValue>>,
    #[props(default)]
    pub on_end_value_change: Option<EventHandler<Option<DateValue>>>,
    #[props(default)]
    pub on_end_value_change_details: Option<EventHandler<ChangeEvent<Option<DateValue>>>>,
    #[props(default)]
    pub placeholder: Option<DateValue>,
    #[props(default)]
    pub default_placeholder: Option<DateValue>,
    #[props(default)]
    pub on_placeholder_change: Option<EventHandler<DateValue>>,
    #[props(default)]
    pub on_placeholder_change_details: Option<EventHandler<ChangeEvent<DateValue>>>,
    #[props(default)]
    pub open: Option<bool>,
    #[props(default = false)]
    pub default_open: bool,
    #[props(default)]
    pub on_open_change: Option<EventHandler<bool>>,
    #[props(default)]
    pub on_open_change_details: Option<EventHandler<ChangeEvent<bool>>>,
    #[props(default)]
    pub on_open_change_complete: Option<EventHandler<bool>>,
    #[props(default)]
    pub on_focus_out: Option<EventHandler<FocusEvent>>,
    #[props(default = true)]
    pub close_on_range_select: bool,
    #[props(default)]
    pub granularity: Option<Granularity>,
    #[props(default)]
    pub hour_cycle: Option<HourCycle>,
    #[props(default)]
    pub locale: Option<CalendarLocale>,
    #[props(default = false)]
    pub hide_time_zone: bool,
    #[props(default)]
    pub min_value: Option<DateValue>,
    #[props(default)]
    pub max_value: Option<DateValue>,
    #[props(default)]
    pub min_days: Option<usize>,
    #[props(default)]
    pub max_days: Option<usize>,
    #[props(default = false)]
    pub exclude_disabled: bool,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub read_only: bool,
    #[props(default = false)]
    pub required: bool,
    #[props(default)]
    pub read_only_segments: Vec<EditableSegmentKind>,
    #[props(default)]
    pub validate: Option<Callback<DateValueRange, Option<Vec<String>>>>,
    #[props(default)]
    pub on_invalid: Option<EventHandler<FieldInvalidEvent>>,
    #[props(default, into)]
    pub error_message_id: Option<String>,
    #[props(default)]
    pub is_date_disabled: Option<Callback<CalendarDate, bool>>,
    #[props(default)]
    pub is_date_unavailable: Option<Callback<CalendarDate, bool>>,
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
    #[props(default = String::from("Event"), into)]
    pub calendar_label: String,
    #[props(default = true)]
    pub disable_days_outside_month: bool,
    #[props(default = false)]
    pub initial_focus: bool,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default)]
    pub modal: Option<bool>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(default)]
    pub agent: Option<AgentHints>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Root provider composing `DateRangeFieldRoot`, `PopoverRoot`, and `RangeCalendarRoot` state.
#[component]
pub fn DateRangePickerRoot(props: DateRangePickerRootProps) -> Element {
    let field_ctx = try_use_field_context();
    let effective_disabled = props.disabled || field_ctx.is_some_and(|f| f.is_disabled());
    let effective_read_only = props.read_only || field_ctx.is_some_and(|f| f.is_read_only());
    let effective_required = props.required || field_ctx.is_some_and(|f| f.is_required());
    let effective_invalid_override = field_ctx.map(|f| f.is_invalid());

    let resolved_dir = use_direction_context(props.dir);
    let resolved_locale = use_locale_context(props.locale);
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned)).into_owned();
    let scope = use_hook(|| ScopeHandle::root("date-range-picker").child(stable_id.clone()));
    let rels = use_hook(|| DateRangePickerRelationships::new(scope.clone()));
    let today = use_hook(local_today);

    let default_range = props.default_value.unwrap_or_default();
    let initial_start = props
        .start_value
        .unwrap_or_else(|| props.value.map_or(default_range.start, |v| v.start));
    let initial_end = props
        .end_value
        .unwrap_or_else(|| props.value.map_or(default_range.end, |v| v.end));
    let initial_value = DateValueRange {
        start: initial_start,
        end: initial_end,
    };

    let value_ctl = use_controllable_signal_config(
        ControllableConfig::new(props.value, initial_value)
            .with_explicit_default(props.default_value.is_some())
            .with_on_change(props.on_value_change)
            .with_on_change_details(props.on_value_change_details)
            .with_component("DateRangePickerRoot")
            .with_prop("value"),
    );
    let start_ctl = use_controllable_signal_config(
        ControllableConfig::new(props.start_value, initial_start)
            .with_on_change(props.on_start_value_change)
            .with_on_change_details(props.on_start_value_change_details)
            .with_component("DateRangePickerRoot")
            .with_prop("start_value"),
    );
    let end_ctl = use_controllable_signal_config(
        ControllableConfig::new(props.end_value, initial_end)
            .with_on_change(props.on_end_value_change)
            .with_on_change_details(props.on_end_value_change_details)
            .with_component("DateRangePickerRoot")
            .with_prop("end_value"),
    );

    let has_explicit_default_placeholder = props.default_placeholder.is_some();
    let initial_placeholder = props
        .placeholder
        .or(props.default_placeholder)
        .unwrap_or_else(|| {
            default_date_field_placeholder(
                initial_value.start.or(initial_value.end),
                props.granularity,
                props.min_value,
                props.max_value,
                today,
            )
        });
    let placeholder_ctl = use_controllable_signal_config(
        ControllableConfig::new(props.placeholder, initial_placeholder)
            .with_explicit_default(has_explicit_default_placeholder)
            .with_on_change(props.on_placeholder_change)
            .with_on_change_details(props.on_placeholder_change_details)
            .with_component("DateRangePickerRoot")
            .with_prop("placeholder"),
    );
    let open_ctl = use_controllable_signal_config(
        ControllableConfig::new(props.open, props.default_open)
            .with_explicit_default(props.default_open)
            .with_on_change(props.on_open_change)
            .with_on_change_details(props.on_open_change_details)
            .with_component("DateRangePickerRoot")
            .with_prop("open"),
    );
    let initial_open = open_ctl.peek();

    let def = DateRangePickerDefinition::new()
        .with_close_on_range_select(props.close_on_range_select)
        .with_disabled(effective_disabled)
        .with_read_only(effective_read_only)
        .with_required(effective_required)
        .with_dir(resolved_dir);

    let state = use_signal(|| {
        DateRangePickerState::new(def.clone(), initial_value, initial_placeholder, initial_open)
    });
    let value_sig = use_signal(|| initial_value);
    let initial_cal_range = initial_value.to_date_range();
    let calendar_value_sig = use_signal(|| initial_cal_range);
    let calendar_start_sig = use_signal(|| initial_cal_range.start);
    let calendar_end_sig = use_signal(|| initial_cal_range.end);
    let placeholder_sig = use_signal(|| initial_placeholder);
    let calendar_placeholder_sig = use_signal(|| initial_placeholder.date());
    let open_sig = use_signal(|| if effective_disabled { false } else { initial_open });
    let trigger_dom_id = use_signal(|| None::<String>);
    let content_dom_id = use_signal(|| None::<String>);

    let next_cal_cfg = DateRangePickerCalendarConfig {
        config: CalendarConfig::new()
            .with_min_value(props.min_value.map(|v| v.date()))
            .with_max_value(props.max_value.map(|v| v.date()))
            .with_disabled(effective_disabled)
            .with_read_only(effective_read_only)
            .with_prevent_deselect(props.prevent_deselect)
            .with_paged_navigation(props.paged_navigation)
            .with_fixed_weeks(props.fixed_weeks)
            .with_number_of_months(props.number_of_months)
            .with_week_starts_on(props.week_starts_on)
            .with_weekday_format(props.weekday_format)
            .with_month_format(props.month_format)
            .with_year_format(props.year_format)
            .with_locale(resolved_locale.clone())
            .with_calendar_label(props.calendar_label.clone())
            .with_direction(resolved_dir)
            .with_disable_days_outside_month(props.disable_days_outside_month),
        min_days: props.min_days,
        max_days: props.max_days,
        exclude_disabled: props.exclude_disabled,
        initial_focus: props.initial_focus,
    };
    let calendar_config = use_signal(|| next_cal_cfg.clone());

    let mut on_value_change = use_hook(|| CopyValue::new(props.on_value_change));
    on_value_change.set(props.on_value_change);
    let mut on_value_change_details = use_hook(|| CopyValue::new(props.on_value_change_details));
    on_value_change_details.set(props.on_value_change_details);
    let mut on_start_value_change = use_hook(|| CopyValue::new(props.on_start_value_change));
    on_start_value_change.set(props.on_start_value_change);
    let mut on_start_value_change_details =
        use_hook(|| CopyValue::new(props.on_start_value_change_details));
    on_start_value_change_details.set(props.on_start_value_change_details);
    let mut on_end_value_change = use_hook(|| CopyValue::new(props.on_end_value_change));
    on_end_value_change.set(props.on_end_value_change);
    let mut on_end_value_change_details =
        use_hook(|| CopyValue::new(props.on_end_value_change_details));
    on_end_value_change_details.set(props.on_end_value_change_details);
    let on_composite_focus_in = use_hook(|| CopyValue::new(None::<EventHandler<FocusEvent>>));
    let on_composite_focus_out = use_hook(|| CopyValue::new(None::<EventHandler<FocusEvent>>));

    let mut is_date_disabled = use_signal(|| props.is_date_disabled);
    let mut is_date_unavailable = use_signal(|| props.is_date_unavailable);
    let mut predicate_rev = use_signal(|| 0_u64);
    let has_unavailable_pred =
        props.is_date_disabled.is_some() || props.is_date_unavailable.is_some();
    let mut pred_initialized = use_hook(|| CopyValue::new(false));
    let mut prev_had_pred = use_hook(|| CopyValue::new(has_unavailable_pred));
    if pred_initialized.try_read().is_ok_and(|init| *init) {
        sync_signal(&mut is_date_disabled, props.is_date_disabled);
        sync_signal(&mut is_date_unavailable, props.is_date_unavailable);
        let had_pred = prev_had_pred.try_read().is_ok_and(|had| *had);
        prev_had_pred.set(has_unavailable_pred);
        if has_unavailable_pred || had_pred {
            let next_rev = predicate_rev.try_peek().ok().map(|cur| cur.wrapping_add(1));
            if let Some(next_rev) = next_rev {
                sync_signal(&mut predicate_rev, next_rev);
            }
        }
    } else {
        pred_initialized.set(true);
    }

    let mut runtime = DateRangePickerRuntime {
        core: PickerRuntimeCore {
            state,
            rels: rels.clone(),
            value: value_sig,
            calendar_value: calendar_value_sig,
            placeholder: placeholder_sig,
            calendar_placeholder: calendar_placeholder_sig,
            open: open_sig,
            trigger_dom_id,
            content_dom_id,
            value_ctl,
            placeholder_ctl,
            open_ctl,
            on_composite_focus_in,
            on_composite_focus_out,
            is_date_disabled,
            is_date_unavailable,
            predicate_rev,
        },
        calendar_start: calendar_start_sig,
        calendar_end: calendar_end_sig,
        calendar_config,
        start_ctl,
        end_ctl,
        on_value_change,
        on_value_change_details,
        on_start_value_change,
        on_start_value_change_details,
        on_end_value_change,
        on_end_value_change_details,
    };

    // Synchronous render-time reconciliation for external controlled props.
    runtime.reconcile_external_state(
        props.value,
        props.start_value,
        props.end_value,
        props.placeholder,
        props.open,
    );

    use_effect(use_reactive(
        (
            &def,
            &next_cal_cfg,
            &props.is_date_disabled,
            &props.is_date_unavailable,
        ),
        {
            let mut rt = runtime.clone();
            move |(next_def, next_cfg, next_disabled, next_unavailable)| {
                if *rt.calendar_config.peek() != next_cfg {
                    rt.calendar_config.set(next_cfg);
                }
                if *rt.is_date_disabled.peek() != next_disabled {
                    rt.is_date_disabled.set(next_disabled);
                }
                if *rt.is_date_unavailable.peek() != next_unavailable {
                    rt.is_date_unavailable.set(next_unavailable);
                }
                if *rt.state.peek().def() != next_def {
                    rt.state.write().sync_definition(next_def);
                    rt.reconcile_external_state(None, None, None, None, None);
                }
            }
        },
    ));

    use_effect(use_reactive(
        (
            &props.value,
            &props.start_value,
            &props.end_value,
            &props.placeholder,
            &props.open,
        ),
        {
            let mut rt = runtime.clone();
            move |(ext_val, ext_start, ext_end, ext_ph, ext_open)| {
                rt.reconcile_external_state(ext_val, ext_start, ext_end, ext_ph, ext_open);
            }
        },
    ));

    use_context_provider(|| DateRangePickerContext(runtime.clone()));

    {
        fn build_drp_snap(rt: &DateRangePickerRuntime) -> ComponentSnapshot {
            let open = rt.open.try_peek().map(|v| *v).unwrap_or(false);
            let range = rt.value.try_peek().map(|v| *v).unwrap_or_default();
            let ph = rt
                .placeholder
                .try_peek()
                .map(|p| SnapshotValue::Text(p.to_iso_string()))
                .unwrap_or(SnapshotValue::Null);
            let ro = rt
                .state
                .try_peek()
                .map(|st| st.def().read_only)
                .unwrap_or(false);
            ComponentSnapshot::new()
                .with_field("open", SnapshotValue::Bool(open))
                .with_field(
                    "value",
                    range_snapshot_value(
                        range.start.map(|d| d.to_iso_string()),
                        range.end.map(|d| d.to_iso_string()),
                    ),
                )
                .with_field("placeholder", ph)
                .with_field("read_only", SnapshotValue::Bool(ro))
        }

        let rt_for_surface = runtime.clone();
        let rt_for_dispatch = runtime.clone();
        let rt_for_reader = runtime.clone();

        use_agent_surface(
            props.agent.as_ref(),
            SurfaceMetaDefaults::default(),
            || {
                let mut actions = vec![
                    ActionDescriptor::new("open", "Open date range picker popover", false),
                    ActionDescriptor::new("close", "Close date range picker popover", false),
                ];
                if !effective_read_only {
                    actions.push(ActionDescriptor::new(
                        "set",
                        "Set ISO date range (start/end)",
                        true,
                    ));
                    actions.push(ActionDescriptor::new(
                        "clear",
                        "Clear selected date range",
                        false,
                    ));
                }
                AgentSurface::new("date-range-picker", "Date range picker")
                    .with_disabled(effective_disabled)
                    .with_snapshot(build_drp_snap(&rt_for_surface))
                    .with_actions(actions)
            },
            || {
                let rt_handle = rt_for_dispatch;
                Rc::new(move |dispatch| {
                    let mut rt = rt_handle.clone();
                    match dispatch.action.as_str() {
                    "open" => {
                        let canceled = rt.set_open_with_context(true, dispatch.ctx);
                        DispatchExecResult::ok().with_canceled(canceled)
                    }
                    "close" => {
                        let canceled = rt.set_open_with_context(false, dispatch.ctx);
                        DispatchExecResult::ok().with_canceled(canceled)
                    }
                    "set" => {
                        match parse_agent_range_arg(&dispatch.args, DateValue::from_iso_str, "date")
                        {
                            Ok((start, end)) => {
                                let canceled = rt.handle_field_value_change_with_context(
                                    DateValueRange { start, end },
                                    dispatch.ctx,
                                );
                                DispatchExecResult::ok().with_canceled(canceled)
                            }
                            Err(err) => DispatchExecResult::ok().with_host_error(err),
                        }
                    }
                    "clear" => {
                        let canceled = rt.handle_field_value_change_with_context(
                            DateValueRange {
                                start: None,
                                end: None,
                            },
                            dispatch.ctx,
                        );
                        DispatchExecResult::ok().with_canceled(canceled)
                    }
                    other => DispatchExecResult::ok()
                        .with_host_error(format!("unsupported date-range-picker action: {other}")),
                    }
                })
            },
            || {
                let rt = rt_for_reader;
                Rc::new(move || build_drp_snap(&rt))
            },
        );
    }

    let mut rt_for_field_val = runtime.clone();
    let mut rt_for_field_ph = runtime.clone();
    let mut rt_for_popover_open = runtime.clone();

    let stable_unavailable_cb = use_hook(|| {
        let is_disabled_sig = runtime.is_date_disabled;
        let is_unavailable_sig = runtime.is_date_unavailable;
        Callback::new(move |date: CalendarDate| -> bool {
            let disabled = is_disabled_sig
                .try_read()
                .ok()
                .and_then(|slot| *slot)
                .is_some_and(|cb| cb.call(date));
            let unavailable = is_unavailable_sig
                .try_read()
                .ok()
                .and_then(|slot| *slot)
                .is_some_and(|cb| cb.call(date));
            disabled || unavailable
        })
    });
    let combined_unavailable = has_unavailable_pred.then_some(stable_unavailable_cb);

    let range_field_spec = RangeFieldRootSpec::<DateFieldPolicy> {
        id: Some(rels.field_scope().token().to_string()),
        class: props.class,
        style: props.style,
        value: props.value,
        default_value: props.default_value,
        on_value_change: Some(EventHandler::new(move |next| {
            rt_for_field_val.handle_field_value_change(next);
        })),
        on_value_change_details: None,
        start_value: props.start_value,
        on_start_value_change: None,
        on_start_value_change_details: None,
        end_value: props.end_value,
        on_end_value_change: None,
        on_end_value_change_details: None,
        placeholder: props.placeholder,
        has_explicit_default_placeholder: false,
        on_placeholder_change: Some(EventHandler::new(move |next| {
            rt_for_field_ph.handle_field_placeholder_change(next);
        })),
        on_placeholder_change_details: None,
        on_focus_out: None,
        granularity: props.granularity,
        hour_cycle: props.hour_cycle,
        locale: resolved_locale,
        hide_time_zone: props.hide_time_zone,
        min_value: props.min_value,
        max_value: props.max_value,
        disabled: effective_disabled,
        read_only: effective_read_only,
        required: effective_required,
        read_only_segments: props.read_only_segments,
        is_date_unavailable: combined_unavailable,
        validate: props.validate,
        on_invalid: props.on_invalid,
        error_message_id: props.error_message_id,
        dir: Some(resolved_dir),
        is_invalid_override: effective_invalid_override,
        child: props.child,
        on_mounted: props.on_mounted,
        agent: None,
        attributes: props.attributes,
        children: rsx! {
            PopoverRoot {
                id: Some(rels.popover_scope().token().to_string()),
                open: Some((runtime.open)()),
                on_open_change: move |next| rt_for_popover_open.handle_popover_open_change(next),
                on_open_change_complete: props.on_open_change_complete,
                modal: props.modal,
                {props.children}
            }
        },
    };
    let mut range_field_rt =
        use_range_field_runtime::<DateFieldPolicy>(&range_field_spec, initial_placeholder);

    // Sync calendar/picker-initiated range value & placeholder changes into embedded DateRangeField
    // without forcing uncontrolled DateRangePicker into controlled DateRangeField mode.
    let picker_val_now = (runtime.value)();
    let picker_ph_now = (runtime.placeholder)();
    let mut prev_picker_val = use_hook(|| CopyValue::new(picker_val_now));
    let mut prev_picker_ph = use_hook(|| CopyValue::new(picker_ph_now));
    if prev_picker_val
        .try_read()
        .is_ok_and(|prev| *prev != picker_val_now)
    {
        prev_picker_val.set(picker_val_now);
        if range_field_rt.state.peek().value() != picker_val_now {
            range_field_rt.reconcile_external_state(Some(picker_val_now), None, None, None, None);
        }
    }
    if prev_picker_ph
        .try_read()
        .is_ok_and(|prev| *prev != picker_ph_now)
    {
        prev_picker_ph.set(picker_ph_now);
        if range_field_rt.state.peek().placeholder() != picker_ph_now {
            range_field_rt.reconcile_external_state(None, None, None, Some(picker_ph_now), None);
        }
    }

    let (handle_focus_in, handle_focus_out, handle_region_dismiss) =
        use_composite_focus_out_tracked(
            vec![
                range_field_rt.current_root_dom_id(),
                runtime.current_trigger_dom_id(),
                runtime.current_content_dom_id(),
            ],
            props.on_focus_out,
        );
    runtime.set_composite_focus_handlers(Some(handle_focus_in), Some(handle_focus_out));
    range_field_rt.set_parent_focus_handlers(Some(handle_focus_in), Some(handle_focus_out));
    let is_open_now = *runtime.open.read();
    let mut prev_open_for_focus = use_hook(|| CopyValue::new(is_open_now));
    if prev_open_for_focus
        .try_read()
        .is_ok_and(|was| *was != is_open_now)
    {
        let was_open = prev_open_for_focus.try_read().map(|w| *w).unwrap_or(false);
        prev_open_for_focus.set(is_open_now);
        if was_open && !is_open_now {
            handle_region_dismiss.call(());
        }
    }

    {
        let mut rfrt = range_field_rt.clone();
        let is_disabled_sig = runtime.is_date_disabled;
        let is_unavailable_sig = runtime.is_date_unavailable;
        let pred_rev_sig = runtime.predicate_rev;
        use_effect(move || {
            let _ = *pred_rev_sig.read();
            let _ = *is_disabled_sig.read();
            let _ = *is_unavailable_sig.read();
            rfrt.sync_validation_and_notify();
        });
    }
    render_range_field_root_with_runtime::<DateFieldPolicy>(range_field_rt, range_field_spec)
}

pub use DateRangePickerRoot as DateRangePicker;

/// Props for [`DateRangePickerTrigger`].
#[derive(Clone, PartialEq, Props)]
pub struct DateRangePickerTriggerProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_blur: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Button that opens the `DateRangePicker` popover and participates in `DateRangeField` traversal.
#[component]
pub fn DateRangePickerTrigger(props: DateRangePickerTriggerProps) -> Element {
    let picker_rt = use_date_range_picker_context();
    let range_field_rt = try_use_date_range_field_context();
    let effective_disabled = props.disabled || picker_rt.snapshot().disabled;
    let dir = picker_rt.state.read().def().dir;

    let mut reg_field = range_field_rt.clone();
    let mut unreg_field = range_field_rt.clone();
    let mut drop_field = range_field_rt.clone();
    let focus_field = range_field_rt;
    let reg_picker = picker_rt.clone();
    let fi_picker = picker_rt.clone();
    let fo_picker = picker_rt;

    render_picker_trigger_surface(
        props.id,
        props.class,
        props.style,
        effective_disabled,
        dir,
        props.child,
        props.on_click,
        props.on_key_down,
        props.on_blur,
        Some(EventHandler::new(move |evt| {
            fi_picker.notify_composite_focus_in(evt);
        })),
        Some(EventHandler::new(move |evt| {
            fo_picker.notify_composite_focus_out(evt);
        })),
        props.on_mounted,
        props.attributes,
        props.children,
        move |tid| {
            if let Some(ref id_str) = tid {
                reg_picker.register_trigger_dom_id(id_str.clone());
            }
            if let Some(ref mut rfrt) = reg_field {
                rfrt.register_trigger_id(tid);
            }
        },
        move |tid| {
            if let Some(ref mut rfrt) = unreg_field {
                rfrt.unregister_trigger_id(tid);
            }
        },
        move |tid| {
            if let Some(ref mut rfrt) = drop_field {
                rfrt.unregister_trigger_id(tid);
            }
        },
        move || {
            if let Some(ref rfrt) = focus_field {
                rfrt.focus_last_segment();
            }
        },
    )
}

/// Props for floating [`DateRangePickerContent`].
#[derive(Clone, PartialEq, Props)]
pub struct DateRangePickerContentProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default, into)]
    pub role: Option<String>,
    #[props(default, into)]
    pub aria_labelledby: Option<String>,
    #[props(default, into)]
    pub aria_describedby: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default = false)]
    pub keep_mounted: bool,
    #[props(default)]
    pub side: Option<PlacementSide>,
    #[props(default)]
    pub align: Option<PlacementAlign>,
    #[props(default)]
    pub side_offset: Option<f32>,
    #[props(default)]
    pub align_offset: Option<f32>,
    #[props(default)]
    pub avoid_collisions: Option<bool>,
    #[props(default, into)]
    pub collision_boundary: Option<String>,
    #[props(default)]
    pub collision_padding: Option<f32>,
    #[props(default)]
    pub arrow_padding: Option<f32>,
    #[props(default)]
    pub sticky: Option<StickyStrategy>,
    #[props(default)]
    pub hide_when_detached: Option<bool>,
    #[props(default, into)]
    pub custom_anchor: Option<String>,
    #[props(default)]
    pub open_focus: Option<PopoverOpenFocusPolicy>,
    #[props(default)]
    pub close_focus: Option<PopoverCloseFocusPolicy>,
    #[props(default)]
    pub scroll_lock: Option<PopoverScrollLockPolicy>,
    #[props(default)]
    pub outside_interaction: Option<PopoverOutsideInteractionPolicy>,
    #[props(default)]
    pub prevent_body_pointer_events: Option<bool>,
    #[props(default)]
    pub escape_key_down_behavior: Option<DismissBehavior>,
    #[props(default)]
    pub interact_outside_behavior: Option<DismissBehavior>,
    #[props(default)]
    pub on_open_auto_focus: Option<EventHandler<OpenAutoFocusEvent>>,
    #[props(default)]
    pub on_close_auto_focus: Option<EventHandler<CloseAutoFocusEvent>>,
    #[props(default)]
    pub on_escape_key_down: Option<EventHandler<EscapeKeyDownEvent>>,
    #[props(default)]
    pub on_pointer_down_outside: Option<EventHandler<PointerDownOutsideEvent>>,
    #[props(default)]
    pub on_interact_outside: Option<EventHandler<InteractOutsideEvent>>,
    #[props(default)]
    pub child: Option<Callback<FloatingContentChildProps, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Props for static (non-floating) [`DateRangePickerContentStatic`].
#[derive(Clone, PartialEq, Props)]
pub struct DateRangePickerContentStaticProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default, into)]
    pub role: Option<String>,
    #[props(default, into)]
    pub aria_labelledby: Option<String>,
    #[props(default, into)]
    pub aria_describedby: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default = false)]
    pub keep_mounted: bool,
    #[props(default)]
    pub open_focus: Option<PopoverOpenFocusPolicy>,
    #[props(default)]
    pub close_focus: Option<PopoverCloseFocusPolicy>,
    #[props(default)]
    pub scroll_lock: Option<PopoverScrollLockPolicy>,
    #[props(default)]
    pub outside_interaction: Option<PopoverOutsideInteractionPolicy>,
    #[props(default)]
    pub prevent_body_pointer_events: Option<bool>,
    #[props(default)]
    pub escape_key_down_behavior: Option<DismissBehavior>,
    #[props(default)]
    pub interact_outside_behavior: Option<DismissBehavior>,
    #[props(default)]
    pub on_open_auto_focus: Option<EventHandler<OpenAutoFocusEvent>>,
    #[props(default)]
    pub on_close_auto_focus: Option<EventHandler<CloseAutoFocusEvent>>,
    #[props(default)]
    pub on_escape_key_down: Option<EventHandler<EscapeKeyDownEvent>>,
    #[props(default)]
    pub on_pointer_down_outside: Option<EventHandler<PointerDownOutsideEvent>>,
    #[props(default)]
    pub on_interact_outside: Option<EventHandler<InteractOutsideEvent>>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Floating popover dialog content for `DateRangePicker`.
#[component]
pub fn DateRangePickerContent(props: DateRangePickerContentProps) -> Element {
    let picker_rt = super::runtime::try_use_date_range_picker_context();
    let fi_rt = picker_rt.clone();
    let fo_rt = picker_rt.clone();
    let reg_rt = picker_rt;
    render_picker_content_surface(
        PickerContentSurfaceSpec {
            id: props.id,
            class: props.class,
            style: props.style,
            role: props.role,
            aria_labelledby: props.aria_labelledby,
            aria_describedby: props.aria_describedby,
            force_mount: props.force_mount,
            keep_mounted: props.keep_mounted,
            floating: true,
            side: props.side,
            align: props.align,
            side_offset: props.side_offset,
            align_offset: props.align_offset,
            avoid_collisions: props.avoid_collisions,
            collision_boundary: props.collision_boundary,
            collision_padding: props.collision_padding,
            arrow_padding: props.arrow_padding,
            sticky: props.sticky,
            hide_when_detached: props.hide_when_detached,
            custom_anchor: props.custom_anchor,
            open_focus: props.open_focus,
            close_focus: props.close_focus,
            scroll_lock: props.scroll_lock,
            outside_interaction: props.outside_interaction,
            prevent_body_pointer_events: props.prevent_body_pointer_events,
            escape_key_down_behavior: props.escape_key_down_behavior,
            interact_outside_behavior: props.interact_outside_behavior,
            on_open_auto_focus: props.on_open_auto_focus,
            on_close_auto_focus: props.on_close_auto_focus,
            on_escape_key_down: props.on_escape_key_down,
            on_pointer_down_outside: props.on_pointer_down_outside,
            on_interact_outside: props.on_interact_outside,
            default_open_focus_selector: DATE_RANGE_PICKER_OPEN_FOCUS_SELECTOR,
            extra_data_attr: "data-date-range-picker-content",
            on_focus_in: fi_rt
                .map(|rt| EventHandler::new(move |evt| rt.notify_composite_focus_in(evt))),
            on_focus_out: fo_rt
                .map(|rt| EventHandler::new(move |evt| rt.notify_composite_focus_out(evt))),
            floating_child: props.child,
            static_child: None,
            attributes: props.attributes,
            children: props.children,
        },
        move |cid| {
            if let Some(ref rt) = reg_rt {
                rt.register_content_dom_id(cid);
            }
        },
    )
}

/// Static (non-floating) popover dialog content for `DateRangePicker`.
#[component]
pub fn DateRangePickerContentStatic(props: DateRangePickerContentStaticProps) -> Element {
    let picker_rt = super::runtime::try_use_date_range_picker_context();
    let fi_rt = picker_rt.clone();
    let fo_rt = picker_rt.clone();
    let reg_rt = picker_rt;
    render_picker_content_surface(
        PickerContentSurfaceSpec {
            id: props.id,
            class: props.class,
            style: props.style,
            role: props.role,
            aria_labelledby: props.aria_labelledby,
            aria_describedby: props.aria_describedby,
            force_mount: props.force_mount,
            keep_mounted: props.keep_mounted,
            floating: false,
            side: None,
            align: None,
            side_offset: None,
            align_offset: None,
            avoid_collisions: None,
            collision_boundary: None,
            collision_padding: None,
            arrow_padding: None,
            sticky: None,
            hide_when_detached: None,
            custom_anchor: None,
            open_focus: props.open_focus,
            close_focus: props.close_focus,
            scroll_lock: props.scroll_lock,
            outside_interaction: props.outside_interaction,
            prevent_body_pointer_events: props.prevent_body_pointer_events,
            escape_key_down_behavior: props.escape_key_down_behavior,
            interact_outside_behavior: props.interact_outside_behavior,
            on_open_auto_focus: props.on_open_auto_focus,
            on_close_auto_focus: props.on_close_auto_focus,
            on_escape_key_down: props.on_escape_key_down,
            on_pointer_down_outside: props.on_pointer_down_outside,
            on_interact_outside: props.on_interact_outside,
            default_open_focus_selector: DATE_RANGE_PICKER_OPEN_FOCUS_SELECTOR,
            extra_data_attr: "data-date-range-picker-content",
            on_focus_in: fi_rt
                .map(|rt| EventHandler::new(move |evt| rt.notify_composite_focus_in(evt))),
            on_focus_out: fo_rt
                .map(|rt| EventHandler::new(move |evt| rt.notify_composite_focus_out(evt))),
            floating_child: None,
            static_child: props.child,
            attributes: props.attributes,
            children: props.children,
        },
        move |cid| {
            if let Some(ref rt) = reg_rt {
                rt.register_content_dom_id(cid);
            }
        },
    )
}

/// Props for [`DateRangePickerCalendar`].
#[derive(Clone, PartialEq, Props)]
pub struct DateRangePickerCalendarProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
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

/// Range calendar view synchronized with its ancestor `DateRangePickerRoot`.
///
/// Calls plain-fn [`render_range_calendar_root`] directly (zero nested `#[component]` taking
/// `Option<Callback|EventHandler>` per `Dioxus #5894`).
#[component]
pub fn DateRangePickerCalendar(props: DateRangePickerCalendarProps) -> Element {
    let picker_rt = use_date_range_picker_context();
    let cfg = picker_rt.calendar_config.read().clone();

    let _ = *picker_rt.predicate_rev.read();
    let is_date_disabled = *picker_rt.is_date_disabled.read();
    let is_date_unavailable = *picker_rt.is_date_unavailable.read();

    let mut rt_for_val = picker_rt.clone();
    let mut rt_for_select = picker_rt.clone();
    let mut rt_for_ph = picker_rt.clone();

    let cal_id = props
        .id
        .unwrap_or_else(|| picker_rt.rels.calendar_scope().token().to_string());

    let DateRangePickerCalendarConfig {
        config: cal_cfg,
        min_days,
        max_days,
        exclude_disabled,
        initial_focus,
    } = cfg;

    render_range_calendar_root(RangeCalendarRootProps {
        id: Some(cal_id),
        class: props.class,
        style: props.style,
        value: Some((picker_rt.calendar_value)()),
        default_value: None,
        on_value_change: Some(EventHandler::new(move |next| {
            rt_for_val.handle_calendar_value_change(next);
        })),
        on_value_change_details: None,
        start_value: Some((picker_rt.calendar_start)()),
        on_start_value_change: None,
        on_start_value_change_details: None,
        end_value: Some((picker_rt.calendar_end)()),
        on_end_value_change: None,
        on_end_value_change_details: None,
        on_range_select: Some(EventHandler::new(move |completed| {
            rt_for_select.handle_calendar_range_select(completed);
        })),
        min_days,
        max_days,
        exclude_disabled,
        placeholder: Some((picker_rt.calendar_placeholder)()),
        default_placeholder: None,
        on_placeholder_change: Some(EventHandler::new(move |ph| {
            rt_for_ph.handle_calendar_placeholder_change(ph);
        })),
        on_placeholder_change_details: None,
        min_value: cal_cfg.min_value,
        max_value: cal_cfg.max_value,
        disabled: cal_cfg.disabled,
        read_only: cal_cfg.read_only,
        prevent_deselect: cal_cfg.prevent_deselect,
        paged_navigation: cal_cfg.paged_navigation,
        fixed_weeks: cal_cfg.fixed_weeks,
        number_of_months: cal_cfg.number_of_months,
        week_starts_on: cal_cfg.week_starts_on,
        weekday_format: cal_cfg.weekday_format,
        month_format: cal_cfg.month_format,
        year_format: cal_cfg.year_format,
        locale: Some(cal_cfg.locale),
        calendar_label: cal_cfg.calendar_label,
        dir: Some(cal_cfg.dir),
        disable_days_outside_month: cal_cfg.disable_days_outside_month,
        initial_focus,
        is_date_disabled,
        is_date_unavailable,
        child: props.child,
        render_children: props.render_children,
        on_key_down: props.on_key_down,
        on_mounted: props.on_mounted,
        agent: Some(AgentHints::new().with_hidden(true)),
        attributes: props.attributes,
        children: props.children,
    })
}
