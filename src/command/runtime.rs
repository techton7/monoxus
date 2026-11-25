use std::collections::BTreeMap;

use dioxus::prelude::keyboard_types::{Key, NamedKey};
use dioxus::prelude::*;

use crate::foundation::browser::scroll_element_into_view_nearest;

use super::{
    relationships::CommandRelationships,
    state::{
        CommandRegistry, compute_command_score, resolve_command_navigation, select_by_group,
        select_by_index, select_by_item, select_first_valid_item, should_render_group,
        should_render_item,
    },
    types::{CommandNavKey, CommandNavModifiers, CommandStateSnapshot},
};

pub type CommandFilterCallback = Callback<(String, String, Vec<String>), f64>;

#[derive(Clone, Copy)]
pub struct CommandContext {
    pub relationships: Signal<CommandRelationships>,
    pub registry: Signal<CommandRegistry>,
    pub search: Signal<String>,
    pub value: Signal<String>,
    pub on_value_change: Signal<Option<EventHandler<String>>>,
    pub on_state_change: Signal<Option<EventHandler<CommandStateSnapshot>>>,
    pub should_filter: Signal<bool>,
    pub filter: Signal<Option<CommandFilterCallback>>,
    pub loop_focus: Signal<bool>,
    pub vim_bindings: Signal<bool>,
    pub disable_pointer_selection: Signal<bool>,
    pub disable_initial_scroll: Signal<bool>,
    pub columns: Signal<Option<usize>>,
    pub label: Signal<String>,
    pub input_id: Signal<String>,
    pub label_id: Signal<String>,
    pub list_id: Signal<String>,
    pub viewport_id: Signal<String>,
    pub list_height: Signal<Option<f64>>,
    pub settled: Signal<bool>,
    pub is_composing: Signal<bool>,
    pub has_scrolled_initially: Signal<bool>,
    pub select_handlers: Signal<BTreeMap<String, EventHandler<()>>>,
}

#[derive(Clone, Copy)]
pub struct CommandGroupContext {
    pub value: Signal<String>,
    pub group_id: Signal<String>,
    pub heading_id: Signal<Option<String>>,
    pub force_mount: Signal<bool>,
}

impl CommandContext {
    pub fn recompute_filtered(&mut self) {
        let search = self.search.peek().clone();
        let should_filter = *self.should_filter.peek();
        let custom_filter = *self.filter.peek();

        {
            let mut reg = self.registry.write();
            if let Some(cb) = custom_filter {
                reg.recompute_filtered(&search, should_filter, |val, q, kws| {
                    cb.call((val.to_string(), q.to_string(), kws.to_vec()))
                });
            } else {
                reg.recompute_filtered(&search, should_filter, compute_command_score);
            }
        }

        self.ensure_valid_selection(true);
        self.emit_state_change();
    }

    pub fn emit_state_change(&self) {
        if let Some(handler) = *self.on_state_change.peek() {
            let snapshot = CommandStateSnapshot {
                search: self.search.peek().clone(),
                value: self.value.peek().clone(),
                filtered: self.registry.peek().filtered().clone(),
            };
            handler.call(snapshot);
        }
    }

    pub fn set_search(&mut self, next: String) {
        if *self.search.peek() == next {
            return;
        }
        self.search.set(next);
        self.recompute_filtered();
    }

    pub fn set_value(&mut self, next: String, prevent_scroll: bool) {
        let changed = *self.value.peek() != next;
        if changed {
            self.value.set(next.clone());
            if let Some(handler) = *self.on_value_change.peek() {
                handler.call(next.clone());
            }
            self.emit_state_change();
        }

        if !prevent_scroll && !next.is_empty() {
            let already_scrolled = *self.has_scrolled_initially.peek();
            let disable_init = *self.disable_initial_scroll.peek();
            if !already_scrolled {
                self.has_scrolled_initially.set(true);
                if disable_init {
                    return;
                }
            }
            self.scroll_selected_into_view(&next);
        }
    }

    fn scroll_selected_into_view(&self, value: &str) {
        let search = self.search.peek().clone();
        let should_filter = *self.should_filter.peek();
        let reg = self.registry.peek();
        if let Some(heading_id) = reg.first_in_group_heading_id(value, &search, should_filter) {
            scroll_element_into_view_nearest(heading_id);
        }
        if let Some(dom_id) = reg.item_dom_id(value) {
            scroll_element_into_view_nearest(dom_id);
        }
    }

    pub fn ensure_valid_selection(&mut self, prevent_scroll: bool) {
        let current = self.value.peek().clone();
        let search = self.search.peek().clone();
        let should_filter = *self.should_filter.peek();

        let is_current_valid = {
            let reg = self.registry.peek();
            !current.is_empty()
                && reg
                    .valid_items(&search, should_filter)
                    .iter()
                    .any(|it| it.value == current)
        };

        if !is_current_valid {
            let first = {
                let reg = self.registry.peek();
                select_first_valid_item(&reg, &search, should_filter).unwrap_or_default()
            };
            if *self.value.peek() != first {
                self.set_value(first, prevent_scroll);
            }
        }
    }

    pub fn register_item(
        &mut self,
        value: String,
        dom_id: String,
        group_value: Option<String>,
        disabled: bool,
        force_mount: bool,
        keywords: Vec<String>,
        on_select: Option<EventHandler<()>>,
    ) {
        let normalized = value.trim().to_string();
        if normalized.is_empty() {
            return;
        }
        if let Some(handler) = on_select {
            self.select_handlers
                .write()
                .insert(normalized.clone(), handler);
        } else {
            self.select_handlers.write().remove(&normalized);
        }

        let cleaned_keywords: Vec<String> = keywords
            .iter()
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty())
            .collect();

        let existing_matches = self.registry.peek().items().iter().any(|it| {
            it.value == normalized
                && it.dom_id == dom_id
                && it.group_value == group_value
                && it.disabled == disabled
                && it.force_mount == force_mount
                && it.keywords == cleaned_keywords
        });

        if !existing_matches {
            self.registry.write().register_item(
                normalized,
                dom_id,
                group_value,
                disabled,
                force_mount,
                cleaned_keywords,
            );
            self.recompute_filtered();
        }
    }

    pub fn unregister_item(&mut self, value: &str) {
        let normalized = value.trim().to_string();
        if let Ok(mut handlers) = self.select_handlers.try_write() {
            handlers.remove(&normalized);
        }
        let removed = if let Ok(mut reg) = self.registry.try_write() {
            reg.unregister_item(&normalized)
        } else {
            false
        };
        if removed {
            let search = self
                .search
                .try_peek()
                .map(|s| s.clone())
                .unwrap_or_default();
            let should_filter = self.should_filter.try_peek().map(|b| *b).unwrap_or(true);
            let custom_filter = self.filter.try_peek().ok().and_then(|f| *f);
            if let Ok(mut reg) = self.registry.try_write() {
                if let Some(cb) = custom_filter {
                    reg.recompute_filtered(&search, should_filter, |val, q, kws| {
                        cb.call((val.to_string(), q.to_string(), kws.to_vec()))
                    });
                } else {
                    reg.recompute_filtered(&search, should_filter, compute_command_score);
                }
            }
            let next_selection = if let (Ok(current), Ok(reg)) =
                (self.value.try_peek(), self.registry.try_peek())
            {
                let current_val = current.clone();
                let still_valid = !current_val.is_empty()
                    && reg
                        .valid_items(&search, should_filter)
                        .iter()
                        .any(|it| it.value == current_val);
                if !still_valid {
                    Some(
                        select_first_valid_item(&reg, &search, should_filter).unwrap_or_default(),
                    )
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(next) = next_selection {
                if let Ok(mut val_sig) = self.value.try_write() {
                    *val_sig = next.clone();
                }
                if let Ok(handler_opt) = self.on_value_change.try_peek() {
                    if let Some(handler) = *handler_opt {
                        handler.call(next);
                    }
                }
            }
        }
    }

    pub fn register_group(
        &mut self,
        value: String,
        dom_id: String,
        heading_id: Option<String>,
        force_mount: bool,
    ) {
        let normalized = value.trim().to_string();
        if normalized.is_empty() {
            return;
        }
        let existing_matches = self.registry.peek().groups().iter().any(|g| {
            g.value == normalized
                && g.dom_id == dom_id
                && (heading_id.is_none() || g.heading_id == heading_id)
                && g.force_mount == force_mount
        });
        if !existing_matches {
            self.registry
                .write()
                .register_group(normalized, dom_id, heading_id, force_mount);
            self.recompute_filtered();
        }
    }

    pub fn unregister_group(&mut self, value: &str) {
        if let Ok(mut reg) = self.registry.try_write() {
            reg.unregister_group(value);
        }
    }

    pub fn set_group_heading(&mut self, group_value: &str, heading_id: Option<String>) {
        let already_matches = self
            .registry
            .try_peek()
            .ok()
            .is_some_and(|reg| {
                reg.groups()
                    .iter()
                    .find(|g| g.value == group_value.trim())
                    .is_some_and(|g| g.heading_id == heading_id)
            });
        if !already_matches {
            if let Ok(mut reg) = self.registry.try_write() {
                reg.set_group_heading(group_value, heading_id);
            }
        }
    }

    pub fn selected_item_dom_id(&self) -> Option<String> {
        let val = self.value.read().clone();
        if val.is_empty() {
            return None;
        }
        let search = self.search.read().clone();
        let should_filter = *self.should_filter.read();
        let reg = self.registry.read();
        reg.ranked_visible_items(&search, should_filter)
            .into_iter()
            .find(|it| it.value == val)
            .map(|it| it.dom_id.clone())
    }

    pub fn should_render_item_value(
        &self,
        value: &str,
        item_force_mount: bool,
        group_force_mount: bool,
    ) -> bool {
        let search = self.search.read();
        let should_filter = *self.should_filter.read();
        let reg = self.registry.read();
        let score = reg.item_score(value);
        should_render_item(
            item_force_mount,
            group_force_mount,
            should_filter,
            &search,
            score,
        )
    }

    pub fn should_render_group_value(&self, group_value: &str, group_force_mount: bool) -> bool {
        let search = self.search.read();
        let should_filter = *self.should_filter.read();
        let reg = self.registry.read();
        let visible = reg.is_group_visible(group_value);
        should_render_group(group_force_mount, should_filter, &search, visible)
    }

    pub fn valid_items(&self) -> Vec<String> {
        let search = self.search.peek().clone();
        let should_filter = *self.should_filter.peek();
        self.registry
            .peek()
            .valid_items(&search, should_filter)
            .into_iter()
            .map(|it| it.value.clone())
            .collect()
    }

    pub fn select_index(&mut self, index: usize) {
        let search = self.search.peek().clone();
        let should_filter = *self.should_filter.peek();
        let target = {
            let reg = self.registry.peek();
            select_by_index(&reg, &search, should_filter, index)
        };
        if let Some(next) = target {
            self.set_value(next, false);
        }
    }

    pub fn select_by_item(&mut self, delta: isize) {
        let search = self.search.peek().clone();
        let should_filter = *self.should_filter.peek();
        let current = self.value.peek().clone();
        let loop_focus = *self.loop_focus.peek();
        let target = {
            let reg = self.registry.peek();
            select_by_item(&reg, &search, should_filter, &current, delta, loop_focus)
        };
        if let Some(next) = target {
            self.set_value(next, false);
        }
    }

    pub fn select_by_group(&mut self, delta: isize) {
        let search = self.search.peek().clone();
        let should_filter = *self.should_filter.peek();
        let current = self.value.peek().clone();
        let loop_focus = *self.loop_focus.peek();
        let target = {
            let reg = self.registry.peek();
            select_by_group(&reg, &search, should_filter, &current, delta, loop_focus)
        };
        if let Some(next) = target {
            self.set_value(next, false);
        }
    }

    pub fn activate_selected(&self) -> bool {
        let current = self.value.peek().clone();
        if current.is_empty() {
            return false;
        }
        let search = self.search.peek().clone();
        let should_filter = *self.should_filter.peek();
        let is_valid = self
            .registry
            .peek()
            .valid_items(&search, should_filter)
            .iter()
            .any(|it| it.value == current);
        if !is_valid {
            return false;
        }
        if let Some(handler) = self.select_handlers.peek().get(&current).copied() {
            handler.call(());
            return true;
        }
        true
    }

    pub fn handle_keydown(&mut self, evt: &KeyboardEvent) {
        let mods = evt.modifiers();
        let nav_mods = CommandNavModifiers::new(mods.meta(), mods.alt(), mods.ctrl());
        let vim_bindings = *self.vim_bindings.peek();
        let columns = *self.columns.peek();
        let is_grid = columns.is_some_and(|c| c > 0);

        match evt.key() {
            Key::Named(NamedKey::Enter) => {
                if !evt.data().is_composing() && !*self.is_composing.peek() {
                    evt.prevent_default();
                    self.activate_selected();
                }
                return;
            }
            _ => {}
        }

        let nav_key = match evt.key() {
            Key::Named(NamedKey::ArrowDown) => Some(CommandNavKey::ArrowDown),
            Key::Named(NamedKey::ArrowUp) => Some(CommandNavKey::ArrowUp),
            Key::Named(NamedKey::ArrowRight) if is_grid => Some(CommandNavKey::ArrowRight),
            Key::Named(NamedKey::ArrowLeft) if is_grid => Some(CommandNavKey::ArrowLeft),
            Key::Named(NamedKey::Home) => Some(CommandNavKey::Home),
            Key::Named(NamedKey::End) => Some(CommandNavKey::End),
            Key::Character(ref ch) if vim_bindings && nav_mods.ctrl => match ch.as_str() {
                "n" | "N" | "j" | "J" => Some(CommandNavKey::VimNext),
                "p" | "P" | "k" | "K" => Some(CommandNavKey::VimPrev),
                "l" | "L" if is_grid => Some(CommandNavKey::VimRight),
                "h" | "H" if is_grid => Some(CommandNavKey::VimLeft),
                _ => None,
            },
            _ => None,
        };

        let Some(key) = nav_key else {
            return;
        };

        evt.prevent_default();

        let search = self.search.peek().clone();
        let should_filter = *self.should_filter.peek();
        let current = self.value.peek().clone();
        let loop_focus = *self.loop_focus.peek();

        let target = {
            let reg = self.registry.peek();
            resolve_command_navigation(
                &reg,
                &search,
                should_filter,
                &current,
                key,
                nav_mods,
                columns,
                loop_focus,
                vim_bindings,
            )
        };

        if let Some(next) = target {
            self.set_value(next, false);
        }
    }
}
