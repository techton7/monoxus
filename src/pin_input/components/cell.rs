use dioxus::prelude::*;

use crate::foundation::{compose::merge_global_attributes, state::sync_signal};

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

    let mut cell_sig = use_signal(|| cell_data);
    sync_signal(&mut cell_sig, cell_data);
    let mut display_char_sig = use_signal(|| display_char);
    sync_signal(&mut display_char_sig, display_char);
    let mut disabled_sig = use_signal(|| disabled);
    sync_signal(&mut disabled_sig, disabled);

    let mut registered_cell = use_hook(|| CopyValue::new((props.index, cell_attrs.id.clone())));
    let next_registration = (props.index, cell_attrs.id.clone());
    let prev_registration = registered_cell.try_read().ok().and_then(|prev| {
        if *prev != next_registration {
            Some((*prev).clone())
        } else {
            None
        }
    });
    if let Some((prev_index, prev_id)) = prev_registration {
        runtime.unregister_cell(prev_index, &prev_id);
        registered_cell.set(next_registration);
    }
    runtime.register_cell(props.index, cell_attrs.id.clone());

    let drop_runtime = runtime.clone();
    use_drop(move || {
        if let Ok(live) = registered_cell.try_read() {
            drop_runtime.unregister_cell(live.0, &live.1);
        }
    });

    use_context_provider(|| PinInputCellContext {
        cell: cell_sig,
        display_char: display_char_sig,
        disabled: disabled_sig,
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
            "data-invalid": cell_attrs.data_invalid,
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
