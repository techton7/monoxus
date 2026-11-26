use dioxus::prelude::*;

use crate::foundation::compose::merge_global_attributes;

use super::super::{
    attrs::PinInputCellAttributes,
    runtime::{PinInputCellContext, use_pin_input_context},
    types::PinInputCellData,
};

/// Render props passed to `render_child` when [`PinInputCell`] sets `as_child: true`.
#[derive(Clone)]
pub struct PinInputCellRenderChildProps {
    pub attrs: PinInputCellAttributes,
    pub cell: PinInputCellData,
    pub display_char: Option<char>,
    pub children: Option<Element>,
}

/// Properties for [`PinInputCell`].
#[derive(Clone, PartialEq, Props)]
pub struct PinInputCellProps {
    #[props(default)]
    pub id: Option<String>,
    pub index: usize,
    #[props(default)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<PinInputCellRenderChildProps, Element>>,
    #[props(default)]
    pub children: Option<Element>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// Visual slot inside [`crate::pin_input::PinInputRoot`] mirroring `char`, `is_active`, and
/// `has_fake_caret` projected from the overlaid `<input>`.
#[component]
pub fn PinInputCell(props: PinInputCellProps) -> Element {
    let ctx = use_pin_input_context();
    let runtime = ctx.runtime;

    let cell_data = runtime.cell_data(props.index);
    let cell_attrs = runtime.cell_attributes(props.index, props.id.clone());
    let display_char = runtime.display_char_for_cell(&cell_data);
    let disabled = cell_attrs.is_disabled();

    runtime.register_cell(props.index, cell_attrs.id.clone());
    let drop_runtime = runtime.clone();
    let drop_index = props.index;
    let drop_id = cell_attrs.id.clone();
    use_drop(move || {
        drop_runtime.unregister_cell(drop_index, &drop_id);
    });

    use_context_provider(|| PinInputCellContext {
        cell: cell_data,
        display_char,
        disabled,
    });

    if props.as_child && props.render_child.is_none() {
        #[cfg(debug_assertions)]
        eprintln!(
            "monoxus::PinInputCell: as_child requires render_child; rendering the default div"
        );
    }
    if let Some(render_child) = props.render_child.as_ref().filter(|_| props.as_child) {
        return render_child.call(PinInputCellRenderChildProps {
            attrs: cell_attrs,
            cell: cell_data,
            display_char,
            children: props.children,
        });
    }

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        div {
            id: "{cell_attrs.id}",
            class: merged.class.as_deref().unwrap_or_default(),
            style: merged.style.as_deref().unwrap_or_default(),
            "data-active": cell_attrs.data_active,
            "data-inactive": cell_attrs.data_inactive,
            "data-disabled": cell_attrs.data_disabled,
            "data-monoxus-pin-input-cell": "{cell_attrs.data_monoxus_pin_input_cell}",
            onmounted: move |evt| {
                if let Some(ref handler) = props.on_mounted {
                    handler.call(evt);
                }
            },
            ..merged.rest,
            if let Some(children) = props.children {
                {children}
            } else if let Some(ch) = display_char {
                "{ch}"
            }
        }
    }
}
