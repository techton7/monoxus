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

        if !skip_internal {
            if let Some(handler) = internal.as_mut() {
                handler(event);
            }
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

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::{
        AsChildSlot, AsChildSlotError, EventHandlerOptions, RefHandler, Slottable,
        compose_event_handlers, compose_refs,
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

        let (target, content) = AsChildSlot::new("button").with_slottable(Slottable::new("child"));
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
}
