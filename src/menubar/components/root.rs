use dioxus::prelude::*;

use crate::foundation::shared::{Direction, ScopeHandle, use_direction_context, use_stable_id};
use crate::menu::{MenuPart, MenuVariant, merge_attributes_with_part};
use crate::menubar::{MenubarRelationships, MenubarRootAttributes, MenubarRuntimeContext};

/// Render props passed to `render_child` when `MenubarRoot` uses `as_child: true`.
#[derive(Clone, PartialEq)]
pub struct MenubarRootRenderProps {
    pub attrs: MenubarRootAttributes,
    pub class: Option<String>,
    pub style: Option<String>,
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Props, Clone, PartialEq)]
pub struct MenubarRootProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub value: Option<String>,
    #[props(default)]
    pub default_value: Option<String>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<String>>,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default = true)]
    pub loop_focus: bool,
    #[props(default)]
    pub submenu_open_delay_ms: Option<u64>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<MenubarRootRenderProps, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// Horizontal `role="menubar"` root container coordinating roving focus, hover swaps, and cross-menu keyboard navigation (`MB-1`..`MB-7`).
#[component]
pub fn MenubarRoot(props: MenubarRootProps) -> Element {
    let effective_dir = use_direction_context(props.dir);
    let token = use_stable_id(None);
    let relationships = use_hook(|| MenubarRelationships::new(ScopeHandle::root("menubar").child(token)));
    let root_id = props.id.unwrap_or_else(|| relationships.root_id().to_owned());

    let is_controlled = props.value.is_some();
    let initial_active = props
        .value
        .clone()
        .or_else(|| props.default_value.clone())
        .filter(|v| !v.is_empty());

    let runtime = MenubarRuntimeContext::new(
        relationships,
        initial_active,
        is_controlled,
        props.on_value_change,
        effective_dir,
        props.loop_focus,
        props.submenu_open_delay_ms,
    );

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
    let merged_attrs = merge_attributes_with_part(
        props.attributes,
        MenuPart::Root.static_data_attr_name(MenuVariant::Menubar),
    );

    if props.as_child
        && let Some(render_child) = props.render_child.as_ref()
    {
        return render_child.call(MenubarRootRenderProps {
            attrs,
            class: props.class,
            style: props.style,
            attributes: merged_attrs,
            children: props.children,
        });
    }

    rsx! {
        div {
            id: "{attrs.id}",
            class: props.class.as_deref().unwrap_or_default(),
            style: props.style.as_deref().unwrap_or_default(),
            role: "{attrs.role}",
            "aria-orientation": "{attrs.aria_orientation}",
            dir: "{attrs.dir}",
            ..merged_attrs,
            {props.children}
        }
    }
}

pub use MenubarRoot as Menubar;
