use dioxus::prelude::*;

use crate::foundation::browser::{FocusOptions, focus_element_sync_or_defer};

use super::super::{attrs::CommandInputAttrs, runtime::CommandContext};

#[derive(Clone, PartialEq, Props)]
pub struct CommandInputRenderProps {
    pub id: String,
    pub r#type: &'static str,
    pub role: &'static str,
    pub aria_autocomplete: &'static str,
    pub aria_expanded: &'static str,
    #[props(default)]
    pub aria_controls: Option<String>,
    #[props(default)]
    pub aria_labelledby: Option<String>,
    #[props(default)]
    pub aria_activedescendant: Option<String>,
    pub autocomplete: &'static str,
    pub autocorrect: &'static str,
    pub spellcheck: &'static str,
    pub data_command_input: &'static str,
    pub value: String,
    #[props(default)]
    pub placeholder: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    pub on_input: EventHandler<FormEvent>,
    pub on_compositionstart: EventHandler<CompositionEvent>,
    pub on_compositionend: EventHandler<CompositionEvent>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[derive(Clone, PartialEq, Props)]
pub struct CommandInputProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub value: Option<Signal<String>>,
    #[props(default)]
    pub default_value: Option<String>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<String>>,
    #[props(default)]
    pub placeholder: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub autofocus: bool,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<CommandInputRenderProps, Element>>,
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
    if *ctx.input_id.peek() != resolved_id {
        ctx.input_id.set(resolved_id.clone());
    }

    let fallback_val = use_signal(|| props.default_value.clone().unwrap_or_default());
    let mut val_sig = props.value.unwrap_or(fallback_val);

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

    let autofocus_id = resolved_id.clone();
    let autofocus = props.autofocus;
    use_effect(move || {
        if autofocus {
            focus_element_sync_or_defer(&autofocus_id, FocusOptions::new(false));
        }
    });

    let active_descendant = ctx.selected_item_dom_id();
    let attrs = CommandInputAttrs::new(
        resolved_id,
        Some(ctx.viewport_id.read().clone()),
        Some(ctx.label_id.read().clone()),
        active_descendant,
    );

    let on_value_change = props.on_value_change;
    let on_input_prop = props.on_input;
    let is_controlled = props.value.is_some();
    let mut ctx_for_input = ctx;
    let handle_input = EventHandler::new(move |evt: FormEvent| {
        let next = evt.value();
        if !is_controlled {
            val_sig.set(next.clone());
        }
        ctx_for_input.set_search(next.clone());
        if let Some(cb) = on_value_change {
            cb.call(next);
        }
        if let Some(handler) = on_input_prop {
            handler.call(evt);
        }
    });

    let mut ctx_for_comp_start = ctx;
    let handle_compositionstart = EventHandler::new(move |_evt: CompositionEvent| {
        ctx_for_comp_start.set_composing(true);
    });
    let mut ctx_for_comp_end = ctx;
    let handle_compositionend = EventHandler::new(move |_evt: CompositionEvent| {
        ctx_for_comp_end.set_composing(false);
    });

    if props.as_child {
        if let Some(render_child) = props.render_child.as_ref() {
            return render_child.call(CommandInputRenderProps {
                id: attrs.id,
                r#type: attrs.r#type,
                role: attrs.role,
                aria_autocomplete: attrs.aria_autocomplete,
                aria_expanded: attrs.aria_expanded,
                aria_controls: attrs.aria_controls,
                aria_labelledby: attrs.aria_labelledby,
                aria_activedescendant: attrs.aria_activedescendant,
                autocomplete: attrs.autocomplete,
                autocorrect: attrs.autocorrect,
                spellcheck: attrs.spellcheck,
                data_command_input: attrs.data_command_input,
                value: current_val,
                placeholder: props.placeholder,
                disabled: props.disabled,
                class: props.class,
                style: props.style,
                on_input: handle_input,
                on_compositionstart: handle_compositionstart,
                on_compositionend: handle_compositionend,
                on_mounted: props.on_mounted,
                attributes: props.attributes,
            });
        }
    }

    rsx! {
        input {
            id: "{attrs.id}",
            r#type: "{attrs.r#type}",
            role: "{attrs.role}",
            aria_autocomplete: "{attrs.aria_autocomplete}",
            aria_expanded: "{attrs.aria_expanded}",
            aria_controls: attrs.aria_controls.as_deref(),
            aria_labelledby: attrs.aria_labelledby.as_deref(),
            aria_activedescendant: attrs.aria_activedescendant.as_deref(),
            autocomplete: "{attrs.autocomplete}",
            "autocorrect": "{attrs.autocorrect}",
            spellcheck: "{attrs.spellcheck}",
            "data-command-input": attrs.data_command_input,
            value: "{current_val}",
            placeholder: props.placeholder.as_deref(),
            disabled: props.disabled,
            class: props.class,
            style: props.style,
            oninput: move |evt| handle_input.call(evt),
            oncompositionstart: move |evt| handle_compositionstart.call(evt),
            oncompositionend: move |evt| handle_compositionend.call(evt),
            onmounted: move |evt| {
                if let Some(handler) = props.on_mounted.as_ref() {
                    handler.call(evt);
                }
            },
            ..props.attributes,
        }
    }
}
