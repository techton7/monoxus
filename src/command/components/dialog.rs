use dioxus::prelude::*;

use crate::dialog::{
    DialogContent, DialogDescription, DialogOpenFocusPolicy, DialogOverlay, DialogPortal,
    DialogRoot, DialogTitle,
};
use crate::foundation::browser::{FocusOptions, focus_element_sync_or_defer};
use crate::foundation::overlay::PortalHost;
use crate::visually_hidden::VisuallyHidden;

use super::super::{
    runtime::{CommandContext, CommandFilterCallback},
    types::CommandStateSnapshot,
};
use super::root::CommandRoot;

const DEFAULT_POSITIONER_STYLE: &str = "position: fixed; inset: 0; z-index: 60; display: flex; align-items: center; justify-content: center; pointer-events: none;";

#[derive(Clone, PartialEq, Props)]
pub struct CommandDialogProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub open: Option<Signal<bool>>,
    #[props(default)]
    pub default_open: Option<bool>,
    #[props(default)]
    pub on_open_change: Option<EventHandler<bool>>,
    #[props(default = "Command Palette".to_string())]
    pub title: String,
    #[props(default = "Search for a command to run...".to_string())]
    pub description: String,
    #[props(default)]
    pub portal_host: Option<PortalHost>,
    #[props(default)]
    pub overlay_class: Option<String>,
    #[props(default)]
    pub overlay_style: Option<String>,
    #[props(default)]
    pub positioner_class: Option<String>,
    #[props(default)]
    pub positioner_style: Option<String>,
    #[props(default)]
    pub content_id: Option<String>,
    #[props(default)]
    pub content_class: Option<String>,
    #[props(default)]
    pub content_style: Option<String>,
    #[props(default = true)]
    pub auto_focus_input: bool,
    #[props(default)]
    pub value: Option<Signal<String>>,
    #[props(default)]
    pub default_value: Option<String>,
    #[props(default)]
    pub on_value_change: Option<EventHandler<String>>,
    #[props(default)]
    pub on_state_change: Option<EventHandler<CommandStateSnapshot>>,
    #[props(default)]
    pub label: Option<String>,
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
    #[props(default)]
    pub children: Element,
}

#[component]
fn CommandDialogAutoFocus(enabled: bool) -> Element {
    let ctx = use_context::<CommandContext>();
    let mut did_focus = use_signal(|| false);
    use_effect(move || {
        if enabled && !*did_focus.peek() {
            let input_id = ctx.input_id.read().clone();
            if !input_id.is_empty() {
                did_focus.set(true);
                focus_element_sync_or_defer(&input_id, FocusOptions::new(false));
            }
        }
    });
    rsx! {}
}

/// Headless modal command palette dialog (`4B D-3`) composing `monoxus::dialog`
/// with a full-viewport flex-centering positioner, library-owned `CommandInput` open focus,
/// and screen-reader-only `DialogTitle`/`DialogDescription` around `CommandRoot`.
#[component]
pub fn CommandDialog(props: CommandDialogProps) -> Element {
    let command_label = props.label.unwrap_or_else(|| props.title.clone());
    let resolved_positioner_style = match props.positioner_style.as_deref().map(str::trim) {
        Some(extra) if !extra.is_empty() => format!("{DEFAULT_POSITIONER_STYLE} {extra}"),
        _ => DEFAULT_POSITIONER_STYLE.to_string(),
    };
    let resolved_content_style = match props.content_style.as_deref().map(str::trim) {
        Some(extra) if !extra.is_empty() => {
            if extra.contains("pointer-events:") {
                Some(extra.to_string())
            } else {
                Some(format!("pointer-events: auto; {extra}"))
            }
        }
        _ => Some("pointer-events: auto;".to_string()),
    };
    let open_focus = if props.auto_focus_input {
        None
    } else {
        Some(DialogOpenFocusPolicy::Suppress)
    };

    rsx! {
        DialogRoot {
            id: props.id.clone(),
            open: props.open,
            default_open: props.default_open,
            on_open_change: props.on_open_change,
            open_focus,
            portal_host: props.portal_host.clone(),
            DialogPortal {
                host: props.portal_host,
                DialogOverlay {
                    class: props.overlay_class,
                    style: props.overlay_style,
                }
                div {
                    "data-command-dialog-positioner": "",
                    class: props.positioner_class,
                    style: "{resolved_positioner_style}",
                    DialogContent {
                        id: props.content_id,
                        class: props.content_class,
                        style: resolved_content_style,
                        "data-command-dialog": "",
                        VisuallyHidden {
                            DialogTitle { "{props.title}" }
                            DialogDescription { "{props.description}" }
                        }
                        CommandRoot {
                            value: props.value,
                            default_value: props.default_value,
                            on_value_change: props.on_value_change,
                            on_state_change: props.on_state_change,
                            label: command_label,
                            should_filter: props.should_filter,
                            filter: props.filter,
                            loop_focus: props.loop_focus,
                            vim_bindings: props.vim_bindings,
                            disable_pointer_selection: props.disable_pointer_selection,
                            disable_initial_scroll: props.disable_initial_scroll,
                            columns: props.columns,
                            class: props.class,
                            style: props.style,
                            CommandDialogAutoFocus { enabled: props.auto_focus_input }
                            {props.children}
                        }
                    }
                }
            }
        }
    }
}
