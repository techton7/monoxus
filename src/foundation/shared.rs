use std::{
    borrow::Cow,
    sync::atomic::{AtomicU64, Ordering},
};

use dioxus::prelude::*;

pub use monoxus_core::foundation::shared::*;

static NEXT_STABLE_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy)]
pub struct DirectionContext(pub Signal<Direction>);

#[component]
pub fn DirectionProvider(dir: Direction, children: Element) -> Element {
    let mut signal = use_signal(|| dir);
    if *signal.peek() != dir {
        signal.set(dir);
    }
    use_context_provider(|| DirectionContext(signal));
    children
}

pub fn use_direction_context(override_direction: Option<Direction>) -> Direction {
    let inherited = try_consume_context::<DirectionContext>();
    if let Some(dir) = override_direction {
        return dir;
    }
    if let Some(ctx) = inherited {
        return (ctx.0)();
    }
    Direction::Ltr
}

pub fn use_mount_animation_guard(current_active: bool) -> Option<&'static str> {
    let initial_active = use_hook(|| current_active);
    let mut guard_active = use_signal(|| initial_active);

    if *guard_active.peek() && current_active != initial_active {
        guard_active.set(false);
    }

    use_effect(move || {
        if guard_active.try_peek().is_ok_and(|g| *g) {
            spawn(async move {
                let _ = oxidase::frame::next_frame().await;
                if let Ok(mut guard) = guard_active.try_write() {
                    *guard = false;
                }
            });
        }
    });

    if *guard_active.read() {
        Some("animation-name: none; animation-duration: 0s; transition-duration: 0s;")
    } else {
        None
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
    use super::StableIdGenerator;

    #[test]
    fn stable_id_generation_is_deterministic() {
        let mut generator = StableIdGenerator::with_seed(7);

        assert_eq!(generator.next_id(), "monoxus-7");
        assert_eq!(generator.next_id(), "monoxus-8");
    }
}
