use std::{cell::RefCell, collections::HashMap, ops::{Deref, DerefMut}, rc::Rc};

use dioxus::prelude::keyboard_types::{Key, NamedKey};
use dioxus::prelude::*;

use crate::foundation::{
    browser::{FocusOptions, focus_element_by_id, focus_element_sync_or_defer, local_today},
    date_time::{
        CalendarConfig, CalendarDate, CalendarLocale, CalendarMonth, CalendarNavKey, MonthFormat,
        YearFormat, build_visible_months, default_placeholder_date, default_year_range,
        ensure_non_disabled_placeholder, is_next_page_disabled, is_prev_page_disabled,
        resolve_grid_navigation, resolve_tabbable_date, visible_weekday_headers,
    },
    state::sync_signal,
};

use super::{
    attrs::{
        CalendarCellAttrs, CalendarDayAttrs, CalendarGridAttrs, CalendarHeaderAttrs,
        CalendarHeadingAttrs, CalendarNavButtonAttrs, CalendarRootAttrs, CalendarSelectAttrs,
        CalendarSubpartAttrs,
    },
    relationships::CalendarRelationships,
    state::{CalendarDefinition, apply_multiple_click, apply_single_click},
    types::{
        CalendarCellSnapshot, CalendarKind, CalendarMode, CalendarPart, CalendarSelectOption,
        CalendarSnapshot, build_calendar_month_options, build_calendar_year_options,
    },
};

/// Translates Dioxus `keyboard_types::Key` + Shift modifier into host-neutral [`CalendarNavKey`].
pub fn translate_calendar_nav_key(key: &Key, shift: bool) -> Option<CalendarNavKey> {
    match key {
        Key::Named(NamedKey::ArrowLeft) => Some(CalendarNavKey::Left),
        Key::Named(NamedKey::ArrowRight) => Some(CalendarNavKey::Right),
        Key::Named(NamedKey::ArrowUp) => Some(CalendarNavKey::Up),
        Key::Named(NamedKey::ArrowDown) => Some(CalendarNavKey::Down),
        Key::Named(NamedKey::PageUp) => Some(if shift {
            CalendarNavKey::ShiftPageUp
        } else {
            CalendarNavKey::PageUp
        }),
        Key::Named(NamedKey::PageDown) => Some(if shift {
            CalendarNavKey::ShiftPageDown
        } else {
            CalendarNavKey::PageDown
        }),
        Key::Named(NamedKey::Home) => Some(CalendarNavKey::Home),
        Key::Named(NamedKey::End) => Some(CalendarNavKey::End),
        _ => None,
    }
}

/// Returns `true` when `key` activates a focused calendar day cell (`Enter` or `Space`).
pub fn is_calendar_activate_key(key: &Key) -> bool {
    matches!(key, Key::Named(NamedKey::Enter))
        || matches!(key, Key::Character(ch) if ch == " ")
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CalendarViewConfig {
    pub week_starts_on: crate::foundation::date_time::Weekday,
    pub fixed_weeks: bool,
    pub number_of_months: usize,
}

/// Shared base runtime embedded in both [`CalendarRuntime`] and
/// [`crate::range_calendar::RangeCalendarRuntime`], eliminating the 2-arm dispatch enum.
#[derive(Clone)]
pub struct CalendarBaseRuntime {
    pub kind: CalendarKind,
    pub config: Signal<CalendarConfig>,
    pub rels: CalendarRelationships,
    pub root_dom_id: Signal<String>,
    pub placeholder: Signal<CalendarDate>,
    pub months: Signal<Vec<CalendarMonth>>,
    pub announcement: Signal<String>,
    pub today_date: Signal<CalendarDate>,
    pub initial_placeholder_year: i32,
    pub is_date_disabled_cb: CopyValue<Option<Callback<CalendarDate, bool>>>,
    pub is_date_unavailable_cb: CopyValue<Option<Callback<CalendarDate, bool>>>,
    pub predicate_rev: Signal<u64>,
    pub on_placeholder_change: CopyValue<Option<EventHandler<CalendarDate>>>,
    pub day_id_overrides: Rc<RefCell<HashMap<(usize, CalendarDate), String>>>,
    pub(crate) pending_focus_target: Rc<RefCell<Option<(usize, CalendarDate)>>>,
    pub(crate) pending_focus_rev: Signal<u64>,
    pub(crate) last_view_config: CopyValue<CalendarViewConfig>,
}

/// Hook initializing a [`CalendarBaseRuntime`] with synchronous frame-1 SSR month grid &
/// non-disabled placeholder resolution and guarded `use_effect` prop reconciliation.
#[allow(clippy::too_many_arguments)]
pub(crate) fn use_calendar_base_runtime(
    kind: CalendarKind,
    rels: CalendarRelationships,
    dom_root_id: String,
    config: CalendarConfig,
    placeholder_prop: Option<Signal<CalendarDate>>,
    default_placeholder: Option<CalendarDate>,
    initial_default_from_value: Option<CalendarDate>,
    initial_focus: bool,
    is_date_disabled: Option<Callback<CalendarDate, bool>>,
    is_date_unavailable: Option<Callback<CalendarDate, bool>>,
    on_placeholder_change: Option<EventHandler<CalendarDate>>,
) -> CalendarBaseRuntime {
    let today = use_hook(local_today);
    let today_sig = use_signal(|| today);

    let effective_week_start = config.effective_week_starts_on();
    let view_config = CalendarViewConfig {
        week_starts_on: effective_week_start,
        fixed_weeks: config.fixed_weeks,
        number_of_months: config.number_of_months,
    };

    // Compute frame-1 placeholder and month grid synchronously so single-pass SSR renders
    // the exact month grid and `tabindex="0"` day cell with zero render-time signal writes
    // and zero per-render `Vec<CalendarMonth>` cloning.
    let (initial_placeholder, months_sig) = use_hook(|| {
        let raw_ph = if let Some(ext_ph) = placeholder_prop {
            *ext_ph.peek()
        } else if let Some(def_ph) = default_placeholder {
            def_ph.clamp(config.min_value, config.max_value)
        } else {
            default_placeholder_date(
                initial_default_from_value,
                config.min_value,
                config.max_value,
                today,
            )
        };
        let months = build_visible_months(
            raw_ph,
            config.number_of_months,
            effective_week_start,
            config.fixed_weeks,
        );
        let resolved_ph = if placeholder_prop.is_none() {
            ensure_non_disabled_placeholder(raw_ph, &months, |d| {
                let custom = is_date_disabled.is_some_and(|cb| cb.call(d));
                config.is_date_disabled_with(d, custom)
            })
        } else {
            raw_ph
        };
        (resolved_ph, Signal::new(months))
    });

    let fallback_placeholder = use_signal(|| initial_placeholder);
    let placeholder_sig = placeholder_prop.unwrap_or(fallback_placeholder);
    let initial_placeholder_year = initial_placeholder.year;

    let config_sig = use_signal(|| config.clone());
    let root_dom_id_sig = use_signal(|| dom_root_id.clone());
    let announcement_sig = use_signal(String::new);

    let mut is_date_disabled_cb = use_hook(|| CopyValue::new(is_date_disabled));
    let mut is_date_unavailable_cb = use_hook(|| CopyValue::new(is_date_unavailable));
    let mut on_placeholder_change_cb = use_hook(|| CopyValue::new(on_placeholder_change));
    is_date_disabled_cb.set(is_date_disabled);
    is_date_unavailable_cb.set(is_date_unavailable);
    on_placeholder_change_cb.set(on_placeholder_change);
    let mut predicate_rev = use_signal(|| 0_u64);
    let has_pred = is_date_disabled.is_some() || is_date_unavailable.is_some();
    let mut pred_initialized = use_hook(|| CopyValue::new(false));
    let mut prev_had_pred = use_hook(|| CopyValue::new(has_pred));
    // Dioxus 0.8 `Props::memoize` updates `Some(cb1) -> Some(cb2)` in place via
    // `Callback::__point_to`, preserving `GenerationalBox` pointer equality. Bump
    // `predicate_rev` on post-mount re-renders whenever a predicate is (or was)
    // present; `CalendarRoot`/`RangeCalendarRoot` never subscribe to `predicate_rev`,
    // so only cells/days and range reconciliation re-run (zero render loop).
    if pred_initialized.try_read().is_ok_and(|init| *init) {
        let had_pred = prev_had_pred.try_read().is_ok_and(|had| *had);
        prev_had_pred.set(has_pred);
        if has_pred || had_pred {
            let next_rev = predicate_rev.try_peek().ok().map(|cur| cur.wrapping_add(1));
            if let Some(next_rev) = next_rev {
                sync_signal(&mut predicate_rev, next_rev);
            }
        }
    } else {
        pred_initialized.set(true);
    }

    let day_id_overrides = use_hook(|| Rc::new(RefCell::new(HashMap::new())));
    let pending_focus_target = use_hook(|| Rc::new(RefCell::new(None)));
    let pending_focus_rev = use_signal(|| 0_u64);
    let last_view_config = use_hook(|| CopyValue::new(view_config.clone()));

    let base = CalendarBaseRuntime {
        kind,
        config: config_sig,
        rels,
        root_dom_id: root_dom_id_sig,
        placeholder: placeholder_sig,
        months: months_sig,
        announcement: announcement_sig,
        today_date: today_sig,
        initial_placeholder_year,
        is_date_disabled_cb,
        is_date_unavailable_cb,
        predicate_rev,
        on_placeholder_change: on_placeholder_change_cb,
        day_id_overrides,
        pending_focus_target,
        pending_focus_rev,
        last_view_config,
    };

    // Guarded `use_effect` prop reconciliation (frame-1 values are already synchronized above).
    {
        let mut rt = base.clone();
        use_effect(use_reactive(
            (&config, &dom_root_id, &view_config),
            move |(next_cfg, next_dom_id, next_view_cfg)| {
                sync_signal(&mut rt.config, next_cfg);
                sync_signal(&mut rt.root_dom_id, next_dom_id);
                let view_changed = rt
                    .last_view_config
                    .try_read()
                    .is_ok_and(|cur| *cur != next_view_cfg);
                if view_changed {
                    rt.last_view_config.set(next_view_cfg);
                    let anchor = rt
                        .months
                        .peek()
                        .first()
                        .map_or(*rt.placeholder.peek(), |m| m.value);
                    rt.rebuild_months_from(anchor);
                }
            },
        ));
    }

    // Focus tabbable day cell on initial mount when `initial_focus = true`.
    let did_initial_focus = use_hook(|| CopyValue::new(false));
    {
        let rt = base.clone();
        let disabled = config.disabled;
        let mut did_focus = did_initial_focus;
        use_effect(move || {
            if initial_focus && !disabled && !*did_focus.read() {
                did_focus.set(true);
                let target_date = rt.tabbable_date();
                let month_idx = rt.month_index_for(target_date);
                let day_id = rt.resolve_day_id(month_idx, target_date);
                focus_element_by_id(&day_id);
            }
        });
    }

    // Focus target day cell after cross-month keyboard paging mounts the new month's cells.
    {
        let rt = base.clone();
        use_effect(move || {
            let _ = (rt.pending_focus_rev)();
            if let Some((month_idx, target_date)) = rt.pending_focus_target.borrow_mut().take() {
                let target_id = rt.resolve_day_id(month_idx, target_date);
                focus_element_sync_or_defer(&target_id, FocusOptions::new(true));
            }
        });
    }

    base
}

impl CalendarBaseRuntime {
    pub const fn kind(&self) -> CalendarKind {
        self.kind
    }

    pub fn disabled(&self) -> bool {
        self.config.read().disabled
    }

    pub fn read_only(&self) -> bool {
        self.config.read().read_only
    }

    pub fn locale(&self) -> CalendarLocale {
        self.config.read().locale.clone()
    }

    pub fn current_placeholder(&self) -> CalendarDate {
        (self.placeholder)()
    }

    pub fn current_month(&self) -> u8 {
        self.current_placeholder().month
    }

    pub fn current_year(&self) -> i32 {
        self.current_placeholder().year
    }

    pub fn current_months(&self) -> Vec<CalendarMonth> {
        (self.months)()
    }

    pub fn month_index_for(&self, month: CalendarDate) -> usize {
        self.months
            .read()
            .iter()
            .position(|m| m.value.is_same_month(month))
            .unwrap_or(0)
    }

    pub fn register_day_id(&self, month_idx: usize, date: CalendarDate, id: String) {
        self.day_id_overrides
            .borrow_mut()
            .insert((month_idx, date), id);
    }

    pub fn unregister_day_id(&self, month_idx: usize, date: CalendarDate) {
        self.day_id_overrides.borrow_mut().remove(&(month_idx, date));
    }

    pub fn resolve_day_id(&self, month_idx: usize, date: CalendarDate) -> String {
        self.day_id_overrides
            .borrow()
            .get(&(month_idx, date))
            .cloned()
            .unwrap_or_else(|| self.rels.day_id(month_idx, date))
    }

    pub fn predicate_rev(&self) -> u64 {
        *self.predicate_rev.read()
    }

    pub fn is_date_disabled(&self, date: CalendarDate) -> bool {
        let cfg = self.config.read();
        let custom = self
            .is_date_disabled_cb
            .try_read()
            .ok()
            .and_then(|slot| *slot)
            .is_some_and(|cb| cb.call(date));
        cfg.is_date_disabled_with(date, custom)
    }

    pub fn is_date_unavailable(&self, date: CalendarDate) -> bool {
        self.is_date_unavailable_cb
            .try_read()
            .ok()
            .and_then(|slot| *slot)
            .is_some_and(|cb| cb.call(date))
    }

    pub fn is_outside_visible_months(&self, date: CalendarDate) -> bool {
        !self
            .months
            .read()
            .iter()
            .any(|month| month.value.is_same_month(date))
    }

    pub fn tabbable_date(&self) -> CalendarDate {
        let placeholder = self.current_placeholder();
        let months = self.current_months();
        resolve_tabbable_date(placeholder, &months, |d| self.is_date_disabled(d))
    }

    pub fn ensure_valid_initial_placeholder(&mut self) {
        let placeholder = *self.placeholder.peek();
        let months = self.months.peek().clone();
        let next = ensure_non_disabled_placeholder(placeholder, &months, |d| {
            self.is_date_disabled(d)
        });
        if next != placeholder {
            self.set_placeholder_internal(next, false);
        }
    }

    pub fn heading_value(&self) -> String {
        let cfg = self.config.read();
        let month_dates: Vec<CalendarDate> = self.months.read().iter().map(|m| m.value).collect();
        cfg.locale
            .heading_label(&month_dates, cfg.month_format, cfg.year_format)
    }

    pub fn full_calendar_label(&self) -> String {
        let label = self.config.read().calendar_label.clone();
        format!("{label} {}", self.heading_value())
    }

    pub fn weekdays(&self) -> Vec<String> {
        let cfg = self.config.read();
        let months = self.months.read();
        if let Some(first) = months.first() {
            visible_weekday_headers(first, &cfg.locale, cfg.weekday_format)
        } else {
            Vec::new()
        }
    }

    pub fn default_years(&self) -> Vec<i32> {
        let cfg = self.config.read();
        let today = *self.today_date.read();
        default_year_range(
            self.initial_placeholder_year,
            today.year,
            cfg.min_value,
            cfg.max_value,
        )
    }

    pub fn is_prev_disabled(&self) -> bool {
        let cfg = self.config.read();
        let months = self.months.read();
        is_prev_page_disabled(&months, cfg.min_value, cfg.disabled)
    }

    pub fn is_next_disabled(&self) -> bool {
        let cfg = self.config.read();
        let months = self.months.read();
        is_next_page_disabled(&months, cfg.max_value, cfg.disabled)
    }

    pub fn rebuild_months_from(&mut self, start_month: CalendarDate) {
        let cfg = self.config.read().clone();
        let next = build_visible_months(
            start_month,
            cfg.number_of_months,
            cfg.effective_week_starts_on(),
            cfg.fixed_weeks,
        );
        self.months.set(next);
    }

    pub fn set_placeholder(&mut self, next: CalendarDate) {
        self.set_placeholder_internal(next, true);
    }

    pub(crate) fn set_placeholder_internal(
        &mut self,
        next: CalendarDate,
        rebuild_if_outside: bool,
    ) {
        let prev = *self.placeholder.peek();
        if prev != next {
            self.placeholder.set(next);
            if let Some(handler) = self
                .on_placeholder_change
                .try_read()
                .ok()
                .and_then(|slot| *slot)
            {
                handler.call(next);
            }
        }
        if rebuild_if_outside {
            let in_view = self
                .months
                .peek()
                .iter()
                .any(|m| m.value.is_same_month(next));
            if !in_view {
                self.rebuild_months_from(next);
            }
        }
    }

    /// Unified paging transition: `sign < 0` pages backward, `sign > 0` pages forward.
    pub fn page_by(&mut self, sign: i32) {
        if sign == 0
            || (sign < 0 && self.is_prev_disabled())
            || (sign > 0 && self.is_next_disabled())
        {
            return;
        }
        let cfg = self.config.read().clone();
        let step = if cfg.paged_navigation {
            cfg.number_of_months.max(1) as i32
        } else {
            1
        };
        let first = self
            .months
            .peek()
            .first()
            .map_or(*self.placeholder.peek(), |m| m.value);
        let new_first = first.add_months_clamped(step * sign.signum());
        self.rebuild_months_from(new_first);
        self.set_placeholder_internal(new_first, false);
    }

    pub fn prev_page(&mut self) {
        self.page_by(-1);
    }

    pub fn next_page(&mut self) {
        self.page_by(1);
    }

    pub fn set_month(&mut self, month: u8) {
        if self.config.read().disabled {
            return;
        }
        let next = self.placeholder.peek().with_month_clamped(month);
        self.set_placeholder(next);
    }

    pub fn set_year(&mut self, year: i32) {
        if self.config.read().disabled {
            return;
        }
        let next = self.placeholder.peek().with_year_clamped(year);
        self.set_placeholder(next);
    }

    pub fn handle_nav_key_target(
        &mut self,
        current_date: CalendarDate,
        key: CalendarNavKey,
    ) -> Option<CalendarDate> {
        let cfg = self.config.read().clone();
        let months = self.months.peek().clone();
        let nav = resolve_grid_navigation(
            current_date,
            &months,
            cfg.number_of_months,
            cfg.effective_week_starts_on(),
            cfg.dir,
            key,
            cfg.min_value,
            cfg.max_value,
            cfg.disabled,
            cfg.disable_days_outside_month,
            |d| self.is_date_disabled(d),
        )?;

        if nav.paged
            && let Some(view_start) = nav.next_view_start
        {
            self.rebuild_months_from(view_start);
        }
        self.set_placeholder_internal(nav.next_placeholder, !nav.paged);

        let target_month_idx = self.month_index_for(nav.target_date);
        if nav.paged {
            *self.pending_focus_target.borrow_mut() = Some((target_month_idx, nav.target_date));
            let next_rev = self.pending_focus_rev.peek().wrapping_add(1);
            self.pending_focus_rev.set(next_rev);
        } else {
            let target_id = self.resolve_day_id(target_month_idx, nav.target_date);
            focus_element_sync_or_defer(&target_id, FocusOptions::new(true));
        }
        Some(nav.target_date)
    }

    pub fn root_attrs_with_invalid(
        &self,
        is_invalid: bool,
        custom_id: Option<String>,
    ) -> CalendarRootAttrs {
        let cfg = self.config.read();
        let id = custom_id.unwrap_or_else(|| (self.root_dom_id)());
        CalendarRootAttrs::new(
            id,
            self.kind,
            self.full_calendar_label(),
            cfg.dir,
            is_invalid,
            cfg.disabled,
            cfg.read_only,
        )
    }

    pub fn header_attrs(&self, custom_id: Option<String>) -> CalendarHeaderAttrs {
        let id = custom_id.unwrap_or_else(|| self.rels.header_id().to_string());
        CalendarHeaderAttrs::new(id, self.kind, self.disabled(), self.read_only())
    }

    pub fn heading_attrs(&self, custom_id: Option<String>) -> CalendarHeadingAttrs {
        let id = custom_id.unwrap_or_else(|| self.rels.heading_id().to_string());
        CalendarHeadingAttrs::new(id, self.kind, self.disabled(), self.read_only())
    }

    pub fn prev_button_attrs(&self, custom_id: Option<String>) -> CalendarNavButtonAttrs {
        let id = custom_id.unwrap_or_else(|| self.rels.prev_button_id().to_string());
        CalendarNavButtonAttrs::prev(id, self.kind, self.is_prev_disabled())
    }

    pub fn next_button_attrs(&self, custom_id: Option<String>) -> CalendarNavButtonAttrs {
        let id = custom_id.unwrap_or_else(|| self.rels.next_button_id().to_string());
        CalendarNavButtonAttrs::next(id, self.kind, self.is_next_disabled())
    }

    pub fn grid_attrs(&self, month_idx: usize, custom_id: Option<String>) -> CalendarGridAttrs {
        let id = custom_id.unwrap_or_else(|| self.rels.grid_id(month_idx));
        CalendarGridAttrs::new(id, self.kind, self.disabled(), self.read_only())
    }

    pub fn subpart_attrs(
        &self,
        part: CalendarPart,
        default_id: String,
        custom_id: Option<String>,
    ) -> CalendarSubpartAttrs {
        CalendarSubpartAttrs::new(
            custom_id.unwrap_or(default_id),
            self.kind,
            part,
            self.disabled(),
            self.read_only(),
        )
    }

    pub fn month_select_attrs(
        &self,
        custom_id: Option<String>,
        aria_label: Option<String>,
        disabled: bool,
    ) -> CalendarSelectAttrs {
        let id = custom_id.unwrap_or_else(|| self.rels.month_select_id().to_string());
        CalendarSelectAttrs::month(id, self.kind, aria_label, self.disabled() || disabled)
    }

    pub fn year_select_attrs(
        &self,
        custom_id: Option<String>,
        aria_label: Option<String>,
        disabled: bool,
    ) -> CalendarSelectAttrs {
        let id = custom_id.unwrap_or_else(|| self.rels.year_select_id().to_string());
        CalendarSelectAttrs::year(id, self.kind, aria_label, self.disabled() || disabled)
    }

    pub fn month_options(&self, format: MonthFormat) -> Vec<CalendarSelectOption> {
        let locale = &self.config.read().locale;
        build_calendar_month_options(None, locale, format)
    }

    pub fn year_options(&self, format: YearFormat) -> Vec<CalendarSelectOption> {
        let years = self.default_years();
        let locale = &self.config.read().locale;
        build_calendar_year_options(&years, locale, format)
    }
}

/// Reactive runtime for `Calendar`.
#[derive(Clone)]
pub struct CalendarRuntime {
    pub base: CalendarBaseRuntime,
    pub def: Signal<CalendarDefinition>,
    pub value: Signal<Option<CalendarDate>>,
    pub values: Signal<Vec<CalendarDate>>,
    pub on_value_change: CopyValue<Option<EventHandler<Option<CalendarDate>>>>,
    pub on_values_change: CopyValue<Option<EventHandler<Vec<CalendarDate>>>>,
    pub on_date_select: CopyValue<Option<EventHandler<CalendarDate>>>,
}

impl Deref for CalendarRuntime {
    type Target = CalendarBaseRuntime;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for CalendarRuntime {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl CalendarRuntime {
    pub fn is_date_selected(&self, date: CalendarDate) -> bool {
        let mode = self.def.read().mode;
        match mode {
            CalendarMode::Single => self.value.read().is_some_and(|d| d == date),
            CalendarMode::Multiple => self.values.read().contains(&date),
        }
    }

    pub fn is_invalid(&self) -> bool {
        let mode = self.def.read().mode;
        match mode {
            CalendarMode::Single => self
                .value
                .read()
                .is_some_and(|d| self.is_date_disabled(d) || self.is_date_unavailable(d)),
            CalendarMode::Multiple => self
                .values
                .read()
                .iter()
                .any(|d| self.is_date_disabled(*d) || self.is_date_unavailable(*d)),
        }
    }

    pub fn snapshot(&self) -> CalendarSnapshot {
        CalendarSnapshot {
            placeholder: self.current_placeholder(),
            months: self.current_months(),
            weekdays: self.weekdays(),
            heading: self.heading_value(),
            full_label: self.full_calendar_label(),
            is_invalid: self.is_invalid(),
            is_prev_disabled: self.is_prev_disabled(),
            is_next_disabled: self.is_next_disabled(),
            tabbable_date: self.tabbable_date(),
        }
    }

    pub fn activate_date(&mut self, date: CalendarDate) {
        let def = self.def.read().clone();
        let is_disabled = self.is_date_disabled(date);
        let is_unavailable = self.is_date_unavailable(date);

        match def.mode {
            CalendarMode::Single => {
                let prev = *self.value.peek();
                let Some(update) = apply_single_click(
                    prev,
                    date,
                    def.prevent_deselect,
                    def.read_only,
                    is_disabled,
                    is_unavailable,
                    &def.locale,
                ) else {
                    return;
                };
                if *self.value.peek() != update.next_value {
                    self.value.set(update.next_value);
                    if let Some(handler) = self
                        .on_value_change
                        .try_read()
                        .ok()
                        .and_then(|slot| *slot)
                    {
                        handler.call(update.next_value);
                    }
                }
                if let Some(next_ph) = update.next_placeholder {
                    self.set_placeholder(next_ph);
                }
                self.announcement.set(update.announcement);
                if let Some(selected) = update.date_selected
                    && let Some(handler) = self
                        .on_date_select
                        .try_read()
                        .ok()
                        .and_then(|slot| *slot)
                {
                    handler.call(selected);
                }
            }
            CalendarMode::Multiple => {
                let prev = self.values.peek().clone();
                let Some(update) = apply_multiple_click(
                    &prev,
                    date,
                    def.prevent_deselect,
                    def.max_days,
                    def.read_only,
                    is_disabled,
                    is_unavailable,
                ) else {
                    return;
                };
                if *self.values.peek() != update.next_values {
                    self.values.set(update.next_values.clone());
                    if let Some(handler) = self
                        .on_values_change
                        .try_read()
                        .ok()
                        .and_then(|slot| *slot)
                    {
                        handler.call(update.next_values);
                    }
                }
                if let Some(next_ph) = update.next_placeholder {
                    self.set_placeholder(next_ph);
                }
            }
        }
    }

    pub fn handle_nav_key(&mut self, current_date: CalendarDate, key: CalendarNavKey) -> bool {
        self.base.handle_nav_key_target(current_date, key).is_some()
    }

    pub fn root_attrs(&self, custom_id: Option<String>) -> CalendarRootAttrs {
        self.base.root_attrs_with_invalid(self.is_invalid(), custom_id)
    }

    pub fn cell_attrs(
        &self,
        date: CalendarDate,
        month: CalendarDate,
        custom_id: Option<String>,
    ) -> CalendarCellAttrs {
        let cfg = self.config.read();
        let month_idx = self.month_index_for(month);
        let id = custom_id.unwrap_or_else(|| self.rels.cell_id(month_idx, date));
        let is_outside_month = !date.is_same_month(month);
        CalendarCellAttrs::new(
            id,
            date,
            self.is_date_selected(date),
            self.is_date_disabled(date),
            self.is_date_unavailable(date),
            date == *self.today_date.read(),
            is_outside_month,
            self.is_outside_visible_months(date),
            date == self.current_placeholder(),
            cfg.disable_days_outside_month,
        )
    }

    pub fn day_attrs(
        &self,
        date: CalendarDate,
        month: CalendarDate,
        cell_attrs: &CalendarCellAttrs,
        custom_id: Option<String>,
    ) -> CalendarDayAttrs {
        let cfg = self.config.read();
        let month_idx = self.month_index_for(month);
        let id = custom_id.unwrap_or_else(|| self.rels.day_id(month_idx, date));
        let is_tabbable = date == self.tabbable_date();
        CalendarDayAttrs::from_cell(id, date, &cfg.locale, cell_attrs, is_tabbable)
    }

    pub fn cell_snapshot(
        &self,
        date: CalendarDate,
        month: CalendarDate,
        cell_attrs: &CalendarCellAttrs,
    ) -> CalendarCellSnapshot {
        CalendarCellSnapshot {
            date,
            month,
            day_text: date.day.to_string(),
            is_disabled: cell_attrs.data_disabled.is_some(),
            is_unavailable: cell_attrs.data_unavailable.is_some(),
            is_selected: cell_attrs.data_selected.is_some(),
            is_today: cell_attrs.data_today.is_some(),
            is_outside_month: cell_attrs.data_outside_month.is_some(),
            is_outside_visible_months: cell_attrs.data_outside_visible_months.is_some(),
            is_focused: cell_attrs.data_focused.is_some(),
        }
    }
}

#[derive(Clone)]
pub struct CalendarContext {
    pub runtime: CalendarRuntime,
}

impl CalendarContext {
    pub fn snapshot(&self) -> CalendarSnapshot {
        self.runtime.snapshot()
    }

    pub fn months(&self) -> Vec<CalendarMonth> {
        self.runtime.current_months()
    }

    pub fn weekdays(&self) -> Vec<String> {
        self.runtime.weekdays()
    }

    pub fn heading(&self) -> String {
        self.runtime.heading_value()
    }

    pub fn prev_page(&mut self) {
        self.runtime.prev_page();
    }

    pub fn next_page(&mut self) {
        self.runtime.next_page();
    }

    pub fn current_month(&self) -> u8 {
        self.runtime.current_month()
    }

    pub fn current_year(&self) -> i32 {
        self.runtime.current_year()
    }

    pub fn set_month(&mut self, month: u8) {
        self.runtime.set_month(month);
    }

    pub fn set_year(&mut self, year: i32) {
        self.runtime.set_year(year);
    }

    pub fn default_years(&self) -> Vec<i32> {
        self.runtime.default_years()
    }

    pub fn month_options(&self, format: MonthFormat) -> Vec<CalendarSelectOption> {
        self.runtime.month_options(format)
    }

    pub fn year_options(&self, format: YearFormat) -> Vec<CalendarSelectOption> {
        self.runtime.year_options(format)
    }
}

pub fn use_calendar_context() -> CalendarContext {
    use_context::<CalendarContext>()
}

pub fn try_use_calendar_context() -> Option<CalendarContext> {
    try_consume_context::<CalendarContext>()
}

#[derive(Clone)]
pub struct CalendarSharedContext {
    pub runtime: CalendarBaseRuntime,
}

pub fn use_calendar_shared_context() -> CalendarSharedContext {
    use_context::<CalendarSharedContext>()
}
