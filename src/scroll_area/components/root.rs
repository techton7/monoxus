use std::rc::Rc;

use dioxus::prelude::*;

use crate::foundation::shared::{Direction, ScopeHandle, use_direction_context};

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
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub r#type: ScrollAreaType,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default = 600)]
    pub scroll_hide_delay: u64,
    pub as_child: Option<bool>,
    pub children: Element,
    pub class: Option<String>,
    pub style: Option<String>,
}

#[component]
pub fn ScrollAreaRoot(props: ScrollAreaRootProps) -> Element {
    let dir = use_direction_context(props.dir);

    let scope_id = props
        .id
        .clone()
        .unwrap_or_else(|| "scroll-area".to_string());
    let scope = ScopeHandle::root("scroll-area").child(scope_id);
    let mut relationships = ScrollAreaRelationships::new(scope);
    if let Some(ref custom_id) = props.id {
        relationships = relationships.with_root_id(custom_id.clone());
    }

    let runtime = use_scroll_area_runtime();
    let hide_timer = use_hook(|| Rc::new(HideDelayTimer::new()));

    let mut relationships_sig = use_signal(|| relationships.clone());
    use_effect(use_reactive((&props.id,), move |(id_opt,)| {
        if let Some(id) = id_opt {
            if relationships_sig.peek().root_id() != id {
                let updated = relationships_sig.peek().clone().with_root_id(id);
                relationships_sig.set(updated);
            }
        }
    }));

    let context = ScrollAreaContext {
        runtime,
        relationships: relationships_sig,
        r#type: props.r#type,
        dir,
        scroll_hide_delay: props.scroll_hide_delay,
    };
    use_context_provider(|| context);

    let corner_w = runtime.corner_width();
    let corner_h = runtime.corner_height();

    let mut root_style = format!(
        "position: relative; --monoxus-scroll-area-corner-width: {corner_w}px; --monoxus-scroll-area-corner-height: {corner_h}px; --radix-scroll-area-corner-width: {corner_w}px; --radix-scroll-area-corner-height: {corner_h}px; "
    );
    if let Some(user_style) = &props.style {
        root_style.push_str(user_style);
    }

    let root_attrs = ScrollAreaRootAttributes::new(relationships_sig.read().root_id().to_string())
        .with_dir(Some(dir));

    let onpointerenter = {
        let timer = Rc::clone(&hide_timer);
        let mut hover_visible = runtime.hover_visible;
        move |_| {
            if props.r#type == ScrollAreaType::Hover {
                timer.cancel();
                hover_visible.set(true);
            }
        }
    };

    let onpointerleave = {
        let timer = Rc::clone(&hide_timer);
        let mut hover_visible = runtime.hover_visible;
        let delay = props.scroll_hide_delay;
        move |_| {
            if props.r#type == ScrollAreaType::Hover {
                timer.schedule(delay, move || {
                    hover_visible.set(false);
                });
            }
        }
    };

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

    rsx! {
        div {
            id: "{root_attrs.id()}",
            class: props.class.as_deref(),
            style: "{root_style}",
            dir: "{dir_str}",
            "data-monoxus-scroll-area-root": "",
            onpointerenter: onpointerenter,
            onpointerleave: onpointerleave,
            {props.children}
        }
    }
}

#[component]
pub fn ScrollArea(props: ScrollAreaRootProps) -> Element {
    rsx! {
        ScrollAreaRoot { ..props }
    }
}
