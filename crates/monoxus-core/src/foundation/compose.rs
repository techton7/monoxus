use std::{error::Error, fmt};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AsChildSlotError {
    MissingProjectionTarget,
    MultipleProjectionTargets { count: usize },
}

impl fmt::Display for AsChildSlotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingProjectionTarget => {
                formatter.write_str("AsChildSlot requires exactly one projection target")
            }
            Self::MultipleProjectionTargets { count } => write!(
                formatter,
                "AsChildSlot expected exactly one projection target but received {count}",
            ),
        }
    }
}

impl Error for AsChildSlotError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AsChildSlot<T> {
    target: T,
}

impl<T> AsChildSlot<T> {
    pub const fn new(target: T) -> Self {
        Self { target }
    }

    pub fn try_from_option(target: Option<T>) -> Result<Self, AsChildSlotError> {
        target
            .map(Self::new)
            .ok_or(AsChildSlotError::MissingProjectionTarget)
    }

    pub fn try_from_iter<I>(targets: I) -> Result<Self, AsChildSlotError>
    where
        I: IntoIterator<Item = T>,
    {
        let mut targets = targets.into_iter();
        let Some(target) = targets.next() else {
            return Err(AsChildSlotError::MissingProjectionTarget);
        };

        let count = 1 + targets.count();
        if count == 1 {
            Ok(Self::new(target))
        } else {
            Err(AsChildSlotError::MultipleProjectionTargets { count })
        }
    }

    pub fn target(&self) -> &T {
        &self.target
    }

    pub fn into_target(self) -> T {
        self.target
    }

    pub fn with_slottable<C>(self, content: Slottable<C>) -> (T, C) {
        (self.target, content.into_inner())
    }
}

impl<T> TryFrom<Option<T>> for AsChildSlot<T> {
    type Error = AsChildSlotError;

    fn try_from(value: Option<T>) -> Result<Self, Self::Error> {
        Self::try_from_option(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slottable<T> {
    content: T,
}

impl<T> Slottable<T> {
    pub const fn new(content: T) -> Self {
        Self { content }
    }

    pub fn content(&self) -> &T {
        &self.content
    }

    pub fn into_inner(self) -> T {
        self.content
    }

    pub fn map<U>(self, project: impl FnOnce(T) -> U) -> Slottable<U> {
        Slottable::new(project(self.content))
    }
}

impl<T> From<T> for Slottable<T> {
    fn from(content: T) -> Self {
        Self::new(content)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EventHandlerOptions<E> {
    check_default_prevented: bool,
    is_default_prevented: fn(&E) -> bool,
}

impl<E> EventHandlerOptions<E> {
    pub const fn always_invoke_internal() -> Self {
        Self {
            check_default_prevented: false,
            is_default_prevented: never_default_prevented::<E>,
        }
    }

    pub const fn cancelable(is_default_prevented: fn(&E) -> bool) -> Self {
        Self {
            check_default_prevented: true,
            is_default_prevented,
        }
    }
}

impl<E> Default for EventHandlerOptions<E> {
    fn default() -> Self {
        Self::always_invoke_internal()
    }
}

fn never_default_prevented<E>(_: &E) -> bool {
    false
}

pub fn compose_event_handlers<E, C, I>(
    consumer: Option<C>,
    internal: Option<I>,
    options: EventHandlerOptions<E>,
) -> impl FnMut(&mut E)
where
    C: FnMut(&mut E),
    I: FnMut(&mut E),
{
    let mut consumer = consumer;
    let mut internal = internal;

    move |event| {
        if let Some(handler) = consumer.as_mut() {
            handler(event);
        }

        let skip_internal =
            options.check_default_prevented && (options.is_default_prevented)(event);

        if !skip_internal && let Some(handler) = internal.as_mut() {
            handler(event);
        }
    }
}

pub type RefHandler<T> = Box<dyn FnMut(T) + 'static>;

pub fn compose_refs<T, I>(refs: I) -> impl FnMut(T)
where
    T: Clone + 'static,
    I: IntoIterator<Item = Option<RefHandler<T>>>,
{
    let mut refs: Vec<RefHandler<T>> = refs.into_iter().flatten().collect();

    move |resolved| {
        for ref_handler in refs.iter_mut() {
            ref_handler(resolved.clone());
        }
    }
}

pub fn project_as_child<T, C>(target: T, content: Slottable<C>) -> (T, C) {
    AsChildSlot::new(target).with_slottable(content)
}

/// Projects any cloneable primitive attribute bag alongside an `as_child` slot target and content.
pub fn project_attrs_as_child<A, T, C>(
    attrs: &A,
    target: T,
    content: Slottable<C>,
) -> (T, A, C)
where
    A: Clone,
{
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

/// Joins non-empty trimmed CSS class tokens with a single `' '` separator.
/// Returns `None` if no non-empty tokens are present.
pub fn join_classes<'a>(parts: impl IntoIterator<Item = &'a str>) -> Option<String> {
    let mut out = String::new();
    for part in parts {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(trimmed);
    }
    (!out.is_empty()).then_some(out)
}

/// Joins non-empty trimmed inline CSS style segments, inserting `';'` between segments only when
/// missing (never producing `";;"`). Returns `None` if no non-empty declarations are present.
pub fn join_styles<'a>(parts: impl IntoIterator<Item = &'a str>) -> Option<String> {
    let mut out = String::new();
    for part in parts {
        let trimmed = part.trim().trim_start_matches(';').trim_start();
        if trimmed.is_empty() {
            continue;
        }
        if !out.is_empty() {
            if !out.ends_with(';') {
                out.push(';');
            }
            out.push(' ');
        }
        out.push_str(trimmed);
    }
    (!out.is_empty()).then_some(out)
}

pub fn compose_part_event_handlers<E, C, I>(
    consumer: Option<C>,
    internal: Option<I>,
    options: EventHandlerOptions<E>,
) -> impl FnMut(&mut E)
where
    C: FnMut(&mut E),
    I: FnMut(&mut E),
{
    compose_event_handlers(consumer, internal, options)
}

pub fn compose_part_refs<T, I>(refs: I) -> impl FnMut(T)
where
    T: Clone + 'static,
    I: IntoIterator<Item = Option<RefHandler<T>>>,
{
    compose_refs(refs)
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::{
        AsChildSlot, AsChildSlotError, EventHandlerOptions, RefHandler, Slottable,
        compose_event_handlers, compose_refs, project_as_child,
    };

    #[derive(Default)]
    struct TestEvent {
        calls: Vec<&'static str>,
        default_prevented: bool,
    }

    fn is_default_prevented(event: &TestEvent) -> bool {
        event.default_prevented
    }

    #[test]
    fn as_child_slot_requires_exactly_one_target() {
        assert_eq!(
            AsChildSlot::<u8>::try_from_iter(std::iter::empty()),
            Err(AsChildSlotError::MissingProjectionTarget),
        );
        assert_eq!(
            AsChildSlot::try_from_iter([1_u8, 2]),
            Err(AsChildSlotError::MultipleProjectionTargets { count: 2 }),
        );

        let (target, content) = project_as_child("button", Slottable::new("child"));
        assert_eq!(target, "button");
        assert_eq!(content, "child");
    }

    #[test]
    fn composed_event_handlers_run_consumer_first_and_support_cancellation() {
        let mut event = TestEvent::default();
        let mut handlers = compose_event_handlers(
            Some(|event: &mut TestEvent| {
                event.calls.push("consumer");
                event.default_prevented = true;
            }),
            Some(|event: &mut TestEvent| event.calls.push("internal")),
            EventHandlerOptions::cancelable(is_default_prevented),
        );
        handlers(&mut event);
        assert_eq!(event.calls, vec!["consumer"]);

        let mut non_cancelable_event = TestEvent::default();
        let mut non_cancelable = compose_event_handlers(
            Some(|event: &mut TestEvent| {
                event.calls.push("consumer");
                event.default_prevented = true;
            }),
            Some(|event: &mut TestEvent| event.calls.push("internal")),
            EventHandlerOptions::default(),
        );
        non_cancelable(&mut non_cancelable_event);
        assert_eq!(non_cancelable_event.calls, vec!["consumer", "internal"]);
    }

    #[test]
    fn composed_refs_fan_out_the_same_resolved_handle() {
        let first = Rc::new(RefCell::new(Vec::new()));
        let second = Rc::new(RefCell::new(Vec::new()));
        let refs: Vec<Option<RefHandler<String>>> = vec![
            Some(Box::new({
                let first = Rc::clone(&first);
                move |value| first.borrow_mut().push(format!("first:{value}"))
            })),
            None,
            Some(Box::new({
                let second = Rc::clone(&second);
                move |value| second.borrow_mut().push(format!("second:{value}"))
            })),
        ];

        let mut composed = compose_refs(refs);
        composed(String::from("node"));

        assert_eq!(&*first.borrow(), &[String::from("first:node")]);
        assert_eq!(&*second.borrow(), &[String::from("second:node")]);
    }

    #[test]
    fn join_classes_trims_skips_empty_and_joins_with_single_space() {
        use super::join_classes;

        assert_eq!(join_classes(std::iter::empty::<&str>()), None);
        assert_eq!(join_classes(["", "   ", "\t\n"]), None);
        assert_eq!(
            join_classes(["  btn  ", "", "   ", "btn-primary  active "]),
            Some(String::from("btn btn-primary  active"))
        );
    }

    #[test]
    fn join_styles_normalizes_semicolons_without_double_semicolons() {
        use super::join_styles;

        assert_eq!(join_styles(std::iter::empty::<&str>()), None);
        assert_eq!(join_styles(["", "   ", ";", " ; "]), None);
        assert_eq!(
            join_styles(["color: red", "margin: 0"]),
            Some(String::from("color: red; margin: 0"))
        );
        assert_eq!(
            join_styles(["color: red;", "margin: 0;"]),
            Some(String::from("color: red; margin: 0;"))
        );
        assert_eq!(
            join_styles(["  --x: 1px; ", "", "; color: red  ", "z-index: 10;"]),
            Some(String::from("--x: 1px; color: red; z-index: 10;"))
        );
        assert!(!join_styles(["a: 1;", "b: 2;"]).unwrap().contains(";;"));
    }

    #[test]
    fn project_attrs_as_child_clones_attrs_with_target_and_slottable_content() {
        use super::project_attrs_as_child;

        #[derive(Clone, Debug, PartialEq, Eq)]
        struct DummyAttrs(&'static str);

        let attrs = DummyAttrs("role=button");
        let (target, projected, content) =
            project_attrs_as_child(&attrs, "a", Slottable::new("Click"));
        assert_eq!(target, "a");
        assert_eq!(projected, DummyAttrs("role=button"));
        assert_eq!(content, "Click");
    }
}
