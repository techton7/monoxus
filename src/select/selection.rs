use dioxus::prelude::*;

use super::{runtime::SelectRuntime, types::SelectMode};

impl SelectRuntime {
    pub fn select_item(&self, val: &str) {
        if self.is_disabled() {
            return;
        }

        let select_def = self.select.read();
        let mode = select_def.mode();
        let value_controlled = select_def.is_value_controlled();
        let values_controlled = select_def.is_values_controlled();
        drop(select_def);

        // AC-6 / BI-P3.5-004: Clear-item placeholder reset (value = "")
        if val.is_empty() {
            match mode {
                SelectMode::Single { .. } => {
                    let current_val = self.value();
                    if current_val.is_some() {
                        if !value_controlled {
                            let mut val_sig = self.state.value;
                            val_sig.set(None);
                            let mut vals_sig = self.state.values;
                            vals_sig.set(Vec::new());
                        }
                        if let Some(cb) = self.on_value_change.try_read().ok().and_then(|g| g.clone()) {
                            cb(None);
                        }
                        if let Some(cb) = self.on_values_change.try_read().ok().and_then(|g| g.clone()) {
                            cb(Vec::new());
                        }
                    }
                }
                SelectMode::Multiple => {
                    let current_vals = self.values();
                    if !current_vals.is_empty() {
                        if !values_controlled {
                            let mut vals_sig = self.state.values;
                            vals_sig.set(Vec::new());
                            let mut val_sig = self.state.value;
                            val_sig.set(None);
                        }
                        if let Some(cb) = self.on_values_change.try_read().ok().and_then(|g| g.clone()) {
                            cb(Vec::new());
                        }
                        if let Some(cb) = self.on_value_change.try_read().ok().and_then(|g| g.clone()) {
                            cb(None);
                        }
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
            SelectMode::Single { allow_deselect } => {
                let current_val = self.value();
                let is_current = current_val.as_deref() == Some(val);
                let next_val = if allow_deselect && is_current {
                    None
                } else {
                    Some(val.to_owned())
                };

                if next_val != current_val {
                    if !value_controlled {
                        let mut val_sig = self.state.value;
                        val_sig.set(next_val.clone());
                        let mut vals_sig = self.state.values;
                        vals_sig.set(
                            next_val
                                .as_ref()
                                .map(|v| vec![v.clone()])
                                .unwrap_or_default(),
                        );
                    }

                    if let Some(cb) = self.on_value_change.try_read().ok().and_then(|g| g.clone()) {
                        cb(next_val.clone());
                    }
                    if let Some(cb) = self.on_values_change.try_read().ok().and_then(|g| g.clone()) {
                        cb(next_val.map(|v| vec![v]).unwrap_or_default());
                    }
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
                    if !values_controlled {
                        let mut vals_sig = self.state.values;
                        vals_sig.set(next_vals.clone());

                        let mut val_sig = self.state.value;
                        val_sig.set(next_vals.first().cloned());
                    }

                    if let Some(cb) = self.on_values_change.try_read().ok().and_then(|g| g.clone()) {
                        cb(next_vals.clone());
                    }
                    if let Some(cb) = self.on_value_change.try_read().ok().and_then(|g| g.clone()) {
                        cb(next_vals.first().cloned());
                    }
                }
            }
        }
    }

    pub fn toggle_item(&self, val: &str) {
        self.select_item(val);
    }
}
