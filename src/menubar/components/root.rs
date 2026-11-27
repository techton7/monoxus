use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props},
    shared::{Direction, ScopeHandle, use_direction_context, use_stable_id},
    state::use_controllable_signal,
};
use crate::menu::{MenuPart, MenuVariant};
use crate::menubar::{MenubarRelationships, MenubarRootAttributes, MenubarRuntimeContext};

#[derive(Props, Clone, PartialEq)]
pub struct MenubarRootProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default, into)]
    pub value: Option<String>,
    #[props(default, into)]
    pub default_value: Option<String>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<String>>,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default = true)]
    pub loop_focus: bool,
    #[props(default)]
    pub submenu_open_delay_ms: Option<u64>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Horizontal `role="menubar"` root container coordinating roving focus, hover swaps, and cross-menu keyboard navigation (`MB-1`..`MB-7`).
#[component]
pub fn MenubarRoot(props: MenubarRootProps) -> Element {
    let effective_dir = use_direction_context(props.dir);
    let token = use_stable_id(None);
    let relationships =
        use_hook(|| MenubarRelationships::new(ScopeHandle::root("menubar").child(token)));
    let root_id = props.id.unwrap_or_else(|| relationships.root_id().to_owned());

    let value_state = use_controllable_signal(
        props.value.clone(),
        props.default_value.unwrap_or_default(),
        props.on_value_change,
    );

    let runtime = MenubarRuntimeContext::new(
        relationships,
        value_state,
        effective_dir,
        props.loop_focus,
        props.submenu_open_delay_ms,
    );
    runtime.set_root_id(root_id.clone());

    if let Some(controlled) = props.value.as_deref() {
        runtime.sync_controlled_value(Some(controlled));
    }

    {
        let mut st = runtime.state;
        let changed = st.try_peek().is_ok_and(|s| {
            s.direction() != effective_dir || s.loop_focus() != props.loop_focus
        });
        if changed {
            let _ = st.try_write().map(|mut w| {
                let next = w
                    .clone()
                    .with_direction(effective_dir)
                    .with_loop_focus(props.loop_focus);
                *w = next;
            });
        }
    }

    use_context_provider(|| runtime.clone());

    let attrs = MenubarRootAttributes::new(root_id, effective_dir);
    let part_attr = MenuPart::Root.static_data_attr_name(MenuVariant::Menubar);

    let base = vec![
        Attribute::new("id", attrs.id, None, false),
        Attribute::new("role", attrs.role, None, false),
        Attribute::new("aria-orientation", attrs.aria_orientation, None, false),
        Attribute::new("dir", attrs.dir, None, false),
        Attribute::new(part_attr, "", None, false),
    ];
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

pub use MenubarRoot as Menubar;
