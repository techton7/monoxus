use dioxus::core::{AttributeValue, Event, ListenerCallback};
use dioxus::html::PlatformEventData;
use dioxus::prelude::*;

use crate::collapsible::attrs::{format_collapsible_css_vars, merge_disclosure_styles};
use crate::collapsible::components::CollapsibleContext;
use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_attr_opt, push_bool_data_attr},
    overlay::use_presence,
    shared::use_disclosure_size,
    state::sync_signal,
};

#[derive(Props, Clone, PartialEq)]
pub struct CollapsibleContentProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default = false)]
    pub hidden_until_found: bool,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn CollapsibleContent(props: CollapsibleContentProps) -> Element {
    let ctx = use_context::<CollapsibleContext>();
    let runtime = ctx.runtime.read();
    let is_open = runtime.is_open();

    let content = runtime.content();
    let content_id = props.id.clone().unwrap_or_else(|| content.id().to_string());

    let mut published_custom_id: CopyValue<Option<String>> = use_hook(|| CopyValue::new(None));
    {
        let mut cid_sig = ctx.content_id;
        let prev_own = published_custom_id
            .try_read()
            .ok()
            .and_then(|g| g.clone());
        match props.id.as_ref() {
            Some(id) => {
                sync_signal(&mut cid_sig, Some(id.clone()));
                if prev_own.as_deref() != Some(id.as_str()) {
                    published_custom_id.set(Some(id.clone()));
                }
            }
            None => {
                if let Some(prev) = prev_own.as_deref()
                    && cid_sig
                        .try_peek()
                        .is_ok_and(|cur| cur.as_deref() == Some(prev))
                {
                    sync_signal(&mut cid_sig, None);
                }
                if prev_own.is_some() {
                    published_custom_id.set(None);
                }
            }
        }
    }

    let presence = use_presence(is_open, &content_id, true, None);
    let disclosure = use_disclosure_size(&content_id, is_open, presence);

    let is_presence_mounted = presence.should_render();
    let should_render = is_open || is_presence_mounted || props.force_mount;

    {
        let mut mounted_sig = ctx.content_mounted;
        sync_signal(&mut mounted_sig, should_render);
    }

    use_drop({
        let mut mounted_sig = ctx.content_mounted;
        let mut cid_sig = ctx.content_id;
        move || {
            if mounted_sig.try_peek().is_ok_and(|m| *m)
                && let Ok(mut m) = mounted_sig.try_write()
            {
                *m = false;
            }
            if let Some(own_id) = published_custom_id
                .try_read()
                .ok()
                .and_then(|g| g.clone())
                && cid_sig
                    .try_peek()
                    .is_ok_and(|cur| cur.as_deref() == Some(own_id.as_str()))
            {
                sync_signal(&mut cid_sig, None);
            }
        }
    });

    if !should_render {
        return rsx! {};
    }

    let hidden_attr: Option<&'static str> = if is_open || is_presence_mounted {
        None
    } else if props.hidden_until_found {
        Some("until-found")
    } else {
        Some("true")
    };

    let css_vars = disclosure.size().map(format_collapsible_css_vars);
    let base_style =
        merge_disclosure_styles(None, disclosure.guard_style(), css_vars.as_deref());

    let mut base = Vec::with_capacity(9);
    base.push(Attribute::new("id", content_id, None, false));
    push_attr_opt(&mut base, "style", base_style);
    push_attr_opt(&mut base, "hidden", hidden_attr);
    base.push(Attribute::new(
        "data-state",
        content.data_state_str(),
        None,
        false,
    ));
    push_bool_data_attr(&mut base, "data-disabled", content.is_disabled());
    base.push(Attribute::new(
        "onmounted",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |_: Event<PlatformEventData>| {
                disclosure.on_mounted();
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "ontransitionend",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |_: Event<PlatformEventData>| {
                disclosure.on_motion_end();
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onanimationend",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |_: Event<PlatformEventData>| {
                disclosure.on_motion_end();
            })
            .erase(),
        ),
        None,
        false,
    ));

    let user = collect_user_attrs(props.class, props.style, props.attributes);
    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            {props.children}
        }
    }
}
