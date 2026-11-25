use dioxus::prelude::*;

use crate::dialog::{
    DialogContent, DialogDescription, DialogOverlay, DialogPortal, DialogRoot, DialogTitle,
};
use crate::foundation::overlay::PortalHost;
use crate::visually_hidden::VisuallyHidden;

use super::super::{runtime::CommandFilterCallback, types::CommandStateSnapshot};
use super::root::CommandRoot;

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
    pub content_id: Option<String>,
    #[props(default)]
    pub content_class: Option<String>,
    #[props(default)]
    pub content_style: Option<String>,
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

/// Headless modal command palette dialog (`4B D-3`) composing `monoxus::dialog`
/// and screen-reader-only `DialogTitle`/`DialogDescription` around `CommandRoot`.
#[component]
pub fn CommandDialog(props: CommandDialogProps) -> Element {
    let command_label = props.label.unwrap_or_else(|| props.title.clone());

    rsx! {
        DialogRoot {
            id: props.id.clone(),
            open: props.open,
            default_open: props.default_open,
            on_open_change: props.on_open_change,
            portal_host: props.portal_host.clone(),
            DialogPortal {
                host: props.portal_host,
                DialogOverlay {
                    class: props.overlay_class,
                    style: props.overlay_style,
                }
                DialogContent {
                    id: props.content_id,
                    class: props.content_class,
                    style: props.content_style,
                    div {
                        "data-command-dialog": "",
                        style: "display: contents;",
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
                            {props.children}
                        }
                    }
                }
            }
        }
    }
}
