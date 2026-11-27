use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props},
    shared::{Direction, Orientation, ScopeHandle, use_direction_context, use_stable_id},
};
use crate::toolbar::{
    components::ToolbarRootContext,
    runtime::use_toolbar_runtime,
    state::Toolbar as ToolbarState,
};

#[derive(Clone, PartialEq, Props)]
pub struct ToolbarRootProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default = Orientation::Horizontal)]
    pub orientation: Orientation,
    #[props(default)]
    pub dir: Option<Direction>,
    #[props(default = true)]
    pub loop_focus: bool,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
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
    let resolved_id = props
        .id
        .unwrap_or_else(|| root_attrs.id().to_string());

    let mut base = Vec::with_capacity(6);
    base.push(Attribute::new("id", resolved_id, None, false));
    base.push(Attribute::new("role", root_attrs.role(), None, false));
    base.push(Attribute::new(
        "aria-orientation",
        root_attrs.aria_orientation(),
        None,
        false,
    ));
    base.push(Attribute::new("dir", root_attrs.dir(), None, false));
    base.push(Attribute::new(
        "data-orientation",
        root_attrs.data_orientation(),
        None,
        false,
    ));
    base.push(Attribute::new(
        "data-toolbar-root",
        root_attrs.data_toolbar_root(),
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

pub use ToolbarRoot as Toolbar;
