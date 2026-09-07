use dioxus::prelude::*;

use super::runtime::SelectRuntime;

impl SelectRuntime {
    pub fn handle_typeahead(&self, ch: char, now_ms: f64) {
        let last_time = *self.state.last_key_timestamp_ms.read();
        let mut buf = self.state.typeahead_buffer.read().clone();

        // 500ms synchronous threshold per interact.md #2
        if now_ms - last_time > 500.0 {
            buf.clear();
        }
        buf.push(ch);

        let mut buf_sig = self.state.typeahead_buffer;
        buf_sig.set(buf.clone());
        let mut time_sig = self.state.last_key_timestamp_ms;
        time_sig.set(now_ms);

        let prefix = buf.to_lowercase();
        let items = self.state.items.read();
        if let Some(matched) = items
            .iter()
            .find(|i| !i.disabled && i.text.to_lowercase().starts_with(&prefix))
        {
            self.set_highlighted(Some(matched.value.clone()));
        }
    }

    pub fn handle_closed_typeahead(&self, ch: char, now_ms: f64) {
        let last_time = *self.state.last_key_timestamp_ms.read();
        let mut buf = self.state.typeahead_buffer.read().clone();
        if now_ms - last_time > 500.0 {
            buf.clear();
        }
        buf.push(ch);

        let mut buf_sig = self.state.typeahead_buffer;
        buf_sig.set(buf.clone());
        let mut time_sig = self.state.last_key_timestamp_ms;
        time_sig.set(now_ms);

        let prefix = buf.to_lowercase();
        let items = self.state.items.read();
        let matched = items
            .iter()
            .find(|i| {
                !i.disabled
                    && (i.text.to_lowercase().starts_with(&prefix)
                        || i.value.to_lowercase().starts_with(&prefix))
            })
            .map(|i| i.value.clone())
            .or_else(|| {
                self.select
                    .items()
                    .iter()
                    .find(|i| {
                        !i.disabled
                            && (i.text.to_lowercase().starts_with(&prefix)
                                || i.value.to_lowercase().starts_with(&prefix))
                    })
                    .map(|i| i.value.clone())
            });

        if let Some(val) = matched {
            self.select_item(&val);
        }
    }

    pub fn handle_trigger_keydown(&self, event: &KeyboardEvent) {
        if self.is_disabled() || self.is_open() {
            return;
        }
        #[cfg(target_arch = "wasm32")]
        let now_ms = js_sys::Date::now();
        #[cfg(not(target_arch = "wasm32"))]
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs_f64() * 1000.0)
            .unwrap_or(0.0);

        let key = event.key().to_string();
        match key.as_str() {
            "ArrowDown" | "ArrowUp" | " " | "Enter" => {
                event.prevent_default();
                self.open_dropdown();
            }
            _ => {
                if key.len() == 1 {
                    let ch = key.chars().next().unwrap();
                    if ch.is_alphanumeric() || ch.is_whitespace() {
                        event.prevent_default();
                        self.handle_closed_typeahead(ch, now_ms);
                    }
                }
            }
        }
    }
}
