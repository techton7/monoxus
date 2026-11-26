use crate::foundation::browser::{
    FocusOptions, blur_active_element, focus_element_sync_or_defer, is_element_focused_by_id,
    is_focus_within_element as foundation_is_focus_within_element, spawn_root_task,
};
use crate::toast::runtime::TOAST_STORE;
use crate::toast::types::ToastId;
use dioxus::prelude::*;

/// Checks whether active keyboard focus currently resides on or within `element_id`.
pub(crate) fn is_focus_within_element(element_id: &str) -> bool {
    foundation_is_focus_within_element(element_id)
}

fn is_focus_within_viewport_or_toast(viewport_id: &str) -> bool {
    if is_focus_within_element(viewport_id) {
        return true;
    }
    let Some(doc) = oxidase::dom::Document::current() else {
        return false;
    };
    let Some(active) = doc.active_element() else {
        return false;
    };
    if active
        .id()
        .or_else(|| active.get_attribute("id"))
        .is_some_and(|id| id.starts_with("monoxus-toast-"))
    {
        return true;
    }
    active.closest("[data-sonner-toast]").is_some()
        || active.closest("[data-sonner-toaster]").is_some()
}

/// Restores keyboard focus to the enclosing toast viewport landmark when a focused toast
/// (or interactive control inside it) is dismissed, or exits focus cleanly when no active
/// toasts remain.
#[allow(dead_code)]
pub(crate) fn restore_viewport_focus_if_within_toast(
    toast_id: ToastId,
    fallback_viewport_id: &str,
) {
    handle_post_dismiss_focus(Some(toast_id), fallback_viewport_id);
}

std::thread_local! {
    static POST_DISMISS_SEQ: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// Reconciles focus and interaction state after a toast (or the entire toast queue) is dismissed.
pub(crate) fn handle_post_dismiss_focus(
    dismissed_toast_id: Option<ToastId>,
    fallback_viewport_id: &str,
) {
    if dioxus::core::Runtime::try_current().is_none() {
        return;
    }

    let has_active_toasts = TOAST_STORE.peek().has_active_toasts();
    if has_active_toasts {
        let Some(toast_id) = dismissed_toast_id else {
            return;
        };
        let toast_dom_id = format!("monoxus-toast-{}", toast_id.0);
        let keyboard_focused = TOAST_STORE.peek().interaction_state.focused;
        if !keyboard_focused {
            if is_focus_within_element(&toast_dom_id) {
                let _ = blur_active_element();
            }
            return;
        }
        if is_focus_within_element(&toast_dom_id) {
            let viewport_id = resolve_enclosing_viewport_id(&toast_dom_id)
                .unwrap_or_else(|| fallback_viewport_id.to_string());
            focus_landmark_viewport(&viewport_id);
        } else if oxidase::dom::Document::current()
            .is_some_and(|doc| doc.element_by_id(fallback_viewport_id).is_none())
        {
            let fallback_vp = fallback_viewport_id.to_string();
            spawn_root_task(async move {
                let _ = oxidase::frame::next_frame().await;
                if TOAST_STORE.peek().has_active_toasts()
                    && is_focus_within_element(&toast_dom_id)
                {
                    let vp = resolve_enclosing_viewport_id(&toast_dom_id).unwrap_or(fallback_vp);
                    let opts = FocusOptions::with_focus_visible(true, Some(true));
                    focus_element_sync_or_defer(&vp, opts);
                    if is_focus_within_element(&vp) {
                        TOAST_STORE.write().set_focused(true);
                    }
                }
            });
        }
        return;
    }

    TOAST_STORE.write().clear_interaction_if_queue_empty();

    let viewport_id = dismissed_toast_id
        .and_then(|tid| resolve_enclosing_viewport_id(&format!("monoxus-toast-{}", tid.0)))
        .unwrap_or_else(|| fallback_viewport_id.to_string());

    let seq = POST_DISMISS_SEQ.with(|c| {
        let next = c.get().wrapping_add(1);
        c.set(next);
        next
    });

    if is_focus_within_viewport_or_toast(&viewport_id) {
        let prev_id = TOAST_STORE.peek().last_focused_element_id.clone();
        let focus_opts = FocusOptions::with_focus_visible(false, Some(false));
        let restored_sync = if let Some(prev) = prev_id.as_deref() {
            if let Some(doc) = oxidase::dom::Document::current()
                && let Some(el) = doc.element_by_id(prev)
            {
                let _ = el.focus_sync_with_options(focus_opts);
            }
            is_element_focused_by_id(prev)
        } else {
            let _ = blur_active_element();
            !is_focus_within_viewport_or_toast(&viewport_id)
        };
        if restored_sync {
            TOAST_STORE.write().last_focused_element_id = None;
        }
    }

    let vp_id = viewport_id;
    spawn_root_task(async move {
        let _ = oxidase::frame::next_frame().await;
        if POST_DISMISS_SEQ.with(|c| c.get()) != seq {
            return;
        }
        if !TOAST_STORE.peek().has_active_toasts() {
            TOAST_STORE.write().clear_interaction_if_queue_empty();
            if is_focus_within_viewport_or_toast(&vp_id) {
                let prev_id = TOAST_STORE.write().last_focused_element_id.take();
                let focus_opts = FocusOptions::with_focus_visible(false, Some(false));
                if let Some(prev) = prev_id.as_deref()
                    && let Some(doc) = oxidase::dom::Document::current()
                    && let Some(el) = doc.element_by_id(prev)
                {
                    let _ = el.focus_with_options(focus_opts).await;
                } else {
                    let _ = blur_active_element();
                }
            } else {
                TOAST_STORE.write().last_focused_element_id = None;
            }
        }
    });
}

pub(crate) fn resolve_enclosing_viewport_id(toast_dom_id: &str) -> Option<String> {
    let doc = oxidase::dom::Document::current()?;
    let toast_el = doc.element_by_id(toast_dom_id)?;
    let vp_el = toast_el.closest("[data-sonner-toaster]")?;
    vp_el
        .id()
        .or_else(|| vp_el.get_attribute("id"))
        .filter(|s| !s.is_empty())
}

/// Checks if a keyboard event key matches the configured landmark hotkey.
///
/// Supports exact and case-insensitive matching (e.g. "F8" matches "f8" or "F8").
pub fn matches_landmark_hotkey(pressed_key: &str, configured_hotkey: &str) -> bool {
    if pressed_key.eq_ignore_ascii_case(configured_hotkey) {
        return true;
    }
    let stripped_pressed = pressed_key.strip_prefix("Key").unwrap_or(pressed_key);
    let stripped_config = configured_hotkey
        .strip_prefix("Key")
        .unwrap_or(configured_hotkey);
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
/// Dispatches immediate focus where supported and schedules next-frame confirmation
/// when running inside an active Dioxus runtime.
pub fn focus_landmark_viewport(viewport_id: &str) -> bool {
    let opts = FocusOptions::with_focus_visible(true, Some(true));

    if dioxus::core::Runtime::try_current().is_some() {
        TOAST_STORE.write().clear_viewport_pointer_press();
    }

    focus_element_sync_or_defer(viewport_id, opts);

    if dioxus::core::Runtime::try_current().is_some() {
        if is_focus_within_element(viewport_id) && TOAST_STORE.peek().has_active_toasts() {
            TOAST_STORE.write().set_focused(true);
        }
        let target_id = viewport_id.to_string();
        spawn_root_task(async move {
            for _ in 0..5 {
                let _ = oxidase::frame::next_frame().await;
                if !TOAST_STORE.peek().has_active_toasts() && !TOAST_STORE.peek().toasts.is_empty()
                {
                    break;
                }
                let doc = oxidase::dom::Document::current().unwrap_or_else(oxidase::document);
                if let Some(el) = doc.element_by_id(&target_id)
                    && el.focus_with_options(opts).await.is_ok()
                {
                    if is_focus_within_element(&target_id)
                        && TOAST_STORE.peek().has_active_toasts()
                    {
                        TOAST_STORE.write().set_focused(true);
                    }
                    break;
                }
            }
        });
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
    handle_landmark_keydown_with_origin(pressed_key, configured_hotkey, viewport_id, None, false)
}

pub(crate) fn handle_landmark_keydown_with_origin(
    pressed_key: &str,
    configured_hotkey: &str,
    viewport_id: &str,
    origin_target_id: Option<&str>,
    origin_inside_viewport: bool,
) -> bool {
    if evaluate_landmark_hotkey(pressed_key, configured_hotkey, viewport_id).is_some() {
        if dioxus::core::Runtime::try_current().is_some()
            && !origin_inside_viewport
            && !is_focus_within_element(viewport_id)
        {
            let prev_focus_id = oxidase::dom::Document::current()
                .and_then(|d| d.active_element())
                .and_then(|el| el.id().or_else(|| el.get_attribute("id")))
                .filter(|id| !id.is_empty())
                .or_else(|| {
                    origin_target_id.map(str::to_string).filter(|id| {
                        !id.is_empty() && id != viewport_id && !id.starts_with("monoxus-toast-")
                    })
                });
            TOAST_STORE.write().last_focused_element_id = prev_focus_id;
        }
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

        let trans1 = manager.handle_visibility_change(false);
        assert_eq!(trans1, VisibilityTransition::Paused);
        assert!(manager.page_hidden);

        let trans2 = manager.handle_visibility_change(false);
        assert_eq!(trans2, VisibilityTransition::Unchanged);

        let trans3 = manager.handle_visibility_change(true);
        assert_eq!(trans3, VisibilityTransition::Resumed);
        assert!(!manager.page_hidden);

        let trans4 = manager.handle_visibility_change(true);
        assert_eq!(trans4, VisibilityTransition::Unchanged);
    }

    #[test]
    fn test_visibility_pause_disabled() {
        let mut manager = ToastVisibilityManager::new(false);

        let trans1 = manager.handle_visibility_change(false);
        assert_eq!(trans1, VisibilityTransition::Unchanged);
        assert!(manager.page_hidden);

        let trans2 = manager.handle_visibility_change(true);
        assert_eq!(trans2, VisibilityTransition::Unchanged);
        assert!(!manager.page_hidden);
    }
}
