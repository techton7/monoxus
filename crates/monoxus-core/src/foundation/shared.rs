use std::{borrow::Cow, cell::Cell, rc::Rc};

/// Unified preventable callback event carrying shared interior cancellation state across clones.
#[derive(Clone, Debug)]
pub struct PreventableEvent<T = ()> {
    prevented: Rc<Cell<bool>>,
    detail: T,
}

impl<T> PreventableEvent<T> {
    /// Creates a new non-prevented event carrying `detail`.
    pub fn new(detail: T) -> Self {
        Self {
            prevented: Rc::new(Cell::new(false)),
            detail,
        }
    }

    /// Cancels the primitive's default action (shared across all clones of this event).
    #[inline]
    pub fn prevent_default(&self) {
        self.prevented.set(true);
    }

    /// Returns `true` if `prevent_default` was called on this event or any clone of it.
    #[inline]
    pub fn is_default_prevented(&self) -> bool {
        self.prevented.get()
    }

    /// Returns a shared reference to the event payload.
    #[inline]
    pub fn detail(&self) -> &T {
        &self.detail
    }

    /// Consumes the event and returns its payload.
    #[inline]
    pub fn into_detail(self) -> T {
        self.detail
    }
}

impl PreventableEvent<()> {
    /// Creates an empty non-prevented event with unit payload `()`.
    pub fn empty() -> Self {
        Self::new(())
    }
}

impl<T: Default> Default for PreventableEvent<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: PartialEq> PartialEq for PreventableEvent<T> {
    fn eq(&self, other: &Self) -> bool {
        self.is_default_prevented() == other.is_default_prevented() && self.detail == other.detail
    }
}

impl<T: Eq> Eq for PreventableEvent<T> {}

pub type EscapeKeyDownEvent = PreventableEvent<()>;
pub type OpenAutoFocusEvent = PreventableEvent<()>;
pub type CloseAutoFocusEvent = PreventableEvent<()>;
pub type InteractOutsideEvent = PreventableEvent<Option<String>>;
pub type PointerDownOutsideEvent = PreventableEvent<Option<String>>;
pub type FocusOutsideEvent = PreventableEvent<Option<String>>;

#[inline]
pub const fn data_disabled_attr(disabled: bool) -> Option<&'static str> {
    if disabled { Some("") } else { None }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ElementSize {
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ElementScrollMetrics {
    pub scroll_left: f64,
    pub scroll_top: f64,
    pub scroll_width: f64,
    pub scroll_height: f64,
    pub client_width: f64,
    pub client_height: f64,
}

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
    pub const fn new(key: K, value: V) -> Self {
        Self { key, value }
    }

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

    pub fn reorder_by_keys(&mut self, ordered_keys: &[K]) {
        let mut reordered = Vec::with_capacity(self.entries.len());
        for key in ordered_keys {
            if let Some(pos) = self.entries.iter().position(|entry| entry.key.eq(key)) {
                reordered.push(self.entries.remove(pos));
            }
        }
        reordered.append(&mut self.entries);
        self.entries = reordered;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Direction {
    #[default]
    Ltr,
    Rtl,
}

impl Direction {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Ltr => "ltr",
            Self::Rtl => "rtl",
        }
    }

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

pub fn resolve_direction(
    override_direction: Option<Direction>,
    inherited_direction: Option<Direction>,
) -> Direction {
    Direction::resolve(override_direction, inherited_direction)
}

pub use resolve_direction as use_direction;

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

    pub const fn intent_for_key(&self, key: RovingFocusKey) -> Option<RovingFocusIntent> {
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
mod tests {
    use super::{
        CollectionRegistry, Direction, Orientation, RovingFocusController, RovingFocusIntent,
        RovingFocusKey, ScopeHandle, data_disabled_attr, resolve_direction, use_direction,
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
        let scope = ScopeHandle::root("collapsible").child("accordion");
        assert_eq!(scope.token(), "11:collapsible|9:accordion");
        assert_eq!(
            scope.qualify("trigger"),
            "11:collapsible|9:accordion|7:trigger"
        );
    }

    #[test]
    fn collection_registry_preserves_registration_order_when_updating() {
        let mut registry = CollectionRegistry::new();
        assert!(registry.is_empty());
        assert!(registry.register("first", 1_u8));
        assert!(registry.register("second", 2_u8));
        assert!(!registry.register("first", 3_u8));
        assert_eq!(registry.len(), 2);

        let keys: Vec<_> = registry.iter().map(|entry| *entry.key()).collect();
        assert_eq!(keys, vec!["first", "second"]);
        assert_eq!(registry.get(&"first"), Some(&3_u8));
        assert_eq!(registry.unregister(&"second"), Some(2_u8));
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn direction_resolution_prefers_override_then_inherited_then_default() {
        assert_eq!(
            use_direction(Some(Direction::Rtl), Some(Direction::Ltr)),
            Direction::Rtl
        );
        assert_eq!(
            resolve_direction(None, Some(Direction::Rtl)),
            Direction::Rtl
        );
        assert_eq!(Direction::resolve(None, None), Direction::Ltr);
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

        assert_eq!(controller.direction(), Direction::Rtl);
        assert_eq!(controller.orientation(), Orientation::Horizontal);
        assert!(controller.looping());

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
            controller.navigate_by_key(
                &registry,
                Some(&"first"),
                RovingFocusKey::ArrowRight,
                |focusable| *focusable,
            ),
            Some("third"),
        );
        assert_eq!(
            controller.navigate_by_key(
                &registry,
                Some(&"first"),
                RovingFocusKey::ArrowDown,
                |focusable| *focusable,
            ),
            None,
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
    fn data_disabled_attr_omits_when_enabled() {
        assert_eq!(data_disabled_attr(false), None);
        assert_eq!(data_disabled_attr(true), Some(""));
    }

    #[test]
    fn direction_as_str_returns_ltr_and_rtl() {
        assert_eq!(Direction::Ltr.as_str(), "ltr");
        assert_eq!(Direction::Rtl.as_str(), "rtl");
    }

    #[test]
    fn collection_registry_reorders_only_matching_keys_and_preserves_tail() {
        let mut registry = CollectionRegistry::new();
        registry.register("a", 1);
        registry.register("b", 2);
        registry.register("c", 3);
        registry.register("d", 4);

        registry.reorder_by_keys(&["c", "absent", "a"]);

        let keys: Vec<_> = registry.iter().map(|e| *e.key()).collect();
        assert_eq!(keys, vec!["c", "a", "b", "d"]);
        assert_eq!(registry.get(&"c"), Some(&3));
        assert_eq!(registry.get(&"a"), Some(&1));
        assert_eq!(registry.get(&"b"), Some(&2));
        assert_eq!(registry.get(&"d"), Some(&4));
    }

    #[test]
    fn preventable_event_shares_cancellation_state_across_clones_and_supports_aliases() {
        use super::{
            CloseAutoFocusEvent, EscapeKeyDownEvent, FocusOutsideEvent, InteractOutsideEvent,
            OpenAutoFocusEvent, PointerDownOutsideEvent, PreventableEvent,
        };

        let original: EscapeKeyDownEvent = PreventableEvent::empty();
        let cloned = original.clone();
        assert!(!original.is_default_prevented());
        assert!(!cloned.is_default_prevented());
        assert_eq!(original, cloned);

        cloned.prevent_default();
        assert!(original.is_default_prevented());
        assert!(cloned.is_default_prevented());
        assert_eq!(original, cloned);

        let fresh: OpenAutoFocusEvent = PreventableEvent::default();
        assert_ne!(original, fresh);

        let close_evt: CloseAutoFocusEvent = PreventableEvent::default();
        assert!(!close_evt.is_default_prevented());

        let outside: InteractOutsideEvent = PreventableEvent::new(Some(String::from("outside-btn")));
        let outside_clone = outside.clone();
        outside_clone.prevent_default();
        assert!(outside.is_default_prevented());
        assert_eq!(outside.detail().as_deref(), Some("outside-btn"));
        assert_eq!(outside.into_detail(), Some(String::from("outside-btn")));

        let ptr: PointerDownOutsideEvent = PreventableEvent::default();
        let foc: FocusOutsideEvent = PreventableEvent::default();
        assert_eq!(ptr.detail(), &None);
        assert_eq!(foc.detail(), &None);
    }
}
