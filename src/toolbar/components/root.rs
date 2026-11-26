use dioxus::prelude::*;

use crate::foundation::{
    compose::merge_global_attributes,
    shared::{Direction, Orientation, ScopeHandle, use_direction_context, use_stable_id},
};
use crate::toolbar::{
    components::ToolbarRootContext,
    runtime::use_toolbar_runtime,
    state::Toolbar as ToolbarState,
};

#[derive(Clone, PartialEq, Props)]
pub struct ToolbarRootProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = Orientation::Horizontal)]
    pub orientation: Orientation,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default = true)]
    pub loop_focus: bool,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn ToolbarRoot(props: ToolbarRootProps) -> Element {
    let resolved_dir = use_direction_context(props.dir);
    let stable_id = use_stable_id(None);
    let scope_id = props.id.clone().unwrap_or_else(|| stable_id.into_owned());
    let scope = ScopeHandle::root("toolbar").child(scope_id);

    let toolbar = ToolbarState::new(scope)
        .with_orientation(props.orientation)
        .with_direction(resolved_dir)
        .with_loop_focus(props.loop_focus);
    let initial_dom_id = props
        .id
        .clone()
        .unwrap_or_else(|| toolbar.relationships().root_id().to_owned());

    let runtime = use_toolbar_runtime(toolbar, initial_dom_id.clone());
    runtime.sync_props(
        props.orientation,
        resolved_dir,
        props.loop_focus,
        &initial_dom_id,
    );

    use_context_provider(|| ToolbarRootContext {
        runtime: runtime.clone(),
    });

    let root_attrs = runtime.root_attributes();
    let resolved_id = props.id.as_deref().unwrap_or(root_attrs.id());
    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        div {
            id: "{resolved_id}",
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref().unwrap_or_default(),
            role: root_attrs.role(),
            "aria-orientation": root_attrs.aria_orientation(),
            dir: root_attrs.dir(),
            "data-orientation": root_attrs.data_orientation(),
            "data-toolbar-root": root_attrs.data_toolbar_root(),
            ..merged.rest,
            {props.children}
        }
    }
}

pub use ToolbarRoot as Toolbar;
