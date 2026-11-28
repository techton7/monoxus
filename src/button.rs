use std::{
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use dioxus::{core::AttributeValue, prelude::*};
use ui_sdk::agent::{
    ActionDescriptor, AgentHints, AgentSurface, ComponentSnapshot, SnapshotValue,
};

use crate::{
    agent::{
        ActionDispatcher, DispatchExecResult, SnapshotReader, SurfaceMetaDefaults,
        use_agent_bridge, use_agent_surface,
    },
    foundation::{
        browser::{announce_polite, click_element_by_id, spawn_root_task},
        compose::{
            MergePropsConfig, merge_props, merge_props_with_config, push_attr_opt,
            push_event_handler,
        },
        keyboard::is_activation_key,
    },
};

pub use ui_sdk::button::*;

static NEXT_AGENT_BUTTON_ID: AtomicU64 = AtomicU64::new(1);
const PENDING_ANNOUNCEMENT_DELAY: Duration = Duration::from_millis(1000);

/// Zero-overhead `Copy` handle for coordinating an async button action with
/// double-submit guarding and unmount cancellation safety (`S2-LD-09`).
#[derive(Clone, Copy, PartialEq)]
pub struct AsyncActionHandle {
    pending: Signal<bool>,
    generation: CopyValue<u64>,
    alive: CopyValue<bool>,
}

struct AsyncPendingResetGuard {
    pending: Signal<bool>,
    generation: CopyValue<u64>,
    alive: CopyValue<bool>,
    expected_gen: u64,
}

impl Drop for AsyncPendingResetGuard {
    fn drop(&mut self) {
        let still_alive = self.alive.try_read().is_ok_and(|flag| *flag);
        let matches_gen = self
            .generation
            .try_read()
            .is_ok_and(|g| *g == self.expected_gen);
        if still_alive && matches_gen {
            let mut pending = self.pending;
            if let Ok(mut guard) = pending.try_write() {
                *guard = false;
            }
        }
    }
}

impl AsyncActionHandle {
    /// Returns `true` while an async action launched through this handle is in flight,
    /// subscribing the active reactive scope.
    pub fn pending(&self) -> bool {
        (self.pending)()
    }

    /// Non-subscribing, drop-safe check of whether an action is currently in flight.
    pub fn peek_pending(&self) -> bool {
        self.pending.try_peek().is_ok_and(|flag| *flag)
    }

    /// Launches `fut` unless another action is already pending on this handle.
    ///
    /// Automatically clears the pending state when `fut` settles, and never writes
    /// to dropped signals if the owning component unmounts before settlement.
    pub fn run(&self, fut: impl Future<Output = ()> + 'static) {
        if self.peek_pending() || !self.alive.try_read().is_ok_and(|flag| *flag) {
            return;
        }

        let mut gen_slot = self.generation;
        let expected_gen = gen_slot
            .try_write()
            .map(|mut g| {
                *g = g.wrapping_add(1);
                *g
            })
            .unwrap_or(0);

        let mut pending_sig = self.pending;
        if let Ok(mut guard) = pending_sig.try_write() {
            *guard = true;
        }

        let guard = AsyncPendingResetGuard {
            pending: self.pending,
            generation: self.generation,
            alive: self.alive,
            expected_gen,
        };

        spawn_root_task(async move {
            let _reset_guard = guard;
            fut.await;
        });
    }
}

/// Creates a component-scoped [`AsyncActionHandle`] with double-submit guarding
/// and unmount-safe signal writes.
pub fn use_async_action() -> AsyncActionHandle {
    let pending = use_signal(|| false);
    let mut generation = use_hook(|| CopyValue::new(0_u64));
    let mut alive = use_hook(|| CopyValue::new(true));

    use_drop(move || {
        if let Ok(mut flag) = alive.try_write() {
            *flag = false;
        }
        if let Ok(mut g) = generation.try_write() {
            *g = g.wrapping_add(1);
        }
    });

    AsyncActionHandle {
        pending,
        generation,
        alive,
    }
}

/// Cloneable async click action wrapper for [`ButtonProps::on_action`].
#[derive(Clone)]
pub struct AsyncButtonAction {
    handler: Rc<dyn Fn(MouseEvent) -> Pin<Box<dyn Future<Output = ()>>>>,
}

impl PartialEq for AsyncButtonAction {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.handler, &other.handler)
    }
}

impl std::fmt::Debug for AsyncButtonAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AsyncButtonAction(..)")
    }
}

impl AsyncButtonAction {
    /// Wraps an async click handler closure into an [`AsyncButtonAction`].
    pub fn new<F, Fut>(handler: F) -> Self
    where
        F: Fn(MouseEvent) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        Self {
            handler: Rc::new(move |evt| Box::pin(handler(evt))),
        }
    }

    /// Invokes the underlying handler to produce the action future.
    pub fn call(&self, evt: MouseEvent) -> Pin<Box<dyn Future<Output = ()>>> {
        (self.handler)(evt)
    }
}

fn extract_aria_label(attrs: &[Attribute]) -> Option<String> {
    attrs.iter().find_map(|attr| {
        if attr.name == "aria-label" {
            match &attr.value {
                AttributeValue::Text(s) if !s.trim().is_empty() => Some(s.clone()),
                _ => None,
            }
        } else {
            None
        }
    })
}

/// Properties for the headless `Button` component.
#[derive(Clone, PartialEq, Props)]
pub struct ButtonProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub r#type: ButtonType,
    #[props(default, into)]
    pub form: Option<String>,
    #[props(default, into)]
    pub href: Option<String>,
    #[props(default, into)]
    pub target: Option<String>,
    #[props(default, into)]
    pub rel: Option<String>,
    #[props(default, into)]
    pub download: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub pending: bool,
    #[props(default = false)]
    pub focusable_when_disabled: bool,
    #[props(default, into)]
    pub pending_announcement: Option<String>,
    #[props(default)]
    pub tabindex: Option<i32>,
    #[props(default, into)]
    pub role: Option<String>,
    #[props(default)]
    pub on_press: Option<EventHandler<()>>,
    #[props(default)]
    pub on_action: Option<AsyncButtonAction>,
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(default)]
    pub agent: Option<AgentHints>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Headless polymorphic button component rendering `<button>` or `<a>`.
#[component]
pub fn Button(props: ButtonProps) -> Element {
    let internal_action = use_async_action();
    let is_pending = props.pending || internal_action.pending();
    let disabled = props.disabled;
    let effective_pending = is_pending && !disabled;
    let is_blocked = disabled || effective_pending;

    // 1000ms delayed polite announcement while continuously pending (`S2-LD-09`).
    let mut announce_armed = use_hook(|| CopyValue::new(false));
    let mut announce_gen = use_hook(|| CopyValue::new(0_u64));
    let mut announce_alive = use_hook(|| CopyValue::new(true));
    let mut announce_msg_slot = use_hook(|| CopyValue::new(props.pending_announcement.clone()));
    announce_msg_slot.set(props.pending_announcement.clone());

    if effective_pending {
        let already_armed = announce_armed.try_read().is_ok_and(|flag| *flag);
        if !already_armed {
            announce_armed.set(true);
            let expected_gen = announce_gen
                .try_write()
                .map(|mut g| {
                    *g = g.wrapping_add(1);
                    *g
                })
                .unwrap_or(0);
            if dioxus::core::Runtime::try_current().is_some() {
                spawn_root_task(async move {
                    futures_timer::Delay::new(PENDING_ANNOUNCEMENT_DELAY).await;
                    let still_valid = announce_alive.try_read().is_ok_and(|a| *a)
                        && announce_armed.try_read().is_ok_and(|armed| *armed)
                        && announce_gen.try_read().is_ok_and(|g| *g == expected_gen);
                    if still_valid {
                        let custom = announce_msg_slot
                            .try_read()
                            .ok()
                            .and_then(|slot| slot.clone());
                        let message = custom.as_deref().unwrap_or("Loading");
                        if !message.trim().is_empty() {
                            announce_polite(message);
                        }
                    }
                });
            }
        }
    } else if announce_armed.try_read().is_ok_and(|flag| *flag) {
        announce_armed.set(false);
        if let Ok(mut g) = announce_gen.try_write() {
            *g = g.wrapping_add(1);
        }
    }

    use_drop(move || {
        if let Ok(mut flag) = announce_alive.try_write() {
            *flag = false;
        }
        if let Ok(mut armed) = announce_armed.try_write() {
            *armed = false;
        }
        if let Ok(mut g) = announce_gen.try_write() {
            *g = g.wrapping_add(1);
        }
    });

    // Allocate a stable DOM id only when inside <AgentProvider> and caller omitted `id`.
    let has_agent_bridge = use_agent_bridge().is_some();
    let mut auto_dom_id = use_hook(|| CopyValue::new(Option::<String>::None));
    let effective_id = if props.id.is_some() {
        props.id.clone()
    } else if has_agent_bridge {
        let existing = auto_dom_id.try_read().ok().and_then(|g| g.clone());
        match existing {
            Some(id) => Some(id),
            None => {
                let id = format!(
                    "monoxus-button-{}",
                    NEXT_AGENT_BUTTON_ID.fetch_add(1, Ordering::Relaxed)
                );
                auto_dom_id.set(Some(id.clone()));
                Some(id)
            }
        }
    } else {
        None
    };

    // Sync slots for zero-bloat AgentSurface dispatcher & snapshot reader (`S2-LD-04`).
    let mut on_press_slot = use_hook(|| CopyValue::new(props.on_press));
    on_press_slot.set(props.on_press);
    let mut dom_id_slot = use_hook(|| CopyValue::new(effective_id.clone()));
    dom_id_slot.set(effective_id.clone());
    let mut blocked_slot = use_hook(|| CopyValue::new(is_blocked));
    blocked_slot.set(is_blocked);
    let mut pending_slot = use_hook(|| CopyValue::new(is_pending));
    pending_slot.set(is_pending);

    let agent_aria_label = if has_agent_bridge {
        extract_aria_label(&props.attributes)
    } else {
        None
    };

    use_agent_surface(
        props.agent.as_ref(),
        SurfaceMetaDefaults::default(),
        || {
            let label = agent_aria_label
                .or_else(|| effective_id.clone())
                .unwrap_or_else(|| String::from("button"));
            AgentSurface::new("button", label)
                .with_disabled(disabled || is_pending)
                .with_snapshot(
                    ComponentSnapshot::new()
                        .with_field("pending", SnapshotValue::Bool(is_pending))
                        .with_pending(is_pending),
                )
                .with_action(ActionDescriptor::new("press", "Press the button", false))
        },
        || {
            let dispatcher: ActionDispatcher = Rc::new(move |dispatch| {
                if dispatch.action != "press" {
                    return DispatchExecResult::ok()
                        .with_host_error(format!("unsupported action `{}`", dispatch.action));
                }
                if blocked_slot.try_read().is_ok_and(|blocked| *blocked) {
                    return DispatchExecResult::ok().with_canceled(true);
                }
                if let Some(press) = on_press_slot.try_read().ok().and_then(|slot| *slot) {
                    press.call(());
                } else if let Some(dom_id) =
                    dom_id_slot.try_read().ok().and_then(|slot| slot.clone())
                {
                    let _ = click_element_by_id(&dom_id);
                }
                DispatchExecResult::ok()
            });
            dispatcher
        },
        || {
            let reader: SnapshotReader = Rc::new(move || {
                let pending_now = pending_slot.try_read().map(|p| *p).unwrap_or(false);
                ComponentSnapshot::new()
                    .with_field("pending", SnapshotValue::Bool(pending_now))
                    .with_pending(pending_now)
            });
            reader
        },
    );

    let state = ButtonState {
        id: effective_id,
        button_type: props.r#type,
        href: props.href.clone(),
        disabled,
        role: props.role.clone(),
        tabindex: props.tabindex,
    };
    let attrs = state.attributes();

    let mut base = Vec::with_capacity(18);
    push_attr_opt(&mut base, "id", attrs.id);
    if attrs.tag == ButtonTag::Anchor {
        push_attr_opt(&mut base, "href", attrs.href);
        push_attr_opt(&mut base, "role", attrs.role);
        push_attr_opt(&mut base, "tabindex", attrs.tabindex);
        let aria_disabled = if disabled || effective_pending {
            Some("true")
        } else {
            attrs.aria_disabled
        };
        push_attr_opt(&mut base, "aria-disabled", aria_disabled);
        if !disabled {
            push_attr_opt(&mut base, "target", props.target);
            push_attr_opt(&mut base, "rel", props.rel);
            push_attr_opt(&mut base, "download", props.download);
        }
    } else {
        push_attr_opt(&mut base, "type", attrs.r#type);
        push_attr_opt(&mut base, "form", props.form);
        if disabled && !props.focusable_when_disabled {
            base.push(Attribute::new("disabled", true, None, false));
        }
        if (disabled && props.focusable_when_disabled) || effective_pending {
            base.push(Attribute::new("aria-disabled", "true", None, false));
        }
        push_attr_opt(&mut base, "role", attrs.role);
        push_attr_opt(&mut base, "tabindex", attrs.tabindex);
    }
    if effective_pending {
        base.push(Attribute::new("aria-busy", "true", None, false));
        base.push(Attribute::new("data-pending", "", None, false));
    }
    base.push(Attribute::new(
        "data-button-root",
        attrs.data_button_root,
        None,
        false,
    ));
    push_attr_opt(&mut base, "data-disabled", attrs.data_disabled);

    if is_blocked {
        push_event_handler(
            &mut base,
            "onclick",
            Some(EventHandler::new(move |evt: MouseEvent| {
                evt.prevent_default();
            })),
        );
        push_event_handler(
            &mut base,
            "onkeydown",
            Some(EventHandler::new(move |evt: KeyboardEvent| {
                if is_activation_key(&evt.key()) {
                    evt.prevent_default();
                }
            })),
        );
    } else {
        let on_press_cb = props.on_press;
        let on_click_cb = props.on_click;
        let on_action_cb = props.on_action.clone();
        let action_for_click = internal_action;
        if on_press_cb.is_some() || on_click_cb.is_some() || on_action_cb.is_some() {
            push_event_handler(
                &mut base,
                "onclick",
                Some(EventHandler::new(move |evt: MouseEvent| {
                    if action_for_click.peek_pending() {
                        evt.prevent_default();
                        return;
                    }
                    if let Some(press) = on_press_cb {
                        press.call(());
                    }
                    if let Some(click) = on_click_cb {
                        click.call(evt.clone());
                    }
                    if let Some(ref action) = on_action_cb
                        && evt.default_action_enabled()
                    {
                        let fut = action.call(evt);
                        action_for_click.run(fut);
                    }
                })),
            );
        }
    }

    let mut user = Vec::with_capacity(props.attributes.len() + 5);
    push_attr_opt(&mut user, "class", props.class);
    push_attr_opt(&mut user, "style", props.style);
    if !is_blocked {
        push_event_handler(&mut user, "onkeydown", props.on_key_down);
    } else if let Some(handler) = props.on_key_down {
        push_event_handler(
            &mut user,
            "onkeydown",
            Some(EventHandler::new(move |evt: KeyboardEvent| {
                if is_activation_key(&evt.key()) {
                    evt.prevent_default();
                    return;
                }
                handler.call(evt);
            })),
        );
    }
    push_event_handler(&mut user, "onmounted", props.on_mounted);
    user.extend(props.attributes);

    let merged = if is_blocked {
        merge_props_with_config(
            base,
            user,
            MergePropsConfig {
                invariant_attrs: &[],
                bookkeeping_events: &["onclick", "onkeydown"],
            },
        )
    } else {
        merge_props(base, user)
    };

    if let Some(child) = props.child {
        return child.call(merged);
    }

    if attrs.tag == ButtonTag::Anchor {
        rsx! {
            a {
                ..merged,
                {props.children}
            }
        }
    } else {
        rsx! {
            button {
                ..merged,
                {props.children}
            }
        }
    }
}

pub use Button as ButtonRoot;
