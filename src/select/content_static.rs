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
    let readiness_sig = ctx.runtime.state.content_readiness;

    let readiness_runtime = ctx.runtime.clone();
    use_effect(use_reactive((&is_open, &should_render), {
        move |(open, renderable)| {
            if open && renderable {
                let mut readiness = readiness_runtime.state.content_readiness;
                if *readiness.peek() != crate::foundation::overlay::FloatingReadiness::Ready {
                    readiness.set(crate::foundation::overlay::FloatingReadiness::Ready);
                }
            }
        }
    }));

    let focus_cid = content_id.clone();
    let focus_runtime = ctx.runtime.clone();
    use_effect(use_reactive((&is_open, &readiness_sig), {
        let cid = focus_cid.clone();
        let rt = focus_runtime.clone();
        move |(open, readiness)| {
            if open && readiness() == crate::foundation::overlay::FloatingReadiness::Ready {
                restore_focus_element_by_id(&cid);
                if let Some(ref v) = rt.highlighted_value() {
                    let item_id = rt.relationships().item_id(v);
                    let vp_id = rt.relationships().viewport_id();
                    super::viewport::scroll_item_within_container(&vp_id, &cid, &item_id);
                }
            }
        }
    }));

    let order_cid = content_id.clone();
    let order_runtime = ctx.runtime.clone();
    let reg_sig = ctx.runtime.state.registry;
    use_effect(use_reactive((&is_open, &reg_sig), move |(open, _)| {
        if open {
            order_runtime.sync_dom_order(&order_cid);
        }
    }));

    if !should_render {
        return rsx! {};
    }

    let hl = ctx.runtime.highlighted_value();
    let activedescendant = hl.map(|v| rels.item_id(&v));
    // Live content attributes reading live runtime.is_open()
    let attrs = ctx.runtime.content_attributes(activedescendant);
    let combined_style = format!(
        "display: flex; flex-direction: column; box-sizing: border-box; outline: none; {}",
        style.as_deref().unwrap_or_default()
    );

    rsx! {
        div {
            id: "{attrs.id()}",
            role: "{attrs.role()}",
            tabindex: "{attrs.tabindex()}",
            aria_activedescendant: attrs.aria_activedescendant(),
            class: class.as_deref().unwrap_or_default(),
            style: "{combined_style}",
            "data-state": if is_open { "open" } else { "closed" },
            "data-side": "{attrs.data_side_str()}",
            "data-align": "{attrs.data_align_str()}",
            onkeydown: {
                let runtime = ctx.runtime.clone();
                move |evt| {
                    let now_ms = oxidase::time::monotonic_now_ms();

                    runtime.handle_content_keydown(&evt, now_ms);
                }
            },
            {children}
        }
    }
}
