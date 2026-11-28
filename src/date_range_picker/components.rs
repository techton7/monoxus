use std::borrow::Cow;

use dioxus::prelude::*;
use ui_sdk::foundation::date_time::field_policy::DateFieldPolicy;

use crate::{
    date_field::default_date_field_placeholder,
    date_picker::{
        components::{
            DatePickerContentProps, render_picker_content_surface, render_picker_trigger_surface,
        },
        runtime::PickerRuntimeCore,
    },
    date_range_field::{
        DateRangeFieldRootRenderProps,
        components::{
            RangeFieldRootSpec, render_range_field_root_with_runtime, use_range_field_runtime,
        },
        try_use_date_range_field_context,
    },
    field::try_use_field_context,
    foundation::{
        browser::{focus::use_composite_focus_out_tracked, local_today},
        date_time::{
            CalendarConfig, CalendarDate, CalendarLocale, DateValue, DateValueRange,
            EditableSegmentKind, FieldInvalidEvent, Granularity, HourCycle, MonthFormat, Weekday,
            WeekdayFormat, YearFormat,
        },
        overlay::{PlacementAlign, PlacementSide, PortalHost},
        shared::{Direction, ScopeHandle, use_direction_context, use_stable_id},
        state::ControlledBinding,
    },
    popover::{
        CloseAutoFocusEvent, EscapeKeyDownEvent, InteractOutsideEvent, OpenAutoFocusEvent,
        PopoverCloseFocusPolicy, PopoverOpenFocusPolicy, PopoverOutsideInteractionPolicy,
        PopoverRoot, PopoverScrollLockPolicy,
    },
    range_calendar::{
        RangeCalendarRootProps, RangeCalendarRootRenderProps, RangeCalendarSnapshot,
        components::render_range_calendar_root,
    },
};

use super::{
    attrs::{DateRangePickerContentAttrs, DateRangePickerTriggerAttrs},
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
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub value: Option<Signal<DateValueRange>>,
    #[props(default)]
    pub controlled_value: Option<ReadSignal<DateValueRange>>,
    #[props(default)]
    pub default_value: DateValueRange,
    #[props(default)]
    pub on_value_change: Option<EventHandler<DateValueRange>>,
    #[props(default)]
    pub start_value: Option<Signal<Option<DateValue>>>,
    #[props(default)]
    pub controlled_start_value: Option<ReadSignal<Option<DateValue>>>,
    #[props(default)]
    pub on_start_value_change: Option<EventHandler<Option<DateValue>>>,
    #[props(default)]
    pub end_value: Option<Signal<Option<DateValue>>>,
    #[props(default)]
    pub controlled_end_value: Option<ReadSignal<Option<DateValue>>>,
    #[props(default)]
    pub on_end_value_change: Option<EventHandler<Option<DateValue>>>,
    #[props(default)]
    pub placeholder: Option<Signal<DateValue>>,
    #[props(default)]
    pub controlled_placeholder: Option<ReadSignal<DateValue>>,
    #[props(default)]
    pub default_placeholder: Option<DateValue>,
    #[props(default)]
    pub on_placeholder_change: Option<EventHandler<DateValue>>,
    #[props(default)]
    pub open: Option<Signal<bool>>,
    #[props(default)]
    pub controlled_open: Option<ReadSignal<bool>>,
    #[props(default = false)]
    pub default_open: bool,
    #[props(default)]
    pub on_open_change: Option<EventHandler<bool>>,
    #[props(default)]
    pub on_open_change_complete: Option<EventHandler<bool>>,
    #[props(default)]
    pub on_focus_out: Option<EventHandler<FocusEvent>>,
    #[props(default)]
    pub is_invalid_override: Option<bool>,
    #[props(default = true)]
    pub close_on_range_select: bool,
    #[props(default)]
    pub granularity: Option<Granularity>,
    #[props(default)]
    pub hour_cycle: Option<HourCycle>,
    #[props(default)]
    pub locale: CalendarLocale,
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
    pub readonly_segments: Vec<EditableSegmentKind>,
    #[props(default)]
    pub validate: Option<Callback<DateValueRange, Option<Vec<String>>>>,
    #[props(default)]
    pub on_invalid: Option<EventHandler<FieldInvalidEvent>>,
    #[props(default)]
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
    #[props(default = String::from("Event"))]
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
    pub open_focus: Option<PopoverOpenFocusPolicy>,
    #[props(default)]
    pub close_focus: Option<PopoverCloseFocusPolicy>,
    #[props(default)]
    pub scroll_lock: Option<PopoverScrollLockPolicy>,
    #[props(default)]
    pub outside_interaction: Option<PopoverOutsideInteractionPolicy>,
    #[props(default)]
    pub portal_host: Option<PortalHost>,
    #[props(default)]
    pub side: Option<PlacementSide>,
    #[props(default)]
    pub side_offset: Option<f32>,
    #[props(default)]
    pub align: Option<PlacementAlign>,
    #[props(default)]
    pub align_offset: Option<f32>,
    #[props(default)]
    pub avoid_collisions: Option<bool>,
    #[props(default)]
    pub collision_padding: Option<f32>,
    #[props(default)]
    pub sticky: Option<bool>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<DateRangeFieldRootRenderProps, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
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
    let effective_invalid_override = props
        .is_invalid_override
        .or_else(|| field_ctx.map(|f| f.is_invalid()));

    let resolved_dir = use_direction_context(props.dir);
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned)).into_owned();
    let scope = use_hook(|| ScopeHandle::root("date-range-picker").child(stable_id.clone()));
    let rels = use_hook(|| DateRangePickerRelationships::new(scope.clone()));
    let today = use_hook(local_today);

    let value_binding = ControlledBinding::from_props(props.value, props.controlled_value);
    let start_binding =
        ControlledBinding::from_props(props.start_value, props.controlled_start_value);
    let end_binding = ControlledBinding::from_props(props.end_value, props.controlled_end_value);
    let placeholder_binding =
        ControlledBinding::from_props(props.placeholder, props.controlled_placeholder);
    let open_binding = ControlledBinding::from_props(props.open, props.controlled_open);

    let mut initial_value = value_binding.initial_or(props.default_value);
    if let Some(s) = start_binding.peek_cloned() {
        initial_value.start = s;
    }
    if let Some(e) = end_binding.peek_cloned() {
        initial_value.end = e;
    }

    let initial_placeholder = placeholder_binding
        .peek_cloned()
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
    let initial_open = open_binding.initial_or(props.default_open);

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
            .with_locale(props.locale.clone())
            .with_calendar_label(props.calendar_label.clone())
            .with_direction(resolved_dir)
            .with_disable_days_outside_month(props.disable_days_outside_month),
        min_days: props.min_days,
        max_days: props.max_days,
        exclude_disabled: props.exclude_disabled,
        initial_focus: props.initial_focus,
    };
    let calendar_config = use_signal(|| next_cal_cfg.clone());

    let mut controlled_value = use_hook(|| CopyValue::new(value_binding));
    controlled_value.set(value_binding);
    let mut controlled_start_value = use_hook(|| CopyValue::new(start_binding));
    controlled_start_value.set(start_binding);
    let mut controlled_end_value = use_hook(|| CopyValue::new(end_binding));
    controlled_end_value.set(end_binding);
    let mut controlled_placeholder = use_hook(|| CopyValue::new(placeholder_binding));
    controlled_placeholder.set(placeholder_binding);
    let mut controlled_open = use_hook(|| CopyValue::new(open_binding));
    controlled_open.set(open_binding);
    let mut on_value_change = use_hook(|| CopyValue::new(props.on_value_change));
    on_value_change.set(props.on_value_change);
    let mut on_start_value_change = use_hook(|| CopyValue::new(props.on_start_value_change));
    on_start_value_change.set(props.on_start_value_change);
    let mut on_end_value_change = use_hook(|| CopyValue::new(props.on_end_value_change));
    on_end_value_change.set(props.on_end_value_change);
    let mut on_placeholder_change = use_hook(|| CopyValue::new(props.on_placeholder_change));
    on_placeholder_change.set(props.on_placeholder_change);
    let mut on_open_change = use_hook(|| CopyValue::new(props.on_open_change));
    on_open_change.set(props.on_open_change);
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
        crate::foundation::state::sync_signal(&mut is_date_disabled, props.is_date_disabled);
        crate::foundation::state::sync_signal(&mut is_date_unavailable, props.is_date_unavailable);
        let had_pred = prev_had_pred.try_read().is_ok_and(|had| *had);
        prev_had_pred.set(has_unavailable_pred);
        if has_unavailable_pred || had_pred {
            let next_rev = predicate_rev.try_peek().ok().map(|cur| cur.wrapping_add(1));
            if let Some(next_rev) = next_rev {
                crate::foundation::state::sync_signal(&mut predicate_rev, next_rev);
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
            controlled_value,
            controlled_placeholder,
            controlled_open,
            on_value_change,
            on_placeholder_change,
            on_open_change,
            on_composite_focus_in,
            on_composite_focus_out,
            is_date_disabled,
            is_date_unavailable,
            predicate_rev,
        },
        calendar_start: calendar_start_sig,
        calendar_end: calendar_end_sig,
        calendar_config,
        controlled_start_value,
        controlled_end_value,
        on_start_value_change,
        on_end_value_change,
    };

    // Synchronous render-time reconciliation for external controlled signals.
    {
        let ext_val = value_binding.read_cloned();
        let ext_start = start_binding.read_cloned();
        let ext_end = end_binding.read_cloned();
        let ext_ph = placeholder_binding.read_cloned();
        let ext_open = open_binding.read_cloned();
        runtime.reconcile_external_state(ext_val, ext_start, ext_end, ext_ph, ext_open);
    }

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
            &value_binding,
            &start_binding,
            &end_binding,
            &placeholder_binding,
            &open_binding,
        ),
        {
            let mut rt = runtime.clone();
            move |(val_bind, start_bind, end_bind, ph_bind, open_bind)| {
                let ext_val = val_bind.read_cloned();
                let ext_start = start_bind.read_cloned();
                let ext_end = end_bind.read_cloned();
                let ext_ph = ph_bind.read_cloned();
                let ext_open = open_bind.read_cloned();
                rt.reconcile_external_state(ext_val, ext_start, ext_end, ext_ph, ext_open);
            }
        },
    ));

    use_context_provider(|| DateRangePickerContext(runtime.clone()));

    let mut rt_for_field_val = runtime.clone();
    let mut rt_for_field_ph = runtime.clone();
    let mut rt_for_popover_open = runtime.clone();

    let resolved_open_focus = props.open_focus.unwrap_or_else(|| {
        PopoverOpenFocusPolicy::Selector(DATE_RANGE_PICKER_OPEN_FOCUS_SELECTOR.to_string())
    });
    let resolved_close_focus = props
        .close_focus
        .unwrap_or(PopoverCloseFocusPolicy::Trigger);

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

    let render_child = props.render_child;
    let range_field_spec = RangeFieldRootSpec::<DateFieldPolicy> {
        id: Some(rels.field_scope().token().to_string()),
        class: props.class,
        style: props.style,
        value: None,
        controlled_value: Some(runtime.value.into()),
        default_value: props.default_value,
        on_value_change: Some(EventHandler::new(move |next| {
            rt_for_field_val.handle_field_value_change(next);
        })),
        on_start_value_change: None,
        on_end_value_change: None,
        placeholder: None,
        controlled_placeholder: Some(runtime.placeholder.into()),
        on_placeholder_change: Some(EventHandler::new(move |next| {
            rt_for_field_ph.handle_field_placeholder_change(next);
        })),
        on_focus_out: None,
        granularity: props.granularity,
        hour_cycle: props.hour_cycle,
        locale: props.locale,
        hide_time_zone: props.hide_time_zone,
        min_value: props.min_value,
        max_value: props.max_value,
        disabled: effective_disabled,
        read_only: effective_read_only,
        required: effective_required,
        readonly_segments: props.readonly_segments,
        is_date_unavailable: combined_unavailable,
        validate: props.validate,
        on_invalid: props.on_invalid,
        error_message_id: props.error_message_id,
        dir: Some(resolved_dir),
        is_invalid_override: effective_invalid_override,
        as_child: props.as_child,
        on_mounted: props.on_mounted,
        attributes: props.attributes,
        children: rsx! {
            PopoverRoot {
                id: Some(rels.popover_scope().token().to_string()),
                open: Some(runtime.open),
                on_open_change: move |next| rt_for_popover_open.handle_popover_open_change(next),
                on_open_change_complete: props.on_open_change_complete,
                modal: props.modal,
                open_focus: Some(resolved_open_focus),
                close_focus: Some(resolved_close_focus),
                scroll_lock: props.scroll_lock,
                outside_interaction: props.outside_interaction,
                portal_host: props.portal_host,
                side: props.side,
                side_offset: props.side_offset,
                align: props.align,
                align_offset: props.align_offset,
                avoid_collisions: props.avoid_collisions,
                collision_padding: props.collision_padding,
                sticky: props.sticky,
                {props.children}
            }
        },
        debug_name: "DateRangePickerRoot",
    };
    let range_field_rt =
        use_range_field_runtime::<DateFieldPolicy>(&range_field_spec, initial_placeholder);

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
    render_range_field_root_with_runtime::<DateFieldPolicy, _>(
        range_field_rt,
        range_field_spec,
        move |view| {
            render_child.map(|cb| {
                cb.call(DateRangeFieldRootRenderProps {
                    attrs: view.attrs,
                    class: view.class,
                    style: view.style,
                    on_mounted: view.on_mounted,
                    attributes: view.attributes,
                    children: view.children,
                })
            })
        },
    )
}

pub use DateRangePickerRoot as DateRangePicker;

/// Render props passed to `as_child` for [`DateRangePickerTrigger`].
#[derive(Clone, PartialEq, Props)]
pub struct DateRangePickerTriggerRenderProps {
    pub attrs: DateRangePickerTriggerAttrs,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    pub on_click: EventHandler<MouseEvent>,
    pub on_pointer_down: EventHandler<PointerEvent>,
    pub on_key_down: EventHandler<KeyboardEvent>,
    pub on_mounted: EventHandler<MountedEvent>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Props for [`DateRangePickerTrigger`].
#[derive(Clone, PartialEq, Props)]
pub struct DateRangePickerTriggerProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<DateRangePickerTriggerRenderProps, Element>>,
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
    let render_child = props.render_child;

    render_picker_trigger_surface(
        props.id,
        props.class,
        props.style,
        effective_disabled,
        dir,
        props.as_child,
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
        move |attrs, class, style, on_click, on_pointer_down, on_key_down, on_mounted, attributes, children| {
            render_child.map(|cb| {
                cb.call(DateRangePickerTriggerRenderProps {
                    attrs,
                    class,
                    style,
                    on_click,
                    on_pointer_down,
                    on_key_down,
                    on_mounted,
                    attributes,
                    children,
                })
            })
        },
    )
}

/// Render props passed to `as_child` for [`DateRangePickerContent`] and [`DateRangePickerContentStatic`].
#[derive(Clone, PartialEq, Props)]
pub struct DateRangePickerContentRenderProps {
    pub attrs: DateRangePickerContentAttrs,
    #[props(default)]
    pub wrapper_id: Option<String>,
    #[props(default)]
    pub wrapper_style: Option<String>,
    pub role: String,
    pub aria_modal: bool,
    pub aria_labelledby: Option<String>,
    pub aria_describedby: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    pub style: String,
    pub on_mounted: EventHandler<MountedEvent>,
    pub on_key_down: EventHandler<KeyboardEvent>,
    pub on_click: EventHandler<MouseEvent>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Props for [`DateRangePickerContent`] and [`DateRangePickerContentStatic`].
#[derive(Clone, PartialEq, Props)]
pub struct DateRangePickerContentProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub role: Option<String>,
    #[props(default)]
    pub aria_labelledby: Option<String>,
    #[props(default)]
    pub aria_describedby: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
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
    #[props(default)]
    pub collision_padding: Option<f32>,
    #[props(default)]
    pub sticky: Option<bool>,
    #[props(default)]
    pub on_open_auto_focus: Option<EventHandler<OpenAutoFocusEvent>>,
    #[props(default)]
    pub on_close_auto_focus: Option<EventHandler<CloseAutoFocusEvent>>,
    #[props(default)]
    pub on_escape_key_down: Option<EventHandler<EscapeKeyDownEvent>>,
    #[props(default)]
    pub on_interact_outside: Option<EventHandler<InteractOutsideEvent>>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<DateRangePickerContentRenderProps, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

fn render_date_range_picker_content(
    props: DateRangePickerContentProps,
    floating: bool,
) -> Element {
    let picker_rt = super::runtime::try_use_date_range_picker_context();
    let fi_rt = picker_rt.clone();
    let fo_rt = picker_rt.clone();
    let reg_rt = picker_rt;
    let render_child = props.render_child;
    render_picker_content_surface(
        DatePickerContentProps {
            id: props.id,
            class: props.class,
            style: props.style,
            role: props.role,
            aria_labelledby: props.aria_labelledby,
            aria_describedby: props.aria_describedby,
            force_mount: props.force_mount,
            side: props.side,
            align: props.align,
            side_offset: props.side_offset,
            align_offset: props.align_offset,
            avoid_collisions: props.avoid_collisions,
            collision_padding: props.collision_padding,
            sticky: props.sticky,
            on_open_auto_focus: props.on_open_auto_focus,
            on_close_auto_focus: props.on_close_auto_focus,
            on_escape_key_down: props.on_escape_key_down,
            on_interact_outside: props.on_interact_outside,
            as_child: props.as_child,
            render_child: None,
            attributes: props.attributes,
            children: props.children,
        },
        floating,
        "data-date-range-picker-content",
        fi_rt.map(|rt| EventHandler::new(move |evt| rt.notify_composite_focus_in(evt))),
        fo_rt.map(|rt| EventHandler::new(move |evt| rt.notify_composite_focus_out(evt))),
        move |cid| {
            if let Some(ref rt) = reg_rt {
                rt.register_content_dom_id(cid);
            }
        },
        move |attrs,
              wrapper_id,
              wrapper_style,
              role,
              aria_modal,
              aria_labelledby,
              aria_describedby,
              class,
              style,
              on_mounted,
              on_key_down,
              on_click,
              attributes,
              children| {
            render_child.map(|cb| {
                cb.call(DateRangePickerContentRenderProps {
                    attrs,
                    wrapper_id,
                    wrapper_style,
                    role,
                    aria_modal,
                    aria_labelledby,
                    aria_describedby,
                    class,
                    style,
                    on_mounted,
                    on_key_down,
                    on_click,
                    attributes,
                    children,
                })
            })
        },
    )
}

/// Floating popover dialog content for `DateRangePicker`.
#[component]
pub fn DateRangePickerContent(props: DateRangePickerContentProps) -> Element {
    render_date_range_picker_content(props, true)
}

/// Static (non-floating) popover dialog content for `DateRangePicker`.
#[component]
pub fn DateRangePickerContentStatic(props: DateRangePickerContentProps) -> Element {
    render_date_range_picker_content(props, false)
}

/// Props for [`DateRangePickerCalendar`].
#[derive(Clone, PartialEq, Props)]
pub struct DateRangePickerCalendarProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
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
        value: Some(picker_rt.calendar_value),
        default_value: None,
        on_value_change: Some(EventHandler::new(move |next| {
            rt_for_val.handle_calendar_value_change(next);
        })),
        start_value: Some(picker_rt.calendar_start),
        on_start_value_change: None,
        end_value: Some(picker_rt.calendar_end),
        on_end_value_change: None,
        on_range_select: Some(EventHandler::new(move |completed| {
            rt_for_select.handle_calendar_range_select(completed);
        })),
        min_days,
        max_days,
        exclude_disabled,
        placeholder: Some(picker_rt.calendar_placeholder),
        default_placeholder: None,
        on_placeholder_change: Some(EventHandler::new(move |ph| {
            rt_for_ph.handle_calendar_placeholder_change(ph);
        })),
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
        locale: cal_cfg.locale,
        calendar_label: cal_cfg.calendar_label,
        dir: Some(cal_cfg.dir),
        disable_days_outside_month: cal_cfg.disable_days_outside_month,
        initial_focus,
        is_date_disabled,
        is_date_unavailable,
        as_child: props.as_child,
        render_child: props.render_child,
        render_children: props.render_children,
        on_key_down: props.on_key_down,
        on_mounted: props.on_mounted,
        attributes: props.attributes,
        children: props.children,
    })
}
