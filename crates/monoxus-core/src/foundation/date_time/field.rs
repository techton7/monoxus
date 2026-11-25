use super::{
    date::{
        CalendarDate, CalendarDateTime, CalendarTime, DateValue, DayPeriod, Granularity, HourCycle,
        TimeGranularity, TimeValue, days_in_month, hour_12_display, hour_12_to_24,
    },
    locale::{FieldLocaleFormat, FieldLocaleTemplate, FieldTemplateToken},
};

/// Kind of segment rendered inside a segmented date or time field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SegmentKind {
    Year,
    Month,
    Day,
    Hour,
    Minute,
    Second,
    DayPeriod,
    TimeZoneName,
    Literal,
}

impl SegmentKind {
    /// Returns `true` for user-editable spinbutton segments.
    #[inline]
    pub const fn is_editable(self) -> bool {
        self.as_editable().is_some()
    }

    /// Returns `true` for focusable segments (editable segments plus `TimeZoneName`).
    #[inline]
    pub const fn is_interactive(self) -> bool {
        !matches!(self, Self::Literal)
    }

    /// Narrows this segment kind to [`EditableSegmentKind`] if editable.
    #[inline]
    pub const fn as_editable(self) -> Option<EditableSegmentKind> {
        match self {
            Self::Year => Some(EditableSegmentKind::Year),
            Self::Month => Some(EditableSegmentKind::Month),
            Self::Day => Some(EditableSegmentKind::Day),
            Self::Hour => Some(EditableSegmentKind::Hour),
            Self::Minute => Some(EditableSegmentKind::Minute),
            Self::Second => Some(EditableSegmentKind::Second),
            Self::DayPeriod => Some(EditableSegmentKind::DayPeriod),
            Self::TimeZoneName | Self::Literal => None,
        }
    }

    /// Returns the canonical `data-segment` attribute string matching `bits-ui` and Sub-wave 5C truth tables.
    #[inline]
    pub const fn as_data_segment(self) -> &'static str {
        match self {
            Self::Year => "year",
            Self::Month => "month",
            Self::Day => "day",
            Self::Hour => "hour",
            Self::Minute => "minute",
            Self::Second => "second",
            Self::DayPeriod => "dayPeriod",
            Self::TimeZoneName => "timeZoneName",
            Self::Literal => "literal",
        }
    }
}

/// Editable spinbutton segment kinds (usable in `readonly_segments` and edit-state indexing).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EditableSegmentKind {
    Year,
    Month,
    Day,
    Hour,
    Minute,
    Second,
    DayPeriod,
}

impl EditableSegmentKind {
    /// Returns the canonical `data-segment` attribute string.
    #[inline]
    pub const fn as_data_segment(self) -> &'static str {
        self.to_segment_kind().as_data_segment()
    }

    /// Widens to [`SegmentKind`].
    #[inline]
    pub const fn to_segment_kind(self) -> SegmentKind {
        match self {
            Self::Year => SegmentKind::Year,
            Self::Month => SegmentKind::Month,
            Self::Day => SegmentKind::Day,
            Self::Hour => SegmentKind::Hour,
            Self::Minute => SegmentKind::Minute,
            Self::Second => SegmentKind::Second,
            Self::DayPeriod => SegmentKind::DayPeriod,
        }
    }
}

impl From<EditableSegmentKind> for SegmentKind {
    #[inline]
    fn from(kind: EditableSegmentKind) -> Self {
        kind.to_segment_kind()
    }
}

/// A rendered segment ready for WAI-ARIA `spinbutton` / `literal` presentation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FieldSegment {
    pub kind: SegmentKind,
    pub value: String,
    pub is_placeholder: bool,
    pub min: Option<i32>,
    pub max: Option<i32>,
    pub now: Option<i32>,
    pub value_text: Option<String>,
}

/// Alias for [`FieldSegment`] matching Sub-wave 5C specification naming.
pub type RenderedSegment = FieldSegment;

/// Host-neutral user interaction intent targeting a spinbutton segment.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FieldSegmentIntent {
    Increment,
    Decrement,
    PageUp,
    PageDown,
    Min,
    Max,
    Digit(u8),
    DayPeriodKey(DayPeriod),
    Backspace,
    Delete,
    NextSegment,
    PrevSegment,
    Blur,
    Paste(String),
}

/// Requested focus traversal direction returned by [`SegmentEngine::apply_intent`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SegmentFocusMove {
    Next,
    Prev,
}

/// Per-segment digit-entry and focus-reentry buffer state.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SegmentEditState {
    pub last_key_zero: bool,
    pub has_left_focus: bool,
    pub year_pressed_digits: Vec<u8>,
    pub year_backspace_count: u8,
    pub year_post_backspace_mode: bool,
}

impl SegmentEditState {
    /// Resets transient digit-entry buffers while marking the segment as having left focus.
    pub fn mark_left_focus(&mut self) {
        self.last_key_zero = false;
        self.has_left_focus = true;
        self.year_pressed_digits.clear();
        self.year_backspace_count = 0;
        self.year_post_backspace_mode = false;
    }

    /// Resets transient digit-entry buffers for active editing.
    pub fn reset_active(&mut self) {
        self.last_key_zero = false;
        self.has_left_focus = false;
        self.year_pressed_digits.clear();
        self.year_backspace_count = 0;
        self.year_post_backspace_mode = false;
    }
}

/// Collection of [`SegmentEditState`]s across all editable segment kinds.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SegmentEditStates {
    pub year: SegmentEditState,
    pub month: SegmentEditState,
    pub day: SegmentEditState,
    pub hour: SegmentEditState,
    pub minute: SegmentEditState,
    pub second: SegmentEditState,
    pub day_period: SegmentEditState,
}

impl SegmentEditStates {
    /// Returns a shared reference to the edit state for `kind`.
    pub fn for_kind(&self, kind: EditableSegmentKind) -> &SegmentEditState {
        match kind {
            EditableSegmentKind::Year => &self.year,
            EditableSegmentKind::Month => &self.month,
            EditableSegmentKind::Day => &self.day,
            EditableSegmentKind::Hour => &self.hour,
            EditableSegmentKind::Minute => &self.minute,
            EditableSegmentKind::Second => &self.second,
            EditableSegmentKind::DayPeriod => &self.day_period,
        }
    }

    /// Returns a mutable reference to the edit state for `kind`.
    pub fn for_kind_mut(&mut self, kind: EditableSegmentKind) -> &mut SegmentEditState {
        match kind {
            EditableSegmentKind::Year => &mut self.year,
            EditableSegmentKind::Month => &mut self.month,
            EditableSegmentKind::Day => &mut self.day,
            EditableSegmentKind::Hour => &mut self.hour,
            EditableSegmentKind::Minute => &mut self.minute,
            EditableSegmentKind::Second => &mut self.second,
            EditableSegmentKind::DayPeriod => &mut self.day_period,
        }
    }

    /// Marks every editable segment as having left focus (overwrite-on-reentry).
    pub fn mark_all_left_focus(&mut self) {
        self.year.mark_left_focus();
        self.month.mark_left_focus();
        self.day.mark_left_focus();
        self.hour.mark_left_focus();
        self.minute.mark_left_focus();
        self.second.mark_left_focus();
        self.day_period.mark_left_focus();
    }
}

/// Raw buffered string values for each editable segment.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SegmentValues {
    pub year: Option<String>,
    pub month: Option<String>,
    pub day: Option<String>,
    pub hour: Option<String>,
    pub minute: Option<String>,
    pub second: Option<String>,
    pub day_period: Option<DayPeriod>,
}

impl SegmentValues {
    /// Clears all buffered segment values.
    pub fn clear(&mut self) {
        self.year = None;
        self.month = None;
        self.day = None;
        self.hour = None;
        self.minute = None;
        self.second = None;
        self.day_period = None;
    }
}

/// Output of applying a [`FieldSegmentIntent`] to a [`SegmentEngine`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SegmentTransitionResult {
    pub updated_segments: Vec<FieldSegment>,
    pub move_focus: Option<SegmentFocusMove>,
    pub committed_date_value: Option<DateValue>,
    pub committed_time_value: Option<TimeValue>,
    pub cleared_value: bool,
    pub announcement: Option<String>,
}

/// Validation failure reason emitted by segmented date/time fields and range fields.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FieldInvalidReason {
    Min,
    Max,
    RangeOrder,
    Unavailable,
    Custom(Vec<String>),
}

/// Payload delivered to `on_invalid` callbacks.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FieldInvalidEvent {
    pub reason: FieldInvalidReason,
    pub messages: Vec<String>,
}

/// Operating mode of a [`SegmentEngine`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FieldMode {
    Date {
        granularity: Granularity,
        include_time_zone: bool,
    },
    Time {
        granularity: TimeGranularity,
    },
}

/// Parses a single key string (`"a"`, `"A"`, `"p"`, `"P"`) into [`DayPeriod`].
///
/// Fixes `bits-ui`'s truthy key comparison bug (`5C D-4`).
#[inline]
pub fn parse_day_period_key(key: &str) -> Option<DayPeriod> {
    match key {
        "a" | "A" => Some(DayPeriod::Am),
        "p" | "P" => Some(DayPeriod::Pm),
        _ => None,
    }
}

/// Validates a single [`DateValue`] following Sub-wave 5C precedence:
/// `Custom` -> `Min` -> `Max`.
pub fn validate_date_value(
    value: Option<DateValue>,
    min: Option<DateValue>,
    max: Option<DateValue>,
    custom_errors: Option<Vec<String>>,
) -> Result<(), FieldInvalidReason> {
    let Some(val) = value else {
        return Ok(());
    };
    if let Some(msgs) = custom_errors
        && !msgs.is_empty()
    {
        return Err(FieldInvalidReason::Custom(msgs));
    }
    if let Some(min_val) = min
        && val.compare_chronological(&min_val).is_lt()
    {
        return Err(FieldInvalidReason::Min);
    }
    if let Some(max_val) = max
        && val.compare_chronological(&max_val).is_gt()
    {
        return Err(FieldInvalidReason::Max);
    }
    Ok(())
}

/// Validates a single [`TimeValue`] following Sub-wave 5C precedence:
/// `Custom` -> `Min` -> `Max`.
pub fn validate_time_value(
    value: Option<TimeValue>,
    min: Option<TimeValue>,
    max: Option<TimeValue>,
    custom_errors: Option<Vec<String>>,
) -> Result<(), FieldInvalidReason> {
    let Some(val) = value else {
        return Ok(());
    };
    if let Some(msgs) = custom_errors
        && !msgs.is_empty()
    {
        return Err(FieldInvalidReason::Custom(msgs));
    }
    if let Some(min_val) = min
        && val.time() < min_val.time()
    {
        return Err(FieldInvalidReason::Min);
    }
    if let Some(max_val) = max
        && val.time() > max_val.time()
    {
        return Err(FieldInvalidReason::Max);
    }
    Ok(())
}

/// Pure-Rust spinbutton segment state machine shared across `DateField`, `TimeField`,
/// `DateRangeField`, `TimeRangeField`, `DatePicker`, and `DateRangePicker`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SegmentEngine {
    mode: FieldMode,
    locale: FieldLocaleFormat,
    hour_cycle: Option<HourCycle>,
    placeholder_date: DateValue,
    placeholder_time: TimeValue,
    values: SegmentValues,
    edit_states: SegmentEditStates,
    day_period_hint: DayPeriod,
}

impl SegmentEngine {
    /// Creates a [`SegmentEngine`] configured for a date or date-time field.
    pub fn new_date(
        locale: FieldLocaleFormat,
        granularity: Granularity,
        hour_cycle: Option<HourCycle>,
        hide_time_zone: bool,
        placeholder: DateValue,
    ) -> Self {
        let placeholder_time = placeholder.time().unwrap_or_else(CalendarTime::midnight);
        let mut edit_states = SegmentEditStates::default();
        edit_states.mark_all_left_focus();
        Self {
            mode: FieldMode::Date {
                granularity,
                include_time_zone: !hide_time_zone && granularity.includes_time(),
            },
            locale,
            hour_cycle,
            placeholder_date: placeholder,
            placeholder_time: TimeValue::Time(placeholder_time),
            values: SegmentValues::default(),
            edit_states,
            day_period_hint: placeholder_time.day_period(),
        }
    }

    /// Creates a [`SegmentEngine`] configured for a time field.
    pub fn new_time(
        locale: FieldLocaleFormat,
        granularity: TimeGranularity,
        hour_cycle: Option<HourCycle>,
        placeholder: TimeValue,
    ) -> Self {
        let mut edit_states = SegmentEditStates::default();
        edit_states.mark_all_left_focus();
        Self {
            mode: FieldMode::Time { granularity },
            locale,
            hour_cycle,
            placeholder_date: DateValue::Date(
                placeholder.date().unwrap_or_else(CalendarDate::default),
            ),
            placeholder_time: placeholder,
            values: SegmentValues::default(),
            edit_states,
            day_period_hint: placeholder.time().day_period(),
        }
    }

    /// Returns the operating [`FieldMode`].
    #[inline]
    pub const fn mode(&self) -> FieldMode {
        self.mode
    }

    /// Returns the configured [`FieldLocaleFormat`].
    #[inline]
    pub fn locale(&self) -> &FieldLocaleFormat {
        &self.locale
    }

    /// Updates the locale format.
    pub fn set_locale(&mut self, locale: FieldLocaleFormat) {
        self.locale = locale;
    }

    /// Returns the effective [`HourCycle`] (explicit override or locale default).
    pub fn effective_hour_cycle(&self) -> HourCycle {
        self.hour_cycle
            .unwrap_or_else(|| self.locale.default_hour_cycle())
    }

    /// Updates the explicit [`HourCycle`] override, converting any populated hour segment.
    pub fn set_hour_cycle(&mut self, hour_cycle: Option<HourCycle>) {
        let prev_cycle = self.effective_hour_cycle();
        self.hour_cycle = hour_cycle;
        let next_cycle = self.effective_hour_cycle();
        if prev_cycle == next_cycle {
            return;
        }
        if let Some(hour_str) = &self.values.hour
            && let Ok(parsed) = hour_str.parse::<u8>()
        {
            match (prev_cycle, next_cycle) {
                (HourCycle::H24, HourCycle::H12) if parsed < 24 => {
                    let period = if parsed < 12 {
                        DayPeriod::Am
                    } else {
                        DayPeriod::Pm
                    };
                    self.values.hour = Some(format!("{:02}", hour_12_display(parsed)));
                    self.values.day_period = Some(period);
                    self.day_period_hint = period;
                }
                (HourCycle::H12, HourCycle::H24) => {
                    let period = self.values.day_period.unwrap_or(self.day_period_hint);
                    if let Some(h24) = hour_12_to_24(parsed, period) {
                        self.values.hour = Some(format!("{h24:02}"));
                    }
                }
                _ => {}
            }
        }
    }

    /// Updates the placeholder [`DateValue`].
    pub fn set_placeholder_date(&mut self, placeholder: DateValue) {
        self.placeholder_date = placeholder;
        if let Some(t) = placeholder.time() {
            self.placeholder_time = TimeValue::Time(t);
            if self.values.day_period.is_none() {
                self.day_period_hint = t.day_period();
            }
        }
    }

    /// Updates the placeholder [`TimeValue`].
    pub fn set_placeholder_time(&mut self, placeholder: TimeValue) {
        self.placeholder_time = placeholder;
        if self.values.day_period.is_none() {
            self.day_period_hint = placeholder.time().day_period();
        }
    }

    /// Returns a shared reference to the raw buffered segment values.
    #[inline]
    pub fn values(&self) -> &SegmentValues {
        &self.values
    }

    /// Returns the [`SegmentEditState`] for an editable segment.
    #[inline]
    pub fn edit_state(&self, kind: EditableSegmentKind) -> &SegmentEditState {
        self.edit_states.for_kind(kind)
    }

    /// Returns a mutable reference to the [`SegmentEditState`] for an editable segment.
    #[inline]
    pub fn edit_state_mut(&mut self, kind: EditableSegmentKind) -> &mut SegmentEditState {
        self.edit_states.for_kind_mut(kind)
    }

    /// Synchronizes all segment buffers from an external `Option<DateValue>`.
    pub fn sync_from_date_value(&mut self, value: Option<DateValue>) {
        match value {
            None => {
                self.values.clear();
            }
            Some(dv) => {
                let d = dv.date();
                self.values.year = Some(format!("{:04}", d.year.clamp(1, 9999)));
                self.values.month = Some(format!("{:02}", d.month));
                self.values.day = Some(format!("{:02}", d.day));
                if let Some(t) = dv.time() {
                    self.sync_time_components(t);
                } else if matches!(
                    self.mode,
                    FieldMode::Date {
                        granularity: Granularity::Day,
                        ..
                    }
                ) {
                    self.values.hour = None;
                    self.values.minute = None;
                    self.values.second = None;
                    self.values.day_period = None;
                } else {
                    self.sync_time_components(CalendarTime::midnight());
                }
            }
        }
        self.edit_states.mark_all_left_focus();
    }

    /// Synchronizes all time segment buffers from an external `Option<TimeValue>`.
    pub fn sync_from_time_value(&mut self, value: Option<TimeValue>) {
        match value {
            None => {
                self.values.hour = None;
                self.values.minute = None;
                self.values.second = None;
                self.values.day_period = None;
            }
            Some(tv) => {
                self.sync_time_components(tv.time());
            }
        }
        self.edit_states.mark_all_left_focus();
    }

    fn sync_time_components(&mut self, t: CalendarTime) {
        let cycle = self.effective_hour_cycle();
        match cycle {
            HourCycle::H12 => {
                let (h12, period) = t.hour_12();
                self.values.hour = Some(format!("{h12:02}"));
                self.values.day_period = Some(period);
                self.day_period_hint = period;
            }
            HourCycle::H24 => {
                self.values.hour = Some(format!("{:02}", t.hour));
                self.values.day_period = Some(t.day_period());
                self.day_period_hint = t.day_period();
            }
        }
        self.values.minute = Some(format!("{:02}", t.minute));
        self.values.second = Some(format!("{:02}", t.second));
    }

    /// Returns the committed [`DateValue`] when in date mode and all rendered editable segments are filled.
    pub fn to_date_value(&self) -> Option<DateValue> {
        let FieldMode::Date { granularity, .. } = self.mode else {
            return None;
        };
        // Canonical commit order: year, month, day, hour, minute, second, dayPeriod
        let year = self.parse_valid_year()?;
        let month = self.parse_valid_month()?;
        let day = self.parse_valid_day(year, month)?;
        let date = CalendarDate::new(year, month, day)?;

        let Some(time_granularity) = granularity.to_time_granularity() else {
            return Some(DateValue::Date(date));
        };
        let time = self.parse_valid_time(time_granularity)?;
        Some(DateValue::DateTime(CalendarDateTime::new(date, time)))
    }

    /// Returns the committed [`TimeValue`] when in time mode and all rendered editable segments are filled.
    pub fn to_time_value(&self) -> Option<TimeValue> {
        let FieldMode::Time { granularity } = self.mode else {
            return None;
        };
        let time = self.parse_valid_time(granularity)?;
        Some(match self.placeholder_time {
            TimeValue::DateTime(dt) => TimeValue::DateTime(dt.with_time(time)),
            TimeValue::Time(_) => TimeValue::Time(time),
        })
    }

    fn parse_valid_year(&self) -> Option<i32> {
        let raw = self.values.year.as_deref()?;
        let year: i32 = raw.parse().ok()?;
        (1..=9999).contains(&year).then_some(year)
    }

    fn parse_valid_month(&self) -> Option<u8> {
        let raw = self.values.month.as_deref()?;
        let month: u8 = raw.parse().ok()?;
        (1..=12).contains(&month).then_some(month)
    }

    fn parse_valid_day(&self, year: i32, month: u8) -> Option<u8> {
        let raw = self.values.day.as_deref()?;
        let day: u8 = raw.parse().ok()?;
        let max_day = days_in_month(year, month);
        (1..=max_day).contains(&day).then_some(day)
    }

    fn parse_valid_time(&self, granularity: TimeGranularity) -> Option<CalendarTime> {
        let cycle = self.effective_hour_cycle();
        let raw_hour = self.values.hour.as_deref()?;
        let parsed_hour: u8 = raw_hour.parse().ok()?;
        let hour_24 = match cycle {
            HourCycle::H12 => {
                let period = self.values.day_period?;
                hour_12_to_24(parsed_hour, period)?
            }
            HourCycle::H24 => {
                if parsed_hour >= 24 {
                    return None;
                }
                parsed_hour
            }
        };

        let minute = if granularity.includes_minute() {
            let raw_min = self.values.minute.as_deref()?;
            let m: u8 = raw_min.parse().ok()?;
            if m >= 60 {
                return None;
            }
            m
        } else {
            0
        };

        let second = if granularity.includes_second() {
            let raw_sec = self.values.second.as_deref()?;
            let s: u8 = raw_sec.parse().ok()?;
            if s >= 60 {
                return None;
            }
            s
        } else {
            0
        };

        CalendarTime::new(hour_24, minute, second, 0)
    }

    fn current_max_day(&self) -> u8 {
        let Some(month) = self.parse_valid_month() else {
            return 31;
        };
        if let Some(year_str) = self.values.year.as_deref()
            && year_str.len() == 4
            && let Ok(year) = year_str.parse::<i32>()
            && (1..=9999).contains(&year)
        {
            return days_in_month(year, month);
        }
        // When year is not yet complete, allow Feb 29 (clamped once a non-leap year is entered).
        if month == 2 {
            29
        } else {
            days_in_month(2024, month)
        }
    }

    fn clamp_day_if_needed(&mut self, require_complete_year: bool) {
        let Some(month) = self.parse_valid_month() else {
            return;
        };
        let max_day = if let Some(year_str) = self.values.year.as_deref() {
            if require_complete_year && year_str.len() < 4 {
                return;
            }
            if let Ok(year) = year_str.parse::<i32>()
                && (1..=9999).contains(&year)
            {
                days_in_month(year, month)
            } else {
                return;
            }
        } else if month == 2 {
            29
        } else {
            days_in_month(2024, month)
        };

        if let Some(day_str) = &self.values.day
            && let Ok(day) = day_str.parse::<u8>()
            && day > max_day
        {
            self.values.day = Some(format!("{max_day:02}"));
        }
    }

    fn active_tokens(&self, template: &FieldLocaleTemplate) -> Vec<FieldTemplateToken> {
        let cycle = self.effective_hour_cycle();
        match self.mode {
            FieldMode::Date {
                granularity,
                include_time_zone,
            } => template.date_field_tokens(granularity, Some(cycle), include_time_zone),
            FieldMode::Time { granularity } => template.time_tokens(cycle, granularity),
        }
    }

    /// Renders all segments (editable, timezone, and literal) with WAI-ARIA metadata.
    pub fn render_segments(&self) -> Vec<FieldSegment> {
        let template = self.locale.template();
        let tokens = self.active_tokens(&template);
        let cycle = self.effective_hour_cycle();
        let max_day = self.current_max_day() as i32;

        tokens
            .into_iter()
            .map(|token| match token {
                FieldTemplateToken::Literal(text) => FieldSegment {
                    kind: SegmentKind::Literal,
                    value: text.into_owned(),
                    is_placeholder: false,
                    min: None,
                    max: None,
                    now: None,
                    value_text: None,
                },
                FieldTemplateToken::Segment(kind) => {
                    self.render_single_segment(kind, &template, cycle, max_day)
                }
            })
            .collect()
    }

    fn render_single_segment(
        &self,
        kind: SegmentKind,
        template: &FieldLocaleTemplate,
        cycle: HourCycle,
        max_day: i32,
    ) -> FieldSegment {
        match kind {
            SegmentKind::Literal => FieldSegment {
                kind,
                value: String::new(),
                is_placeholder: false,
                min: None,
                max: None,
                now: None,
                value_text: None,
            },
            SegmentKind::TimeZoneName => FieldSegment {
                kind,
                value: template.placeholders.time_zone_name.to_string(),
                is_placeholder: false,
                min: None,
                max: None,
                now: None,
                value_text: None,
            },
            SegmentKind::DayPeriod => {
                let is_placeholder = self.values.day_period.is_none();
                let effective_period = self.values.day_period.unwrap_or(self.day_period_hint);
                let label = if is_placeholder {
                    template.placeholders.day_period.to_string()
                } else {
                    template.day_period_label(effective_period).to_string()
                };
                let now = if effective_period == DayPeriod::Pm {
                    12
                } else {
                    0
                };
                FieldSegment {
                    kind,
                    value: label.clone(),
                    is_placeholder,
                    min: Some(0),
                    max: Some(12),
                    now: Some(now),
                    value_text: Some(label),
                }
            }
            SegmentKind::Year
            | SegmentKind::Month
            | SegmentKind::Day
            | SegmentKind::Hour
            | SegmentKind::Minute
            | SegmentKind::Second => {
                let (min, max) = match kind {
                    SegmentKind::Year => (1, 9999),
                    SegmentKind::Month => (1, 12),
                    SegmentKind::Day => (1, max_day),
                    SegmentKind::Hour => match cycle {
                        HourCycle::H12 => (1, 12),
                        HourCycle::H24 => (0, 23),
                    },
                    SegmentKind::Minute | SegmentKind::Second => (0, 59),
                    _ => unreachable!(),
                };
                let raw = self.numeric_value_ref(kind);
                let is_placeholder = raw.is_none();
                let value = raw
                    .map(ToString::to_string)
                    .unwrap_or_else(|| template.placeholders.for_kind(kind).to_string());
                let now = raw.and_then(|s| s.parse::<i32>().ok());
                let value_text = match now {
                    None => Some(template.empty_value_text.to_string()),
                    Some(n) if kind == SegmentKind::Hour && cycle == HourCycle::H12 => {
                        let period = self.values.day_period.unwrap_or(self.day_period_hint);
                        let period_label = template.day_period_label(period);
                        Some(format!("{n} {period_label}"))
                    }
                    Some(n) => Some(n.to_string()),
                };
                FieldSegment {
                    kind,
                    value,
                    is_placeholder,
                    min: Some(min),
                    max: Some(max),
                    now,
                    value_text,
                }
            }
        }
    }

    fn numeric_value_ref(&self, kind: SegmentKind) -> Option<&str> {
        match kind {
            SegmentKind::Year => self.values.year.as_deref(),
            SegmentKind::Month => self.values.month.as_deref(),
            SegmentKind::Day => self.values.day.as_deref(),
            SegmentKind::Hour => self.values.hour.as_deref(),
            SegmentKind::Minute => self.values.minute.as_deref(),
            SegmentKind::Second => self.values.second.as_deref(),
            _ => None,
        }
    }

    fn numeric_value_slot(&mut self, kind: SegmentKind) -> Option<&mut Option<String>> {
        match kind {
            SegmentKind::Year => Some(&mut self.values.year),
            SegmentKind::Month => Some(&mut self.values.month),
            SegmentKind::Day => Some(&mut self.values.day),
            SegmentKind::Hour => Some(&mut self.values.hour),
            SegmentKind::Minute => Some(&mut self.values.minute),
            SegmentKind::Second => Some(&mut self.values.second),
            _ => None,
        }
    }

    fn had_committed_value(&self) -> bool {
        match self.mode {
            FieldMode::Date { .. } => self.to_date_value().is_some(),
            FieldMode::Time { .. } => self.to_time_value().is_some(),
        }
    }

    /// Applies a host-neutral [`FieldSegmentIntent`] to `kind` and returns the updated segments,
    /// focus movement request, committed value, and optional announcement.
    pub fn apply_intent(
        &mut self,
        kind: SegmentKind,
        intent: FieldSegmentIntent,
    ) -> SegmentTransitionResult {
        let was_committed = self.had_committed_value();
        let mut move_focus = None;

        match intent {
            FieldSegmentIntent::NextSegment => {
                move_focus = Some(SegmentFocusMove::Next);
            }
            FieldSegmentIntent::PrevSegment => {
                move_focus = Some(SegmentFocusMove::Prev);
            }
            FieldSegmentIntent::Paste(text) => {
                self.apply_paste(&text);
            }
            FieldSegmentIntent::Blur => {
                self.apply_blur(kind);
            }
            FieldSegmentIntent::Digit(d) => {
                if d <= 9 {
                    move_focus = self.apply_digit(kind, d);
                }
            }
            FieldSegmentIntent::DayPeriodKey(period) => {
                if kind == SegmentKind::DayPeriod {
                    self.values.day_period = Some(period);
                    self.day_period_hint = period;
                    self.edit_states.day_period.reset_active();
                }
            }
            FieldSegmentIntent::Increment => {
                self.apply_step(kind, 1);
            }
            FieldSegmentIntent::Decrement => {
                self.apply_step(kind, -1);
            }
            FieldSegmentIntent::PageUp => {
                let delta = page_step(kind);
                self.apply_step(kind, delta);
            }
            FieldSegmentIntent::PageDown => {
                let delta = -page_step(kind);
                self.apply_step(kind, delta);
            }
            FieldSegmentIntent::Min => {
                self.apply_bound(kind, false);
            }
            FieldSegmentIntent::Max => {
                self.apply_bound(kind, true);
            }
            FieldSegmentIntent::Backspace => {
                move_focus = self.apply_backspace(kind);
            }
            FieldSegmentIntent::Delete => {
                self.apply_delete(kind);
            }
        }

        let committed_date_value = self.to_date_value();
        let committed_time_value = self.to_time_value();
        let now_committed = committed_date_value.is_some() || committed_time_value.is_some();
        let cleared_value = was_committed && !now_committed;
        let updated_segments = self.render_segments();

        let announcement = if let Some(dv) = committed_date_value {
            Some(dv.to_iso_string())
        } else if let Some(tv) = committed_time_value {
            Some(tv.to_iso_string())
        } else {
            None
        };

        SegmentTransitionResult {
            updated_segments,
            move_focus,
            committed_date_value,
            committed_time_value,
            cleared_value,
            announcement,
        }
    }

    fn apply_paste(&mut self, text: &str) {
        match self.mode {
            FieldMode::Date { granularity, .. } => {
                if let Some(dv) = parse_pasted_date_value(text, granularity) {
                    self.sync_from_date_value(Some(dv));
                }
            }
            FieldMode::Time { .. } => {
                if let Some(tv) = parse_pasted_time_value(text) {
                    self.sync_from_time_value(Some(tv));
                }
            }
        }
    }

    fn apply_blur(&mut self, kind: SegmentKind) {
        let Some(editable) = kind.as_editable() else {
            return;
        };
        self.edit_states.for_kind_mut(editable).mark_left_focus();

        match kind {
            SegmentKind::Year => {
                if let Some(raw) = self.values.year.clone() {
                    if let Ok(year) = raw.parse::<i32>() {
                        if year == 0 {
                            self.values.year = None;
                        } else {
                            self.values.year = Some(format!("{:04}", year.clamp(1, 9999)));
                            self.clamp_day_if_needed(false);
                        }
                    } else {
                        self.values.year = None;
                    }
                }
            }
            SegmentKind::Month
            | SegmentKind::Day
            | SegmentKind::Hour
            | SegmentKind::Minute
            | SegmentKind::Second => {
                let min_val = self.segment_min(kind);
                if let Some(slot) = self.numeric_value_slot(kind)
                    && let Some(raw) = slot.clone()
                {
                    if let Ok(n) = raw.parse::<u8>() {
                        if n < min_val {
                            *slot = None;
                        } else if raw.len() == 1 {
                            *slot = Some(format!("{n:02}"));
                        }
                    } else {
                        *slot = None;
                    }
                }
                if kind == SegmentKind::Month {
                    self.clamp_day_if_needed(true);
                }
            }
            SegmentKind::DayPeriod | SegmentKind::TimeZoneName | SegmentKind::Literal => {}
        }
    }

    fn segment_min(&self, kind: SegmentKind) -> u8 {
        match kind {
            SegmentKind::Month | SegmentKind::Day => 1,
            SegmentKind::Hour => match self.effective_hour_cycle() {
                HourCycle::H12 => 1,
                HourCycle::H24 => 0,
            },
            SegmentKind::Minute | SegmentKind::Second => 0,
            _ => 0,
        }
    }

    fn segment_max(&self, kind: SegmentKind) -> u8 {
        match kind {
            SegmentKind::Month => 12,
            SegmentKind::Day => self.current_max_day(),
            SegmentKind::Hour => match self.effective_hour_cycle() {
                HourCycle::H12 => 12,
                HourCycle::H24 => 23,
            },
            SegmentKind::Minute | SegmentKind::Second => 59,
            _ => 0,
        }
    }

    fn apply_digit(&mut self, kind: SegmentKind, d: u8) -> Option<SegmentFocusMove> {
        let Some(editable) = kind.as_editable() else {
            return None;
        };
        if editable == EditableSegmentKind::DayPeriod {
            return None;
        }

        if self.edit_states.for_kind(editable).has_left_focus {
            if let Some(slot) = self.numeric_value_slot(kind) {
                *slot = None;
            }
            self.edit_states.for_kind_mut(editable).reset_active();
        }

        if kind == SegmentKind::Year {
            return self.apply_year_digit(d);
        }

        let min = self.segment_min(kind);
        let max = self.segment_max(kind);
        let max_start = max / 10;
        let allows_zero = min == 0;

        let prev = self.numeric_value_ref(kind).map(ToString::to_string);
        let last_key_zero = self.edit_states.for_kind(editable).last_key_zero;

        let (next_str, next_last_zero, auto_advance) = match prev {
            None => {
                if d == 0 {
                    ("0".to_string(), true, false)
                } else if d > max_start {
                    (format!("{d:02}"), false, true)
                } else {
                    (format!("{d}"), false, false)
                }
            }
            Some(_) if last_key_zero => {
                if d != 0 {
                    (format!("{d:02}"), false, true)
                } else if allows_zero {
                    ("00".to_string(), false, true)
                } else {
                    ("0".to_string(), true, false)
                }
            }
            Some(prev_str) => {
                let prev_num = prev_str.parse::<u8>().unwrap_or(0);
                let combined = prev_num.saturating_mul(10).saturating_add(d);
                if combined <= max {
                    (format!("{combined:02}"), false, true)
                } else if d == 0 {
                    ("0".to_string(), true, false)
                } else {
                    (format!("{d:02}"), false, true)
                }
            }
        };

        if let Some(slot) = self.numeric_value_slot(kind) {
            *slot = Some(next_str);
        }
        let state = self.edit_states.for_kind_mut(editable);
        state.last_key_zero = next_last_zero;
        state.has_left_focus = false;

        if kind == SegmentKind::Month {
            self.clamp_day_if_needed(true);
        }

        auto_advance.then_some(SegmentFocusMove::Next)
    }

    fn apply_year_digit(&mut self, d: u8) -> Option<SegmentFocusMove> {
        let post_bs = self.edit_states.year.year_post_backspace_mode;
        let pressed_empty = self.edit_states.year.year_pressed_digits.is_empty();

        if post_bs {
            let mut buf = self.values.year.clone().unwrap_or_default();
            buf.push((b'0' + d) as char);
            if buf.len() >= 4 {
                let tail = &buf[buf.len() - 4..];
                let normalized = if tail == "0000" {
                    "0001".to_string()
                } else {
                    tail.to_string()
                };
                self.values.year = Some(normalized);
                self.edit_states.year.reset_active();
                self.clamp_day_if_needed(false);
                return Some(SegmentFocusMove::Next);
            }
            self.values.year = Some(buf);
            return None;
        }

        self.edit_states.year.year_pressed_digits.push(d);
        let mut buf = if pressed_empty {
            String::new()
        } else {
            self.values.year.clone().unwrap_or_default()
        };
        buf.push((b'0' + d) as char);

        if buf.len() >= 4 || self.edit_states.year.year_pressed_digits.len() >= 4 {
            let start = buf.len().saturating_sub(4);
            let tail = &buf[start..];
            let parsed = tail.parse::<i32>().unwrap_or(1).clamp(1, 9999);
            self.values.year = Some(format!("{parsed:04}"));
            self.edit_states.year.year_pressed_digits.clear();
            self.clamp_day_if_needed(false);
            return Some(SegmentFocusMove::Next);
        }

        self.values.year = Some(buf);
        None
    }

    fn apply_step(&mut self, kind: SegmentKind, delta: i32) {
        let Some(editable) = kind.as_editable() else {
            return;
        };

        if kind == SegmentKind::DayPeriod {
            let next = match self.values.day_period {
                None => self.placeholder_time.time().day_period(),
                Some(curr) => curr.toggled(),
            };
            self.values.day_period = Some(next);
            self.day_period_hint = next;
            self.edit_states.day_period.reset_active();
            return;
        }

        let cycle = self.effective_hour_cycle();
        let placeholder_date = self.placeholder_date.date();
        let placeholder_time = match self.mode {
            FieldMode::Date { .. } => self
                .placeholder_date
                .time()
                .unwrap_or_else(CalendarTime::midnight),
            FieldMode::Time { .. } => self.placeholder_time.time(),
        };

        match kind {
            SegmentKind::Year => {
                let next = match self
                    .values
                    .year
                    .as_deref()
                    .and_then(|s| s.parse::<i32>().ok())
                {
                    None => placeholder_date.year.clamp(1, 9999),
                    Some(curr) => curr.saturating_add(delta).clamp(1, 9999),
                };
                self.values.year = Some(format!("{next:04}"));
                self.clamp_day_if_needed(false);
            }
            SegmentKind::Month => {
                let next = match self
                    .values
                    .month
                    .as_deref()
                    .and_then(|s| s.parse::<i32>().ok())
                {
                    None => placeholder_date.month as i32,
                    Some(curr) => (curr - 1 + delta).rem_euclid(12) + 1,
                };
                self.values.month = Some(format!("{next:02}"));
                self.clamp_day_if_needed(true);
            }
            SegmentKind::Day => {
                let max_day = self.current_max_day() as i32;
                let next = match self
                    .values
                    .day
                    .as_deref()
                    .and_then(|s| s.parse::<i32>().ok())
                {
                    None => (placeholder_date.day as i32).clamp(1, max_day),
                    Some(curr) => (curr - 1 + delta).rem_euclid(max_day) + 1,
                };
                self.values.day = Some(format!("{next:02}"));
            }
            SegmentKind::Hour => {
                let next = match self
                    .values
                    .hour
                    .as_deref()
                    .and_then(|s| s.parse::<i32>().ok())
                {
                    None => match cycle {
                        HourCycle::H12 => placeholder_time.hour_12_display() as i32,
                        HourCycle::H24 => placeholder_time.hour as i32,
                    },
                    Some(curr) => match cycle {
                        HourCycle::H12 => (curr - 1 + delta).rem_euclid(12) + 1,
                        HourCycle::H24 => (curr + delta).rem_euclid(24),
                    },
                };
                self.values.hour = Some(format!("{next:02}"));
            }
            SegmentKind::Minute => {
                let next = match self
                    .values
                    .minute
                    .as_deref()
                    .and_then(|s| s.parse::<i32>().ok())
                {
                    None => placeholder_time.minute as i32,
                    Some(curr) => (curr + delta).rem_euclid(60),
                };
                self.values.minute = Some(format!("{next:02}"));
            }
            SegmentKind::Second => {
                let next = match self
                    .values
                    .second
                    .as_deref()
                    .and_then(|s| s.parse::<i32>().ok())
                {
                    None => placeholder_time.second as i32,
                    Some(curr) => (curr + delta).rem_euclid(60),
                };
                self.values.second = Some(format!("{next:02}"));
            }
            SegmentKind::DayPeriod | SegmentKind::TimeZoneName | SegmentKind::Literal => {}
        }

        self.edit_states.for_kind_mut(editable).reset_active();
    }

    fn apply_bound(&mut self, kind: SegmentKind, is_max: bool) {
        let Some(editable) = kind.as_editable() else {
            return;
        };
        let cycle = self.effective_hour_cycle();
        match kind {
            SegmentKind::Year => {
                self.values.year = Some(if is_max { "9999" } else { "0001" }.to_string());
                self.clamp_day_if_needed(false);
            }
            SegmentKind::Month => {
                self.values.month = Some(if is_max { "12" } else { "01" }.to_string());
                self.clamp_day_if_needed(true);
            }
            SegmentKind::Day => {
                let max_day = self.current_max_day();
                self.values.day = Some(if is_max {
                    format!("{max_day:02}")
                } else {
                    "01".to_string()
                });
            }
            SegmentKind::Hour => {
                self.values.hour = Some(match (cycle, is_max) {
                    (HourCycle::H12, false) => "01".to_string(),
                    (HourCycle::H12, true) => "12".to_string(),
                    (HourCycle::H24, false) => "00".to_string(),
                    (HourCycle::H24, true) => "23".to_string(),
                });
            }
            SegmentKind::Minute => {
                self.values.minute = Some(if is_max { "59" } else { "00" }.to_string());
            }
            SegmentKind::Second => {
                self.values.second = Some(if is_max { "59" } else { "00" }.to_string());
            }
            SegmentKind::DayPeriod => {
                let period = if is_max {
                    DayPeriod::Pm
                } else {
                    DayPeriod::Am
                };
                self.values.day_period = Some(period);
                self.day_period_hint = period;
            }
            SegmentKind::TimeZoneName | SegmentKind::Literal => {}
        }
        self.edit_states.for_kind_mut(editable).reset_active();
    }

    fn apply_backspace(&mut self, kind: SegmentKind) -> Option<SegmentFocusMove> {
        let Some(editable) = kind.as_editable() else {
            return None;
        };
        if kind == SegmentKind::DayPeriod {
            self.values.day_period = Some(DayPeriod::Am);
            self.day_period_hint = DayPeriod::Am;
            self.edit_states.day_period.reset_active();
            return None;
        }

        let current = self.numeric_value_ref(kind).map(ToString::to_string);
        let Some(mut s) = current else {
            return Some(SegmentFocusMove::Prev);
        };

        if kind == SegmentKind::Year {
            if s.len() <= 1 {
                self.values.year = None;
                self.edit_states.year.reset_active();
            } else {
                s.pop();
                self.values.year = Some(s);
                self.edit_states.year.has_left_focus = false;
                self.edit_states.year.last_key_zero = false;
                self.edit_states.year.year_pressed_digits.clear();
                self.edit_states.year.year_post_backspace_mode = true;
                self.edit_states.year.year_backspace_count =
                    self.edit_states.year.year_backspace_count.saturating_add(1);
            }
            return None;
        }

        if s.len() <= 1 || (s.len() == 2 && s.starts_with('0')) {
            if let Some(slot) = self.numeric_value_slot(kind) {
                *slot = None;
            }
        } else {
            s.pop();
            if let Some(slot) = self.numeric_value_slot(kind) {
                *slot = Some(s);
            }
        }
        self.edit_states.for_kind_mut(editable).reset_active();
        None
    }

    fn apply_delete(&mut self, kind: SegmentKind) {
        let Some(editable) = kind.as_editable() else {
            return;
        };
        if kind == SegmentKind::DayPeriod {
            self.values.day_period = Some(DayPeriod::Am);
            self.day_period_hint = DayPeriod::Am;
            self.edit_states.day_period.reset_active();
            return;
        }
        if let Some(slot) = self.numeric_value_slot(kind) {
            *slot = None;
        }
        self.edit_states.for_kind_mut(editable).reset_active();
    }
}

const fn page_step(kind: SegmentKind) -> i32 {
    match kind {
        SegmentKind::Year => 10,
        SegmentKind::Month => 3,
        SegmentKind::Day => 7,
        SegmentKind::Hour => 2,
        SegmentKind::Minute | SegmentKind::Second => 10,
        SegmentKind::DayPeriod | SegmentKind::TimeZoneName | SegmentKind::Literal => 1,
    }
}

/// Parses a pasted date or date-time string into [`DateValue`] respecting `granularity`.
///
/// Accepts:
/// - ISO-8601 date: `YYYY-MM-DD`
/// - ISO-8601 date-time: `YYYY-MM-DDTHH:MM[:SS[.mmm]]` or `YYYY-MM-DD HH:MM[:SS[.mmm]]`
/// - US slash date: `MM/DD/YYYY` (and 1-digit `M/D/YYYY`)
/// - Dot date (`ko-KR`): `YYYY. MM. DD.` or `YYYY.MM.DD`
pub fn parse_pasted_date_value(text: &str, granularity: Granularity) -> Option<DateValue> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(dt) = CalendarDateTime::from_iso_str(trimmed) {
        return Some(match granularity {
            Granularity::Day => DateValue::Date(dt.date),
            _ => DateValue::DateTime(dt),
        });
    }

    if let Some(date) = CalendarDate::from_iso_str(trimmed) {
        return Some(match granularity {
            Granularity::Day => DateValue::Date(date),
            _ => DateValue::DateTime(CalendarDateTime::from_date(date)),
        });
    }

    // Try `MM/DD/YYYY` (optionally followed by whitespace + time)
    if let Some((date_part, rest_time)) = split_date_and_optional_time(trimmed, '/') {
        let mut parts = date_part.split('/');
        let m: u8 = parts.next()?.trim().parse().ok()?;
        let d: u8 = parts.next()?.trim().parse().ok()?;
        let y: i32 = parts.next()?.trim().parse().ok()?;
        if parts.next().is_some() {
            return None;
        }
        let date = CalendarDate::new(y, m, d)?;
        return Some(finish_pasted_date(date, rest_time, granularity));
    }

    // Try `YYYY. MM. DD.` or `YYYY.MM.DD`
    if trimmed.contains('.') {
        let cleaned = trimmed.trim_end_matches('.');
        let parts: Vec<&str> = cleaned
            .split('.')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        if parts.len() == 3 {
            let y: i32 = parts[0].parse().ok()?;
            let m: u8 = parts[1].parse().ok()?;
            let d: u8 = parts[2].parse().ok()?;
            let date = CalendarDate::new(y, m, d)?;
            return Some(finish_pasted_date(date, None, granularity));
        }
    }

    None
}

fn split_date_and_optional_time(input: &str, sep: char) -> Option<(&str, Option<&str>)> {
    if !input.contains(sep) {
        return None;
    }
    if let Some((left, right)) = input.split_once(|c: char| c.is_whitespace() || c == ',') {
        let r = right.trim_start_matches(|c: char| c.is_whitespace() || c == ',');
        Some((left.trim(), (!r.is_empty()).then_some(r)))
    } else {
        Some((input, None))
    }
}

fn finish_pasted_date(
    date: CalendarDate,
    rest_time: Option<&str>,
    granularity: Granularity,
) -> DateValue {
    match granularity {
        Granularity::Day => DateValue::Date(date),
        _ => {
            let time = rest_time
                .and_then(parse_pasted_time_value)
                .map(|tv| tv.time())
                .unwrap_or_else(CalendarTime::midnight);
            DateValue::DateTime(CalendarDateTime::new(date, time))
        }
    }
}

/// Parses a pasted time or date-time string into [`TimeValue`].
///
/// Accepts:
/// - ISO-8601 date-time: `YYYY-MM-DDTHH:MM[:SS[.mmm]]`
/// - 24-hour time: `HH:MM[:SS[.mmm]]` (and 1-digit `H:MM[:SS]`)
/// - 12-hour time with meridiem: `h:mm AM`, `hh:mm:ss PM`, `h:mmAM`, `오전 9:30`, `오후 02:15:30`
pub fn parse_pasted_time_value(text: &str) -> Option<TimeValue> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(dt) = CalendarDateTime::from_iso_str(trimmed) {
        return Some(TimeValue::DateTime(dt));
    }

    if let Some(time) = CalendarTime::from_iso_str(trimmed) {
        return Some(TimeValue::Time(time));
    }

    // Check leading Korean meridiem (`오전` / `오후`)
    let (meridiem, core) = if let Some(rest) = trimmed.strip_prefix("오전") {
        (Some(DayPeriod::Am), rest.trim())
    } else if let Some(rest) = trimmed.strip_prefix("오후") {
        (Some(DayPeriod::Pm), rest.trim())
    } else {
        // Check trailing ASCII `AM` / `PM` (case-insensitive)
        let upper = trimmed.to_ascii_uppercase();
        if upper.ends_with("AM") {
            let cut = trimmed.len() - 2;
            (Some(DayPeriod::Am), trimmed[..cut].trim())
        } else if upper.ends_with("PM") {
            let cut = trimmed.len() - 2;
            (Some(DayPeriod::Pm), trimmed[..cut].trim())
        } else {
            (None, trimmed)
        }
    };

    let mut parts = core.split(':');
    let h_str = parts.next()?.trim();
    let m_str = parts.next()?.trim();
    let s_str = parts.next().map(str::trim);
    if parts.next().is_some() {
        return None;
    }
    let raw_hour: u8 = h_str.parse().ok()?;
    let minute: u8 = m_str.parse().ok()?;
    let second: u8 = match s_str {
        None => 0,
        Some(s) => s.parse().ok()?,
    };

    let hour_24 = match meridiem {
        Some(period) => hour_12_to_24(raw_hour, period)?,
        None => {
            if raw_hour >= 24 {
                return None;
            }
            raw_hour
        }
    };

    CalendarTime::new(hour_24, minute, second, 0).map(TimeValue::Time)
}
