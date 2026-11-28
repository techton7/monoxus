use dioxus::prelude::*;

use super::{runtime::SelectRuntime, types::SelectMode};

impl SelectRuntime {
    pub(crate) fn propose_single_value(&self, next_val: Option<String>) {
        let next_vals = next_val
            .as_ref()
            .map(|v| vec![v.clone()])
            .unwrap_or_default();

        if let Some(val_ctl) = self.state.value_ctl {
            let is_val_controlled = val_ctl.is_controlled();
            val_ctl.set(next_val.clone());
            if let Some(vals_ctl) = self.state.values_ctl {
                if is_val_controlled && !vals_ctl.is_controlled() {
                    if let Some(cb) = self.on_values_change.try_read().ok().and_then(|g| g.clone())
                    {
                        cb(next_vals);
                    }
                } else {
                    vals_ctl.set(next_vals);
                }
            }
            return;
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
    }

    pub(crate) fn propose_multiple_values(&self, next_vals: Vec<String>) {
        let next_first = next_vals.first().cloned();

        if let Some(vals_ctl) = self.state.values_ctl {
            let is_vals_controlled = vals_ctl.is_controlled();
            vals_ctl.set(next_vals.clone());
            if let Some(val_ctl) = self.state.value_ctl {
                if is_vals_controlled && !val_ctl.is_controlled() {
                    if let Some(cb) = self.on_value_change.try_read().ok().and_then(|g| g.clone()) {
                        cb(next_first);
                    }
                } else {
                    val_ctl.set(next_first);
                }
            }
            return;
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
    }

    pub fn select_item(&self, val: &str) {
        if self.is_disabled() {
            return;
        }

        let (mode, allow_deselect) = {
            let select = self.select.read();
            (select.mode(), select.allows_deselect())
        };

        // AC-6 / BI-P3.5-004: Clear-item placeholder reset (value = "")
        if val.is_empty() {
            match mode {
                SelectMode::Single => {
                    if self.value().is_some() {
                        self.propose_single_value(None);
                    }
                }
                SelectMode::Multiple => {
                    if !self.values().is_empty() {
                        self.propose_multiple_values(Vec::new());
                    }
                }
            }

            self.close_dropdown();
            return;
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

                if next_val != current_val {
                    self.propose_single_value(next_val);
                }

                self.close_dropdown();
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
                    self.propose_multiple_values(next_vals);
                }
            }
        }
    }

    pub fn toggle_item(&self, val: &str) {
        self.select_item(val);
    }
}
