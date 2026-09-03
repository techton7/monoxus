use std::{
    borrow::Cow,
    sync::atomic::{AtomicU64, Ordering},
};

use dioxus::prelude::use_hook;

static NEXT_STABLE_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ScopeHandle {
    segments: Vec<Cow<'static, str>>,
}

impl ScopeHandle {
    pub fn root(segment: impl Into<Cow<'static, str>>) -> Self {
        Self {
            segments: vec![segment.into()],
        }
    }

    pub fn child(&self, segment: impl Into<Cow<'static, str>>) -> Self {
        let mut segments = self.segments.clone();
        segments.push(segment.into());
        Self { segments }
    }

    pub fn segments(&self) -> &[Cow<'static, str>] {
        &self.segments
    }

    pub fn token(&self) -> String {
        encode_scope_segments(self.segments.iter().map(Cow::as_ref))
    }

    pub fn qualify(&self, leaf: impl AsRef<str>) -> String {
        encode_scope_segments(
            self.segments
                .iter()
                .map(Cow::as_ref)
                .chain(std::iter::once(leaf.as_ref())),
        )
    }
}

fn encode_scope_segments<'a>(segments: impl IntoIterator<Item = &'a str>) -> String {
    let mut token = String::new();

    for (index, segment) in segments.into_iter().enumerate() {
        if index > 0 {
            token.push('|');
        }

        token.push_str(&segment.len().to_string());
        token.push(':');
        token.push_str(segment);
    }

    token
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollectionEntry<K, V> {
    key: K,
    value: V,
}

impl<K, V> CollectionEntry<K, V> {
    pub fn key(&self) -> &K {
        &self.key
    }

    pub fn value(&self) -> &V {
        &self.value
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollectionRegistry<K, V> {
    entries: Vec<CollectionEntry<K, V>>,
}

impl<K, V> Default for CollectionRegistry<K, V> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl<K, V> CollectionRegistry<K, V>
where
    K: PartialEq,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn register(&mut self, key: K, value: V) -> bool {
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.key.eq(&key)) {
            entry.value = value;
            return false;
        }

        self.entries.push(CollectionEntry { key, value });
        true
    }

    pub fn unregister(&mut self, key: &K) -> Option<V> {
        let position = self.entries.iter().position(|entry| entry.key.eq(key))?;
        Some(self.entries.remove(position).value)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.entries
            .iter()
            .find(|entry| entry.key.eq(key))
            .map(CollectionEntry::value)
    }

    pub fn iter(&self) -> impl Iterator<Item = &CollectionEntry<K, V>> {
        self.entries.iter()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Direction {
    #[default]
    Ltr,
    Rtl,
}

impl Direction {
    pub const fn resolve(
        override_direction: Option<Self>,
        inherited_direction: Option<Self>,
    ) -> Self {
        match override_direction {
            Some(direction) => direction,
            None => match inherited_direction {
                Some(direction) => direction,
                None => Self::Ltr,
            },
        }
    }
}

pub fn use_direction(
    override_direction: Option<Direction>,
    inherited_direction: Option<Direction>,
) -> Direction {
    Direction::resolve(override_direction, inherited_direction)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Orientation {
    #[default]
    Horizontal,
    Vertical,
    Both,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RovingFocusIntent {
    First,
    Last,
    Next,
    Previous,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RovingFocusKey {
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    Home,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RovingFocusController {
    direction: Direction,
    orientation: Orientation,
    looping: bool,
}

impl Default for RovingFocusController {
    fn default() -> Self {
        Self::new(Direction::default())
    }
}

impl RovingFocusController {
    pub const fn new(direction: Direction) -> Self {
        Self {
            direction,
            orientation: Orientation::Horizontal,
            looping: false,
        }
    }

    pub const fn with_orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub const fn with_looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }

    pub const fn direction(&self) -> Direction {
        self.direction
    }

    pub const fn orientation(&self) -> Orientation {
        self.orientation
    }

    pub const fn looping(&self) -> bool {
        self.looping
    }

    pub fn navigate<K, V>(
        &self,
        registry: &CollectionRegistry<K, V>,
        current: Option<&K>,
        intent: RovingFocusIntent,
        is_focusable: impl Fn(&V) -> bool,
    ) -> Option<K>
    where
        K: Clone + PartialEq,
    {
        let focusable_indices: Vec<usize> = registry
            .entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| is_focusable(&entry.value).then_some(index))
            .collect();

        let target_index = match intent {
            RovingFocusIntent::First => focusable_indices.first().copied(),
            RovingFocusIntent::Last => focusable_indices.last().copied(),
            RovingFocusIntent::Next => next_focusable_index(
                &focusable_indices,
                current.and_then(|key| registry_index(registry, key)),
                self.looping,
            ),
            RovingFocusIntent::Previous => previous_focusable_index(
                &focusable_indices,
                current.and_then(|key| registry_index(registry, key)),
                self.looping,
            ),
        }?;

        registry
            .entries
            .get(target_index)
            .map(|entry| entry.key.clone())
    }

    pub fn navigate_by_key<K, V>(
        &self,
        registry: &CollectionRegistry<K, V>,
        current: Option<&K>,
        key: RovingFocusKey,
        is_focusable: impl Fn(&V) -> bool,
    ) -> Option<K>
    where
        K: Clone + PartialEq,
    {
        let intent = self.intent_for_key(key)?;
        self.navigate(registry, current, intent, is_focusable)
    }

    fn intent_for_key(&self, key: RovingFocusKey) -> Option<RovingFocusIntent> {
        match key {
            RovingFocusKey::Home => Some(RovingFocusIntent::First),
            RovingFocusKey::End => Some(RovingFocusIntent::Last),
            RovingFocusKey::ArrowLeft => match self.orientation {
                Orientation::Vertical => None,
                Orientation::Horizontal | Orientation::Both => Some(match self.direction {
                    Direction::Ltr => RovingFocusIntent::Previous,
                    Direction::Rtl => RovingFocusIntent::Next,
                }),
            },
            RovingFocusKey::ArrowRight => match self.orientation {
                Orientation::Vertical => None,
                Orientation::Horizontal | Orientation::Both => Some(match self.direction {
                    Direction::Ltr => RovingFocusIntent::Next,
                    Direction::Rtl => RovingFocusIntent::Previous,
                }),
            },
            RovingFocusKey::ArrowUp => match self.orientation {
                Orientation::Horizontal => None,
                Orientation::Vertical | Orientation::Both => Some(RovingFocusIntent::Previous),
            },
            RovingFocusKey::ArrowDown => match self.orientation {
                Orientation::Horizontal => None,
                Orientation::Vertical | Orientation::Both => Some(RovingFocusIntent::Next),
            },
        }
    }
}

pub fn use_stable_id(custom_id: Option<Cow<'static, str>>) -> Cow<'static, str> {
    let generated = use_hook(next_global_stable_id);

    custom_id.unwrap_or_else(|| Cow::Owned(generated.clone()))
}

fn next_global_stable_id() -> String {
    let next = NEXT_STABLE_ID.fetch_add(1, Ordering::Relaxed);
    format!("monoxus-{next}")
}

fn registry_index<K, V>(registry: &CollectionRegistry<K, V>, key: &K) -> Option<usize>
where
    K: PartialEq,
{
    registry.entries.iter().position(|entry| entry.key.eq(key))
}

fn next_focusable_index(
    focusable_indices: &[usize],
    current_index: Option<usize>,
    looping: bool,
) -> Option<usize> {
    let current_position = current_index.and_then(|index| {
        focusable_indices
            .iter()
            .position(|candidate| *candidate == index)
    });

    match current_position {
        Some(position) if position + 1 < focusable_indices.len() => {
            focusable_indices.get(position + 1).copied()
        }
        Some(_) if looping => focusable_indices.first().copied(),
        Some(_) => None,
        None => focusable_indices.first().copied(),
    }
}

fn previous_focusable_index(
    focusable_indices: &[usize],
    current_index: Option<usize>,
    looping: bool,
) -> Option<usize> {
    let current_position = current_index.and_then(|index| {
        focusable_indices
            .iter()
            .position(|candidate| *candidate == index)
    });

    match current_position {
        Some(position) if position > 0 => focusable_indices.get(position - 1).copied(),
        Some(_) if looping => focusable_indices.last().copied(),
        Some(_) => None,
        None => focusable_indices.last().copied(),
    }
}

#[cfg(test)]
#[derive(Clone, Debug, Default)]
struct StableIdGenerator {
    next: u64,
}

#[cfg(test)]
impl StableIdGenerator {
    fn with_seed(next: u64) -> Self {
        Self { next }
    }

    fn next_id(&mut self) -> String {
        let next = self.next;
        self.next += 1;
        format!("monoxus-{next}")
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CollectionRegistry, Direction, Orientation, RovingFocusController, RovingFocusIntent,
        RovingFocusKey, ScopeHandle, StableIdGenerator, use_direction,
    };

    #[test]
    fn scope_handles_build_collision_safe_nested_tokens() {
        let first = ScopeHandle::root("a").child("bc").qualify("leaf");
        let second = ScopeHandle::root("ab").child("c").qualify("leaf");

        assert_ne!(first, second);
        assert_eq!(
            ScopeHandle::root("menu").child("item").token(),
            "4:menu|4:item"
        );
    }

    #[test]
    fn collection_registry_preserves_registration_order_when_updating() {
        let mut registry = CollectionRegistry::new();
        assert!(registry.register("first", 1_u8));
        assert!(registry.register("second", 2_u8));
        assert!(!registry.register("first", 3_u8));

        let keys: Vec<_> = registry.iter().map(|entry| *entry.key()).collect();
        assert_eq!(keys, vec!["first", "second"]);
        assert_eq!(registry.get(&"first"), Some(&3_u8));
    }

    #[test]
    fn direction_resolution_prefers_override_then_inherited_then_default() {
        assert_eq!(
            use_direction(Some(Direction::Rtl), Some(Direction::Ltr)),
            Direction::Rtl
        );
        assert_eq!(use_direction(None, Some(Direction::Rtl)), Direction::Rtl);
        assert_eq!(use_direction(None, None), Direction::Ltr);
    }

    #[test]
    fn roving_focus_navigation_consumes_external_state_and_direction() {
        let mut registry = CollectionRegistry::new();
        registry.register("first", true);
        registry.register("second", false);
        registry.register("third", true);

        let controller = RovingFocusController::new(Direction::Rtl)
            .with_orientation(Orientation::Horizontal)
            .with_looping(true);

        assert_eq!(
            controller.navigate_by_key(
                &registry,
                Some(&"first"),
                RovingFocusKey::ArrowLeft,
                |focusable| *focusable,
            ),
            Some("third"),
        );
        assert_eq!(
            controller.navigate(
                &registry,
                Some(&"third"),
                RovingFocusIntent::Previous,
                |focusable| *focusable,
            ),
            Some("first"),
        );
    }

    #[test]
    fn stable_id_generation_is_deterministic() {
        let mut generator = StableIdGenerator::with_seed(7);

        assert_eq!(generator.next_id(), "monoxus-7");
        assert_eq!(generator.next_id(), "monoxus-8");
    }
}
