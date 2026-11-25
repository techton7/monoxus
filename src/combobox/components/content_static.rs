use dioxus::prelude::keyboard_types::{Key, NamedKey};
use dioxus::prelude::*;

use crate::foundation::overlay::{
    OverlayBranchContext, PlacementAlign, PlacementSide, is_topmost_overlay_layer,
    register_active_overlay_layer, unregister_active_overlay_layer,
};

use crate::combobox::{
    attrs::ComboboxContentAttrs, monitors::ComboboxInteractOutsideEvent,
    runtime::use_combobox_context,
};

#[derive(Clone, PartialEq)]
pub struct ComboboxContentStaticRenderProps {
    pub attrs: ComboboxContentAttrs,
    pub class: Option<String>,
    pub style: String,
    pub on_key_down: EventHandler<KeyboardEvent>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Props, Clone, PartialEq)]
pub struct ComboboxContentStaticProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub force_mount: bool,
    #[props(default)]
    pub on_interact_outside: Option<EventHandler<ComboboxInteractOutsideEvent>>,
    #[props(default)]
    pub on_escape_key_down: Option<EventHandler<KeyboardEvent>>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<ComboboxContentStaticRenderProps, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn ComboboxContentStatic(props: ComboboxContentStaticProps) -> Element {
    let ctx = use_combobox_context();
    let is_open = ctx.runtime.is_open();
    let content_id = props
        .id
        .clone()
        .unwrap_or_else(|| ctx.runtime.relationships().content_id().to_owned());
    ctx.runtime.set_content_id(content_id.clone());

    {
        let mut fm = ctx.runtime.state.force_mount;
        if fm.try_peek().is_ok_and(|g| *g != props.force_mount) {
            let _ = fm.try_write().map(|mut w| *w = props.force_mount);
        }
        let mut oio = ctx.runtime.state.on_interact_outside;
        let _ = oio.try_write().map(|mut w| *w = props.on_interact_outside);
        let mut oek = ctx.runtime.state.on_escape_key_down;
        let _ = oek.try_write().map(|mut w| *w = props.on_escape_key_down);
    }

    let parent_branch = try_consume_context::<OverlayBranchContext>();
    {
        let cid = content_id.clone();
        let branch = parent_branch.clone();
        use_effect(use_reactive((&is_open,), move |(open,)| {
            if open {
                register_active_overlay_layer(&cid);
                if let Some(ref b) = branch {
                    b.register_branch(cid.clone());
                }
            } else {
                unregister_active_overlay_layer(&cid);
                if let Some(ref b) = branch {
                    b.unregister_branch(cid.clone());
                }
            }
        }));
    }

    {
        let cid = content_id.clone();
        let branch = parent_branch;
        use_drop(move || {
            unregister_active_overlay_layer(&cid);
            if let Some(ref b) = branch {
                b.unregister_branch(cid);
            }
        });
    }

    {
        let cid = content_id.clone();
        let rt = ctx.runtime.clone();
        let reg_sig = ctx.runtime.state.registry;
        use_effect(use_reactive((&is_open, &reg_sig), move |(open, _)| {
            if open {
                rt.sync_dom_order(&cid);
                rt.refresh_scroll_buttons();
            }
        }));
    }

    if !is_open && !props.force_mount {
        return rsx! {};
    }

    let attrs = ctx.runtime.content_attributes(
        content_id.clone(),
        PlacementSide::Bottom,
        PlacementAlign::Start,
    );

    let force_mount_style = if !is_open && props.force_mount {
        "display: none;"
    } else {
        ""
    };
    let combined_style = format!(
        "display: flex; flex-direction: column; box-sizing: border-box; outline: none; {force_mount_style} {}",
        props.style.as_deref().unwrap_or_default()
    );

    let key_rt = ctx.runtime.clone();
    let esc_cb = props.on_escape_key_down;
    let esc_cid = content_id;
    let on_key_down = EventHandler::new(move |evt: KeyboardEvent| {
        if evt.key() == Key::Named(NamedKey::Escape) {
            if !is_topmost_overlay_layer(&esc_cid) {
                return;
            }
            evt.prevent_default();
            if let Some(ref cb) = esc_cb {
                cb.call(evt.clone());
                if !evt.default_action_enabled() {
                    return;
                }
            }
            key_rt.close_menu();
        }
    });

    if props.as_child {
        if let Some(ref render_child) = props.render_child {
            return render_child.call(ComboboxContentStaticRenderProps {
                attrs,
                class: props.class,
                style: combined_style,
                on_key_down,
                attributes: props.attributes,
                children: props.children,
            });
        }
    }

    rsx! {
        div {
            id: "{attrs.id()}",
            role: "{attrs.role()}",
            tabindex: "{attrs.tabindex()}",
            "aria-multiselectable": attrs.aria_multiselectable(),
            "data-combobox-content": "",
            "data-state": "{attrs.data_state_str()}",
            class: props.class.as_deref().unwrap_or_default(),
            style: "{combined_style}",
            onkeydown: move |evt| on_key_down.call(evt),
            ..props.attributes,
            {props.children}
        }
    }
}
