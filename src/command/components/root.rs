use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

use dioxus::prelude::*;

use crate::foundation::{
    compose::merge_global_attributes,
    shared::{ScopeHandle, use_stable_id},
    state::sync_signal,
};
use crate::visually_hidden::VisuallyHiddenState;

use super::super::{
    attrs::{CommandInputLabelAttrs, CommandRootAttrs},
    relationships::CommandRelationships,
    runtime::{CommandContext, CommandFilterCallback},
    state::CommandRegistry,
    types::CommandStateSnapshot,
};

#[derive(Clone, PartialEq, Props)]
pub struct CommandRootRenderProps {
    pub id: String,
    pub role: &'static str,
    pub tabindex: &'static str,
    #[props(default)]
    pub aria_label: Option<String>,
    pub data_command_root: &'static str,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    pub on_keydown: EventHandler<KeyboardEvent>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[derive(Clone, PartialEq, Props)]
pub struct CommandRootProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub value: Option<Signal<String>>,
    #[props(default)]
    pub default_value: Option<String>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<String>>,
    #[props(default)]
    pub on_state_change: Option<EventHandler<CommandStateSnapshot>>,
    #[props(default)]
    pub label: String,
    #[props(default = true)]
    pub should_filter: bool,
    #[props(default)]
    pub filter: Option<CommandFilterCallback>,
    #[props(default = false)]
    pub loop_focus: bool,
    #[props(default = true)]
    pub vim_bindings: bool,
    #[props(default = false)]
    pub disable_pointer_selection: bool,
    #[props(default = false)]
    pub disable_initial_scroll: bool,
    #[props(default)]
    pub columns: Option<usize>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub style: Option<String>,
    #[props(default = false)]
    pub as_child: bool,
    #[props(default)]
    pub render_child: Option<Callback<CommandRootRenderProps, Element>>,
    #[props(default)]
    pub on_keydown: Option<EventHandler<KeyboardEvent>>,
    #[props(default)]
    pub on_mounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn CommandRoot(props: CommandRootProps) -> Element {
    let stable_id = use_stable_id(props.id.clone().map(Cow::Owned));
    let root_id = stable_id.into_owned();
    let scope = ScopeHandle::root("command").child(root_id.clone());
    let rels = CommandRelationships::new(scope);

    let relationships = use_signal({
        let rels = rels.clone();
        move || rels
    });
    let registry = use_signal(CommandRegistry::new);
    let search = use_signal(String::new);
    let fallback_value = use_signal(|| props.default_value.clone().unwrap_or_default());
    let value_sig = props.value.unwrap_or(fallback_value);

    let mut on_value_change = use_hook(|| CopyValue::new(props.on_value_change));
    on_value_change.set(props.on_value_change);
    let mut on_state_change = use_hook(|| CopyValue::new(props.on_state_change));
    on_state_change.set(props.on_state_change);

    let mut should_filter = use_signal(|| props.should_filter);
    let should_filter_changed = sync_signal(&mut should_filter, props.should_filter);

    let mut filter = use_hook(|| CopyValue::new(props.filter));
    let filter_changed = filter.try_read().is_ok_and(|cur| *cur != props.filter);
    if filter_changed {
        filter.set(props.filter);
    }

    let mut loop_focus = use_signal(|| props.loop_focus);
    sync_signal(&mut loop_focus, props.loop_focus);
    let mut vim_bindings = use_signal(|| props.vim_bindings);
    sync_signal(&mut vim_bindings, props.vim_bindings);
    let mut disable_pointer_selection = use_signal(|| props.disable_pointer_selection);
    sync_signal(&mut disable_pointer_selection, props.disable_pointer_selection);
    let mut disable_initial_scroll = use_signal(|| props.disable_initial_scroll);
    sync_signal(&mut disable_initial_scroll, props.disable_initial_scroll);
    let resolved_columns = props.columns.filter(|&c| c > 0);
    let mut columns = use_signal(|| resolved_columns);
    sync_signal(&mut columns, resolved_columns);
    let mut label_sig = use_signal(|| props.label.clone());
    sync_signal(&mut label_sig, props.label.clone());

    let input_id = use_signal(|| rels.input_id());
    let label_id = use_signal(|| rels.label_id());
    let list_id = use_signal(|| rels.list_id());
    let viewport_id = use_signal(|| rels.viewport_id());
    let list_height = use_signal(|| None::<f64>);
    let mut settled = use_signal(|| false);
    let is_composing = use_signal(|| false);
    let has_scrolled_initially = use_signal(|| false);
    let select_handlers = use_signal(BTreeMap::<String, EventHandler<()>>::new);
    let link_items = use_signal(BTreeSet::<String>::new);

    let mut ctx = use_context_provider(|| CommandContext {
        relationships,
        registry,
        search,
        value: value_sig,
        on_value_change,
        on_state_change,
        should_filter,
        filter,
        loop_focus,
        vim_bindings,
        disable_pointer_selection,
        disable_initial_scroll,
        columns,
        label: label_sig,
        input_id,
        label_id,
        list_id,
        viewport_id,
        list_height,
        settled,
        is_composing,
        has_scrolled_initially,
        select_handlers,
        link_items,
    });

    if should_filter_changed || filter_changed {
        ctx.recompute_filtered(false);
    }

    let mut ctx_for_init = ctx;
    use_effect(move || {
        if !*settled.peek() {
            settled.set(true);
        }
        if !*ctx_for_init.has_scrolled_initially.peek() {
            let current = ctx_for_init.value.peek().clone();
            if !current.is_empty() {
                ctx_for_init.has_scrolled_initially.set(true);
                if !*ctx_for_init.disable_initial_scroll.peek() {
                    ctx_for_init.scroll_selected_into_view(&current);
                }
            }
        }
    });

    let attrs = CommandRootAttrs::new(root_id, None);
    let label_attrs = CommandInputLabelAttrs::new(label_id.read().clone(), input_id.read().clone());
    let vh_attrs = VisuallyHiddenState::new().attributes();

    let mut consumer_keydown = use_hook(|| CopyValue::new(props.on_keydown));
    consumer_keydown.set(props.on_keydown);
    let mut ctx_for_key = ctx;
    let handle_keydown = EventHandler::new(move |evt: KeyboardEvent| {
        if let Some(handler) = consumer_keydown.try_read().ok().and_then(|s| *s) {
            handler.call(evt.clone());
        }
        ctx_for_key.handle_keydown(&evt);
    });

    let inner = rsx! {
        label {
            id: "{label_attrs.id}",
            r#for: "{label_attrs.r#for}",
            style: "{vh_attrs.style}",
            "data-command-input-label": label_attrs.data_command_input_label,
            "data-visually-hidden-root": vh_attrs.data_visually_hidden_root,
            "{props.label}"
        }
        {props.children}
    };

    if props.as_child
        && let Some(render_child) = props.render_child.as_ref()
    {
        return render_child.call(CommandRootRenderProps {
            id: attrs.id,
            role: attrs.role,
            tabindex: attrs.tabindex,
            aria_label: attrs.aria_label,
            data_command_root: attrs.data_command_root,
            class: props.class,
            style: props.style,
            on_keydown: handle_keydown,
            on_mounted: props.on_mounted,
            attributes: props.attributes,
            children: inner,
        });
    }

    let merged = merge_global_attributes(
        props.class.as_deref(),
        props.style.as_deref(),
        props.attributes,
    );

    rsx! {
        div {
            id: "{attrs.id}",
            role: "{attrs.role}",
            tabindex: "{attrs.tabindex}",
            aria_label: attrs.aria_label.as_deref(),
            "data-command-root": attrs.data_command_root,
            class: merged.class,
            style: merged.style,
            onkeydown: move |evt| handle_keydown.call(evt),
            onmounted: move |evt| {
                if let Some(handler) = props.on_mounted.as_ref() {
                    handler.call(evt);
                }
            },
            ..merged.rest,
            {inner}
        }
    }
}

pub use CommandRoot as Command;
