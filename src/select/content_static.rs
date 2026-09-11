use dioxus::prelude::*;

use crate::foundation::browser::restore_focus_element_by_id;

use super::types::SelectContext;

#[component]
pub fn SelectContentStatic(
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    children: Element,
) -> Element {
    let ctx = use_context::<SelectContext>();
    let is_open = ctx.runtime.is_open();
    let should_render = ctx.runtime.should_render_content();
    let rels = ctx.runtime.relationships();
    let content_id = rels.content_id().to_owned();
    let runtime = ctx.runtime.clone();
    use_effect(use_reactive((&is_open,), {
        let cid = content_id.clone();
        let rt = runtime.clone();
        move |(open,)| {
            if open {
                restore_focus_element_by_id(&cid);
                rt.sync_dom_order();
            }
        }
    }));

    if !should_render {
        return rsx! {};
    }

    let hl = ctx.runtime.highlighted_value();
    let activedescendant = hl.map(|v| rels.item_id(&v));
    // Live content attributes reading live runtime.is_open()
    let attrs = ctx.runtime.content_attributes(activedescendant);

    rsx! {
        div {
            id: "{attrs.id()}",
            role: "{attrs.role()}",
            tabindex: "{attrs.tabindex()}",
            aria_activedescendant: attrs.aria_activedescendant(),
            class: class.as_deref().unwrap_or_default(),
            style: style.as_deref().unwrap_or_default(),
            "data-state": if is_open { "open" } else { "closed" },
            "data-side": "{attrs.data_side_str()}",
            "data-align": "{attrs.data_align_str()}",
            onkeydown: {
                let runtime = ctx.runtime.clone();
                move |evt| {
                    #[cfg(target_arch = "wasm32")]
                    let now_ms = js_sys::Date::now();
                    #[cfg(not(target_arch = "wasm32"))]
                    let now_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs_f64() * 1000.0)
                        .unwrap_or(0.0);

                    runtime.handle_content_keydown(&evt, now_ms);
                }
            },
            {children}
        }
    }
}
