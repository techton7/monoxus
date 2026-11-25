use std::{borrow::Cow, cell::RefCell, rc::Rc};

use dioxus::prelude::keyboard_types::{Key, NamedKey};
use dioxus::prelude::*;

use crate::{
    date_field::{default_date_field_placeholder, local_today},
    date_range_field::{
        DateRangeFieldRoot, DateRangeFieldRootRenderProps, try_use_date_range_field_context,
    },
    foundation::{
        date_time::{
            CalendarDate, CalendarLocale, DateValue, DateValueRange, EditableSegmentKind,
            FieldInvalidEvent, Granularity, HourCycle, MonthFormat, Weekday, WeekdayFormat,
            YearFormat,
        },
        overlay::{
            OverlayBranchContext, OverlayEscapeGuardContext, OverlayEscapeGuardFn, PlacementAlign,
            PlacementSide, PortalHost,
        },
        shared::{Direction, ScopeHandle, use_direction_context, use_stable_id},
        state::DataState,
    },
    popover::{
        PopoverCloseFocusPolicy, PopoverContext, PopoverOpenFocusPolicy,
        PopoverOutsideInteractionPolicy, PopoverRoot, PopoverScrollLockPolicy,
    },
    range_calendar::{RangeCalendarRoot, RangeCalendarRootRenderProps, RangeCalendarSnapshot},
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
    pub default_value: DateValueRange,
    #[props(default)]
    pub on_value_change: Option<EventHandler<DateValueRange>>,
    #[props(default)]
    pub start_value: Option<Signal<Option<DateValue>>>,
    #[props(default)]
    pub on_start_value_change: Option<EventHandler<Option<DateValue>>>,
    #[props(default)]
    pub end_value: Option<Signal<Option<DateValue>>>,
    #[props(default)]
    pub on_end_value_change: Option<EventHandler<Option<DateValue>>>,
    #[props(default)]
    pub placeholder: Option<Signal<DateValue>>,
    #[props(default)]
    pub default_placeholder: Option<DateValue>,
    #[props(default)]
    pub on_placeholder_change: Option<EventHandler<DateValue>>,
    #[props(default)]
    pub open: Option<Signal<bool>>,
    #[props(default = false)]
    pub default_open: bool,
    #[props(default)]
    pub on_open_change: Option<EventHandler<bool>>,
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
    let resolved_dir = use_direction_context(props.dir);
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned)).into_owned();
    let scope = use_hook(|| ScopeHandle::root("date-range-picker").child(stable_id.clone()));
    let rels = use_hook(|| DateRangePickerRelationships::new(scope.clone()));
    let today = use_hook(local_today);

    let mut initial_value = props
        .value
        .map(|s| *s.peek())
        .unwrap_or(props.default_value);
    if let Some(s) = props.start_value {
        initial_value.start = *s.peek();
    }
    if let Some(e) = props.end_value {
        initial_value.end = *e.peek();
    }

    let initial_placeholder = props
        .placeholder
        .map(|s| *s.peek())
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
    let initial_open = props
        .open
        .map(|s| *s.peek())
        .unwrap_or(props.default_open);

    let def = DateRangePickerDefinition::new()
        .with_close_on_range_select(props.close_on_range_select)
        .with_disabled(props.disabled)
        .with_read_only(props.read_only)
        .with_required(props.required)
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
    let open_sig = use_signal(|| if props.disabled { false } else { initial_open });

    let next_cal_cfg = DateRangePickerCalendarConfig {
        min_value: props.min_value.map(|v| v.date()),
        max_value: props.max_value.map(|v| v.date()),
        min_days: props.min_days,
        max_days: props.max_days,
        exclude_disabled: props.exclude_disabled,
        disabled: props.disabled,
        read_only: props.read_only,
        prevent_deselect: props.prevent_deselect,
        paged_navigation: props.paged_navigation,
        fixed_weeks: props.fixed_weeks,
        number_of_months: props.number_of_months,
        week_starts_on: props.week_starts_on,
        weekday_format: props.weekday_format,
        month_format: props.month_format,
        year_format: props.year_format,
        locale: props.locale.clone(),
        calendar_label: props.calendar_label.clone(),
        dir: resolved_dir,
        disable_days_outside_month: props.disable_days_outside_month,
        initial_focus: props.initial_focus,
    };
    let mut calendar_config = use_signal(|| next_cal_cfg.clone());
    if *calendar_config.peek() != next_cal_cfg {
        calendar_config.set(next_cal_cfg);
    }

    let controlled_value = use_hook(|| Rc::new(RefCell::new(props.value)));
    *controlled_value.borrow_mut() = props.value;

    let controlled_start_value = use_hook(|| Rc::new(RefCell::new(props.start_value)));
    *controlled_start_value.borrow_mut() = props.start_value;

    let controlled_end_value = use_hook(|| Rc::new(RefCell::new(props.end_value)));
    *controlled_end_value.borrow_mut() = props.end_value;

    let controlled_placeholder = use_hook(|| Rc::new(RefCell::new(props.placeholder)));
    *controlled_placeholder.borrow_mut() = props.placeholder;

    let controlled_open = use_hook(|| Rc::new(RefCell::new(props.open)));
    *controlled_open.borrow_mut() = props.open;

    let on_value_change = use_hook(|| Rc::new(RefCell::new(props.on_value_change)));
    *on_value_change.borrow_mut() = props.on_value_change;

    let on_start_value_change = use_hook(|| Rc::new(RefCell::new(props.on_start_value_change)));
    *on_start_value_change.borrow_mut() = props.on_start_value_change;

    let on_end_value_change = use_hook(|| Rc::new(RefCell::new(props.on_end_value_change)));
    *on_end_value_change.borrow_mut() = props.on_end_value_change;

    let on_placeholder_change = use_hook(|| Rc::new(RefCell::new(props.on_placeholder_change)));
    *on_placeholder_change.borrow_mut() = props.on_placeholder_change;

    let on_open_change = use_hook(|| Rc::new(RefCell::new(props.on_open_change)));
    *on_open_change.borrow_mut() = props.on_open_change;

    let is_date_disabled = use_hook(|| Rc::new(RefCell::new(props.is_date_disabled)));
    *is_date_disabled.borrow_mut() = props.is_date_disabled;

    let is_date_unavailable = use_hook(|| Rc::new(RefCell::new(props.is_date_unavailable)));
    *is_date_unavailable.borrow_mut() = props.is_date_unavailable;

    let mut runtime = DateRangePickerRuntime {
        state,
        rels: rels.clone(),
        value: value_sig,
        calendar_value: calendar_value_sig,
        calendar_start: calendar_start_sig,
        calendar_end: calendar_end_sig,
        placeholder: placeholder_sig,
        calendar_placeholder: calendar_placeholder_sig,
        open: open_sig,
        calendar_config,
        controlled_value,
        controlled_start_value,
        controlled_end_value,
        controlled_placeholder,
        controlled_open,
        on_value_change,
        on_start_value_change,
        on_end_value_change,
        on_placeholder_change,
        on_open_change,
        is_date_disabled,
        is_date_unavailable,
    };

    if *runtime.state.peek().def() != def {
        runtime.state.write().sync_definition(def);
    }
    let ext_val = props.value.map(|s| *s.read());
    let ext_start = props.start_value.map(|s| *s.read());
    let ext_end = props.end_value.map(|s| *s.read());
    let ext_ph = props.placeholder.map(|s| *s.read());
    let ext_open = props.open.map(|s| *s.read());
    runtime.reconcile_external_state(ext_val, ext_start, ext_end, ext_ph, ext_open);

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

    rsx! {
        DateRangeFieldRoot {
            id: Some(rels.field_scope().token().to_string()),
            class: props.class,
            style: props.style,
            value: Some(runtime.value),
            on_value_change: move |next| rt_for_field_val.handle_field_value_change(next),
            placeholder: Some(runtime.placeholder),
            on_placeholder_change: move |next| rt_for_field_ph.handle_field_placeholder_change(next),
            granularity: props.granularity,
            hour_cycle: props.hour_cycle,
            locale: props.locale,
            hide_time_zone: props.hide_time_zone,
            min_value: props.min_value,
            max_value: props.max_value,
            disabled: props.disabled,
            read_only: props.read_only,
            required: props.required,
            readonly_segments: props.readonly_segments,
            validate: props.validate,
            on_invalid: props.on_invalid,
            error_message_id: props.error_message_id,
            dir: Some(resolved_dir),
            as_child: props.as_child,
            render_child: props.render_child,
            on_mounted: props.on_mounted,
            attributes: props.attributes,
            PopoverRoot {
                id: Some(rels.popover_scope().token().to_string()),
                open: Some(runtime.open),
                on_open_change: move |next| rt_for_popover_open.handle_popover_open_change(next),
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
                {props.children}
            }
        }
    }
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
    let popover_ctx = use_context::<PopoverContext>();
    let range_field_rt = try_use_date_range_field_context();

    let popover_rt = popover_ctx.runtime.read().clone();
    let trigger = popover_rt.trigger();
    let trigger_id = props
        .id
        .clone()
        .unwrap_or_else(|| trigger.id().to_string());
    popover_rt.register_trigger_id(&trigger_id);
    let popover_rt_for_drop = popover_rt.clone();
    let popover_tid_for_drop = trigger_id.clone();
    use_drop(move || {
        popover_rt_for_drop.unregister_trigger_id(&popover_tid_for_drop);
    });

    let picker_snap = picker_rt.snapshot();
    let effective_disabled = props.disabled || picker_snap.disabled;
    let dir = picker_rt.state.read().def().dir;

    if let Some(mut rfrt) = range_field_rt.clone() {
        rfrt.register_trigger_id(if effective_disabled {
            None
        } else {
            Some(trigger_id.clone())
        });
    }

    let range_field_rt_for_drop = range_field_rt.clone();
    let tid_for_drop = trigger_id.clone();
    use_drop(move || {
        if let Some(mut rfrt) = range_field_rt_for_drop {
            if rfrt
                .trailing_trigger_id
                .try_peek()
                .ok()
                .and_then(|v| v.clone())
                .as_deref()
                == Some(tid_for_drop.as_str())
            {
                rfrt.register_trigger_id(None);
            }
        }
    });

    let attrs = DateRangePickerTriggerAttrs::new(
        trigger_id,
        trigger.aria_controls().unwrap_or_default().to_string(),
        popover_rt.is_open(),
        effective_disabled,
    );

    let on_mounted = {
        let mut mount_cb = popover_rt.mount_trigger();
        let user_cb = props.on_mounted;
        EventHandler::new(move |evt: MountedEvent| {
            mount_cb(evt.clone());
            if let Some(cb) = user_cb {
                cb.call(evt);
            }
        })
    };

    let on_pointer_down = {
        let mut pd_cb = popover_rt.trigger_pointer_down();
        EventHandler::new(move |evt: PointerEvent| {
            if !effective_disabled {
                pd_cb(evt);
            }
        })
    };

    let on_click = {
        let mut click_cb = popover_rt.trigger_click();
        let user_cb = props.on_click;
        EventHandler::new(move |evt: MouseEvent| {
            if !effective_disabled {
                click_cb(evt.clone());
            }
            if let Some(cb) = user_cb {
                cb.call(evt);
            }
        })
    };

    let on_key_down = {
        let rfrt = range_field_rt;
        let user_cb = props.on_key_down;
        EventHandler::new(move |evt: KeyboardEvent| {
            if !effective_disabled {
                let move_to_prev = match evt.key() {
                    Key::Named(NamedKey::ArrowLeft) => dir == Direction::Ltr,
                    Key::Named(NamedKey::ArrowRight) => dir == Direction::Rtl,
                    _ => false,
                };
                if move_to_prev {
                    evt.prevent_default();
                    if let Some(ref field) = rfrt {
                        field.focus_last_segment();
                    }
                }
            }
            if let Some(cb) = user_cb {
                cb.call(evt);
            }
        })
    };

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(DateRangePickerTriggerRenderProps {
                attrs,
                class: props.class,
                style: props.style,
                on_click,
                on_pointer_down,
                on_key_down,
                on_mounted,
                attributes: props.attributes,
                children: props.children,
            });
        }
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::DateRangePickerTrigger: as_child requires render_child; rendering the default button"
        );
    }

    rsx! {
        button {
            id: "{attrs.id}",
            r#type: "button",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            disabled: attrs.disabled,
            tabindex: attrs.tabindex,
            aria_haspopup: attrs.aria_haspopup,
            aria_expanded: attrs.aria_expanded,
            aria_controls: (!attrs.aria_controls.is_empty()).then_some(attrs.aria_controls.as_str()),
            "data-segment": attrs.data_segment,
            "data-state": attrs.data_state,
            "data-disabled": attrs.data_disabled,
            onmounted: move |evt| on_mounted.call(evt),
            onpointerdown: move |evt| on_pointer_down.call(evt),
            onclick: move |evt| on_click.call(evt),
            onkeydown: move |evt| on_key_down.call(evt),
            ..props.attributes,
            {props.children}
        }
    }
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
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Floating popover dialog content for `DateRangePicker`.
#[component]
pub fn DateRangePickerContent(props: DateRangePickerContentProps) -> Element {
    let mut ctx = use_context::<PopoverContext>();
    let runtime = ctx.runtime.read().clone();
    let should_render = props.force_mount || runtime.should_render_content();
    let is_open = runtime.is_open();

    let content = runtime.content();
    let content_id = props.id.unwrap_or_else(|| content.id().to_string());
    let wrapper_id = format!("{}-wrapper", content_id);

    if should_render {
        if ctx.content_id.peek().as_deref() != Some(content_id.as_str()) {
            ctx.content_id.set(Some(content_id.clone()));
        }
        runtime.register_content_id(&content_id);
    } else {
        if ctx.content_id.peek().as_deref() == Some(content_id.as_str()) {
            ctx.content_id.set(None);
        }
        runtime.unregister_content_id(&content_id);
    }
    let cid_for_drop = content_id.clone();
    let mut cid_sig_drop = ctx.content_id;
    let runtime_for_drop = runtime.clone();
    use_drop(move || {
        if let Ok(mut sig) = cid_sig_drop.try_write() {
            if sig.as_deref() == Some(cid_for_drop.as_str()) {
                *sig = None;
            }
        }
        runtime_for_drop.unregister_content_id(&cid_for_drop);
        crate::foundation::overlay::unregister_active_overlay_layer(&cid_for_drop);
    });

    let effect_wrapper_id = wrapper_id.clone();
    let runtime_for_shield = runtime.clone();
    let branches_sig = runtime.branches_signal();
    use_effect(use_reactive(
        (&should_render, &is_open, &effect_wrapper_id),
        move |(render_val, _open_val, wid)| {
            let _ = branches_sig.read();
            if render_val {
                runtime_for_shield.sync_modal_shield_with_wrapper(Some(&wid));
            } else {
                runtime_for_shield.sync_modal_shield_with_wrapper(None);
            }
        },
    ));

    let runtime_for_branch = runtime.clone();
    use_context_provider(|| {
        let rt_reg = runtime_for_branch.clone();
        let rt_unreg = runtime_for_branch.clone();
        OverlayBranchContext::new(
            move |branch: String| {
                rt_reg.register_branch(branch);
            },
            move |branch: String| {
                rt_unreg.unregister_branch(&branch);
            },
        )
    });

    let runtime_for_escape_guard = runtime.clone();
    use_context_provider(|| {
        let rt_reg = runtime_for_escape_guard.clone();
        let rt_unreg = runtime_for_escape_guard.clone();
        OverlayEscapeGuardContext::new(
            move |id: String, guard: OverlayEscapeGuardFn| {
                rt_reg.register_escape_guard(id, guard);
            },
            move |id: String| {
                rt_unreg.unregister_escape_guard(&id);
            },
        )
    });

    let mut layer_registered = use_signal(|| false);
    if is_open && !*layer_registered.peek() {
        crate::foundation::overlay::register_active_overlay_layer(&content_id);
        layer_registered.set(true);
    } else if !is_open && *layer_registered.peek() {
        crate::foundation::overlay::unregister_active_overlay_layer(&content_id);
        layer_registered.set(false);
    }

    if !should_render {
        return rsx! {};
    }

    let wrapper_style = runtime.wrapper_style();
    let css_vars = runtime.content_css_custom_properties();

    let title_id = (ctx.title_id)();
    let desc_id = (ctx.description_id)();
    let labelledby = props.aria_labelledby.or(title_id);
    let describedby = props.aria_describedby.or(desc_id);
    let resolved_role = props.role.as_deref().unwrap_or(content.role());

    let attrs = DateRangePickerContentAttrs::new(
        content_id,
        is_open,
        Some(content.data_side()),
        Some(content.data_align()),
    );

    let mut inner_style = format!("{css_vars} {}", props.style.as_deref().unwrap_or_default());
    if !is_open || content.data_state() == &DataState::Closed {
        if !inner_style.trim().is_empty() && !inner_style.trim_end().ends_with(';') {
            inner_style.push(';');
        }
        inner_style.push_str(" pointer-events: none;");
    } else if runtime.should_force_pointer_events_auto()
        && !inner_style.contains("pointer-events:")
    {
        if !inner_style.trim().is_empty() && !inner_style.trim_end().ends_with(';') {
            inner_style.push(';');
        }
        inner_style.push_str(" pointer-events: auto;");
    }

    rsx! {
        div {
            id: "{wrapper_id}",
            "data-monoxus-floating-content-wrapper": "",
            style: "{wrapper_style}",
            div {
                id: "{attrs.id}",
                role: resolved_role,
                aria_modal: content.aria_modal(),
                aria_labelledby: labelledby.as_deref(),
                aria_describedby: describedby.as_deref(),
                class: props.class.as_deref().unwrap_or_default(),
                style: "{inner_style}",
                "data-state": content.data_state().as_str(),
                "data-side": attrs.data_side,
                "data-align": attrs.data_align,
                "data-date-range-picker-content": attrs.data_date_range_picker_content,
                tabindex: attrs.tabindex,
                onmounted: runtime.mount_content(),
                onkeydown: runtime.escape_keydown(),
                onclick: move |evt: MouseEvent| evt.stop_propagation(),
                ..props.attributes,
                {props.children}
            }
        }
    }
}

/// Static (non-floating) popover dialog content for `DateRangePicker`.
#[component]
pub fn DateRangePickerContentStatic(props: DateRangePickerContentProps) -> Element {
    let mut ctx = use_context::<PopoverContext>();
    let runtime = ctx.runtime.read().clone();
    let should_render = props.force_mount || runtime.should_render_content();
    let is_open = runtime.is_open();

    let content = runtime.content();
    let content_id = props.id.unwrap_or_else(|| content.id().to_string());

    if should_render {
        if ctx.content_id.peek().as_deref() != Some(content_id.as_str()) {
            ctx.content_id.set(Some(content_id.clone()));
        }
        runtime.register_content_id(&content_id);
    } else {
        if ctx.content_id.peek().as_deref() == Some(content_id.as_str()) {
            ctx.content_id.set(None);
        }
        runtime.unregister_content_id(&content_id);
    }
    let cid_for_drop = content_id.clone();
    let mut cid_sig_drop = ctx.content_id;
    let runtime_for_drop = runtime.clone();
    use_drop(move || {
        if let Ok(mut sig) = cid_sig_drop.try_write() {
            if sig.as_deref() == Some(cid_for_drop.as_str()) {
                *sig = None;
            }
        }
        runtime_for_drop.unregister_content_id(&cid_for_drop);
        crate::foundation::overlay::unregister_active_overlay_layer(&cid_for_drop);
    });

    let runtime_for_branch = runtime.clone();
    use_context_provider(|| {
        let rt_reg = runtime_for_branch.clone();
        let rt_unreg = runtime_for_branch.clone();
        OverlayBranchContext::new(
            move |branch: String| {
                rt_reg.register_branch(branch);
            },
            move |branch: String| {
                rt_unreg.unregister_branch(&branch);
            },
        )
    });

    let runtime_for_escape_guard = runtime.clone();
    use_context_provider(|| {
        let rt_reg = runtime_for_escape_guard.clone();
        let rt_unreg = runtime_for_escape_guard.clone();
        OverlayEscapeGuardContext::new(
            move |id: String, guard: OverlayEscapeGuardFn| {
                rt_reg.register_escape_guard(id, guard);
            },
            move |id: String| {
                rt_unreg.unregister_escape_guard(&id);
            },
        )
    });

    let mut layer_registered = use_signal(|| false);
    if is_open && !*layer_registered.peek() {
        crate::foundation::overlay::register_active_overlay_layer(&content_id);
        layer_registered.set(true);
    } else if !is_open && *layer_registered.peek() {
        crate::foundation::overlay::unregister_active_overlay_layer(&content_id);
        layer_registered.set(false);
    }

    if !should_render {
        return rsx! {};
    }

    let title_id = (ctx.title_id)();
    let desc_id = (ctx.description_id)();
    let labelledby = props.aria_labelledby.or(title_id);
    let describedby = props.aria_describedby.or(desc_id);
    let resolved_role = props.role.as_deref().unwrap_or(content.role());

    let attrs = DateRangePickerContentAttrs::new(content_id, is_open, None, None);

    rsx! {
        div {
            id: "{attrs.id}",
            role: resolved_role,
            aria_modal: content.aria_modal(),
            aria_labelledby: labelledby.as_deref(),
            aria_describedby: describedby.as_deref(),
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            "data-state": content.data_state().as_str(),
            "data-date-range-picker-content": attrs.data_date_range_picker_content,
            tabindex: attrs.tabindex,
            onmounted: runtime.mount_content(),
            onkeydown: runtime.escape_keydown(),
            onclick: move |evt: MouseEvent| evt.stop_propagation(),
            ..props.attributes,
            {props.children}
        }
    }
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
#[component]
pub fn DateRangePickerCalendar(props: DateRangePickerCalendarProps) -> Element {
    let picker_rt = use_date_range_picker_context();
    let cfg = picker_rt.calendar_config.read().clone();

    let is_date_disabled = *picker_rt.is_date_disabled.borrow();
    let is_date_unavailable = *picker_rt.is_date_unavailable.borrow();

    let mut rt_for_val = picker_rt.clone();
    let mut rt_for_select = picker_rt.clone();
    let mut rt_for_ph = picker_rt.clone();

    let cal_id = props
        .id
        .unwrap_or_else(|| picker_rt.rels.calendar_scope().token().to_string());

    rsx! {
        RangeCalendarRoot {
            id: Some(cal_id),
            class: props.class,
            style: props.style,
            value: Some(picker_rt.calendar_value),
            start_value: Some(picker_rt.calendar_start),
            end_value: Some(picker_rt.calendar_end),
            on_value_change: move |next| rt_for_val.handle_calendar_value_change(next),
            on_range_select: move |completed| rt_for_select.handle_calendar_range_select(completed),
            placeholder: Some(picker_rt.calendar_placeholder),
            on_placeholder_change: move |ph| rt_for_ph.handle_calendar_placeholder_change(ph),
            min_value: cfg.min_value,
            max_value: cfg.max_value,
            min_days: cfg.min_days,
            max_days: cfg.max_days,
            exclude_disabled: cfg.exclude_disabled,
            disabled: cfg.disabled,
            read_only: cfg.read_only,
            prevent_deselect: cfg.prevent_deselect,
            paged_navigation: cfg.paged_navigation,
            fixed_weeks: cfg.fixed_weeks,
            number_of_months: cfg.number_of_months,
            week_starts_on: cfg.week_starts_on,
            weekday_format: cfg.weekday_format,
            month_format: cfg.month_format,
            year_format: cfg.year_format,
            locale: cfg.locale,
            calendar_label: cfg.calendar_label,
            dir: Some(cfg.dir),
            disable_days_outside_month: cfg.disable_days_outside_month,
            initial_focus: cfg.initial_focus,
            is_date_disabled,
            is_date_unavailable,
            as_child: props.as_child,
            render_child: props.render_child,
            render_children: props.render_children,
            on_key_down: props.on_key_down,
            on_mounted: props.on_mounted,
            attributes: props.attributes,
            {props.children}
        }
    }
}
