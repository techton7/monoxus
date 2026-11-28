use dioxus::prelude::*;
use ui_sdk::kernel::{ChangeContext, ChangeReason, ChangeSource};

use crate::foundation::state::ChangeOutcome;

use super::{runtime::SelectRuntime, types::SelectMode};

impl SelectRuntime {
    pub(crate) fn propose_single_value(&self, next_val: Option<String>) -> ChangeOutcome {
        self.propose_single_value_with_context(
            next_val,
            ChangeContext::new(ChangeReason::Selection, ChangeSource::Mouse),
        )
    }

    pub(crate) fn propose_single_value_with_context(
        &self,
        next_val: Option<String>,
        ctx: ChangeContext,
    ) -> ChangeOutcome {
        let next_vals = next_val
            .as_ref()
            .map(|v| vec![v.clone()])
            .unwrap_or_default();

        if let Some(val_ctl) = self.state.value_ctl {
            let is_val_controlled = val_ctl.is_controlled();
            let outcome = val_ctl.set_with_context(next_val.clone(), ctx);
            if outcome.is_accepted()
                && let Some(vals_ctl) = self.state.values_ctl
            {
                if is_val_controlled && !vals_ctl.is_controlled() {
                    if let Some(cb) = self.on_values_change.try_read().ok().and_then(|g| g.clone())
                    {
                        cb(next_vals);
                    }
                } else {
                    let _ = vals_ctl.set_with_context(next_vals, ctx);
                }
            }
            return outcome;
        }

        let value_controlled = self.select.read().is_value_controlled();
        if !value_controlled {
            let mut val_sig = self.state.value;
            val_sig.set(next_val.clone());
            let mut vals_sig = self.state.values;
            vals_sig.set(next_vals.clone());
        }
        if let Some(cb) = self.on_value_change.try_read().ok().and_then(|g| g.clone()) {
            cb(next_val);
        }
        if let Some(cb) = self.on_values_change.try_read().ok().and_then(|g| g.clone()) {
            cb(next_vals);
        }
        if value_controlled {
            ChangeOutcome::ControlledPending(ctx)
        } else {
            ChangeOutcome::Committed(ctx)
        }
    }

    pub(crate) fn propose_multiple_values(&self, next_vals: Vec<String>) -> ChangeOutcome {
        self.propose_multiple_values_with_context(
            next_vals,
            ChangeContext::new(ChangeReason::Selection, ChangeSource::Mouse),
        )
    }

    pub(crate) fn propose_multiple_values_with_context(
        &self,
        next_vals: Vec<String>,
        ctx: ChangeContext,
    ) -> ChangeOutcome {
        let next_first = next_vals.first().cloned();

        if let Some(vals_ctl) = self.state.values_ctl {
            let is_vals_controlled = vals_ctl.is_controlled();
            let outcome = vals_ctl.set_with_context(next_vals.clone(), ctx);
            if outcome.is_accepted()
                && let Some(val_ctl) = self.state.value_ctl
            {
                if is_vals_controlled && !val_ctl.is_controlled() {
                    if let Some(cb) = self.on_value_change.try_read().ok().and_then(|g| g.clone()) {
                        cb(next_first);
                    }
                } else {
                    let _ = val_ctl.set_with_context(next_first, ctx);
                }
            }
            return outcome;
        }

        let values_controlled = self.select.read().is_values_controlled();
        if !values_controlled {
            let mut vals_sig = self.state.values;
            vals_sig.set(next_vals.clone());
            let mut val_sig = self.state.value;
            val_sig.set(next_first.clone());
        }
        if let Some(cb) = self.on_values_change.try_read().ok().and_then(|g| g.clone()) {
            cb(next_vals);
        }
        if let Some(cb) = self.on_value_change.try_read().ok().and_then(|g| g.clone()) {
            cb(next_first);
        }
        if values_controlled {
            ChangeOutcome::ControlledPending(ctx)
        } else {
            ChangeOutcome::Committed(ctx)
        }
    }

    pub fn select_item(&self, val: &str) {
        let _ = self.select_item_with_context(
            val,
            ChangeContext::new(ChangeReason::Selection, ChangeSource::Mouse),
        );
    }

    pub(crate) fn select_item_with_context(&self, val: &str, ctx: ChangeContext) -> ChangeOutcome {
        if self.is_disabled() {
            return ChangeOutcome::Unchanged;
        }

        let (mode, allow_deselect) = {
            let select = self.select.read();
            (select.mode(), select.allows_deselect())
        };

        // AC-6 / BI-P3.5-004: Clear-item placeholder reset (value = "")
        if val.is_empty() {
            let outcome = match mode {
                SelectMode::Single => {
                    if self.value().is_some() {
                        self.propose_single_value_with_context(None, ctx)
                    } else {
                        ChangeOutcome::Unchanged
                    }
                }
                SelectMode::Multiple => {
                    if !self.values().is_empty() {
                        self.propose_multiple_values_with_context(Vec::new(), ctx)
                    } else {
                        ChangeOutcome::Unchanged
                    }
                }
            };

            if !outcome.is_canceled() {
                let _ = self.close_dropdown_with_context(true, ctx);
            }
            return outcome;
        }

        if let Some(lbl) = self.item_label(val) {
            let mut labels_sig = self.state.selected_labels;
            if let Ok(mut labels) = labels_sig.try_write() {
                labels.insert(val.to_string(), lbl);
            }
        }

        match mode {
            SelectMode::Single => {
                let current_val = self.value();
                let is_current = current_val.as_deref() == Some(val);
                let next_val = if allow_deselect && is_current {
                    None
                } else {
                    Some(val.to_owned())
                };

                let outcome = if next_val != current_val {
                    self.propose_single_value_with_context(next_val, ctx)
                } else {
                    ChangeOutcome::Unchanged
                };

                if !outcome.is_canceled() {
                    let _ = self.close_dropdown_with_context(true, ctx);
                }
                outcome
            }
            SelectMode::Multiple => {
                let current_vals = self.values();
                let mut next_vals = current_vals.clone();
                if let Some(pos) = next_vals.iter().position(|v| v == val) {
                    next_vals.remove(pos);
                } else {
                    next_vals.push(val.to_owned());
                }

                if next_vals != current_vals {
                    self.propose_multiple_values_with_context(next_vals, ctx)
                } else {
                    ChangeOutcome::Unchanged
                }
            }
        }
    }

    pub fn toggle_item(&self, val: &str) {
        self.select_item(val);
    }
}
