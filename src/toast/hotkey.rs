use crate::toast::runtime::TOAST_STORE;
use dioxus::prelude::*;

/// Checks if a keyboard event key matches the configured landmark hotkey.
///
/// Supports exact and case-insensitive matching (e.g. "F8" matches "f8" or "F8").
pub fn matches_landmark_hotkey(pressed_key: &str, configured_hotkey: &str) -> bool {
    if pressed_key.eq_ignore_ascii_case(configured_hotkey) {
        return true;
    }
    // Also match normalized variants like "KeyF8" vs "F8"
    let stripped_pressed = pressed_key.strip_prefix("Key").unwrap_or(pressed_key);
    let stripped_config = configured_hotkey.strip_prefix("Key").unwrap_or(configured_hotkey);
    stripped_pressed.eq_ignore_ascii_case(stripped_config)
}

/// Evaluates a keydown event against the toast landmark hotkey configuration.
///
/// Returns `Some(viewport_id)` if the hotkey matches and landmark focus should be triggered,
/// or `None` if the key is unrelated.
pub fn evaluate_landmark_hotkey(
    pressed_key: &str,
    configured_hotkey: &str,
    viewport_id: &str,
) -> Option<String> {
    if matches_landmark_hotkey(pressed_key, configured_hotkey) {
        Some(viewport_id.to_string())
    } else {
        None
    }
}

/// Focuses the landmark viewport container element by ID.
///
/// Dispatches through `crate::foundation::browser::focus_element_by_id`, which handles
/// immediate focus as well as next-frame retries for substrate resilience.
pub fn focus_landmark_viewport(viewport_id: &str) -> bool {
    crate::foundation::browser::focus_element_by_id(viewport_id);

    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Some(doc) = window.document() {
                if let Some(el) = doc.get_element_by_id(viewport_id) {
                    use wasm_bindgen::JsCast;
                    if let Ok(html_el) = el.dyn_into::<web_sys::HtmlElement>() {
                        let _ = html_el.focus();
                    }
                }
            }
        }
    }

    true
}

/// Handles a keydown event for toast landmark navigation.
///
/// If `pressed_key` matches `configured_hotkey`, focuses `viewport_id` and returns `true`.
/// Otherwise returns `false`.
pub fn handle_landmark_keydown(
    pressed_key: &str,
    configured_hotkey: &str,
    viewport_id: &str,
) -> bool {
    if evaluate_landmark_hotkey(pressed_key, configured_hotkey, viewport_id).is_some() {
        focus_landmark_viewport(viewport_id);
        true
    } else {
        false
    }
}

/// Transition outcome of a document visibility change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisibilityTransition {
    /// Document was hidden and active countdown timers were paused.
    Paused,
    /// Document became visible and paused timers were resumed.
    Resumed,
    /// Visibility state was unchanged or pausing on hidden page is disabled.
    Unchanged,
}

/// State machine managing document/tab visibility changes and pausing toast timers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToastVisibilityManager {
    pub page_hidden: bool,
    pub pause_when_page_is_hidden: bool,
}

impl ToastVisibilityManager {
    pub fn new(pause_when_page_is_hidden: bool) -> Self {
        Self {
            page_hidden: false,
            pause_when_page_is_hidden,
        }
    }

    pub fn with_hidden(mut self, hidden: bool) -> Self {
        self.page_hidden = hidden;
        self
    }

    /// Processes a visibility change event (`is_visible: true` if document is visible, `false` if hidden).
    ///
    /// Computes whether timers should transition to paused or resumed according to `pause_when_page_is_hidden`.
    pub fn handle_visibility_change(&mut self, is_visible: bool) -> VisibilityTransition {
        let hidden = !is_visible;
        if self.page_hidden == hidden {
            return VisibilityTransition::Unchanged;
        }

        self.page_hidden = hidden;

        if !self.pause_when_page_is_hidden {
            return VisibilityTransition::Unchanged;
        }

        if hidden {
            VisibilityTransition::Paused
        } else {
            VisibilityTransition::Resumed
        }
    }
}

/// Applies a document visibility change directly to the global `TOAST_STORE`.
///
/// Updates the reactive store's `interaction_state.page_hidden` flag, triggering
/// countdown timer suspension when `pause_when_page_is_hidden` is active.
pub fn apply_visibility_change(
    is_visible: bool,
    pause_when_page_is_hidden: bool,
) -> VisibilityTransition {
    let mut manager = ToastVisibilityManager::new(pause_when_page_is_hidden)
        .with_hidden(TOAST_STORE.peek().interaction_state.page_hidden);

    let transition = manager.handle_visibility_change(is_visible);
    if transition != VisibilityTransition::Unchanged {
        TOAST_STORE.write().set_page_hidden(!is_visible);
    }
    transition
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_landmark_hotkey_matching() {
        assert!(matches_landmark_hotkey("F8", "F8"));
        assert!(matches_landmark_hotkey("f8", "F8"));
        assert!(matches_landmark_hotkey("F8", "f8"));
        assert!(matches_landmark_hotkey("KeyF8", "F8"));
        assert!(matches_landmark_hotkey("F8", "KeyF8"));

        assert!(!matches_landmark_hotkey("F9", "F8"));
        assert!(!matches_landmark_hotkey("Escape", "F8"));
        assert!(!matches_landmark_hotkey("Enter", "F8"));

        // Custom hotkeys
        assert!(matches_landmark_hotkey("KeyT", "KeyT"));
        assert!(matches_landmark_hotkey("t", "KeyT"));
        assert!(!matches_landmark_hotkey("k", "KeyT"));
    }

    #[test]
    fn test_landmark_hotkey_evaluation() {
        let viewport_id = "custom-viewport-landmark";
        assert_eq!(
            evaluate_landmark_hotkey("F8", "F8", viewport_id),
            Some(viewport_id.to_string())
        );
        assert_eq!(
            evaluate_landmark_hotkey("f8", "F8", viewport_id),
            Some(viewport_id.to_string())
        );
        assert_eq!(evaluate_landmark_hotkey("F9", "F8", viewport_id), None);
    }

    #[test]
    fn test_visibility_state_transitions() {
        let mut manager = ToastVisibilityManager::new(true);
        assert!(!manager.page_hidden);

        // Document hidden -> transitions to Paused
        let trans1 = manager.handle_visibility_change(false);
        assert_eq!(trans1, VisibilityTransition::Paused);
        assert!(manager.page_hidden);

        // Repeated hidden event is idempotent -> Unchanged
        let trans2 = manager.handle_visibility_change(false);
        assert_eq!(trans2, VisibilityTransition::Unchanged);

        // Document visible -> transitions to Resumed
        let trans3 = manager.handle_visibility_change(true);
        assert_eq!(trans3, VisibilityTransition::Resumed);
        assert!(!manager.page_hidden);

        // Repeated visible event is idempotent -> Unchanged
        let trans4 = manager.handle_visibility_change(true);
        assert_eq!(trans4, VisibilityTransition::Unchanged);
    }

    #[test]
    fn test_visibility_pause_disabled() {
        let mut manager = ToastVisibilityManager::new(false);

        // When pause_when_page_is_hidden is false, transitions remain Unchanged
        let trans1 = manager.handle_visibility_change(false);
        assert_eq!(trans1, VisibilityTransition::Unchanged);
        assert!(manager.page_hidden);

        let trans2 = manager.handle_visibility_change(true);
        assert_eq!(trans2, VisibilityTransition::Unchanged);
        assert!(!manager.page_hidden);
    }
}
