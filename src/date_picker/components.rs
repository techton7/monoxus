use std::{borrow::Cow, cell::RefCell, rc::Rc};

use dioxus::prelude::keyboard_types::{Key, NamedKey};
use dioxus::prelude::*;
use ui_sdk::foundation::date_time::field_policy::{DateFieldPolicy, FieldRangeRole};
use ui_sdk::kernel::ChangeEvent;

use crate::{
    agent::{
        ActionDescriptor, AgentHints, AgentSurface, ComponentSnapshot, DispatchExecResult,
        SnapshotValue, SurfaceMetaDefaults, use_agent_surface,
    },
    calendar::{CalendarMode, CalendarRootProps, CalendarSnapshot, components::render_calendar_root},
    date_field::{
        components::{
            SegmentFieldRootSpec, render_segment_field_chrome, use_segment_field_runtime,
        },
        default_date_field_placeholder, try_use_date_field_context,
    },
    field::try_use_field_context,
    foundation::{
        browser::{focus::use_composite_focus_out_tracked, local_today},
        compose::{
            FloatingContentChildProps, collect_user_attrs, join_styles, merge_props, push_attr_opt,
            push_bool_data_attr, push_event_handler,
        },
        config::use_locale_context,
        date_time::{
            CalendarConfig, CalendarDate, CalendarLocale, DateValue, EditableSegmentKind,
            FieldInvalidEvent, Granularity, HourCycle, MonthFormat, Weekday, WeekdayFormat,
            YearFormat,
        },
        keyboard::is_ime_composing_key,
        overlay::{
            DismissBehavior, OverlayBranchContext, OverlayEscapeGuardContext, OverlayEscapeGuardFn,
            PlacementAlign, PlacementSide, StickyStrategy, use_overlay_branch_provider,
            use_register_parent_overlay_branches,
        },
        shared::{Direction, ScopeHandle, use_direction_context, use_stable_id},
        state::{ControllableConfig, DataState, sync_signal, use_controllable_signal_config},
    },
    popover::{
        CloseAutoFocusEvent, EscapeKeyDownEvent, InteractOutsideEvent, OpenAutoFocusEvent,
        PointerDownOutsideEvent, PopoverCloseFocusPolicy, PopoverContext, PopoverOpenFocusPolicy,
        PopoverOutsideInteractionPolicy, PopoverRoot, PopoverScrollLockPolicy,
    },
};

use super::{
    relationships::{DATE_PICKER_OPEN_FOCUS_SELECTOR, DatePickerRelationships},
    runtime::{
        DatePickerCalendarConfig, DatePickerContext, DatePickerRuntime, PickerRuntimeCore,
        use_date_picker_context,
    },
    state::{DatePickerDefinition, DatePickerState},
};

/// Shared plain-fn trigger surface composing `PopoverContext` with `data-segment="trigger"`,
/// equality-guarded `trailing_trigger_id` registration, RTL-aware `ArrowLeft`/`ArrowRight` ->
/// `focus_last_segment()` handoff, and unified `merge_props` + `child`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_picker_trigger_surface<FReg, FUnreg, FUnregDrop, FFocus>(
    id: Option<String>,
    class: Option<String>,
    style: Option<String>,
    effective_disabled: bool,
    dir: Direction,
    child: Option<Callback<Vec<Attribute>, Element>>,
    on_click: Option<EventHandler<MouseEvent>>,
    on_key_down: Option<EventHandler<KeyboardEvent>>,
    on_blur: Option<EventHandler<FocusEvent>>,
    on_focus_in: Option<EventHandler<FocusEvent>>,
    on_focus_out: Option<EventHandler<FocusEvent>>,
    on_mounted: Option<EventHandler<MountedEvent>>,
    attributes: Vec<Attribute>,
    children: Element,
    mut register_trigger: FReg,
    mut unregister_trigger: FUnreg,
    mut unregister_trigger_drop: FUnregDrop,
    focus_last_segment: FFocus,
) -> Element
where
    FReg: FnMut(Option<String>),
    FUnreg: FnMut(&str),
    FUnregDrop: FnMut(&str) + 'static,
    FFocus: Fn() + 'static,
{
    let popover_ctx = use_context::<PopoverContext>();
    let runtime = popover_ctx.runtime.read().clone();
    let trigger = runtime.trigger();
    let trigger_id = id.unwrap_or_else(|| trigger.id().to_string());

    let latest_tid = use_hook(|| Rc::new(RefCell::new(trigger_id.clone())));
    {
        let mut prev = latest_tid.borrow_mut();
        if *prev != trigger_id {
            runtime.unregister_trigger_id(&prev);
            unregister_trigger(&prev);
            *prev = trigger_id.clone();
        }
    }
    runtime.register_trigger_id(&trigger_id);
    if effective_disabled {
        unregister_trigger(&trigger_id);
    } else {
        register_trigger(Some(trigger_id.clone()));
    }
    let tid_for_drop = Rc::clone(&latest_tid);
    let runtime_for_drop = runtime.clone();
    use_drop(move || {
        let current = tid_for_drop.borrow().clone();
        runtime_for_drop.unregister_trigger_id(&current);
        unregister_trigger_drop(&current);
    });

    let disabled = effective_disabled;
    let tabindex = (!disabled).then_some("0");
    let data_state = match trigger.data_state() {
        DataState::Open => "open",
        _ => "closed",
    };
    let aria_expanded = if trigger.aria_expanded() {
        "true"
    } else {
        "false"
    };
    let aria_controls = trigger.aria_controls().map(str::to_owned);

    let mut mount_cb = runtime.mount_trigger();
    let mut pd_cb = runtime.trigger_pointer_down();
    let mut click_cb = runtime.trigger_click();

    let mut base = Vec::with_capacity(17);
    base.push(Attribute::new("id", trigger_id, None, false));
    base.push(Attribute::new("type", "button", None, false));
    if disabled {
        base.push(Attribute::new("disabled", true, None, false));
    }
    push_attr_opt(&mut base, "tabindex", tabindex);
    base.push(Attribute::new("aria-haspopup", "dialog", None, false));
    base.push(Attribute::new("aria-expanded", aria_expanded, None, false));
    push_attr_opt(&mut base, "aria-controls", aria_controls);
    base.push(Attribute::new("data-state", data_state, None, false));
    push_bool_data_attr(&mut base, "data-disabled", disabled);
    base.push(Attribute::new("data-segment", "trigger", None, false));
    push_event_handler(
        &mut base,
        "onmounted",
        Some(EventHandler::new(move |evt: MountedEvent| mount_cb(evt))),
    );
    push_event_handler(
        &mut base,
        "onpointerdown",
        Some(EventHandler::new(move |evt: PointerEvent| {
            if !disabled {
                pd_cb(evt);
            }
        })),
    );
    push_event_handler(
        &mut base,
        "onclick",
        Some(EventHandler::new(move |evt: MouseEvent| {
            if disabled {
                evt.prevent_default();
                return;
            }
            if evt.default_action_enabled() {
                click_cb(evt);
            }
        })),
    );
    push_event_handler(
        &mut base,
        "onkeydown",
        Some(EventHandler::new(move |evt: KeyboardEvent| {
            if !disabled && !is_ime_composing_key(&evt) && evt.default_action_enabled() {
                let move_to_prev = match evt.key() {
                    Key::Named(NamedKey::ArrowLeft) => dir == Direction::Ltr,
                    Key::Named(NamedKey::ArrowRight) => dir == Direction::Rtl,
                    _ => false,
                };
                if move_to_prev {
                    evt.prevent_default();
                    focus_last_segment();
                }
            }
        })),
    );
    push_event_handler(&mut base, "onfocusin", on_focus_in);
    push_event_handler(&mut base, "onfocusout", on_focus_out);

    let mut user = collect_user_attrs(class, style, attributes);
    push_event_handler(&mut user, "onmounted", on_mounted);
    if !disabled {
        push_event_handler(&mut user, "onclick", on_click);
    }
    push_event_handler(&mut user, "onkeydown", on_key_down);
    push_event_handler(&mut user, "onblur", on_blur);

    let merged = merge_props(base, user);

    if let Some(child) = child {
        return child.call(merged);
    }

    rsx! {
        button {
            ..merged,
            {children}
        }
    }
}

/// Internal specification for [`render_picker_content_surface`].
pub(crate) struct PickerContentSurfaceSpec {
    pub id: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub role: Option<String>,
    pub aria_labelledby: Option<String>,
    pub aria_describedby: Option<String>,
    pub force_mount: bool,
    pub keep_mounted: bool,
    pub floating: bool,
    pub side: Option<PlacementSide>,
    pub align: Option<PlacementAlign>,
    pub side_offset: Option<f32>,
    pub align_offset: Option<f32>,
    pub avoid_collisions: Option<bool>,
    pub collision_boundary: Option<String>,
    pub collision_padding: Option<f32>,
    pub arrow_padding: Option<f32>,
    pub sticky: Option<StickyStrategy>,
    pub hide_when_detached: Option<bool>,
    pub custom_anchor: Option<String>,
    pub open_focus: Option<PopoverOpenFocusPolicy>,
    pub close_focus: Option<PopoverCloseFocusPolicy>,
    pub scroll_lock: Option<PopoverScrollLockPolicy>,
    pub outside_interaction: Option<PopoverOutsideInteractionPolicy>,
    pub prevent_body_pointer_events: Option<bool>,
    pub escape_key_down_behavior: Option<DismissBehavior>,
    pub interact_outside_behavior: Option<DismissBehavior>,
    pub on_open_auto_focus: Option<EventHandler<OpenAutoFocusEvent>>,
    pub on_close_auto_focus: Option<EventHandler<CloseAutoFocusEvent>>,
    pub on_escape_key_down: Option<EventHandler<EscapeKeyDownEvent>>,
    pub on_pointer_down_outside: Option<EventHandler<PointerDownOutsideEvent>>,
    pub on_interact_outside: Option<EventHandler<InteractOutsideEvent>>,
    pub default_open_focus_selector: &'static str,
    pub extra_data_attr: &'static str,
    pub on_focus_in: Option<EventHandler<FocusEvent>>,
    pub on_focus_out: Option<EventHandler<FocusEvent>>,
    pub floating_child: Option<Callback<FloatingContentChildProps, Element>>,
    pub static_child: Option<Callback<Vec<Attribute>, Element>>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Shared plain-fn content surface for floating and static `DatePickerContent*` /
/// `DateRangePickerContent*`, wiring `keep_mounted`, `TransitionStatus`, 4-way `DismissBehavior`,
/// placement/policy overrides, z-index cleanup, `OverlayEscapeGuardContext`, and `merge_props`.
pub(crate) fn render_picker_content_surface(
    spec: PickerContentSurfaceSpec,
    mut register_content_id: impl FnMut(String),
) -> Element {
    let mut ctx = use_context::<PopoverContext>();
    let runtime = ctx.runtime.read().clone();

    runtime.set_keep_mounted(spec.keep_mounted);
    if spec.floating {
        runtime.set_content_placement_overrides(
            spec.side,
            spec.align,
            spec.side_offset,
            spec.align_offset,
            spec.avoid_collisions,
            spec.collision_boundary,
            spec.collision_padding,
            spec.arrow_padding,
            spec.sticky,
            spec.hide_when_detached,
            spec.custom_anchor,
        );
    }
    let resolved_open_focus = Some(spec.open_focus.unwrap_or_else(|| {
        PopoverOpenFocusPolicy::Selector(spec.default_open_focus_selector.to_string())
    }));
    let resolved_close_focus = Some(spec.close_focus.unwrap_or(PopoverCloseFocusPolicy::Trigger));
    runtime.set_content_policy_overrides(
        resolved_open_focus,
        resolved_close_focus,
        spec.scroll_lock,
        spec.outside_interaction,
        spec.prevent_body_pointer_events,
        spec.escape_key_down_behavior,
        spec.interact_outside_behavior,
    );
    runtime.set_content_event_handlers(
        spec.on_open_auto_focus,
        spec.on_close_auto_focus,
        spec.on_escape_key_down,
        spec.on_pointer_down_outside,
        spec.on_interact_outside,
    );

    let should_render = spec.force_mount || runtime.should_render_content();
    let is_open = runtime.is_open();

    let content = runtime.content();
    let content_id = spec.id.unwrap_or_else(|| content.id().to_string());
    register_content_id(content_id.clone());
    let wrapper_id = format!("{content_id}-wrapper");
    let portal_id = runtime.relationships().scope().qualify("portal");

    let parent_branch = try_consume_context::<OverlayBranchContext>();
    let branch_ids = if spec.floating {
        vec![content_id.clone(), wrapper_id.clone(), portal_id]
    } else {
        vec![content_id.clone(), portal_id]
    };
    use_register_parent_overlay_branches(parent_branch, is_open, branch_ids);

    let latest_cid = use_hook(|| Rc::new(RefCell::new(content_id.clone())));
    {
        let mut prev = latest_cid.borrow_mut();
        if *prev != content_id {
            if ctx.content_id.peek().as_deref() == Some(prev.as_str()) {
                ctx.content_id.set(None);
            }
            runtime.unregister_content_id(&prev);
            *prev = content_id.clone();
        }
    }

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

    let cid_for_drop = Rc::clone(&latest_cid);
    let mut cid_sig_drop = ctx.content_id;
    let runtime_for_drop = runtime.clone();
    use_drop(move || {
        let current_cid = cid_for_drop.borrow().clone();
        if let Ok(mut sig) = cid_sig_drop.try_write()
            && sig.as_deref() == Some(current_cid.as_str())
        {
            *sig = None;
        }
        runtime_for_drop.unregister_content_id(&current_cid);
        runtime_for_drop.clear_content_z_index();
        runtime_for_drop.set_keep_mounted(false);
        runtime_for_drop.set_content_placement_overrides(
            None, None, None, None, None, None, None, None, None, None, None,
        );
        runtime_for_drop.set_content_policy_overrides(None, None, None, None, None, None, None);
        runtime_for_drop.set_content_event_handlers(None, None, None, None, None);
    });

    let effect_wrapper_id = wrapper_id.clone();
    let runtime_for_shield = runtime.clone();
    let is_floating = spec.floating;
    use_effect(use_reactive(
        (&should_render, &is_floating, &effect_wrapper_id),
        move |(render_val, floating_val, wid)| {
            if render_val && floating_val {
                runtime_for_shield.sync_modal_shield_with_wrapper(Some(&wid));
            } else {
                runtime_for_shield.sync_modal_shield_with_wrapper(None);
            }
        },
    ));

    let rt_reg_branch = runtime.clone();
    let rt_unreg_branch = runtime.clone();
    use_overlay_branch_provider(
        move |branch: String| rt_reg_branch.register_branch(branch),
        move |branch: String| rt_unreg_branch.unregister_branch(&branch),
    );

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

    if !should_render {
        return rsx! {};
    }

    let wrapper_style = spec.floating.then(|| runtime.wrapper_style());
    let css_vars = if spec.floating {
        runtime.content_css_custom_properties()
    } else {
        String::new()
    };

    let title_id = (ctx.title_id)();
    let desc_id = (ctx.description_id)();
    let labelledby = spec.aria_labelledby.or(title_id);
    let describedby = spec.aria_describedby.or(desc_id);
    let resolved_role = spec.role.unwrap_or_else(|| content.role().to_string());
    let aria_modal = if content.aria_modal() {
        "true"
    } else {
        "false"
    };
    let data_state = match content.data_state() {
        DataState::Open => "open",
        _ => "closed",
    };
    let data_side = content.data_side();
    let data_align = content.data_align();
    let is_closed_state = *content.data_state() == DataState::Closed;
    let is_hidden = !spec.force_mount && runtime.is_hidden();

    let mut style_parts: Vec<&str> = Vec::with_capacity(2);
    if !css_vars.trim().is_empty() {
        style_parts.push(css_vars.as_str());
    }
    let has_pointer_events = spec
        .style
        .as_deref()
        .is_some_and(|s| s.contains("pointer-events:"));
    if !is_open || is_closed_state {
        style_parts.push("pointer-events: none;");
    } else if runtime.should_force_pointer_events_auto() && !has_pointer_events {
        style_parts.push("pointer-events: auto;");
    }
    let base_style = join_styles(style_parts);

    let mut mount_cb = runtime.mount_content();
    let mut keydown_cb = runtime.escape_keydown();

    let mut base = Vec::with_capacity(20);
    base.push(Attribute::new("id", content_id, None, false));
    base.push(Attribute::new("role", resolved_role, None, false));
    base.push(Attribute::new("aria-modal", aria_modal, None, false));
    push_attr_opt(&mut base, "aria-labelledby", labelledby);
    push_attr_opt(&mut base, "aria-describedby", describedby);
    push_attr_opt(&mut base, "style", base_style);
    base.push(Attribute::new("data-state", data_state, None, false));
    if spec.floating {
        base.push(Attribute::new("data-side", data_side, None, false));
        base.push(Attribute::new("data-align", data_align, None, false));
    }
    push_attr_opt(
        &mut base,
        "data-starting-style",
        runtime.data_starting_style(),
    );
    push_attr_opt(&mut base, "data-ending-style", runtime.data_ending_style());
    if is_hidden {
        base.push(Attribute::new("hidden", true, None, false));
    }
    base.push(Attribute::new("tabindex", "-1", None, false));
    base.push(Attribute::new(spec.extra_data_attr, "", None, false));
    push_event_handler(
        &mut base,
        "onmounted",
        Some(EventHandler::new(move |evt: MountedEvent| mount_cb(evt))),
    );
    push_event_handler(
        &mut base,
        "onkeydown",
        Some(EventHandler::new(move |evt: KeyboardEvent| {
            keydown_cb(evt)
        })),
    );
    push_event_handler(
        &mut base,
        "onclick",
        Some(EventHandler::new(move |evt: MouseEvent| {
            evt.stop_propagation();
        })),
    );
    push_event_handler(&mut base, "onfocusin", spec.on_focus_in);
    push_event_handler(&mut base, "onfocusout", spec.on_focus_out);

    let user = collect_user_attrs(spec.class, spec.style, spec.attributes);
    let merged = merge_props(base, user);

    if let Some(ws) = wrapper_style {
        let mut wrapper_props = Vec::with_capacity(4);
        wrapper_props.push(Attribute::new("id", wrapper_id, None, false));
        wrapper_props.push(Attribute::new(
            "data-monoxus-floating-content-wrapper",
            "",
            None,
            false,
        ));
        if !ws.is_empty() {
            wrapper_props.push(Attribute::new("style", ws, None, false));
        }
        if is_hidden {
            wrapper_props.push(Attribute::new("hidden", true, None, false));
        }

        if let Some(child) = spec.floating_child {
            return child.call(FloatingContentChildProps::new(wrapper_props, merged));
        }

        rsx! {
            div {
                ..wrapper_props,
                div {
                    ..merged,
                    {spec.children}
                }
            }
        }
    } else {
        if let Some(child) = spec.static_child {
            return child.call(merged);
        }

        rsx! {
            div {
                ..merged,
                {spec.children}
            }
        }
    }
}

/// Props for [`DatePickerRoot`].
#[derive(Clone, PartialEq, Props)]
pub struct DatePickerRootProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default)]
    pub value: Option<Option<DateValue>>,
    #[props(default)]
    pub default_value: Option<DateValue>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<Option<DateValue>>>,
    #[props(default)]
    pub on_value_change_details: Option<EventHandler<ChangeEvent<Option<DateValue>>>>,
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
    pub close_on_date_select: bool,
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
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub read_only: bool,
    #[props(default = false)]
    pub required: bool,
    #[props(default)]
    pub read_only_segments: Vec<EditableSegmentKind>,
    #[props(default)]
    pub validate: Option<Callback<DateValue, Option<Vec<String>>>>,
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
    pub agent: Option<AgentHints>,
    #[props(default)]
    pub children: Element,
}

/// Root provider composing `DateFieldRoot`, `PopoverRoot`, and `CalendarRoot` state.
#[component]
pub fn DatePickerRoot(props: DatePickerRootProps) -> Element {
    let field_ctx = try_use_field_context();
    let effective_disabled = props.disabled || field_ctx.is_some_and(|f| f.is_disabled());
    let effective_read_only = props.read_only || field_ctx.is_some_and(|f| f.is_read_only());
    let effective_required = props.required || field_ctx.is_some_and(|f| f.is_required());
    let effective_invalid_override = field_ctx.map(|f| f.is_invalid());

    let resolved_dir = use_direction_context(props.dir);
    let resolved_locale = use_locale_context(props.locale);
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned)).into_owned();
    let scope = use_hook(|| ScopeHandle::root("date-picker").child(stable_id.clone()));
    let rels = use_hook(|| DatePickerRelationships::new(scope.clone()));
    let today = use_hook(local_today);

    let value_ctl = use_controllable_signal_config(
        ControllableConfig::new(props.value, props.default_value)
            .with_explicit_default(props.default_value.is_some())
            .with_on_change(props.on_value_change)
            .with_on_change_details(props.on_value_change_details)
            .with_component("DatePickerRoot")
            .with_prop("value"),
    );
    let initial_value = value_ctl.peek();
    let has_explicit_default_placeholder = props.default_placeholder.is_some();
    let initial_placeholder = props
        .placeholder
        .or(props.default_placeholder)
        .unwrap_or_else(|| {
            default_date_field_placeholder(
                initial_value,
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
            .with_component("DatePickerRoot")
            .with_prop("placeholder"),
    );
    let open_ctl = use_controllable_signal_config(
        ControllableConfig::new(props.open, props.default_open)
            .with_explicit_default(props.default_open)
            .with_on_change(props.on_open_change)
            .with_on_change_details(props.on_open_change_details)
            .with_component("DatePickerRoot")
            .with_prop("open"),
    );
    let initial_open = open_ctl.peek();

    let def = DatePickerDefinition::new()
        .with_close_on_date_select(props.close_on_date_select)
        .with_disabled(effective_disabled)
        .with_read_only(effective_read_only)
        .with_required(effective_required)
        .with_dir(resolved_dir);

    let state = use_signal(|| {
        DatePickerState::new(def.clone(), initial_value, initial_placeholder, initial_open)
    });
    let value_sig = use_signal(|| initial_value);
    let calendar_value_sig = use_signal(|| initial_value.map(|v| v.date()));
    let placeholder_sig = use_signal(|| initial_placeholder);
    let calendar_placeholder_sig = use_signal(|| initial_placeholder.date());
    let open_sig = use_signal(|| if effective_disabled { false } else { initial_open });
    let trigger_dom_id = use_signal(|| None::<String>);
    let content_dom_id = use_signal(|| None::<String>);

    let next_cal_cfg = DatePickerCalendarConfig {
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
        initial_focus: props.initial_focus,
    };
    let calendar_config = use_signal(|| next_cal_cfg.clone());

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

    let mut runtime = DatePickerRuntime {
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
        calendar_config,
    };

    // Synchronous render-time reconciliation for external controlled props.
    runtime.reconcile_external_state(props.value, props.placeholder, props.open);

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
                    rt.reconcile_external_state(None, None, None);
                }
            }
        },
    ));

    use_effect(use_reactive((&props.value, &props.placeholder, &props.open), {
        let mut rt = runtime.clone();
        move |(ext_val, ext_ph, ext_open)| {
            rt.reconcile_external_state(ext_val, ext_ph, ext_open);
        }
    }));

    use_context_provider(|| DatePickerContext(runtime.clone()));

    {
        fn build_dp_snap(rt: &DatePickerRuntime) -> ComponentSnapshot {
            let open = rt.open.try_peek().map(|v| *v).unwrap_or(false);
            let val = rt
                .value
                .try_peek()
                .ok()
                .and_then(|v| *v)
                .map(|d| SnapshotValue::Text(d.to_iso_string()))
                .unwrap_or(SnapshotValue::Null);
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
                .with_field("value", val)
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
                    ActionDescriptor::new("open", "Open date picker popover", false),
                    ActionDescriptor::new("close", "Close date picker popover", false),
                ];
                if !effective_read_only {
                    actions.push(ActionDescriptor::new(
                        "set",
                        "Set ISO date value",
                        true,
                    ));
                    actions.push(ActionDescriptor::new(
                        "clear",
                        "Clear selected date value",
                        false,
                    ));
                }
                AgentSurface::new("date-picker", "Date picker")
                    .with_disabled(effective_disabled)
                    .with_snapshot(build_dp_snap(&rt_for_surface))
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
                    "set" => match dispatch.args.get("value") {
                        Some(SnapshotValue::Null) => {
                            let canceled =
                                rt.handle_field_value_change_with_context(None, dispatch.ctx);
                            DispatchExecResult::ok().with_canceled(canceled)
                        }
                        Some(SnapshotValue::Text(raw)) => {
                            let Some(parsed) = DateValue::from_iso_str(raw.trim()) else {
                                return DispatchExecResult::ok()
                                    .with_host_error(format!("invalid ISO date: {raw}"));
                            };
                            let canceled = rt.handle_field_value_change_with_context(
                                Some(parsed),
                                dispatch.ctx,
                            );
                            DispatchExecResult::ok().with_canceled(canceled)
                        }
                        _ => DispatchExecResult::ok()
                            .with_host_error("missing ISO date argument `value`"),
                    },
                    "clear" => {
                        let canceled =
                            rt.handle_field_value_change_with_context(None, dispatch.ctx);
                        DispatchExecResult::ok().with_canceled(canceled)
                    }
                    other => DispatchExecResult::ok()
                        .with_host_error(format!("unsupported date-picker action: {other}")),
                    }
                })
            },
            || {
                let rt = rt_for_reader;
                Rc::new(move || build_dp_snap(&rt))
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

    let field_spec = SegmentFieldRootSpec::<DateFieldPolicy> {
        id: Some(rels.field_scope().token().to_string()),
        value: props.value,
        default_value: props.default_value,
        on_value_change: Some(EventHandler::new(move |next| {
            rt_for_field_val.handle_field_value_change(next);
        })),
        on_value_change_details: None,
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
        range_role: FieldRangeRole::Standalone,
        label_id_override: None,
        on_prev_boundary: None,
        on_next_boundary: None,
        agent: None,
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
    let mut field_rt = use_segment_field_runtime::<DateFieldPolicy>(&field_spec, initial_placeholder);

    // Sync calendar/picker-initiated value & placeholder changes into the embedded DateField
    // without forcing uncontrolled DatePicker into controlled DateField mode.
    let picker_val_now = (runtime.value)();
    let picker_ph_now = (runtime.placeholder)();
    let mut prev_picker_val = use_hook(|| CopyValue::new(picker_val_now));
    let mut prev_picker_ph = use_hook(|| CopyValue::new(picker_ph_now));
    if prev_picker_val
        .try_read()
        .is_ok_and(|prev| *prev != picker_val_now)
    {
        prev_picker_val.set(picker_val_now);
        field_rt.reconcile_external_state(Some(picker_val_now), None, None);
    }
    if prev_picker_ph
        .try_read()
        .is_ok_and(|prev| *prev != picker_ph_now)
    {
        prev_picker_ph.set(picker_ph_now);
        field_rt.reconcile_external_state(None, Some(picker_ph_now), None);
    }

    let (handle_focus_in, handle_focus_out, handle_region_dismiss) =
        use_composite_focus_out_tracked(
            vec![
                field_rt.current_input_dom_id(),
                runtime.current_trigger_dom_id(),
                runtime.current_content_dom_id(),
            ],
            props.on_focus_out,
        );
    runtime.set_composite_focus_handlers(Some(handle_focus_in), Some(handle_focus_out));
    field_rt.set_parent_focus_handlers(Some(handle_focus_in), Some(handle_focus_out));
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
        let mut frt = field_rt.clone();
        let is_disabled_sig = runtime.is_date_disabled;
        let is_unavailable_sig = runtime.is_date_unavailable;
        let pred_rev_sig = runtime.predicate_rev;
        use_effect(move || {
            let _ = *pred_rev_sig.read();
            let _ = *is_disabled_sig.read();
            let _ = *is_unavailable_sig.read();
            frt.sync_validation_and_notify();
        });
    }
    render_segment_field_chrome(&field_rt, field_spec.children)
}

pub use DatePickerRoot as DatePicker;

/// Props for [`DatePickerTrigger`].
#[derive(Clone, PartialEq, Props)]
pub struct DatePickerTriggerProps {
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

/// Button that opens the `DatePicker` popover and participates in `DateField` arrow-key traversal.
#[component]
pub fn DatePickerTrigger(props: DatePickerTriggerProps) -> Element {
    let picker_rt = use_date_picker_context();
    let field_rt = try_use_date_field_context();
    let effective_disabled = props.disabled || picker_rt.snapshot().disabled;
    let dir = picker_rt.state.read().def().dir;

    let mut reg_field = field_rt.clone();
    let mut unreg_field = field_rt.clone();
    let mut drop_field = field_rt.clone();
    let focus_field = field_rt;
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
            if let Some(ref mut frt) = reg_field {
                frt.register_trigger_id(tid);
            }
        },
        move |tid| {
            if let Some(ref mut frt) = unreg_field {
                frt.unregister_trigger_id(tid);
            }
        },
        move |tid| {
            if let Some(ref mut frt) = drop_field {
                frt.unregister_trigger_id(tid);
            }
        },
        move || {
            if let Some(ref frt) = focus_field {
                frt.focus_last_segment();
            }
        },
    )
}

/// Props for floating [`DatePickerContent`].
#[derive(Clone, PartialEq, Props)]
pub struct DatePickerContentProps {
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

/// Props for static (non-floating) [`DatePickerContentStatic`].
#[derive(Clone, PartialEq, Props)]
pub struct DatePickerContentStaticProps {
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

/// Floating popover dialog content for `DatePicker`.
#[component]
pub fn DatePickerContent(props: DatePickerContentProps) -> Element {
    let picker_rt = super::runtime::try_use_date_picker_context();
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
            default_open_focus_selector: DATE_PICKER_OPEN_FOCUS_SELECTOR,
            extra_data_attr: "data-date-picker-content",
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

/// Static (non-floating) popover dialog content for `DatePicker`.
#[component]
pub fn DatePickerContentStatic(props: DatePickerContentStaticProps) -> Element {
    let picker_rt = super::runtime::try_use_date_picker_context();
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
            default_open_focus_selector: DATE_PICKER_OPEN_FOCUS_SELECTOR,
            extra_data_attr: "data-date-picker-content",
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

/// Props for [`DatePickerCalendar`].
#[derive(Clone, PartialEq, Props)]
pub struct DatePickerCalendarProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
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

/// Calendar view synchronized with its ancestor `DatePickerRoot`.
///
/// Calls plain-fn [`render_calendar_root`] directly (zero nested `#[component]` taking
/// `Option<Callback|EventHandler>` per `Dioxus #5894`).
#[component]
pub fn DatePickerCalendar(props: DatePickerCalendarProps) -> Element {
    let picker_rt = use_date_picker_context();
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

    let DatePickerCalendarConfig {
        config: cal_cfg,
        initial_focus,
    } = cfg;

    render_calendar_root(CalendarRootProps {
        id: Some(cal_id),
        class: props.class,
        style: props.style,
        mode: CalendarMode::Single,
        value: Some((picker_rt.calendar_value)()),
        default_value: None,
        on_value_change: Some(EventHandler::new(move |next| {
            rt_for_val.handle_calendar_value_change(next);
        })),
        on_value_change_details: None,
        values: None,
        default_values: None,
        on_values_change: None,
        on_values_change_details: None,
        on_date_select: Some(EventHandler::new(move |date| {
            rt_for_select.handle_calendar_date_select(date);
        })),
        max_days: None,
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
