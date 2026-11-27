use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
    state::sync_signal,
};

use super::super::runtime::{PinInputCellContext, use_pin_input_context};

/// Properties for [`PinInputCell`].
#[derive(Clone, PartialEq, Props)]
pub struct PinInputCellProps {
    #[props(default, into)]
    pub id: Option<String>,
    pub index: usize,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub children: Option<Element>,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
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

    let mut base = Vec::with_capacity(6);
    base.push(Attribute::new("id", cell_attrs.id, None, false));
    push_attr_opt(&mut base, "data-active", cell_attrs.data_active);
    push_attr_opt(&mut base, "data-inactive", cell_attrs.data_inactive);
    push_attr_opt(&mut base, "data-disabled", cell_attrs.data_disabled);
    push_attr_opt(&mut base, "data-invalid", cell_attrs.data_invalid);
    base.push(Attribute::new(
        "data-monoxus-pin-input-cell",
        cell_attrs.data_monoxus_pin_input_cell,
        None,
        false,
    ));

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler(&mut user, "onmounted", props.on_mounted);

    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            if let Some(children) = props.children {
                {children}
            } else if let Some(ch) = display_char {
                "{ch}"
            }
        }
    }
}
