use crate::foundation::{overlay::GeometryVars, shared::Direction};
use crate::menu::{MenuKey, MenuVariant, menu_css_vars};

use super::{
    attrs::{MenubarRootAttributes, MenubarTriggerAttributes},
    relationships::MenubarRelationships,
    types::{
        MENUBAR_DEFAULT_LOOP_FOCUS, MenubarCrossMenuResult, MenubarMenuEntry, MenubarNavigationStep,
    },
};

/// Pure headless state machine coordinating top-level `Menubar` focus, hover swap, and cross-menu arrows (`MB-1`..`MB-7`).
#[derive(Clone, Debug, PartialEq)]
pub struct MenubarState {
    relationships: MenubarRelationships,
    active_value: Option<String>,
    current_tabstop_value: Option<String>,
    highlighted_value: Option<String>,
    direction: Direction,
    loop_focus: bool,
    menus: Vec<MenubarMenuEntry>,
}

impl MenubarState {
    pub fn new(relationships: MenubarRelationships) -> Self {
        Self {
            relationships,
            active_value: None,
            current_tabstop_value: None,
            highlighted_value: None,
            direction: Direction::Ltr,
            loop_focus: MENUBAR_DEFAULT_LOOP_FOCUS,
            menus: Vec::new(),
        }
    }

    pub fn with_active_value(mut self, active_value: Option<String>) -> Self {
        self.active_value = active_value;
        self
    }

    pub fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn with_loop_focus(mut self, loop_focus: bool) -> Self {
        self.loop_focus = loop_focus;
        self
    }

    pub fn relationships(&self) -> &MenubarRelationships {
        &self.relationships
    }

    pub fn active_value(&self) -> Option<&str> {
        self.active_value.as_deref()
    }

    pub fn set_active_value(&mut self, value: Option<String>) {
        self.active_value = value;
    }

    pub fn current_tabstop_value(&self) -> Option<&str> {
        self.current_tabstop_value.as_deref()
    }

    pub fn set_current_tabstop_value(&mut self, value: Option<String>) {
        self.current_tabstop_value = value;
    }

    pub fn highlighted_value(&self) -> Option<&str> {
        self.highlighted_value.as_deref()
    }

    pub fn set_highlighted_value(&mut self, value: Option<String>) {
        self.highlighted_value = value;
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn loop_focus(&self) -> bool {
        self.loop_focus
    }

    pub fn menus(&self) -> &[MenubarMenuEntry] {
        &self.menus
    }

    pub fn register_menu(&mut self, entry: MenubarMenuEntry) {
        if let Some(existing) = self.menus.iter_mut().find(|m| m.value == entry.value) {
            *existing = entry;
        } else {
            self.menus.push(entry);
        }
        self.menus.sort_by_key(|m| m.dom_order);
        self.ensure_valid_tabstop();
    }

    pub fn unregister_menu(&mut self, value: &str) {
        self.menus.retain(|m| m.value != value);
        if self.active_value.as_deref() == Some(value) {
            self.active_value = None;
        }
        if self.current_tabstop_value.as_deref() == Some(value) {
            self.current_tabstop_value = None;
        }
        if self.highlighted_value.as_deref() == Some(value) {
            self.highlighted_value = None;
        }
        self.ensure_valid_tabstop();
    }

    pub fn sync_dom_order(&mut self, ordered_trigger_ids: &[String]) {
        for entry in &mut self.menus {
            if let Some(pos) = ordered_trigger_ids.iter().position(|id| id == &entry.trigger_id) {
                entry.dom_order = pos;
            }
        }
        self.menus.sort_by_key(|m| m.dom_order);
        self.ensure_valid_tabstop();
    }

    fn ensure_valid_tabstop(&mut self) {
        let valid = self.current_tabstop_value.as_deref().is_some_and(|val| {
            self.menus.iter().any(|m| m.value == val && !m.disabled)
        });
        if !valid {
            self.current_tabstop_value = self
                .menus
                .iter()
                .find(|m| !m.disabled)
                .map(|m| m.value.clone());
        }
    }

    pub fn is_tabstop(&self, value: &str) -> bool {
        if let Some(ref current) = self.current_tabstop_value {
            current == value
        } else {
            self.menus
                .iter()
                .find(|m| !m.disabled)
                .is_some_and(|first| first.value == value)
        }
    }

    /// Open `value`, returning `(previous_active_value, skip_exit_animation)` (`MB-5`).
    pub fn open_menu(&mut self, value: &str) -> (Option<String>, bool) {
        let previous = self.active_value.take();
        let skip_exit_animation = previous.as_deref().is_some_and(|prev| prev != value);
        self.active_value = Some(value.to_owned());
        self.current_tabstop_value = Some(value.to_owned());
        (previous, skip_exit_animation)
    }

    pub fn close_menu(&mut self, value: &str) -> bool {
        if self.active_value.as_deref() == Some(value) {
            self.active_value = None;
            true
        } else {
            false
        }
    }

    pub fn close_all(&mut self) {
        self.active_value = None;
    }

    pub fn is_trigger_element_id(&self, id: &str) -> bool {
        self.menus.iter().any(|m| m.trigger_id == id)
    }

    pub fn first_enabled_menu(&self) -> Option<&MenubarMenuEntry> {
        self.menus.iter().find(|m| !m.disabled)
    }

    pub fn last_enabled_menu(&self) -> Option<&MenubarMenuEntry> {
        self.menus.iter().rev().find(|m| !m.disabled)
    }

    pub fn adjacent_enabled_menu(
        &self,
        current_value: &str,
        forward: bool,
    ) -> Option<&MenubarMenuEntry> {
        let enabled: Vec<&MenubarMenuEntry> = self.menus.iter().filter(|m| !m.disabled).collect();
        if enabled.is_empty() {
            return None;
        }
        let pos = enabled.iter().position(|m| m.value == current_value)?;
        if forward {
            if pos + 1 < enabled.len() {
                Some(enabled[pos + 1])
            } else if self.loop_focus {
                Some(enabled[0])
            } else {
                None
            }
        } else if pos > 0 {
            Some(enabled[pos - 1])
        } else if self.loop_focus {
            Some(enabled[enabled.len() - 1])
        } else {
            None
        }
    }

    /// Hover-switch rule (`MB-3`): hovering a sibling trigger swaps active menu ONLY when one is already open.
    pub fn on_trigger_pointer_enter(&self, value: &str) -> Option<&MenubarMenuEntry> {
        if self.active_value.is_none() || self.active_value.as_deref() == Some(value) {
            return None;
        }
        self.menus.iter().find(|m| m.value == value && !m.disabled)
    }

    /// Resolve keyboard navigation when focus is on a `Menubar.Trigger` (`MB-2`).
    pub fn on_trigger_key(&self, current_value: &str, key: MenuKey) -> MenubarNavigationStep {
        let is_open = self.active_value.is_some();
        let next_key = match self.direction {
            Direction::Ltr => MenuKey::ArrowRight,
            Direction::Rtl => MenuKey::ArrowLeft,
        };
        let prev_key = match self.direction {
            Direction::Ltr => MenuKey::ArrowLeft,
            Direction::Rtl => MenuKey::ArrowRight,
        };

        if key == MenuKey::Escape {
            return MenubarNavigationStep::CloseAll;
        }

        if matches!(key, MenuKey::Enter | MenuKey::Space | MenuKey::ArrowDown) {
            return MenubarNavigationStep::OpenCurrentAndFocusFirst {
                value: current_value.to_owned(),
            };
        }

        if matches!(key, MenuKey::ArrowUp) {
            return MenubarNavigationStep::OpenCurrentAndFocusLast {
                value: current_value.to_owned(),
            };
        }

        let target = if key == next_key {
            self.adjacent_enabled_menu(current_value, true)
        } else if key == prev_key {
            self.adjacent_enabled_menu(current_value, false)
        } else if key == MenuKey::Home {
            self.first_enabled_menu()
        } else if key == MenuKey::End {
            self.last_enabled_menu()
        } else {
            None
        };

        let Some(entry) = target else {
            return MenubarNavigationStep::None;
        };

        if is_open {
            MenubarNavigationStep::FocusAndOpenMenu {
                value: entry.value.clone(),
                trigger_id: entry.trigger_id.clone(),
            }
        } else {
            MenubarNavigationStep::FocusTrigger {
                value: entry.value.clone(),
                trigger_id: entry.trigger_id.clone(),
            }
        }
    }

    /// Resolve cross-menu horizontal arrow navigation from inside an open `Menubar.Content` or `SubContent` (`MB-6`).
    pub fn resolve_cross_menu_arrow(
        &self,
        current_value: &str,
        key: MenuKey,
        on_sub_trigger: bool,
        in_submenu: bool,
    ) -> MenubarCrossMenuResult {
        let forward_key = match self.direction {
            Direction::Ltr => MenuKey::ArrowRight,
            Direction::Rtl => MenuKey::ArrowLeft,
        };
        let backward_key = match self.direction {
            Direction::Ltr => MenuKey::ArrowLeft,
            Direction::Rtl => MenuKey::ArrowRight,
        };

        let next_entry = if key == forward_key && !on_sub_trigger {
            self.adjacent_enabled_menu(current_value, true)
        } else if key == backward_key && !in_submenu {
            self.adjacent_enabled_menu(current_value, false)
        } else {
            None
        };

        match next_entry {
            Some(entry) if entry.value != current_value => MenubarCrossMenuResult::SwitchToMenu {
                value: entry.value.clone(),
                trigger_id: entry.trigger_id.clone(),
            },
            _ => MenubarCrossMenuResult::None,
        }
    }

    pub fn css_vars(geometry: &GeometryVars) -> Vec<(String, String)> {
        menu_css_vars(geometry, MenuVariant::Menubar)
    }

    pub fn root_attrs(&self) -> MenubarRootAttributes {
        MenubarRootAttributes::new(self.relationships.root_id().to_owned(), self.direction)
    }

    pub fn trigger_attrs(
        &self,
        value: &str,
        trigger_id: String,
        content_id: &str,
        disabled: bool,
    ) -> MenubarTriggerAttributes {
        let open = self.active_value.as_deref() == Some(value);
        let highlighted = !disabled && (open || self.highlighted_value.as_deref() == Some(value));
        let tabstop = !disabled && self.is_tabstop(value);
        MenubarTriggerAttributes::from_parts(
            trigger_id,
            content_id,
            open,
            disabled,
            highlighted,
            tabstop,
        )
    }
}
