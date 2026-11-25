use crate::foundation::overlay::PortalHost;

use super::{
    registry::ComboboxOptionRegistry,
    types::{ComboboxMode, ComboboxNavKey, ComboboxScrollAlignment},
};

pub const COMBOBOX_PAGE_STEP: isize = 10;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComboboxState {
    mode: ComboboxMode,
    disabled: bool,
    required: bool,
    loop_focus: bool,
    scroll_alignment: ComboboxScrollAlignment,
    allow_deselect: bool,
    clear_on_deselect: bool,
    name: Option<String>,
    form: Option<String>,
    portal_host: PortalHost,
}

impl Default for ComboboxState {
    fn default() -> Self {
        Self::new()
    }
}

impl ComboboxState {
    pub fn new() -> Self {
        Self {
            mode: ComboboxMode::Single,
            disabled: false,
            required: false,
            loop_focus: false,
            scroll_alignment: ComboboxScrollAlignment::Nearest,
            allow_deselect: true,
            clear_on_deselect: false,
            name: None,
            form: None,
            portal_host: PortalHost::default(),
        }
    }

    pub fn with_mode(mut self, mode: ComboboxMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn with_loop_focus(mut self, loop_focus: bool) -> Self {
        self.loop_focus = loop_focus;
        self
    }

    pub fn with_scroll_alignment(mut self, scroll_alignment: ComboboxScrollAlignment) -> Self {
        self.scroll_alignment = scroll_alignment;
        self
    }

    pub fn with_allow_deselect(mut self, allow_deselect: bool) -> Self {
        self.allow_deselect = allow_deselect;
        self
    }

    pub fn with_clear_on_deselect(mut self, clear_on_deselect: bool) -> Self {
        self.clear_on_deselect = clear_on_deselect;
        self
    }

    pub fn with_name(mut self, name: Option<String>) -> Self {
        self.name = name;
        self
    }

    pub fn with_form(mut self, form: Option<String>) -> Self {
        self.form = form;
        self
    }

    pub fn with_portal_host(mut self, portal_host: PortalHost) -> Self {
        self.portal_host = portal_host;
        self
    }

    pub const fn mode(&self) -> ComboboxMode {
        self.mode
    }

    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub const fn is_required(&self) -> bool {
        self.required
    }

    pub const fn loop_focus(&self) -> bool {
        self.loop_focus
    }

    pub const fn scroll_alignment(&self) -> ComboboxScrollAlignment {
        self.scroll_alignment
    }

    pub const fn allow_deselect(&self) -> bool {
        self.allow_deselect
    }

    pub const fn clear_on_deselect(&self) -> bool {
        self.clear_on_deselect
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn form(&self) -> Option<&str> {
        self.form.as_deref()
    }

    pub fn portal_host(&self) -> &PortalHost {
        &self.portal_host
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComboboxSingleToggleOutcome {
    pub next_value: Option<String>,
    pub next_input_value: Option<String>,
    pub should_close: bool,
    pub changed: bool,
}

pub fn apply_single_toggle(
    current: Option<&str>,
    target_value: &str,
    target_label: &str,
    allow_deselect: bool,
    clear_on_deselect: bool,
) -> ComboboxSingleToggleOutcome {
    if current == Some(target_value) {
        if allow_deselect {
            ComboboxSingleToggleOutcome {
                next_value: None,
                next_input_value: clear_on_deselect.then(String::new),
                should_close: true,
                changed: true,
            }
        } else {
            ComboboxSingleToggleOutcome {
                next_value: Some(target_value.to_string()),
                next_input_value: None,
                should_close: true,
                changed: false,
            }
        }
    } else {
        ComboboxSingleToggleOutcome {
            next_value: Some(target_value.to_string()),
            next_input_value: Some(target_label.to_string()),
            should_close: true,
            changed: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComboboxMultipleToggleOutcome {
    pub next_values: Vec<String>,
    pub next_input_value: Option<String>,
    pub changed: bool,
}

pub fn apply_multiple_toggle(
    current: &[String],
    target_value: &str,
    target_label: &str,
    clear_on_deselect: bool,
) -> ComboboxMultipleToggleOutcome {
    let mut next_values = current.to_vec();
    if let Some(pos) = next_values.iter().position(|val| val == target_value) {
        next_values.remove(pos);
        let next_input_value = if next_values.is_empty() && clear_on_deselect {
            Some(String::new())
        } else {
            None
        };
        ComboboxMultipleToggleOutcome {
            next_values,
            next_input_value,
            changed: true,
        }
    } else {
        next_values.push(target_value.to_string());
        ComboboxMultipleToggleOutcome {
            next_values,
            next_input_value: Some(target_label.to_string()),
            changed: true,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ComboboxKeyOutcome {
    pub prevent_default: bool,
    pub next_open: Option<bool>,
    pub next_highlighted: Option<Option<String>>,
    pub trigger_select: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn resolve_input_keydown(
    registry: &ComboboxOptionRegistry,
    is_open: bool,
    highlighted: Option<&str>,
    selected_value: Option<&str>,
    key: ComboboxNavKey,
    is_composing: bool,
    loop_focus: bool,
    disabled: bool,
) -> ComboboxKeyOutcome {
    if disabled {
        return ComboboxKeyOutcome::default();
    }

    match key {
        // Home and End pass through to the native `<input>` caret.
        ComboboxNavKey::Home | ComboboxNavKey::End => ComboboxKeyOutcome::default(),

        ComboboxNavKey::ArrowDown => {
            if !is_open {
                let candidate = selected_value
                    .filter(|val| registry.contains_enabled(val))
                    .map(str::to_owned)
                    .or_else(|| registry.first_enabled().map(|item| item.value.clone()));
                ComboboxKeyOutcome {
                    prevent_default: true,
                    next_open: Some(true),
                    next_highlighted: Some(candidate),
                    trigger_select: false,
                }
            } else {
                let next = registry
                    .next_enabled(highlighted, loop_focus)
                    .map(|item| item.value.clone());
                ComboboxKeyOutcome {
                    prevent_default: true,
                    next_open: None,
                    next_highlighted: Some(next),
                    trigger_select: false,
                }
            }
        }

        ComboboxNavKey::ArrowUp => {
            if !is_open {
                let candidate = selected_value
                    .filter(|val| registry.contains_enabled(val))
                    .map(str::to_owned)
                    .or_else(|| registry.last_enabled().map(|item| item.value.clone()));
                ComboboxKeyOutcome {
                    prevent_default: true,
                    next_open: Some(true),
                    next_highlighted: Some(candidate),
                    trigger_select: false,
                }
            } else {
                let prev = registry
                    .prev_enabled(highlighted, loop_focus)
                    .map(|item| item.value.clone());
                ComboboxKeyOutcome {
                    prevent_default: true,
                    next_open: None,
                    next_highlighted: Some(prev),
                    trigger_select: false,
                }
            }
        }

        ComboboxNavKey::PageDown => {
            if !is_open {
                return ComboboxKeyOutcome::default();
            }
            let next = registry
                .step_enabled(highlighted, COMBOBOX_PAGE_STEP)
                .map(|item| item.value.clone());
            ComboboxKeyOutcome {
                prevent_default: true,
                next_open: None,
                next_highlighted: Some(next),
                trigger_select: false,
            }
        }

        ComboboxNavKey::PageUp => {
            if !is_open {
                return ComboboxKeyOutcome::default();
            }
            let prev = registry
                .step_enabled(highlighted, -COMBOBOX_PAGE_STEP)
                .map(|item| item.value.clone());
            ComboboxKeyOutcome {
                prevent_default: true,
                next_open: None,
                next_highlighted: Some(prev),
                trigger_select: false,
            }
        }

        ComboboxNavKey::AltArrowUp => {
            if !is_open {
                return ComboboxKeyOutcome::default();
            }
            ComboboxKeyOutcome {
                prevent_default: true,
                next_open: Some(false),
                next_highlighted: None,
                trigger_select: false,
            }
        }

        ComboboxNavKey::Tab => {
            if !is_open {
                return ComboboxKeyOutcome::default();
            }
            // Do NOT prevent default so Tab advances focus naturally.
            ComboboxKeyOutcome {
                prevent_default: false,
                next_open: Some(false),
                next_highlighted: None,
                trigger_select: false,
            }
        }

        ComboboxNavKey::Escape => {
            if !is_open {
                return ComboboxKeyOutcome::default();
            }
            // Prevent default so Blitz Native does not clear `<input>` focus on Escape.
            ComboboxKeyOutcome {
                prevent_default: true,
                next_open: Some(false),
                next_highlighted: None,
                trigger_select: false,
            }
        }

        ComboboxNavKey::Enter => {
            if is_composing || !is_open {
                return ComboboxKeyOutcome::default();
            }
            ComboboxKeyOutcome {
                prevent_default: true,
                next_open: None,
                next_highlighted: None,
                trigger_select: highlighted.is_some(),
            }
        }

        ComboboxNavKey::Printable => {
            if !is_open {
                ComboboxKeyOutcome {
                    prevent_default: false,
                    next_open: Some(true),
                    next_highlighted: None,
                    trigger_select: false,
                }
            } else {
                ComboboxKeyOutcome::default()
            }
        }
    }
}

/// Computes the target `scrollTop` for aligning an option item inside a scrollable viewport.
/// Returns `None` when the item is already aligned or dimensions are non-positive.
#[allow(clippy::too_many_arguments)]
pub fn compute_viewport_scroll_target(
    container_top: f64,
    container_bottom: f64,
    container_height: f64,
    current_scroll_top: f64,
    item_top: f64,
    item_bottom: f64,
    item_height: f64,
    alignment: ComboboxScrollAlignment,
) -> Option<f64> {
    if container_height <= 0.0 || item_height <= 0.0 {
        return None;
    }

    match alignment {
        ComboboxScrollAlignment::Nearest => {
            const EDGE_INSET: f64 = 1.0;
            let visible_bottom = container_bottom - EDGE_INSET;
            if item_top < container_top {
                Some((current_scroll_top + (item_top - container_top)).max(0.0))
            } else if item_bottom > visible_bottom {
                Some((current_scroll_top + (item_bottom - visible_bottom)).max(0.0))
            } else {
                None
            }
        }
        ComboboxScrollAlignment::Center => {
            let item_center = item_top + (item_height / 2.0);
            let container_center = container_top + (container_height / 2.0);
            let delta = item_center - container_center;
            if delta.abs() < 0.5 {
                None
            } else {
                Some((current_scroll_top + delta).max(0.0))
            }
        }
    }
}
