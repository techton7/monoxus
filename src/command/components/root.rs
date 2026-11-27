use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

use dioxus::prelude::*;

use crate::foundation::{
    compose::{collect_user_attrs, merge_props, push_attr_opt, push_event_handler},
    shared::{ScopeHandle, use_stable_id},
    state::{sync_signal, use_controllable_signal},
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
pub struct CommandRootProps {
    #[props(default, into)]
    pub id: Option<String>,
    #[props(default, into)]
    pub value: Option<String>,
    #[props(default, into)]
    pub default_value: Option<String>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<String>>,
    #[props(default)]
    pub on_state_change: Option<EventHandler<CommandStateSnapshot>>,
    #[props(default, into)]
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
    #[props(default, into)]
    pub class: Option<String>,
    #[props(default, into)]
    pub style: Option<String>,
    #[props(default)]
    pub child: Option<Callback<Vec<Attribute>, Element>>,
    #[props(default)]
    pub on_key_down: Option<EventHandler<KeyboardEvent>>,
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
    let value_ctl = use_controllable_signal(
        props.value,
        props.default_value.unwrap_or_default(),
        props.on_value_change,
    );
    let value_sig = value_ctl.signal();

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
        value_ctl,
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

    let mut ctx_for_key = ctx;
    let base_keydown = EventHandler::new(move |evt: KeyboardEvent| {
        ctx_for_key.handle_keydown(&evt);
    });

    let mut base = Vec::with_capacity(6);
    base.push(Attribute::new("id", attrs.id, None, false));
    base.push(Attribute::new("role", attrs.role, None, false));
    base.push(Attribute::new("tabindex", attrs.tabindex, None, false));
    push_attr_opt(&mut base, "aria-label", attrs.aria_label);
    base.push(Attribute::new(
        "data-command-root",
        attrs.data_command_root,
        None,
        false,
    ));
    push_event_handler(&mut base, "onkeydown", Some(base_keydown));

    let mut user = collect_user_attrs(props.class, props.style, props.attributes);
    push_event_handler(&mut user, "onkeydown", props.on_key_down);
    push_event_handler(&mut user, "onmounted", props.on_mounted);
    let merged = merge_props(base, user);

    if let Some(child) = props.child {
        return child.call(merged);
    }

    rsx! {
        div {
            ..merged,
            label {
                id: "{label_attrs.id}",
                r#for: "{label_attrs.r#for}",
                style: "{vh_attrs.style}",
                "data-command-input-label": label_attrs.data_command_input_label,
                "data-visually-hidden-root": vh_attrs.data_visually_hidden_root,
                "{props.label}"
            }
            {props.children}
        }
    }
}

pub use CommandRoot as Command;
