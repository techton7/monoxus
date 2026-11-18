use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::keyboard_types::{Key, Modifiers, NamedKey};
use dioxus::prelude::*;

use super::browser::{focus_element_by_id, query_descendant_order};
use super::shared::CollectionRegistry;

pub use super::shared::{
    Direction, Orientation, RovingFocusController, RovingFocusIntent, RovingFocusKey,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RovingFocusConfig {
    pub orientation: Orientation,
    pub dir: Direction,
    pub loop_focus: bool,
    pub enabled: bool,
}

impl Default for RovingFocusConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl RovingFocusConfig {
    pub const fn new() -> Self {
        Self {
            orientation: Orientation::Horizontal,
            dir: Direction::Ltr,
            loop_focus: false,
            enabled: true,
        }
    }

    pub const fn with_orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub const fn with_direction(mut self, dir: Direction) -> Self {
        self.dir = dir;
        self
    }

    pub const fn with_dir(mut self, dir: Direction) -> Self {
        self.dir = dir;
        self
    }

    pub const fn with_loop_focus(mut self, loop_focus: bool) -> Self {
        self.loop_focus = loop_focus;
        self
    }

    pub const fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub const fn controller(&self) -> RovingFocusController {
        RovingFocusController::new(self.dir)
            .with_orientation(self.orientation)
            .with_looping(self.loop_focus)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RovingModifiers {
    pub alt: bool,
    pub ctrl: bool,
    pub meta: bool,
    pub shift: bool,
}

impl RovingModifiers {
    pub const fn none() -> Self {
        Self {
            alt: false,
            ctrl: false,
            meta: false,
            shift: false,
        }
    }

    pub const fn new(alt: bool, ctrl: bool, meta: bool, shift: bool) -> Self {
        Self {
            alt,
            ctrl,
            meta,
            shift,
        }
    }

    pub const fn any(&self) -> bool {
        self.alt || self.ctrl || self.meta || self.shift
    }
}

impl From<Modifiers> for RovingModifiers {
    fn from(modifiers: Modifiers) -> Self {
        Self {
            alt: modifiers.contains(Modifiers::ALT),
            ctrl: modifiers.contains(Modifiers::CONTROL),
            meta: modifiers.contains(Modifiers::META),
            shift: modifiers.contains(Modifiers::SHIFT),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RovingItemMeta {
    pub dom_id: String,
    pub focusable: bool,
    pub active: bool,
}

impl RovingItemMeta {
    pub fn new(dom_id: impl Into<String>, focusable: bool, active: bool) -> Self {
        Self {
            dom_id: dom_id.into(),
            focusable,
            active,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RovingCandidate {
    pub key: String,
    pub dom_id: String,
    pub focusable: bool,
    pub active: bool,
}

impl RovingCandidate {
    pub fn new(
        key: impl Into<String>,
        dom_id: impl Into<String>,
        focusable: bool,
        active: bool,
    ) -> Self {
        Self {
            key: key.into(),
            dom_id: dom_id.into(),
            focusable,
            active,
        }
    }
}

pub fn candidates_from_registry(
    registry: &CollectionRegistry<String, RovingItemMeta>,
) -> Vec<RovingCandidate> {
    registry
        .iter()
        .map(|entry| {
            let meta = entry.value();
            RovingCandidate {
                key: entry.key().clone(),
                dom_id: meta.dom_id.clone(),
                focusable: meta.focusable,
                active: meta.active,
            }
        })
        .collect()
}

pub fn parse_roving_key(key: &str) -> Option<RovingFocusKey> {
    match key {
        "ArrowLeft" => Some(RovingFocusKey::ArrowLeft),
        "ArrowRight" => Some(RovingFocusKey::ArrowRight),
        "ArrowUp" => Some(RovingFocusKey::ArrowUp),
        "ArrowDown" => Some(RovingFocusKey::ArrowDown),
        "Home" | "PageUp" => Some(RovingFocusKey::Home),
        "End" | "PageDown" => Some(RovingFocusKey::End),
        _ => None,
    }
}

pub fn key_to_roving_str(key: &Key) -> Option<&'static str> {
    match key {
        Key::Named(NamedKey::ArrowLeft) => Some("ArrowLeft"),
        Key::Named(NamedKey::ArrowRight) => Some("ArrowRight"),
        Key::Named(NamedKey::ArrowUp) => Some("ArrowUp"),
        Key::Named(NamedKey::ArrowDown) => Some("ArrowDown"),
        Key::Named(NamedKey::Home) => Some("Home"),
        Key::Named(NamedKey::End) => Some("End"),
        Key::Named(NamedKey::PageUp) => Some("PageUp"),
        Key::Named(NamedKey::PageDown) => Some("PageDown"),
        Key::Character(s) => match s.as_str() {
            "ArrowLeft" => Some("ArrowLeft"),
            "ArrowRight" => Some("ArrowRight"),
            "ArrowUp" => Some("ArrowUp"),
            "ArrowDown" => Some("ArrowDown"),
            "Home" => Some("Home"),
            "End" => Some("End"),
            "PageUp" => Some("PageUp"),
            "PageDown" => Some("PageDown"),
            _ => None,
        },
        _ => None,
    }
}

pub fn is_navigation_key(key: &str) -> bool {
    parse_roving_key(key).is_some()
}

pub fn is_navigation_key_for_orientation(orientation: Orientation, key: &str) -> bool {
    let Some(roving_key) = parse_roving_key(key) else {
        return false;
    };
    RovingFocusController::new(Direction::Ltr)
        .with_orientation(orientation)
        .intent_for_key(roving_key)
        .is_some()
}

/// Resolves the current roving tab-stop key using a 3-tier priority fallback:
/// 1. `current` if present in `candidates` and `focusable`.
/// 2. First candidate where `focusable && active`.
/// 3. First candidate where `focusable`.
pub fn resolve_tab_stop(current: Option<&str>, candidates: &[RovingCandidate]) -> Option<String> {
    if let Some(curr) = current
        && candidates.iter().any(|c| c.key == curr && c.focusable)
    {
        return Some(curr.to_string());
    }

    if let Some(active) = candidates.iter().find(|c| c.focusable && c.active) {
        return Some(active.key.clone());
    }

    candidates
        .iter()
        .find(|c| c.focusable)
        .map(|c| c.key.clone())
}

/// Computes `tabindex` for an item given roving configuration and resolved tab-stop.
/// - Non-focusable (disabled) items always receive `-1`.
/// - When roving focus is disabled (`!config.enabled`), focusable items receive `0`.
/// - Otherwise, the resolved tab-stop receives `0` and all other items receive `-1`.
pub fn compute_tab_index(
    config: &RovingFocusConfig,
    key: &str,
    focusable: bool,
    resolved_tab_stop: Option<&str>,
) -> i32 {
    if !focusable {
        return -1;
    }
    if !config.enabled {
        return 0;
    }
    if resolved_tab_stop == Some(key) {
        0
    } else {
        -1
    }
}

pub fn resolve_next_stop_in_registry<V>(
    config: &RovingFocusConfig,
    registry: &CollectionRegistry<String, V>,
    current: Option<&str>,
    key: &str,
    modifiers: RovingModifiers,
    is_focusable: impl Fn(&V) -> bool,
) -> Option<String> {
    if !config.enabled || modifiers.any() {
        return None;
    }

    let roving_key = parse_roving_key(key)?;
    let current_owned = current.map(ToOwned::to_owned);
    config
        .controller()
        .navigate_by_key(registry, current_owned.as_ref(), roving_key, is_focusable)
}

pub fn resolve_next_stop(
    config: &RovingFocusConfig,
    candidates: &[RovingCandidate],
    current: Option<&str>,
    key: &str,
    modifiers: RovingModifiers,
) -> Option<String> {
    if !config.enabled || modifiers.any() {
        return None;
    }

    let roving_key = parse_roving_key(key)?;
    let mut registry = CollectionRegistry::new();
    for candidate in candidates {
        registry.register(candidate.key.clone(), candidate.focusable);
    }

    let current_owned = current.map(ToOwned::to_owned);
    config.controller().navigate_by_key(
        &registry,
        current_owned.as_ref(),
        roving_key,
        |focusable| *focusable,
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RovingFocusRuntime {
    pub config: Signal<RovingFocusConfig>,
    pub registry: Signal<CollectionRegistry<String, RovingItemMeta>>,
    pub current_tab_stop: Signal<Option<String>>,
    pub root_dom_id: Signal<String>,
}

pub fn use_roving_focus_group(
    config: RovingFocusConfig,
    initial_tab_stop: Option<String>,
    root_dom_id: impl Into<String>,
) -> RovingFocusRuntime {
    let root_dom_id = root_dom_id.into();
    let runtime = RovingFocusRuntime::new(config, initial_tab_stop, root_dom_id.clone());
    runtime.set_config(config);
    runtime.set_root_dom_id(root_dom_id);
    runtime
}

pub fn use_roving_focus_item(
    roving: RovingFocusRuntime,
    key: &str,
    meta: RovingItemMeta,
) {
    let tracked_key = use_hook(|| Rc::new(RefCell::new(key.to_string())));

    use_hook(|| {
        roving.register_item(key.to_string(), meta.clone());
    });

    let effect_key = key.to_string();
    let effect_meta = meta;
    let effect_tracked = Rc::clone(&tracked_key);
    use_effect(use_reactive(
        (&effect_key, &effect_meta),
        move |(next_key, next_meta)| {
            let prev_key = effect_tracked.borrow().clone();
            if prev_key != next_key {
                roving.unregister_item(&prev_key);
                *effect_tracked.borrow_mut() = next_key.clone();
            }
            roving.register_item(next_key, next_meta);
        },
    ));

    let drop_tracked = Rc::clone(&tracked_key);
    use_drop(move || {
        let curr_key = drop_tracked.borrow().clone();
        roving.unregister_item(&curr_key);
    });
}

impl RovingFocusRuntime {
    pub fn new(
        config: RovingFocusConfig,
        initial_tab_stop: Option<String>,
        root_dom_id: impl Into<String>,
    ) -> Self {
        let root_dom_id = root_dom_id.into();
        Self {
            config: use_signal(|| config),
            registry: use_signal(CollectionRegistry::new),
            current_tab_stop: use_signal(|| initial_tab_stop),
            root_dom_id: use_signal(|| root_dom_id),
        }
    }

    pub fn set_config(&self, next: RovingFocusConfig) {
        let mut sig = self.config;
        if *sig.peek() != next {
            sig.set(next);
        }
    }

    pub fn set_orientation(&self, orientation: Orientation) {
        let mut sig = self.config;
        let current = *sig.peek();
        if current.orientation != orientation {
            sig.set(current.with_orientation(orientation));
        }
    }

    pub fn set_direction(&self, dir: Direction) {
        let mut sig = self.config;
        let current = *sig.peek();
        if current.dir != dir {
            sig.set(current.with_direction(dir));
        }
    }

    pub fn set_loop_focus(&self, loop_focus: bool) {
        let mut sig = self.config;
        let current = *sig.peek();
        if current.loop_focus != loop_focus {
            sig.set(current.with_loop_focus(loop_focus));
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        let mut sig = self.config;
        let current = *sig.peek();
        if current.enabled != enabled {
            sig.set(current.with_enabled(enabled));
        }
    }

    pub fn set_root_dom_id(&self, root_dom_id: impl Into<String>) {
        let root_dom_id = root_dom_id.into();
        let mut sig = self.root_dom_id;
        if *sig.peek() != root_dom_id {
            sig.set(root_dom_id);
        }
    }

    pub fn register_item(&self, key: impl Into<String>, meta: RovingItemMeta) {
        let key = key.into();
        if self.registry.peek().get(&key) == Some(&meta) {
            return;
        }
        let mut reg = self.registry;
        if let Ok(mut guard) = reg.try_write() {
            guard.register(key, meta);
        }
    }

    pub fn unregister_item(&self, key: &str) {
        let mut reg_sig = self.registry;
        if let Ok(mut reg) = reg_sig.try_write() {
            reg.unregister(&key.to_string());
        }
        let mut stop_sig = self.current_tab_stop;
        if let Ok(mut current) = stop_sig.try_write()
            && current.as_deref() == Some(key)
        {
            *current = None;
        }
    }

    pub fn item_dom_id(&self, key: &str) -> Option<String> {
        self.registry
            .read()
            .get(&key.to_string())
            .filter(|meta| !meta.dom_id.is_empty())
            .map(|meta| meta.dom_id.clone())
    }

    pub fn reorder_items(&self, ordered_keys: &[String]) {
        let mut reg_sig = self.registry;
        if let Ok(mut reg) = reg_sig.try_write() {
            reg.reorder_by_keys(ordered_keys);
        }
    }

    pub fn sync_dom_order(&self) {
        let root_id = self.root_dom_id.peek().clone();
        self.sync_dom_order_for(&root_id);
    }

    pub fn sync_dom_order_for(&self, root_element_id: &str) {
        if root_element_id.is_empty() {
            return;
        }
        let reg = self.registry.peek().clone();
        if reg.len() <= 1 {
            return;
        }
        let id_to_key: Vec<(String, String)> = reg
            .iter()
            .filter(|entry| !entry.value().dom_id.is_empty())
            .map(|entry| (entry.value().dom_id.clone(), entry.key().clone()))
            .collect();
        if id_to_key.len() <= 1 {
            return;
        }
        let candidate_refs: Vec<&str> = id_to_key.iter().map(|(id, _)| id.as_str()).collect();

        if let Ok(ordered_dom_ids) = query_descendant_order(root_element_id, &candidate_refs)
            && !ordered_dom_ids.is_empty()
        {
            let ordered_keys: Vec<String> = ordered_dom_ids
                .into_iter()
                .filter_map(|dom_id| {
                    id_to_key
                        .iter()
                        .find(|(id, _)| id == &dom_id)
                        .map(|(_, key)| key.clone())
                })
                .collect();
            if !ordered_keys.is_empty() {
                let current_keys: Vec<String> = reg.iter().map(|e| e.key().clone()).collect();
                let mut preview = reg;
                preview.reorder_by_keys(&ordered_keys);
                let next_keys: Vec<String> = preview.iter().map(|e| e.key().clone()).collect();
                if current_keys != next_keys {
                    self.reorder_items(&ordered_keys);
                }
            }
        }
    }

    pub fn candidates(&self) -> Vec<RovingCandidate> {
        let reg = self.registry.read();
        candidates_from_registry(&reg)
    }

    pub fn resolve_tab_stop(&self) -> Option<String> {
        self.resolve_tab_stop_with(|_key, meta| meta.active)
    }

    pub fn resolve_tab_stop_with(
        &self,
        is_active: impl Fn(&str, &RovingItemMeta) -> bool,
    ) -> Option<String> {
        let current = self.current_tab_stop.read();
        let reg = self.registry.read();
        let candidates: Vec<RovingCandidate> = reg
            .iter()
            .map(|entry| {
                let meta = entry.value();
                RovingCandidate {
                    key: entry.key().clone(),
                    dom_id: meta.dom_id.clone(),
                    focusable: meta.focusable,
                    active: is_active(entry.key(), meta),
                }
            })
            .collect();
        resolve_tab_stop(current.as_deref(), &candidates)
    }

    pub fn tab_index_for(&self, key: &str, focusable: bool) -> i32 {
        let config = *self.config.read();
        let resolved = self.resolve_tab_stop();
        compute_tab_index(&config, key, focusable, resolved.as_deref())
    }

    pub fn tab_index_for_with(
        &self,
        key: &str,
        focusable: bool,
        is_active: impl Fn(&str, &RovingItemMeta) -> bool,
    ) -> i32 {
        let config = *self.config.read();
        let resolved = self.resolve_tab_stop_with(is_active);
        compute_tab_index(&config, key, focusable, resolved.as_deref())
    }

    pub fn set_current_tab_stop(&self, tab_stop: Option<String>) {
        let mut sig = self.current_tab_stop;
        if *sig.peek() != tab_stop {
            sig.set(tab_stop);
        }
    }

    pub fn handle_item_focus(&self, key: &str, focusable: bool) {
        if focusable {
            self.set_current_tab_stop(Some(key.to_string()));
        }
    }

    pub fn focus_item(&self, key: &str) -> bool {
        self.set_current_tab_stop(Some(key.to_string()));
        if let Some(dom_id) = self
            .registry
            .peek()
            .get(&key.to_string())
            .map(|meta| meta.dom_id.clone())
            && !dom_id.is_empty()
        {
            focus_element_by_id(&dom_id);
            return true;
        }
        false
    }

    pub fn navigate_key(
        &self,
        current_key: Option<&str>,
        key: &str,
        modifiers: RovingModifiers,
    ) -> Option<String> {
        let config = *self.config.peek();
        if !config.enabled || modifiers.any() {
            return None;
        }
        if !is_navigation_key_for_orientation(config.orientation, key) {
            return None;
        }

        self.sync_dom_order();

        let reg = self.registry.peek();
        let fallback_current = self.current_tab_stop.peek().clone();
        let effective_current = current_key.or(fallback_current.as_deref());

        resolve_next_stop_in_registry(
            &config,
            &reg,
            effective_current,
            key,
            modifiers,
            |meta| meta.focusable,
        )
    }

    pub fn handle_keydown<F>(
        &self,
        current_key: Option<&str>,
        evt: &KeyboardEvent,
        on_focused: Option<F>,
    ) -> Option<String>
    where
        F: FnOnce(&str),
    {
        let config = *self.config.peek();
        if !config.enabled {
            return None;
        }

        let modifiers = RovingModifiers::from(evt.modifiers());
        if modifiers.any() {
            return None;
        }

        let event_key = evt.key();
        let key_str = key_to_roving_str(&event_key)?;
        if !is_navigation_key_for_orientation(config.orientation, key_str) {
            return None;
        }

        evt.prevent_default();
        evt.stop_propagation();

        let next_key = self.navigate_key(current_key, key_str, modifiers)?;
        self.focus_item(&next_key);
        if let Some(callback) = on_focused {
            callback(&next_key);
        }
        Some(next_key)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use dioxus::prelude::*;

    use super::{
        Direction, Orientation, RovingCandidate, RovingFocusConfig, RovingFocusRuntime,
        RovingItemMeta, RovingModifiers, compute_tab_index, is_navigation_key,
        is_navigation_key_for_orientation, resolve_next_stop, resolve_tab_stop,
        use_roving_focus_group, use_roving_focus_item,
    };

    fn sample_candidates() -> Vec<RovingCandidate> {
        vec![
            RovingCandidate::new("a", "dom-a", false, false), // disabled
            RovingCandidate::new("b", "dom-b", true, false),
            RovingCandidate::new("c", "dom-c", true, true), // active
            RovingCandidate::new("d", "dom-d", true, false),
        ]
    }

    #[test]
    fn resolve_tab_stop_follows_three_tier_priority_and_recovers_from_disabled_or_missing() {
        let candidates = sample_candidates();

        // Tier 1: current is valid and focusable -> keeps current ("b") even though "c" is active
        assert_eq!(
            resolve_tab_stop(Some("b"), &candidates),
            Some("b".to_string())
        );

        // Tier 2: current is None -> selects first active focusable ("c")
        assert_eq!(resolve_tab_stop(None, &candidates), Some("c".to_string()));

        // Tier 2 recovery: current points to disabled ("a") or unmounted ("missing") -> falls back to active ("c")
        assert_eq!(
            resolve_tab_stop(Some("a"), &candidates),
            Some("c".to_string())
        );
        assert_eq!(
            resolve_tab_stop(Some("missing"), &candidates),
            Some("c".to_string())
        );

        // Tier 3: no active focusable item -> selects first focusable ("b")
        let inactive_candidates = vec![
            RovingCandidate::new("a", "dom-a", false, true), // active but disabled!
            RovingCandidate::new("b", "dom-b", true, false),
            RovingCandidate::new("d", "dom-d", true, false),
        ];
        assert_eq!(
            resolve_tab_stop(None, &inactive_candidates),
            Some("b".to_string())
        );

        // All disabled or empty -> None
        let all_disabled = vec![RovingCandidate::new("a", "dom-a", false, true)];
        assert_eq!(resolve_tab_stop(Some("a"), &all_disabled), None);
        assert_eq!(resolve_tab_stop(None, &[]), None);
    }

    #[test]
    fn compute_tab_index_respects_focusable_and_enabled_opt_out() {
        let roving_on = RovingFocusConfig::new().with_enabled(true);
        assert_eq!(compute_tab_index(&roving_on, "b", true, Some("b")), 0);
        assert_eq!(compute_tab_index(&roving_on, "c", true, Some("b")), -1);
        assert_eq!(compute_tab_index(&roving_on, "b", false, Some("b")), -1);

        // When enabled == false (e.g. ToggleGroup roving_focus: false), all focusable items get 0
        let roving_off = RovingFocusConfig::new().with_enabled(false);
        assert_eq!(compute_tab_index(&roving_off, "b", true, Some("c")), 0);
        assert_eq!(compute_tab_index(&roving_off, "c", true, Some("c")), 0);
        assert_eq!(compute_tab_index(&roving_off, "a", false, Some("c")), -1);
    }

    #[test]
    fn resolve_next_stop_supports_orientations_rtl_looping_clamping_and_boundaries() {
        let candidates = vec![
            RovingCandidate::new("1", "id-1", true, false),
            RovingCandidate::new("2", "id-2", false, false), // disabled
            RovingCandidate::new("3", "id-3", true, false),
            RovingCandidate::new("4", "id-4", true, false),
        ];

        // Horizontal LTR looping
        let h_ltr = RovingFocusConfig::new()
            .with_orientation(Orientation::Horizontal)
            .with_direction(Direction::Ltr)
            .with_loop_focus(true);
        assert_eq!(
            resolve_next_stop(
                &h_ltr,
                &candidates,
                Some("1"),
                "ArrowRight",
                RovingModifiers::none()
            ),
            Some("3".to_string())
        );
        assert_eq!(
            resolve_next_stop(
                &h_ltr,
                &candidates,
                Some("1"),
                "ArrowLeft",
                RovingModifiers::none()
            ),
            Some("4".to_string())
        );
        assert_eq!(
            resolve_next_stop(
                &h_ltr,
                &candidates,
                Some("1"),
                "ArrowDown",
                RovingModifiers::none()
            ),
            None
        );

        // Horizontal RTL looping (ArrowLeft -> Next, ArrowRight -> Previous)
        let h_rtl = h_ltr.with_direction(Direction::Rtl);
        assert_eq!(
            resolve_next_stop(
                &h_rtl,
                &candidates,
                Some("1"),
                "ArrowLeft",
                RovingModifiers::none()
            ),
            Some("3".to_string())
        );
        assert_eq!(
            resolve_next_stop(
                &h_rtl,
                &candidates,
                Some("1"),
                "ArrowRight",
                RovingModifiers::none()
            ),
            Some("4".to_string())
        );

        // Vertical clamping (loop_focus = false)
        let v_clamp = RovingFocusConfig::new()
            .with_orientation(Orientation::Vertical)
            .with_loop_focus(false);
        assert_eq!(
            resolve_next_stop(
                &v_clamp,
                &candidates,
                Some("1"),
                "ArrowDown",
                RovingModifiers::none()
            ),
            Some("3".to_string())
        );
        assert_eq!(
            resolve_next_stop(
                &v_clamp,
                &candidates,
                Some("1"),
                "ArrowUp",
                RovingModifiers::none()
            ),
            None
        );
        assert_eq!(
            resolve_next_stop(
                &v_clamp,
                &candidates,
                Some("4"),
                "ArrowDown",
                RovingModifiers::none()
            ),
            None
        );
        assert_eq!(
            resolve_next_stop(
                &v_clamp,
                &candidates,
                Some("1"),
                "ArrowRight",
                RovingModifiers::none()
            ),
            None
        );

        // Orientation::Both accepts both axes and respects RTL on horizontal arrows
        let both_rtl = RovingFocusConfig::new()
            .with_orientation(Orientation::Both)
            .with_direction(Direction::Rtl)
            .with_loop_focus(true);
        assert_eq!(
            resolve_next_stop(
                &both_rtl,
                &candidates,
                Some("1"),
                "ArrowLeft",
                RovingModifiers::none()
            ),
            Some("3".to_string())
        );
        assert_eq!(
            resolve_next_stop(
                &both_rtl,
                &candidates,
                Some("1"),
                "ArrowDown",
                RovingModifiers::none()
            ),
            Some("3".to_string())
        );

        // Home / PageUp / End / PageDown
        assert_eq!(
            resolve_next_stop(
                &v_clamp,
                &candidates,
                Some("4"),
                "Home",
                RovingModifiers::none()
            ),
            Some("1".to_string())
        );
        assert_eq!(
            resolve_next_stop(
                &v_clamp,
                &candidates,
                Some("4"),
                "PageUp",
                RovingModifiers::none()
            ),
            Some("1".to_string())
        );
        assert_eq!(
            resolve_next_stop(
                &v_clamp,
                &candidates,
                Some("1"),
                "End",
                RovingModifiers::none()
            ),
            Some("4".to_string())
        );
        assert_eq!(
            resolve_next_stop(
                &v_clamp,
                &candidates,
                Some("1"),
                "PageDown",
                RovingModifiers::none()
            ),
            Some("4".to_string())
        );
    }

    #[test]
    fn resolve_next_stop_guards_modifiers_and_disabled_config() {
        let candidates = sample_candidates();
        let config = RovingFocusConfig::new().with_loop_focus(true);

        for mods in [
            RovingModifiers::new(true, false, false, false),
            RovingModifiers::new(false, true, false, false),
            RovingModifiers::new(false, false, true, false),
            RovingModifiers::new(false, false, false, true),
        ] {
            assert_eq!(
                resolve_next_stop(&config, &candidates, Some("b"), "ArrowRight", mods),
                None
            );
        }

        let disabled_cfg = config.with_enabled(false);
        assert_eq!(
            resolve_next_stop(
                &disabled_cfg,
                &candidates,
                Some("b"),
                "ArrowRight",
                RovingModifiers::none()
            ),
            None
        );

        assert!(is_navigation_key("PageUp"));
        assert!(is_navigation_key("PageDown"));
        assert!(!is_navigation_key("Enter"));
        assert!(is_navigation_key_for_orientation(
            Orientation::Both,
            "ArrowUp"
        ));
        assert!(is_navigation_key_for_orientation(
            Orientation::Both,
            "ArrowLeft"
        ));
    }

    #[component]
    fn RuntimeHarnessItem(roving: RovingFocusRuntime, item_key: String, active: bool) -> Element {
        use_roving_focus_item(
            roving,
            &item_key,
            RovingItemMeta::new(format!("dom-{item_key}"), true, active),
        );
        let idx = roving.tab_index_for(&item_key, true);
        rsx! {
            button { id: "dom-{item_key}", tabindex: "{idx}", "{item_key}" }
        }
    }

    #[component]
    fn RuntimeUnmountHarness(
        mount_ctl: Rc<RefCell<Option<Signal<bool>>>>,
        observed_stop: Rc<RefCell<Option<String>>>,
    ) -> Element {
        let mount_first = use_signal(|| true);
        *mount_ctl.borrow_mut() = Some(mount_first);

        let roving = use_roving_focus_group(
            RovingFocusConfig::new().with_loop_focus(true),
            Some("first".to_string()),
            "harness-root",
        );

        *observed_stop.borrow_mut() = roving.resolve_tab_stop();

        rsx! {
            div { id: "harness-root",
                if mount_first() {
                    RuntimeHarnessItem { roving, item_key: "first".to_string(), active: false }
                }
                RuntimeHarnessItem { roving, item_key: "second".to_string(), active: true }
                RuntimeHarnessItem { roving, item_key: "third".to_string(), active: false }
            }
        }
    }

    #[test]
    fn roving_focus_runtime_recovers_tab_stop_when_current_item_unmounts() {
        let mount_ctl: Rc<RefCell<Option<Signal<bool>>>> = Rc::new(RefCell::new(None));
        let observed = Rc::new(RefCell::new(None));

        let mut dom = VirtualDom::new_with_props(
            RuntimeUnmountHarness,
            RuntimeUnmountHarnessProps {
                mount_ctl: Rc::clone(&mount_ctl),
                observed_stop: Rc::clone(&observed),
            },
        );
        dom.rebuild_in_place();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        assert_eq!(*observed.borrow(), Some("first".to_string()));

        // Unmount "first" -> use_drop unregisters "first" and clears current_tab_stop -> recovers to active "second"
        let mut sig = mount_ctl.borrow().expect("signal captured");
        sig.set(false);
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        assert_eq!(*observed.borrow(), Some("second".to_string()));
    }
}
