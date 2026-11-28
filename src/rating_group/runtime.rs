use std::cell::{Cell, RefCell};
use std::rc::Rc;

use dioxus::prelude::keyboard_types::Key;
use dioxus::prelude::*;
use ui_sdk::kernel::{ChangeContext, ChangeEvent, ChangeReason, ChangeSource};

use crate::foundation::{
    browser::{FocusOptions, focus_element_sync_or_defer, get_element_client_rect},
    keyboard::{NavKey, translate_nav_key},
    state::{
        ChangeOutcome, ControllableConfig, ControllableSignal, sync_f64_signal, sync_signal,
        use_controllable_signal_config,
    },
};

use super::{
    attrs::{RatingGroupItemAttributes, RatingGroupRootAttributes},
    relationships::RatingGroupRelationships,
    state::{
        RatingGroupDefinition, RatingGroupKeySequence, calculate_rating_from_pointer,
        clamp_rating_value, derive_item_state, derive_items, reduce_rating_group_key,
        resolve_item_click_value,
    },
    types::{
        RATING_GROUP_KEY_SEQUENCE_TIMEOUT_MS, RatingGroupAriaValuetext, RatingGroupItemData,
        RatingGroupItemState, RatingGroupKey,
    },
};

fn f64_bits_eq(a: &f64, b: &f64) -> bool {
    a.to_bits() == b.to_bits()
}

/// Translates Dioxus `keyboard_types::Key` into host-neutral [`RatingGroupKey`].
pub fn translate_keyboard_key(key: &Key) -> RatingGroupKey {
    match translate_nav_key(key) {
        Some(NavKey::ArrowUp) => RatingGroupKey::ArrowUp,
        Some(NavKey::ArrowRight) => RatingGroupKey::ArrowRight,
        Some(NavKey::ArrowDown) => RatingGroupKey::ArrowDown,
        Some(NavKey::ArrowLeft) => RatingGroupKey::ArrowLeft,
        Some(NavKey::Home) => RatingGroupKey::Home,
        Some(NavKey::End) => RatingGroupKey::End,
        Some(NavKey::PageUp) => RatingGroupKey::PageUp,
        Some(NavKey::PageDown) => RatingGroupKey::PageDown,
        _ => match key {
            Key::Character(s) => match s.as_str() {
                "." => RatingGroupKey::DecimalPoint,
                _ => {
                    let mut chars = s.chars();
                    match (chars.next(), chars.next()) {
                        (Some(c @ '0'..='9'), None) => RatingGroupKey::Digit(c as u8 - b'0'),
                        _ => RatingGroupKey::Other,
                    }
                }
            },
            _ => RatingGroupKey::Other,
        },
    }
}

/// Reactive runtime coordinating committed/hover rating values, decimal key sequence,
/// pointer hit detection, and attribute publication.
#[derive(Clone)]
pub struct RatingGroupRuntime {
    pub def: Signal<RatingGroupDefinition>,
    pub rels: RatingGroupRelationships,
    pub root_dom_id: Signal<String>,
    pub value: Signal<f64>,
    pub(crate) value_ctl: ControllableSignal<f64>,
    pub hover_value: Signal<Option<f64>>,
    pub is_controlled: Signal<bool>,
    pub key_sequence: Signal<RatingGroupKeySequence>,
    pub decimal_timer_gen: Rc<Cell<u64>>,
    pub decimal_timer_task: Rc<RefCell<Option<dioxus::core::Task>>>,
    pub aria_label: Signal<Option<String>>,
    pub aria_labelledby: Signal<Option<String>>,
    pub aria_describedby: Signal<Option<String>>,
    pub invalid: Signal<bool>,
    pub aria_valuetext: Rc<RefCell<Option<RatingGroupAriaValuetext>>>,
}

impl RatingGroupRuntime {
    /// Returns the committed rating value.
    pub fn current_value(&self) -> f64 {
        (self.value)()
    }

    /// Returns the active display value (hover preview if active, otherwise committed value).
    pub fn display_value(&self) -> f64 {
        (self.hover_value)().unwrap_or_else(|| (self.value)())
    }

    /// Derives the item list snapshot for indices `0..def.max`.
    pub fn items(&self) -> Vec<RatingGroupItemData> {
        let def = self.def.read();
        derive_items(self.display_value(), def.max, def.allow_half)
    }

    /// Derives the visual fill state for item `index`.
    pub fn item_state(&self, index: usize) -> RatingGroupItemState {
        let def = self.def.read();
        derive_item_state(self.display_value(), index, def.allow_half)
    }

    /// Derives WAI-ARIA and `data-*` attributes for `RatingGroupRoot`.
    pub fn root_attrs(&self) -> RatingGroupRootAttributes {
        let def = self.def.read();
        let committed = self.current_value();
        let dom_id = (self.root_dom_id)();
        let custom_valuetext = match self.aria_valuetext.borrow().as_ref() {
            Some(RatingGroupAriaValuetext::Static(text)) => Some(text.clone()),
            Some(RatingGroupAriaValuetext::Formatter(cb)) => Some(cb.call((committed, def.max))),
            None => None,
        };

        RatingGroupRootAttributes::new(
            dom_id,
            committed,
            def.min,
            def.max,
            def.orientation,
            def.direction,
            def.disabled,
            def.read_only,
            def.required,
            (self.aria_label)(),
            (self.aria_labelledby)(),
            custom_valuetext,
        )
        .with_invalid((self.invalid)())
        .with_aria_describedby((self.aria_describedby)())
    }

    /// Derives `data-*` and `role="presentation"` attributes for `RatingGroupItem`.
    pub fn item_attrs(
        &self,
        index: usize,
        item_id: Option<String>,
        item_disabled: bool,
    ) -> RatingGroupItemAttributes {
        let def = self.def.read();
        let state = derive_item_state(self.display_value(), index, def.allow_half);
        RatingGroupItemAttributes::new(
            item_id.unwrap_or_else(|| self.rels.item_id(index)),
            index,
            def.orientation,
            def.disabled || item_disabled,
            def.read_only,
            state,
        )
        .with_invalid((self.invalid)())
    }

    /// Cancels any active decimal key sequence timeout.
    pub fn cancel_decimal_timer(&self) {
        self.decimal_timer_gen
            .set(self.decimal_timer_gen.get().wrapping_add(1));
        if let Some(task) = self.decimal_timer_task.borrow_mut().take() {
            task.cancel();
        }
    }

    /// Starts or resets the 1000ms decimal sequence expiration timer.
    pub fn start_decimal_timer(&mut self) {
        self.cancel_decimal_timer();
        if dioxus::core::Runtime::try_current().is_none() {
            return;
        }
        let next_gen = self.decimal_timer_gen.get().wrapping_add(1);
        self.decimal_timer_gen.set(next_gen);
        let gen_cell = Rc::clone(&self.decimal_timer_gen);
        let mut key_seq = self.key_sequence;
        let task = spawn(async move {
            futures_timer::Delay::new(std::time::Duration::from_millis(
                RATING_GROUP_KEY_SEQUENCE_TIMEOUT_MS,
            ))
            .await;
            if gen_cell.get() == next_gen
                && let Ok(mut seq) = key_seq.try_write()
            {
                *seq = RatingGroupKeySequence::Idle;
            }
        });
        *self.decimal_timer_task.borrow_mut() = Some(task);
    }

    /// Commits a new rating value if enabled/editable and distinct from current value.
    pub fn set_value(&mut self, next: f64) {
        let _ = self.set_value_with_context(
            next,
            ChangeContext::new(ChangeReason::ValueChange, ChangeSource::Programmatic),
        );
    }

    /// Commits a new rating value with explicit [`ChangeContext`].
    pub fn set_value_with_context(&mut self, next: f64, ctx: ChangeContext) -> ChangeOutcome {
        let def = self.def.peek().clone();
        if def.disabled || def.read_only {
            return ChangeOutcome::Unchanged;
        }
        let clamped = clamp_rating_value(next, def.min, def.max);
        self.value_ctl.set_by_with_context(clamped, ctx, f64_bits_eq)
    }

    /// Updates hover preview value without modifying committed value or emitting `on_value_change`.
    pub fn set_hover_value(&mut self, next: Option<f64>) {
        let def = self.def.peek().clone();
        if def.disabled || def.read_only || !def.hover_preview {
            return;
        }
        let clamped = next.map(|v| clamp_rating_value(v, def.min, def.max));
        sync_signal(&mut self.hover_value, clamped);
    }

    /// Clears hover preview unconditionally (e.g. on root pointer leave or key press).
    pub fn clear_hover_value(&mut self) {
        if self.hover_value.peek().is_some() {
            self.hover_value.set(None);
        }
    }

    /// Resets rating value back to `initial` in response to native `<form>` reset.
    pub fn reset_to(&mut self, initial: f64) {
        let def = self.def.peek().clone();
        let clamped = clamp_rating_value(initial, def.min, def.max);
        self.clear_hover_value();
        self.cancel_decimal_timer();
        sync_signal(&mut self.key_sequence, RatingGroupKeySequence::Idle);
        self.value_ctl.set_by_with_context(
            clamped,
            ChangeContext::new(ChangeReason::Reset, ChangeSource::Programmatic),
            f64_bits_eq,
        );
    }

    /// Dispatches a translated keyboard intent; returns `true` if `prevent_default` should run.
    pub fn handle_keydown(&mut self, key: RatingGroupKey) -> bool {
        let def = self.def.peek().clone();
        let current = *self.value.peek();
        let seq = *self.key_sequence.peek();
        let outcome = reduce_rating_group_key(&def, current, seq, key);

        sync_signal(&mut self.key_sequence, outcome.next_sequence);
        if outcome.clear_hover {
            self.clear_hover_value();
        }
        if outcome.clear_decimal_timer {
            self.cancel_decimal_timer();
        }
        if outcome.start_decimal_timer {
            self.start_decimal_timer();
        }
        if let Some(next_value) = outcome.next_value {
            let _ = self.set_value_with_context(
                next_value,
                ChangeContext::new(ChangeReason::ValueChange, ChangeSource::Keyboard),
            );
        }

        outcome.prevent_default
    }

    fn compute_pointer_rating(
        &self,
        index: usize,
        client_x: f64,
        client_y: f64,
        item_dom_id: &str,
    ) -> f64 {
        let def = self.def.peek();
        if !def.allow_half {
            return (index + 1) as f64;
        }
        if let Some(rect) =
            get_element_client_rect(item_dom_id).filter(|r| r.width > 0.0 || r.height > 0.0)
        {
            calculate_rating_from_pointer(
                index,
                true,
                def.orientation,
                def.direction,
                client_x,
                client_y,
                rect.left(),
                rect.top(),
                rect.width,
                rect.height,
            )
        } else {
            (index + 1) as f64
        }
    }

    /// Handles item click: updates/clears rating and focuses `RatingGroupRoot`.
    pub fn handle_item_click(
        &mut self,
        index: usize,
        item_disabled: bool,
        client_x: f64,
        client_y: f64,
        item_dom_id: &str,
    ) {
        let def = self.def.peek().clone();
        if def.disabled || item_disabled || def.read_only {
            return;
        }
        let calculated = self.compute_pointer_rating(index, client_x, client_y, item_dom_id);
        let next =
            resolve_item_click_value(index, *self.value.peek(), def.min, def.max, calculated);
        let _ = self.set_value_with_context(
            next,
            ChangeContext::new(ChangeReason::ValueChange, ChangeSource::Mouse),
        );

        if dioxus::core::Runtime::try_current().is_some() {
            let root_dom_id = self.root_dom_id.peek().clone();
            focus_element_sync_or_defer(&root_dom_id, FocusOptions::new(false));
        }
    }

    /// Handles item pointer move: updates hover preview unless touch or disabled/readonly.
    pub fn handle_item_pointer_move(
        &mut self,
        index: usize,
        item_disabled: bool,
        pointer_type: &str,
        client_x: f64,
        client_y: f64,
        item_dom_id: &str,
    ) {
        let def = self.def.peek().clone();
        if def.disabled
            || item_disabled
            || def.read_only
            || !def.hover_preview
            || pointer_type == "touch"
        {
            return;
        }
        let hover = self.compute_pointer_rating(index, client_x, client_y, item_dom_id);
        self.set_hover_value(Some(hover));
    }
}

/// Initializes reactive runtime signals and registers `use_drop` cleanup for `RatingGroupRoot`.
#[allow(clippy::too_many_arguments)]
pub fn use_rating_group_runtime(
    def: RatingGroupDefinition,
    rels: RatingGroupRelationships,
    root_dom_id: String,
    value: Option<f64>,
    default_value: f64,
    on_value_change: Option<EventHandler<f64>>,
    aria_label: Option<String>,
    aria_labelledby: Option<String>,
    aria_valuetext: Option<RatingGroupAriaValuetext>,
) -> RatingGroupRuntime {
    let default_opt = if default_value.to_bits() == 0.0_f64.to_bits() {
        None
    } else {
        Some(default_value)
    };
    use_rating_group_runtime_with_details(
        def,
        rels,
        root_dom_id,
        value,
        default_opt,
        on_value_change,
        None,
        aria_label,
        aria_labelledby,
        aria_valuetext,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn use_rating_group_runtime_with_details(
    def: RatingGroupDefinition,
    rels: RatingGroupRelationships,
    root_dom_id: String,
    value: Option<f64>,
    default_value: Option<f64>,
    on_value_change: Option<EventHandler<f64>>,
    on_value_change_details: Option<EventHandler<ChangeEvent<f64>>>,
    aria_label: Option<String>,
    aria_labelledby: Option<String>,
    aria_valuetext: Option<RatingGroupAriaValuetext>,
) -> RatingGroupRuntime {
    let min = def.min;
    let max = def.max;
    let has_explicit_default = default_value.is_some();
    let value_ctl = use_controllable_signal_config(
        ControllableConfig::new(value, default_value.unwrap_or(0.0))
            .with_explicit_default(has_explicit_default)
            .with_on_change(on_value_change)
            .with_on_change_details(on_value_change_details)
            .with_sanitize(move |v| clamp_rating_value(v, min, max))
            .with_eq(f64_bits_eq)
            .with_component("RatingGroupRoot")
            .with_prop("value"),
    );

    if !value_ctl.is_controlled() {
        let mut val_sig = value_ctl.signal();
        let current = *val_sig.peek();
        let clamped = clamp_rating_value(current, min, max);
        sync_f64_signal(&mut val_sig, clamped);
    }

    let aria_valuetext_slot = use_hook(|| Rc::new(RefCell::new(None)));
    *aria_valuetext_slot.borrow_mut() = aria_valuetext;

    let decimal_timer_gen = use_hook(|| Rc::new(Cell::new(0_u64)));
    let decimal_timer_task: Rc<RefCell<Option<dioxus::core::Task>>> =
        use_hook(|| Rc::new(RefCell::new(None)));

    let drop_gen = Rc::clone(&decimal_timer_gen);
    let drop_task = Rc::clone(&decimal_timer_task);
    use_drop(move || {
        drop_gen.set(drop_gen.get().wrapping_add(1));
        if let Some(task) = drop_task.borrow_mut().take() {
            task.cancel();
        }
    });

    let is_controlled = value_ctl.is_controlled();
    let should_clear_hover = def.disabled || def.read_only || !def.hover_preview;
    let mut rt = RatingGroupRuntime {
        def: use_signal(|| def.clone()),
        rels,
        root_dom_id: use_signal(|| root_dom_id.clone()),
        value: value_ctl.signal(),
        value_ctl,
        hover_value: use_signal(|| None),
        is_controlled: use_signal(|| is_controlled),
        key_sequence: use_signal(RatingGroupKeySequence::default),
        decimal_timer_gen,
        decimal_timer_task,
        aria_label: use_signal(|| aria_label.clone()),
        aria_labelledby: use_signal(|| aria_labelledby.clone()),
        aria_describedby: use_signal(|| None),
        invalid: use_signal(|| false),
        aria_valuetext: aria_valuetext_slot,
    };

    sync_signal(&mut rt.is_controlled, is_controlled);
    sync_signal(&mut rt.def, def);
    sync_signal(&mut rt.root_dom_id, root_dom_id);
    sync_signal(&mut rt.aria_label, aria_label);
    sync_signal(&mut rt.aria_labelledby, aria_labelledby);
    if should_clear_hover {
        rt.clear_hover_value();
    }

    rt
}

/// Context struct providing [`RatingGroupRuntime`] to `RatingGroupItem` descendants.
#[derive(Clone)]
pub struct RatingGroupContext(pub RatingGroupRuntime);

/// Hook accessing [`RatingGroupContext`] inside a `RatingGroupRoot` subtree.
pub fn use_rating_group_context() -> RatingGroupContext {
    use_context::<RatingGroupContext>()
}

/// Optional variant of [`use_rating_group_context`].
pub fn try_use_rating_group_context() -> Option<RatingGroupContext> {
    try_consume_context::<RatingGroupContext>()
}

/// Context struct exposing reactive state for a single `RatingGroupItem` subtree.
#[derive(Clone)]
pub struct RatingGroupItemContext {
    pub runtime: RatingGroupRuntime,
    pub index: Signal<usize>,
    pub item_disabled: Signal<bool>,
}

impl RatingGroupItemContext {
    pub fn index(&self) -> usize {
        (self.index)()
    }

    pub fn value(&self) -> f64 {
        ((self.index)() + 1) as f64
    }

    pub fn state(&self) -> RatingGroupItemState {
        self.runtime.item_state((self.index)())
    }

    pub fn is_disabled(&self) -> bool {
        self.runtime.def.read().disabled || (self.item_disabled)()
    }
}

/// Hook accessing [`RatingGroupItemContext`] inside a `RatingGroupItem` subtree.
pub fn use_rating_group_item_context() -> RatingGroupItemContext {
    use_context::<RatingGroupItemContext>()
}

/// Optional variant of [`use_rating_group_item_context`].
pub fn try_use_rating_group_item_context() -> Option<RatingGroupItemContext> {
    try_consume_context::<RatingGroupItemContext>()
}
