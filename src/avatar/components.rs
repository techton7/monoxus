use std::{borrow::Cow, cell::Cell, rc::Rc, time::Duration};

use dioxus::prelude::*;
use futures_timer::Delay;

use crate::foundation::{
    compose::merge_global_attributes,
    shared::{ScopeHandle, use_stable_id},
    state::{sync_signal, use_controllable_signal},
};

use super::{
    attrs::{AvatarFallbackAttributes, AvatarImageAttributes, AvatarRootAttributes},
    relationships::AvatarRelationships,
    runtime::{AvatarContext, AvatarImageProbe, use_avatar_context, watch_avatar_image},
    types::AvatarImageLoadingStatus,
};

#[derive(Clone)]
pub struct AvatarRootRenderChildProps {
    pub attrs: AvatarRootAttributes,
    pub children: Element,
}

#[derive(Clone)]
pub struct AvatarImageRenderChildProps {
    pub attrs: AvatarImageAttributes,
    pub src: String,
    pub crossorigin: Option<String>,
    pub referrerpolicy: Option<String>,
    pub children: Element,
}

#[derive(Clone)]
pub struct AvatarFallbackRenderChildProps {
    pub attrs: AvatarFallbackAttributes,
    pub children: Element,
}

#[derive(Props, Clone, PartialEq)]
pub struct AvatarRootProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub loading_status: Option<AvatarImageLoadingStatus>,
    #[props(default)]
    pub on_loading_status_change: Option<EventHandler<AvatarImageLoadingStatus>>,
    #[props(default)]
    pub delay_ms: Option<u64>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<AvatarRootRenderChildProps, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn AvatarRoot(props: AvatarRootProps) -> Element {
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned));
    let scope = ScopeHandle::root("avatar").child(stable_id);
    let relationships = AvatarRelationships::new(&scope);

    let status_ctl = use_controllable_signal(
        props.loading_status,
        AvatarImageLoadingStatus::default(),
        props.on_loading_status_change,
    );
    let mut delay_ms = use_signal(|| props.delay_ms);
    sync_signal(&mut delay_ms, props.delay_ms);
    let image_count = use_signal(|| 0);

    let context = use_context_provider(|| AvatarContext {
        status_ctl,
        delay_ms,
        relationships,
        image_count,
    });

    if props.as_child && props.render_child.is_none() {
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::AvatarRoot: as_child requires render_child; rendering the default span"
        );
    }

    let attrs = context.root_attributes();
    if let Some(render_child) = props.render_child.as_ref().filter(|_| props.as_child) {
        return render_child.call(AvatarRootRenderChildProps {
            attrs,
            children: props.children,
        });
    }

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        span {
            id: props.id.clone().unwrap_or_else(|| context.relationships.root_id().to_string()),
            class: merged.class,
            style: merged.style,
            "data-status": attrs.data_status,
            "data-avatar-root": attrs.data_avatar_root,
            onmounted: move |event| {
                if let Some(handler) = props.on_mounted.as_ref() {
                    handler.call(event);
                }
            },
            ..merged.rest,
            {props.children}
        }
    }
}

pub use AvatarRoot as Avatar;

#[derive(Props, Clone, PartialEq)]
pub struct AvatarImageProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub src: Option<String>,
    #[props(default)]
    pub alt: Option<String>,
    #[props(default)]
    pub crossorigin: Option<String>,
    #[props(default)]
    pub referrerpolicy: Option<String>,
    #[props(default)]
    pub on_loading_status_change: Option<EventHandler<AvatarImageLoadingStatus>>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<AvatarImageRenderChildProps, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn AvatarImage(props: AvatarImageProps) -> Element {
    let context = use_avatar_context();
    let mut probe = AvatarImageProbe::new();
    let alive = probe.alive();
    let mut watcher_slot = probe.watcher_slot();
    let src = props.src.clone();
    let options_crossorigin = props.crossorigin.clone();
    let options_referrerpolicy = props.referrerpolicy.clone();
    let mut callback_slot = use_hook(|| CopyValue::new(props.on_loading_status_change));
    callback_slot.set(props.on_loading_status_change);
    let context_for_effect = context.clone();
    let mut last_request = use_signal(|| None::<(Option<String>, Option<String>, Option<String>)>);

    use_hook({
        let context = context.clone();
        move || {
            context.register_image();
        }
    });
    use_drop({
        let cleanup_context = context.clone();
        move || {
            alive.set(false);
            cleanup_context.unregister_image();
            if let Ok(mut slot) = watcher_slot.try_write()
                && let Some(watcher) = slot.take()
            {
                watcher.stop();
            }
        }
    });

    use_effect(use_reactive(
        (&src, &options_crossorigin, &options_referrerpolicy),
        move |(src, crossorigin, referrerpolicy)| {
            let request = (src.clone(), crossorigin.clone(), referrerpolicy.clone());
            if last_request.peek().as_ref() == Some(&request) {
                return;
            }
            last_request.set(Some(request));
            let generation = probe.begin_generation();
            probe.replace_watcher(None);
            let publish_context = context_for_effect.clone();
            let probe_for_callback = probe.clone();
            let owns = probe.owns_generation(generation);

            let Some(src) = src.filter(|value| !value.is_empty()) else {
                if probe_for_callback
                    .transition(generation, AvatarImageLoadingStatus::Error)
                    .is_some()
                {
                    publish_context.publish(AvatarImageLoadingStatus::Error);
                    if let Some(handler) = callback_slot.try_read().ok().and_then(|s| *s) {
                        handler.call(AvatarImageLoadingStatus::Error);
                    }
                }
                return;
            };

            let watched_context = publish_context.clone();
            let watched_probe = probe_for_callback.clone();
            let watcher = match watch_avatar_image(
                &src,
                oxidase::dom::observer::ImageLoadOptions {
                    crossorigin,
                    referrerpolicy,
                },
                generation,
                probe.alive(),
                owns,
                move |generation, status| {
                    if watched_probe.transition(generation, status).is_some() {
                        watched_context.publish(status);
                        if let Some(handler) = callback_slot.try_read().ok().and_then(|s| *s) {
                            handler.call(status);
                        }
                    }
                },
            ) {
                Ok(watcher) => Some(watcher),
                Err(error) => {
                    #[cfg(debug_assertions)]
                    eprintln!("monoxus::AvatarImage: unable to watch {src:?}: {error}");
                    if probe_for_callback
                        .transition(generation, AvatarImageLoadingStatus::Error)
                        .is_some()
                    {
                        publish_context.publish(AvatarImageLoadingStatus::Error);
                        if let Some(handler) = callback_slot.try_read().ok().and_then(|s| *s) {
                            handler.call(AvatarImageLoadingStatus::Error);
                        }
                    }
                    None
                }
            };
            probe.replace_watcher(watcher);
        },
    ));

    let attrs = context.image_attributes();
    if !attrs.data_status.eq("loaded") {
        return rsx! {};
    }
    let Some(src) = props.src.filter(|value| !value.is_empty()) else {
        return rsx! {};
    };

    if let Some(render_child) = props.render_child.as_ref().filter(|_| props.as_child) {
        return render_child.call(AvatarImageRenderChildProps {
            attrs,
            src,
            crossorigin: props.crossorigin,
            referrerpolicy: props.referrerpolicy,
            children: props.children,
        });
    }
    if props.as_child && props.render_child.is_none() {
        #[cfg(debug_assertions)]
        eprintln!("monoxus::AvatarImage: as_child requires render_child");
    }

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        img {
            id: props.id.clone().unwrap_or_else(|| context.relationships.image_id().to_string()),
            class: merged.class,
            style: merged.style,
            src,
            alt: props.alt,
            crossorigin: props.crossorigin,
            referrerpolicy: props.referrerpolicy,
            "data-status": attrs.data_status,
            "data-avatar-image": attrs.data_avatar_image,
            onmounted: move |event| {
                if let Some(handler) = props.on_mounted.as_ref() {
                    handler.call(event);
                }
            },
            ..merged.rest,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AvatarFallbackProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub delay_ms: Option<u64>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<AvatarFallbackRenderChildProps, Element>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn AvatarFallback(props: AvatarFallbackProps) -> Element {
    let context = use_avatar_context();
    let status = context.status();
    let delay_ms = props.delay_ms.or(*context.delay_ms.read());
    let mut elapsed = use_signal(|| delay_ms.unwrap_or(0) == 0);
    let mut task = use_signal(|| None::<dioxus::core::Task>);
    let generation = use_hook(|| Rc::new(Cell::new(0_u64)));
    let alive = use_hook(|| Rc::new(Cell::new(true)));

    use_drop({
        let generation = generation.clone();
        let alive = alive.clone();
        move || {
            alive.set(false);
            generation.set(generation.get().wrapping_add(1));
            if let Ok(mut slot) = task.try_write()
                && let Some(t) = slot.take()
            {
                t.cancel();
            }
        }
    });

    use_effect(use_reactive((&delay_ms,), {
        let generation = generation.clone();
        let alive = alive.clone();
        move |(delay_ms,)| {
            generation.set(generation.get().wrapping_add(1));
            let current = generation.get();
            if let Ok(mut slot) = task.try_write()
                && let Some(t) = slot.take()
            {
                t.cancel();
            }
            let milliseconds = delay_ms.unwrap_or(0);
            if milliseconds == 0 {
                sync_signal(&mut elapsed, true);
                return;
            }
            sync_signal(&mut elapsed, false);
            let mut fallback_elapsed = elapsed;
            let generation = generation.clone();
            let alive = alive.clone();
            let handle = spawn(async move {
                Delay::new(Duration::from_millis(milliseconds)).await;
                if alive.get()
                    && generation.get() == current
                    && let Ok(mut guard) = fallback_elapsed.try_write()
                {
                    *guard = true;
                }
            });
            if let Ok(mut slot) = task.try_write() {
                *slot = Some(handle);
            }
        }
    }));

    if !*elapsed.read() || status.is_loaded() {
        return rsx! {};
    }

    let attrs = context.fallback_attributes();
    if let Some(render_child) = props.render_child.as_ref().filter(|_| props.as_child) {
        return render_child.call(AvatarFallbackRenderChildProps {
            attrs,
            children: props.children,
        });
    }
    if props.as_child && props.render_child.is_none() {
        #[cfg(debug_assertions)]
        eprintln!("monoxus::AvatarFallback: as_child requires render_child");
    }

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        span {
            id: props.id.clone().unwrap_or_else(|| context.relationships.fallback_id().to_string()),
            class: merged.class,
            style: merged.style,
            "data-status": attrs.data_status,
            "data-avatar-fallback": attrs.data_avatar_fallback,
            onmounted: move |event| {
                if let Some(handler) = props.on_mounted.as_ref() {
                    handler.call(event);
                }
            },
            ..merged.rest,
            {props.children}
        }
    }
}

pub use AvatarFallback as Fallback;
pub use AvatarImage as Image;
