use dioxus::prelude::*;
use ui_sdk::kernel::{ChangeContext, ChangeEvent, ChangeReason, ChangeSource};

use crate::foundation::{
    browser::{FocusOptions, focus_element_sync_or_defer},
    compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
    state::{ControllableConfig, sync_signal, use_controllable_signal_config},
};

use super::super::{attrs::CommandInputAttrs, runtime::CommandContext};

#[derive(Clone, PartialEq, Props)]
pub struct CommandInputProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub value: Option<String>,
    #[props(default, into)]
    pub default_value: Option<String>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<String>>,
    #[props(default)]
    pub on_value_change_details: Option<EventHandler<ChangeEvent<String>>>,
    #[props(default, into)]
    pub placeholder: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub auto_focus: bool,
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub on_input: Option<EventHandler<FormEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[component]
pub fn CommandInput(props: CommandInputProps) -> Element {
    let mut ctx = use_context::<CommandContext>();
    let resolved_id = props
        .id
        .clone()
        .unwrap_or_else(|| ctx.relationships.peek().input_id());
    sync_signal(&mut ctx.input_id, resolved_id.clone());

    let has_explicit_default = props.default_value.is_some();
    let value_ctl = use_controllable_signal_config(
        ControllableConfig::new(props.value, props.default_value.unwrap_or_default())
            .with_explicit_default(has_explicit_default)
            .with_on_change(props.on_value_change)
            .with_on_change_details(props.on_value_change_details)
            .with_component("CommandInput")
            .with_prop("value"),
    );
    let val_sig = value_ctl.signal();

    // Sync controlled or seeded input value into command root search via effect (not mid-render).
    let current_val = val_sig();
    let mut initialized_search = use_signal(|| false);
    let mut ctx_for_sync = ctx;
    use_effect(move || {
        let current = val_sig();
        let was_init = *initialized_search.peek();
        if !was_init {
            initialized_search.set(true);
            if *ctx_for_sync.search.peek() != current {
                ctx_for_sync.search.set(current);
                ctx_for_sync.recompute_filtered(false);
            }
        } else if *ctx_for_sync.search.peek() != current {
            ctx_for_sync.set_search(current);
        }
    });

    let auto_focus_id = resolved_id.clone();
    let auto_focus = props.auto_focus;
    use_effect(move || {
        if auto_focus {
            focus_element_sync_or_defer(&auto_focus_id, FocusOptions::new(false));
        }
    });

    let active_descendant = ctx.selected_item_dom_id();
    let attrs = CommandInputAttrs::new(
        resolved_id,
        Some(ctx.viewport_id.read().clone()),
        Some(ctx.label_id.read().clone()),
        active_descendant,
    );

    let mut ctx_for_input = ctx;
    let base_input = EventHandler::new(move |evt: FormEvent| {
        let next = evt.value();
        let outcome = value_ctl.set_with_context(
            next.clone(),
            ChangeContext::new(ChangeReason::Input, ChangeSource::Keyboard),
        );
        if outcome.is_committed() {
            ctx_for_input.set_search(next);
        }
    });

    let mut ctx_for_comp_start = ctx;
    let base_compositionstart = EventHandler::new(move |_evt: CompositionEvent| {
        ctx_for_comp_start.set_composing(true);
    });
    let mut ctx_for_comp_end = ctx;
    let base_compositionend = EventHandler::new(move |_evt: CompositionEvent| {
        ctx_for_comp_end.set_composing(false);
    });

    let mut base = Vec::with_capacity(18);
    base.push(Attribute::new("id", attrs.id, None, false));
    base.push(Attribute::new("type", attrs.r#type, None, false));
    base.push(Attribute::new("role", attrs.role, None, false));
    base.push(Attribute::new(
        "aria-autocomplete",
        attrs.aria_autocomplete,
        None,
        false,
    ));
    base.push(Attribute::new(
        "aria-expanded",
        attrs.aria_expanded,
        None,
        false,
    ));
    push_attr_opt(&mut base, "aria-controls", attrs.aria_controls);
    push_attr_opt(&mut base, "aria-labelledby", attrs.aria_labelledby);
    push_attr_opt(
        &mut base,
        "aria-activedescendant",
        attrs.aria_activedescendant,
    );
    base.push(Attribute::new(
        "autocomplete",
        attrs.autocomplete,
        None,
        false,
    ));
    base.push(Attribute::new(
        "autocorrect",
        attrs.autocorrect,
        None,
        false,
    ));
    base.push(Attribute::new("spellcheck", attrs.spellcheck, None, false));
    base.push(Attribute::new(
        "data-command-input",
        attrs.data_command_input,
        None,
        false,
    ));
    base.push(Attribute::new("value", current_val, None, false));
    push_attr_opt(&mut base, "placeholder", props.placeholder);
    if props.disabled {
        base.push(Attribute::new("disabled", true, None, false));
    }
    push_event_handler(&mut base, "oninput", Some(base_input));
    push_event_handler(&mut base, "oncompositionstart", Some(base_compositionstart));
    push_event_handler(&mut base, "oncompositionend", Some(base_compositionend));

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler(&mut user, "oninput", props.on_input);
    push_event_handler(&mut user, "onmounted", props.on_mounted);
    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        input {
            ..merged,
        }
    }
}
