use dioxus::prelude::*;

use super::{runtime::SelectRuntime, types::SelectMode};

impl SelectRuntime {
    pub fn select_item(&self, val: &str) {
        if self.is_disabled() {
            return;
        }

        // AC-6 / BI-P3.5-004: Clear-item placeholder reset (value = "")
        if val.is_empty() {
            let mut val_sig = self.state.value;
            val_sig.set(None);
            let mut vals_sig = self.state.values;
            vals_sig.set(Vec::new());

            if let Some(ref cb) = self.on_value_change {
                cb(None);
            }
            if let Some(ref cb) = self.on_values_change {
                cb(Vec::new());
            }

            self.close_dropdown();
            return;
        }

        match self.select.mode() {
            SelectMode::Single { allow_deselect } => {
                let is_current = self.value().as_deref() == Some(val);
                let next_val = if allow_deselect && is_current {
                    None
                } else {
                    Some(val.to_owned())
                };

                let mut val_sig = self.state.value;
                val_sig.set(next_val.clone());
                let mut vals_sig = self.state.values;
                vals_sig.set(
                    next_val
                        .as_ref()
                        .map(|v| vec![v.clone()])
                        .unwrap_or_default(),
                );

                if let Some(ref cb) = self.on_value_change {
                    cb(next_val.clone());
                }
                if let Some(ref cb) = self.on_values_change {
                    cb(next_val.map(|v| vec![v]).unwrap_or_default());
                }

                self.close_dropdown();
            }
            SelectMode::Multiple => {
                let mut vals_sig = self.state.values;
                let mut list = vals_sig.read().clone();
                if let Some(pos) = list.iter().position(|v| v == val) {
                    list.remove(pos);
                } else {
                    list.push(val.to_owned());
                }
                vals_sig.set(list.clone());

                let mut val_sig = self.state.value;
                val_sig.set(list.first().cloned());

                if let Some(ref cb) = self.on_values_change {
                    cb(list.clone());
                }
                if let Some(ref cb) = self.on_value_change {
                    cb(list.first().cloned());
                }
            }
        }
    }

    pub fn toggle_item(&self, val: &str) {
        self.select_item(val);
    }
}
