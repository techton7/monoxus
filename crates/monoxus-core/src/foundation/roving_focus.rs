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

#[cfg(test)]
mod tests {
    use super::{
        CollectionRegistry, Direction, Orientation, RovingCandidate, RovingFocusConfig,
        RovingItemMeta, RovingModifiers, candidates_from_registry, compute_tab_index,
        is_navigation_key, is_navigation_key_for_orientation, resolve_next_stop,
        resolve_next_stop_in_registry, resolve_tab_stop,
    };

    fn sample_candidates() -> Vec<RovingCandidate> {
        vec![
            RovingCandidate::new("a", "dom-a", false, false),
            RovingCandidate::new("b", "dom-b", true, false),
            RovingCandidate::new("c", "dom-c", true, true),
            RovingCandidate::new("d", "dom-d", true, false),
        ]
    }

    #[test]
    fn resolve_tab_stop_follows_three_tier_priority_and_recovers_from_disabled_or_missing() {
        let candidates = sample_candidates();

        assert_eq!(
            resolve_tab_stop(Some("b"), &candidates),
            Some("b".to_string())
        );
        assert_eq!(resolve_tab_stop(None, &candidates), Some("c".to_string()));
        assert_eq!(
            resolve_tab_stop(Some("a"), &candidates),
            Some("c".to_string())
        );
        assert_eq!(
            resolve_tab_stop(Some("missing"), &candidates),
            Some("c".to_string())
        );

        let inactive_candidates = vec![
            RovingCandidate::new("a", "dom-a", false, true),
            RovingCandidate::new("b", "dom-b", true, false),
            RovingCandidate::new("d", "dom-d", true, false),
        ];
        assert_eq!(
            resolve_tab_stop(None, &inactive_candidates),
            Some("b".to_string())
        );

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

        let roving_off = RovingFocusConfig::new().with_enabled(false);
        assert_eq!(compute_tab_index(&roving_off, "b", true, Some("c")), 0);
        assert_eq!(compute_tab_index(&roving_off, "c", true, Some("c")), 0);
        assert_eq!(compute_tab_index(&roving_off, "a", false, Some("c")), -1);
    }

    #[test]
    fn resolve_next_stop_supports_orientations_rtl_looping_clamping_and_boundaries() {
        let candidates = vec![
            RovingCandidate::new("1", "id-1", true, false),
            RovingCandidate::new("2", "id-2", false, false),
            RovingCandidate::new("3", "id-3", true, false),
            RovingCandidate::new("4", "id-4", true, false),
        ];

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

    #[test]
    fn candidates_from_registry_and_in_registry_navigation_stay_aligned() {
        let mut registry = CollectionRegistry::new();
        registry.register("a".to_string(), RovingItemMeta::new("dom-a", true, false));
        registry.register("b".to_string(), RovingItemMeta::new("dom-b", false, false));
        registry.register("c".to_string(), RovingItemMeta::new("dom-c", true, true));

        let candidates = candidates_from_registry(&registry);
        assert_eq!(candidates.len(), 3);
        assert_eq!(resolve_tab_stop(None, &candidates), Some("c".to_string()));

        let config = RovingFocusConfig::new().with_loop_focus(true);
        assert_eq!(
            resolve_next_stop_in_registry(
                &config,
                &registry,
                Some("a"),
                "ArrowRight",
                RovingModifiers::none(),
                |meta| meta.focusable,
            ),
            Some("c".to_string())
        );
    }
}
