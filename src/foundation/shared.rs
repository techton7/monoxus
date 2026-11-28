use std::{
    borrow::Cow,
    sync::atomic::{AtomicU64, Ordering},
};

use dioxus::prelude::*;

pub use ui_sdk::foundation::shared::*;

use super::{
    browser::{
        ResizeObserverBox, get_element_client_rect, is_element_animating,
        measure_element_without_animation, spawn_root_task, start_resize_monitor,
    },
    overlay::{PresenceHandle, use_watcher_slot},
    state::sync_signal,
};

pub const MOUNT_ANIMATION_GUARD_STYLE: &str =
    "animation-name: none; animation-duration: 0s; transition-duration: 0s;";

static NEXT_STABLE_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy)]
pub struct DirectionContext(pub Signal<Direction>);

#[component]
pub fn DirectionProvider(dir: Direction, children: Element) -> Element {
    let mut signal = use_signal(|| dir);
    sync_signal(&mut signal, dir);
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

/// Declarative mount-open animation block cleared after 1 frame or immediate state change
/// (`bits#2209`), with teardown invalidation so unmounting mid-frame never panics (`bits#2149`).
pub fn use_mount_animation_guard(current_active: bool) -> Option<&'static str> {
    let initial_active = use_hook(|| current_active);
    let mut guard_active = use_signal(|| initial_active);
    let alive = use_hook(|| CopyValue::new(true));
    let mut guard_gen = use_hook(|| CopyValue::new(0_u64));

    if *guard_active.peek() && current_active != initial_active {
        guard_active.set(false);
        let next_gen = guard_gen.try_read().map(|g| g.wrapping_add(1)).unwrap_or(1);
        guard_gen.set(next_gen);
    }

    use_effect(move || {
        if guard_active.try_peek().is_ok_and(|g| *g) {
            let expected_gen = guard_gen.try_read().map(|g| *g).unwrap_or(0);
            let mut guard_active = guard_active;
            spawn_root_task(async move {
                let _ = oxidase::frame::next_frame().await;
                if alive.try_read().is_ok_and(|a| *a)
                    && guard_gen.try_read().is_ok_and(|g| *g == expected_gen)
                    && let Ok(mut guard) = guard_active.try_write()
                {
                    *guard = false;
                }
            });
        }
    });

    use_drop(move || {
        let mut alive = alive;
        let mut guard_gen = guard_gen;
        if let Ok(mut flag) = alive.try_write() {
            *flag = false;
        }
        let next_gen = guard_gen.try_read().map(|g| g.wrapping_add(1)).unwrap_or(1);
        if let Ok(mut g) = guard_gen.try_write() {
            *g = next_gen;
        }
    });

    if *guard_active.read() {
        Some(MOUNT_ANIMATION_GUARD_STYLE)
    } else {
        None
    }
}

/// Zero-overhead `Copy` disclosure sizing and motion handle unifying `AccordionContent` and
/// `CollapsibleContent` (`bits#2209/#2149`, `FM-COL-2`).
#[derive(Clone, Copy, PartialEq)]
pub struct DisclosureSizeHandle {
    content_size: Signal<Option<ElementSize>>,
    mounted_rev: Signal<u64>,
    guard_style: CopyValue<Option<&'static str>>,
    element_id: CopyValue<String>,
    is_open: CopyValue<bool>,
    presence: PresenceHandle,
    alive: CopyValue<bool>,
}

impl DisclosureSizeHandle {
    /// Returns the last measured content border-box size (subscribing).
    pub fn size(&self) -> Option<ElementSize> {
        *self.content_size.read()
    }

    /// Returns the declarative mount-open animation block style while active (`bits#2209`).
    pub fn guard_style(&self) -> Option<&'static str> {
        self.guard_style.try_read().ok().and_then(|g| *g)
    }

    /// Invoked from `onmounted` on the disclosure content element.
    ///
    /// Measures natural size without animation and preserves non-zero dimensions during active
    /// open animations across Web and Blitz Native (`animating_zero_height` guard).
    pub fn on_mounted(&self) {
        if !self.alive.try_read().is_ok_and(|a| *a) {
            return;
        }
        let mut rev = self.mounted_rev;
        if let Ok(mut r) = rev.try_write() {
            *r = r.wrapping_add(1);
        }
        if !self.is_open.try_read().is_ok_and(|o| *o) {
            return;
        }
        let cid = self
            .element_id
            .try_read()
            .map(|g| g.clone())
            .unwrap_or_default();
        if cid.is_empty() {
            return;
        }
        if let Some(size) = measure_element_without_animation(&cid)
            && (size.width > 0.0 || size.height > 0.0)
        {
            let existing = self.content_size.try_peek().ok().and_then(|g| *g);
            let animating_zero_height = size.height <= 0.0
                && existing.is_some_and(|prev| prev.height > 0.0)
                && is_element_animating(&cid);
            if !animating_zero_height && existing != Some(size) {
                let mut sig = self.content_size;
                sync_signal(&mut sig, Some(size));
            }
        }
    }

    /// Single motion-completion handler shared by both `ontransitionend` and `onanimationend`.
    pub fn on_motion_end(&self) {
        if !self.alive.try_read().is_ok_and(|a| *a) {
            return;
        }
        let cid = self
            .element_id
            .try_read()
            .map(|g| g.clone())
            .unwrap_or_default();
        if let Some(cycle) = self.presence.active_close_cycle() {
            if !is_element_animating(&cid) {
                self.presence.complete_close_cycle(cycle);
            }
        } else if self.is_open.try_read().is_ok_and(|o| *o)
            && !is_element_animating(&cid)
            && let Some(rect) = get_element_client_rect(&cid)
            && (rect.width > 0.0 || rect.height > 0.0)
        {
            let size = ElementSize {
                width: rect.width,
                height: rect.height,
            };
            let mut sig = self.content_size;
            sync_signal(&mut sig, Some(size));
        }
    }
}

/// Shared zero-overhead disclosure size hook unifying `AccordionContent` and `CollapsibleContent`:
/// - declarative mount-open animation block cleared after 1 frame or immediate close (`bits#2209`),
/// - teardown invalidation preventing mid-frame panics (`bits#2149`),
/// - `measure_element_without_animation` + `start_resize_monitor(BorderBox)` + Blitz+Web
///   `animating_zero_height` guard,
/// - single `on_motion_end()` handler for both `ontransitionend` and `onanimationend`.
pub fn use_disclosure_size(
    element_id: &str,
    is_open: bool,
    presence: PresenceHandle,
) -> DisclosureSizeHandle {
    let guard_style_val = use_mount_animation_guard(is_open);
    let mut guard_style = use_hook(|| CopyValue::new(guard_style_val));
    guard_style.set(guard_style_val);

    let mut content_size = use_signal(|| None::<ElementSize>);
    let mounted_rev = use_signal(|| 0_u64);
    let resize_watcher = use_watcher_slot();
    let mut element_id_slot = use_hook(|| CopyValue::new(element_id.to_owned()));
    let mut is_open_slot = use_hook(|| CopyValue::new(is_open));
    let mut was_open = use_hook(|| CopyValue::new(false));
    let mut last_measured_cid = use_hook(|| CopyValue::new(String::new()));
    let mut alive = use_hook(|| CopyValue::new(true));

    if element_id_slot
        .try_read()
        .is_ok_and(|cur| cur.as_str() != element_id)
    {
        element_id_slot.set(element_id.to_owned());
    }
    is_open_slot.set(is_open);

    let cid_owned = element_id.to_owned();
    use_effect(use_reactive(
        (&is_open, &mounted_rev(), &cid_owned),
        move |(open, _rev, cid)| {
            if !open || cid.is_empty() {
                was_open.set(false);
                resize_watcher.clear();
                return;
            }

            let opened_transition = was_open.try_read().is_ok_and(|prev| !*prev);
            let cid_changed = last_measured_cid
                .try_read()
                .is_ok_and(|prev| prev.as_str() != cid.as_str());
            was_open.set(true);

            let needs_measure = opened_transition
                || cid_changed
                || content_size.try_peek().is_ok_and(|cur| cur.is_none());
            if needs_measure
                && let Some(size) = measure_element_without_animation(&cid)
                && (size.width > 0.0 || size.height > 0.0)
            {
                let existing = content_size.try_peek().ok().and_then(|g| *g);
                let animating_zero_height = size.height <= 0.0
                    && existing.is_some_and(|prev| prev.height > 0.0)
                    && is_element_animating(&cid);
                if !animating_zero_height {
                    sync_signal(&mut content_size, Some(size));
                    last_measured_cid.set(cid.clone());
                }
            }

            let cid_for_watch = cid.clone();
            resize_watcher.arm_with(move |expected_gen| {
                let cid_check = cid_for_watch.clone();
                let mut content_size = content_size;
                start_resize_monitor(
                    &cid_for_watch,
                    ResizeObserverBox::BorderBox,
                    move |new_size| {
                        if !alive.try_read().is_ok_and(|a| *a)
                            || resize_watcher.generation() != expected_gen
                        {
                            return;
                        }
                        if is_element_animating(&cid_check) {
                            return;
                        }
                        if new_size.width <= 0.0 && new_size.height <= 0.0 {
                            return;
                        }
                        sync_signal(&mut content_size, Some(new_size));
                    },
                )
            });
        },
    ));

    use_drop(move || {
        if let Ok(mut flag) = alive.try_write() {
            *flag = false;
        }
        resize_watcher.clear();
    });

    DisclosureSizeHandle {
        content_size,
        mounted_rev,
        guard_style,
        element_id: element_id_slot,
        is_open: is_open_slot,
        presence,
        alive,
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
