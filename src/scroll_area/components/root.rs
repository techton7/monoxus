use std::borrow::Cow;
use std::rc::Rc;

use dioxus::core::{AttributeValue, Event, ListenerCallback};
use dioxus::html::PlatformEventData;
use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props},
    shared::{Direction, ScopeHandle, use_direction_context, use_stable_id},
    state::sync_signal,
};

use super::{
    super::{
        attrs::ScrollAreaRootAttributes,
        relationships::ScrollAreaRelationships,
        runtime::{HideDelayTimer, use_scroll_area_runtime},
        types::ScrollAreaType,
    },
    ScrollAreaContext,
};

#[derive(Clone, PartialEq, Props)]
pub struct ScrollAreaRootProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default)]
    pub r#type: ScrollAreaType,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default = 600)]
    pub scroll_hide_delay: u64,
    #[props(default)]
    pub children: Element,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[component]
pub fn ScrollAreaRoot(props: ScrollAreaRootProps) -> Element {
    let dir = use_direction_context(props.dir);

    let scope_id = use_stable_id(props.id.clone().map(Cow::Owned));
    let scope = ScopeHandle::root("scroll-area").child(scope_id);
    let mut relationships = ScrollAreaRelationships::new(scope);
    if let Some(ref custom_id) = props.id {
        relationships = relationships.with_root_id(custom_id.clone());
    }

    let runtime = use_scroll_area_runtime();
    let hide_timer = use_hook(|| Rc::new(HideDelayTimer::new()));

    let mut relationships_sig = use_signal(|| relationships.clone());
    if relationships_sig.peek().root_id() != relationships.root_id() {
        relationships_sig.set(relationships.clone());
    }

    let mut type_sig = use_signal(|| props.r#type);
    sync_signal(&mut type_sig, props.r#type);
    let mut dir_sig = use_signal(|| dir);
    sync_signal(&mut dir_sig, dir);
    let mut scroll_hide_delay_sig = use_signal(|| props.scroll_hide_delay);
    sync_signal(&mut scroll_hide_delay_sig, props.scroll_hide_delay);

    let context = ScrollAreaContext {
        runtime,
        relationships: relationships_sig,
        r#type: type_sig,
        dir: dir_sig,
        scroll_hide_delay: scroll_hide_delay_sig,
    };
    use_context_provider(|| context);

    let corner_w = runtime.corner_width();
    let corner_h = runtime.corner_height();

    let base_style = format!(
        "position: relative; --monoxus-scroll-area-corner-width: {corner_w}px; --monoxus-scroll-area-corner-height: {corner_h}px; --radix-scroll-area-corner-width: {corner_w}px; --radix-scroll-area-corner-height: {corner_h}px;"
    );

    let root_attrs = ScrollAreaRootAttributes::new(relationships_sig.read().root_id().to_string())
        .with_dir(Some(dir));

    let timer_enter = Rc::clone(&hide_timer);
    let mut hover_visible_enter = runtime.hover_visible;
    let timer_leave = Rc::clone(&hide_timer);
    let hover_visible_leave = runtime.hover_visible;

    use_drop({
        let timer = Rc::clone(&hide_timer);
        move || {
            timer.cancel();
        }
    });

    let dir_str = match dir {
        Direction::Ltr => "ltr",
        Direction::Rtl => "rtl",
    };

    let mut base = Vec::with_capacity(6);
    base.push(Attribute::new("id", root_attrs.id().to_string(), None, false));
    base.push(Attribute::new("style", base_style, None, false));
    base.push(Attribute::new("dir", dir_str, None, false));
    base.push(Attribute::new("data-monoxus-scroll-area-root", "", None, false));
    base.push(Attribute::new(
        "onpointerenter",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |_: Event<PlatformEventData>| {
                if type_sig
                    .try_peek()
                    .is_ok_and(|t| *t == ScrollAreaType::Hover)
                {
                    timer_enter.cancel();
                    hover_visible_enter.set(true);
                }
            })
            .erase(),
        ),
        None,
        false,
    ));
    base.push(Attribute::new(
        "onpointerleave",
        AttributeValue::Listener(
            ListenerCallback::<PlatformEventData>::new(move |_: Event<PlatformEventData>| {
                if type_sig
                    .try_peek()
                    .is_ok_and(|t| *t == ScrollAreaType::Hover)
                {
                    let delay = scroll_hide_delay_sig.try_peek().map(|d| *d).unwrap_or(600);
                    let mut hover_visible = hover_visible_leave;
                    timer_leave.schedule(delay, move || {
                        if let Ok(mut guard) = hover_visible.try_write() {
                            *guard = false;
                        }
                    });
                }
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

pub use ScrollAreaRoot as ScrollArea;
