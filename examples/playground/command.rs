use dioxus::prelude::*;
use monoxus::command::{
    CommandDialog, CommandEmpty, CommandGroup, CommandGroupHeading, CommandGroupItems,
    CommandInput, CommandItem, CommandLinkItem, CommandList, CommandLoading, CommandRoot,
    CommandSeparator, CommandViewport,
};

const CARD_STYLE: &str = "border-radius: 1rem; border: 1px solid #e2e8f0; background-color: #ffffff; padding: 1.5rem; box-shadow: 0 1px 3px rgba(15, 23, 42, 0.06); display: flex; flex-direction: column; gap: 1rem;";
const STAGE_STYLE: &str = "display: flex; flex-direction: column; gap: 0.75rem; padding: 1.25rem; border-radius: 0.75rem; border: 1px dashed #cbd5e1; background-color: #f8fafc;";
const CTRL_BTN_STYLE: &str = "display: inline-flex; align-items: center; gap: 0.375rem; padding: 0.45rem 0.75rem; border-radius: 0.5rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #334155; font-size: 0.75rem; font-weight: 600; cursor: pointer;";
const PRIMARY_BTN_STYLE: &str = "display: inline-flex; align-items: center; gap: 0.4rem; padding: 0.5rem 0.9rem; border-radius: 0.5rem; border: 1px solid #7c3aed; background-color: #7c3aed; color: #ffffff; font-size: 0.875rem; font-weight: 600; cursor: pointer;";
const CMD_ROOT_STYLE: &str = "width: 100%; max-width: 460px; border-radius: 0.75rem; border: 1px solid #cbd5e1; background-color: #ffffff; color: #0f172a; box-shadow: 0 8px 24px rgba(15, 23, 42, 0.08); overflow: hidden;";
const CMD_INPUT_STYLE: &str = "width: 100%; padding: 0.65rem 0.875rem; border: none; border-bottom: 1px solid #e2e8f0; font-size: 0.875rem; color: #0f172a; background-color: #ffffff; outline: none; box-sizing: border-box;";
const CMD_GROUP_HEADING_STYLE: &str = "padding: 0.35rem 0.75rem 0.2rem; font-size: 0.6875rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.05em; color: #64748b;";
const CMD_SEP_STYLE: &str = "height: 1px; margin: 0.25rem 0.5rem; background-color: #e2e8f0;";
const CMD_EMPTY_STYLE: &str = "padding: 1.25rem 0.75rem; text-align: center; font-size: 0.8125rem; color: #64748b;";
const CMD_LOADING_STYLE: &str = "padding: 0.5rem 0.75rem; font-size: 0.75rem; color: #6d28d9; background-color: #f5f3ff; border-bottom: 1px solid #ede9fe;";
const DIALOG_OVERLAY_STYLE: &str = "position: fixed; inset: 0; background-color: rgba(15, 23, 42, 0.45); z-index: 50;";
const DIALOG_CONTENT_STYLE: &str = "pointer-events: auto; width: 92vw; max-width: 460px; border-radius: 0.75rem; background-color: #ffffff; box-shadow: 0 20px 50px rgba(15, 23, 42, 0.28); overflow: hidden;";

const COMMAND_PLAYGROUND_CSS: &str = r#"
[data-command-item] {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 0.45rem 0.75rem;
    margin: 0.125rem 0.375rem;
    border-radius: 0.375rem;
    font-size: 0.8125rem;
    font-weight: 500;
    color: #1e293b;
    text-decoration: none;
    cursor: pointer;
    user-select: none;
}
[data-command-item][data-selected] {
    background-color: #ede9fe;
    color: #5b21b6;
}
[data-command-item][data-disabled] {
    color: #94a3b8;
    opacity: 0.55;
    cursor: not-allowed;
}
[data-command-input]:focus-visible {
    box-shadow: inset 0 -2px 0 #7c3aed;
}
"#;

fn badge_style(active: bool) -> &'static str {
    if active {
        "display: inline-flex; align-items: center; border-radius: 9999px; padding: 0.15rem 0.6rem; font-size: 0.75rem; font-family: monospace; font-weight: 700; background-color: #ede9fe; color: #6d28d9; border: 1px solid #c4b5fd;"
    } else {
        "display: inline-flex; align-items: center; border-radius: 9999px; padding: 0.15rem 0.6rem; font-size: 0.75rem; font-family: monospace; font-weight: 600; background-color: #f1f5f9; color: #475569; border: 1px solid #e2e8f0;"
    }
}

#[component]
pub fn CommandShowcase() -> Element {
    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-8",
            style: "max-width: 56rem; margin: 0 auto; display: flex; flex-direction: column; gap: 2rem;",
            style { "{COMMAND_PLAYGROUND_CSS}" }

            div {
                style: "display: flex; flex-direction: column; gap: 0.375rem;",
                h2 {
                    style: "margin: 0; font-size: 1.5rem; font-weight: 700; color: #0f172a; letter-spacing: -0.025em;",
                    "Command"
                }
                p {
                    style: "margin: 0; font-size: 0.875rem; color: #64748b;",
                    "Headless command palette with fuzzy subsequence scoring, virtual focus (aria-activedescendant), vim bindings, group jumps, CommandDialog modal composition, and 2D grid navigation."
                }
            }

            InlineCommandScenario {}
            CommandDialogScenario {}
            CommandGridScenario {}
        }
    }
}

#[component]
fn InlineCommandScenario() -> Element {
    let mut is_loading = use_signal(|| false);
    let mut active_value = use_signal(String::new);
    let mut last_selected = use_signal(|| String::from("none"));
    let mut select_count = use_signal(|| 0usize);

    let mut record_select = move |name: &'static str| {
        last_selected.set(name.to_string());
        select_count += 1;
    };

    rsx! {
        section {
            id: "sw4b-cmd-inline-card",
            style: "{CARD_STYLE}",
            div {
                style: "display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 0.5rem;",
                div {
                    h3 {
                        style: "margin: 0; font-size: 1.0625rem; font-weight: 700; color: #0f172a;",
                        "1. Inline Command Palette (SW4B-1, SW4B-2, SW4B-3)"
                    }
                    p {
                        style: "margin: 0.2rem 0 0; font-size: 0.8125rem; color: #64748b;",
                        "Fuzzy scoring, keywords, disabled item skip, vim bindings (Ctrl+n/p/j/k), loop focus, and scrollable CommandViewport."
                    }
                }
                div {
                    style: "display: flex; align-items: center; gap: 0.5rem;",
                    button {
                        id: "sw4b-cmd-inline-loading-toggle",
                        r#type: "button",
                        style: "{CTRL_BTN_STYLE}",
                        onclick: move |_| is_loading.toggle(),
                        if is_loading() { "Hide Loading" } else { "Show Loading" }
                    }
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; gap: 0.5rem;",
                span {
                    id: "sw4b-cmd-inline-active",
                    style: "{badge_style(!active_value().is_empty())}",
                    "highlighted: {active_value}"
                }
                span {
                    id: "sw4b-cmd-inline-log",
                    style: "{badge_style(select_count() > 0)}",
                    "selected: {last_selected} ({select_count})"
                }
            }

            div {
                style: "{STAGE_STYLE}",
                CommandRoot {
                    id: Some("sw4b-cmd-inline".to_string()),
                    value: Some(active_value),
                    on_value_change: move |v: String| active_value.set(v),
                    label: "Inline Command Menu".to_string(),
                    vim_bindings: true,
                    loop_focus: true,
                    style: Some(CMD_ROOT_STYLE.to_string()),

                    CommandInput {
                        id: Some("sw4b-cmd-inline-input".to_string()),
                        placeholder: Some("Type a command or search (e.g. calc, docs)...".to_string()),
                        style: Some(CMD_INPUT_STYLE.to_string()),
                    }

                    if is_loading() {
                        CommandLoading {
                            id: Some("sw4b-cmd-inline-loading".to_string()),
                            progress: 65.0,
                            style: Some(CMD_LOADING_STYLE.to_string()),
                            "Indexing workspace commands (65%)..."
                        }
                    }

                    CommandList {
                        id: Some("sw4b-cmd-inline-list".to_string()),
                        CommandViewport {
                            id: Some("sw4b-cmd-inline-viewport".to_string()),
                            style: Some("max-height: 220px; overflow-y: auto; padding: 0.25rem 0;".to_string()),

                            CommandEmpty {
                                id: Some("sw4b-cmd-inline-empty".to_string()),
                                style: Some(CMD_EMPTY_STYLE.to_string()),
                                "No matching commands found."
                            }

                            CommandGroup {
                                id: Some("sw4b-cmd-group-suggestions".to_string()),
                                value: Some("suggestions".to_string()),
                                CommandGroupHeading {
                                    id: Some("sw4b-cmd-heading-suggestions".to_string()),
                                    style: Some(CMD_GROUP_HEADING_STYLE.to_string()),
                                    "Suggestions"
                                }
                                CommandGroupItems {
                                    CommandItem {
                                        id: Some("sw4b-cmd-item-calendar".to_string()),
                                        value: "calendar".to_string(),
                                        keywords: vec!["date".to_string(), "schedule".to_string()],
                                        on_select: move |_| record_select("calendar"),
                                        span { "Calendar" }
                                        span { style: "font-size: 0.7rem; color: #94a3b8;", "date, schedule" }
                                    }
                                    CommandItem {
                                        id: Some("sw4b-cmd-item-calculator".to_string()),
                                        value: "calculator".to_string(),
                                        keywords: vec!["math".to_string(), "numbers".to_string()],
                                        on_select: move |_| record_select("calculator"),
                                        span { "Calculator" }
                                        span { style: "font-size: 0.7rem; color: #94a3b8;", "math" }
                                    }
                                    CommandItem {
                                        id: Some("sw4b-cmd-item-search-emoji".to_string()),
                                        value: "search-emoji".to_string(),
                                        keywords: vec!["icon".to_string(), "smiley".to_string()],
                                        on_select: move |_| record_select("search-emoji"),
                                        span { "Search Emoji" }
                                        span { style: "font-size: 0.7rem; color: #94a3b8;", "icon" }
                                    }
                                    CommandLinkItem {
                                        id: Some("sw4b-cmd-item-docs".to_string()),
                                        value: "documentation".to_string(),
                                        href: Some("#command-docs".to_string()),
                                        keywords: vec!["help".to_string(), "guide".to_string()],
                                        on_select: move |_| record_select("documentation"),
                                        span { "Documentation" }
                                        span { style: "font-size: 0.7rem; color: #94a3b8;", "link" }
                                    }
                                }
                            }

                            CommandSeparator {
                                id: Some("sw4b-cmd-inline-sep".to_string()),
                                style: Some(CMD_SEP_STYLE.to_string()),
                            }

                            CommandGroup {
                                id: Some("sw4b-cmd-group-settings".to_string()),
                                value: Some("settings".to_string()),
                                CommandGroupHeading {
                                    id: Some("sw4b-cmd-heading-settings".to_string()),
                                    style: Some(CMD_GROUP_HEADING_STYLE.to_string()),
                                    "Settings"
                                }
                                CommandGroupItems {
                                    CommandItem {
                                        id: Some("sw4b-cmd-item-profile".to_string()),
                                        value: "profile".to_string(),
                                        keywords: vec!["account".to_string(), "user".to_string()],
                                        on_select: move |_| record_select("profile"),
                                        span { "Profile" }
                                        span { style: "font-size: 0.7rem; color: #94a3b8;", "account" }
                                    }
                                    CommandItem {
                                        id: Some("sw4b-cmd-item-billing".to_string()),
                                        value: "billing".to_string(),
                                        disabled: true,
                                        keywords: vec!["invoice".to_string(), "payment".to_string()],
                                        on_select: move |_| record_select("billing"),
                                        span { "Billing (Disabled)" }
                                        span { style: "font-size: 0.7rem; color: #94a3b8;", "locked" }
                                    }
                                    CommandItem {
                                        id: Some("sw4b-cmd-item-preferences".to_string()),
                                        value: "preferences".to_string(),
                                        keywords: vec!["theme".to_string(), "config".to_string()],
                                        on_select: move |_| record_select("preferences"),
                                        span { "Preferences" }
                                        span { style: "font-size: 0.7rem; color: #94a3b8;", "config" }
                                    }
                                    CommandItem {
                                        id: Some("sw4b-cmd-item-shortcuts".to_string()),
                                        value: "keyboard-shortcuts".to_string(),
                                        keywords: vec!["keys".to_string(), "hotkeys".to_string()],
                                        on_select: move |_| record_select("keyboard-shortcuts"),
                                        span { "Keyboard Shortcuts" }
                                        span { style: "font-size: 0.7rem; color: #94a3b8;", "hotkeys" }
                                    }
                                    CommandItem {
                                        id: Some("sw4b-cmd-item-api-keys".to_string()),
                                        value: "api-keys".to_string(),
                                        keywords: vec!["token".to_string(), "secret".to_string()],
                                        on_select: move |_| record_select("api-keys"),
                                        span { "API Keys" }
                                        span { style: "font-size: 0.7rem; color: #94a3b8;", "token" }
                                    }
                                    CommandItem {
                                        id: Some("sw4b-cmd-item-audit-log".to_string()),
                                        value: "audit-log".to_string(),
                                        keywords: vec!["security".to_string(), "history".to_string()],
                                        on_select: move |_| record_select("audit-log"),
                                        span { "Audit Log" }
                                        span { style: "font-size: 0.7rem; color: #94a3b8;", "security" }
                                    }
                                    CommandItem {
                                        id: Some("sw4b-cmd-item-export-data".to_string()),
                                        value: "export-data".to_string(),
                                        keywords: vec!["download".to_string(), "csv".to_string()],
                                        on_select: move |_| record_select("export-data"),
                                        span { "Export Data" }
                                        span { style: "font-size: 0.7rem; color: #94a3b8;", "csv" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CommandDialogScenario() -> Element {
    let mut open = use_signal(|| false);
    let mut last_command = use_signal(|| String::from("none"));

    let mut run_command = move |cmd: &'static str| {
        last_command.set(cmd.to_string());
        open.set(false);
    };

    rsx! {
        section {
            id: "sw4b-cmd-dialog-card",
            style: "{CARD_STYLE}",
            div {
                h3 {
                    style: "margin: 0; font-size: 1.0625rem; font-weight: 700; color: #0f172a;",
                    "2. CommandDialog Modal (SW4B-4)"
                }
                p {
                    style: "margin: 0.2rem 0 0; font-size: 0.8125rem; color: #64748b;",
                    "Modal CommandDialog composing monoxus::dialog with sr-only title/description, autofocused input, Enter selection, and Escape/overlay dismiss."
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; align-items: center; gap: 0.75rem;",
                button {
                    id: "sw4b-cmd-dialog-trigger",
                    r#type: "button",
                    style: "{PRIMARY_BTN_STYLE}",
                    onclick: move |_| open.set(true),
                    "Open Command Dialog"
                }
                span {
                    id: "sw4b-cmd-dialog-state",
                    style: "{badge_style(open())}",
                    if open() { "dialog: open" } else { "dialog: closed" }
                }
                span {
                    id: "sw4b-cmd-dialog-log",
                    style: "{badge_style(last_command() != \"none\")}",
                    "last command: {last_command}"
                }
            }

            CommandDialog {
                id: Some("sw4b-cmd-dialog".to_string()),
                open: Some(open),
                on_open_change: move |next: bool| open.set(next),
                title: "Workspace Command Palette".to_string(),
                description: "Search and execute a workspace command.".to_string(),
                overlay_style: Some(DIALOG_OVERLAY_STYLE.to_string()),
                content_id: Some("sw4b-cmd-dialog-content".to_string()),
                content_style: Some(DIALOG_CONTENT_STYLE.to_string()),
                loop_focus: true,

                CommandInput {
                    id: Some("sw4b-cmd-dialog-input".to_string()),
                    placeholder: Some("Jump to file or run action...".to_string()),
                    style: Some(CMD_INPUT_STYLE.to_string()),
                }
                CommandList {
                    id: Some("sw4b-cmd-dialog-list".to_string()),
                    CommandViewport {
                        style: Some("max-height: 200px; overflow-y: auto; padding: 0.25rem 0;".to_string()),
                        CommandEmpty {
                            style: Some(CMD_EMPTY_STYLE.to_string()),
                            "No dialog commands matched."
                        }
                        CommandGroup {
                            value: Some("quick-actions".to_string()),
                            CommandGroupHeading {
                                style: Some(CMD_GROUP_HEADING_STYLE.to_string()),
                                "Quick Actions"
                            }
                            CommandGroupItems {
                                CommandItem {
                                    id: Some("sw4b-cmd-dialog-item-new-file".to_string()),
                                    value: "new-file".to_string(),
                                    keywords: vec!["create".to_string()],
                                    on_select: move |_| run_command("new-file"),
                                    "New File"
                                }
                                CommandItem {
                                    id: Some("sw4b-cmd-dialog-item-open-terminal".to_string()),
                                    value: "open-terminal".to_string(),
                                    keywords: vec!["shell".to_string(), "console".to_string()],
                                    on_select: move |_| run_command("open-terminal"),
                                    "Open Terminal"
                                }
                                CommandItem {
                                    id: Some("sw4b-cmd-dialog-item-toggle-theme".to_string()),
                                    value: "toggle-theme".to_string(),
                                    keywords: vec!["dark".to_string(), "light".to_string()],
                                    on_select: move |_| run_command("toggle-theme"),
                                    "Toggle Theme"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CommandGridScenario() -> Element {
    let mut grid_active = use_signal(String::new);
    let mut grid_selected = use_signal(|| String::from("none"));
    let mut manual_query = use_signal(String::new);

    let tiles = [
        ("tile-1", "Alpha"),
        ("tile-2", "Beta"),
        ("tile-3", "Gamma"),
        ("tile-4", "Delta"),
        ("tile-5", "Epsilon"),
        ("tile-6", "Zeta"),
    ];

    rsx! {
        section {
            id: "sw4b-cmd-grid-card",
            style: "{CARD_STYLE}",
            div {
                h3 {
                    style: "margin: 0; font-size: 1.0625rem; font-weight: 700; color: #0f172a;",
                    "3. 2D Grid & Manual Mode (columns: Some(3), should_filter: false)"
                }
                p {
                    style: "margin: 0.2rem 0 0; font-size: 0.8125rem; color: #64748b;",
                    "Uses ArrowLeft/ArrowRight/ArrowUp/ArrowDown (or Ctrl+h/l/k/j) across 3 columns. In manual mode (should_filter: false), all tiles stay mounted and any search change resets the active tile to tile-1."
                }
            }

            div {
                style: "display: flex; flex-wrap: wrap; gap: 0.5rem;",
                span {
                    id: "sw4b-cmd-grid-active",
                    style: "{badge_style(!grid_active().is_empty())}",
                    "active tile: {grid_active}"
                }
                span {
                    id: "sw4b-cmd-grid-log",
                    style: "{badge_style(grid_selected() != \"none\")}",
                    "selected tile: {grid_selected}"
                }
                span {
                    id: "sw4b-cmd-grid-query",
                    style: "{badge_style(!manual_query().is_empty())}",
                    "manual query: {manual_query}"
                }
            }

            div {
                style: "{STAGE_STYLE}",
                CommandRoot {
                    id: Some("sw4b-cmd-grid".to_string()),
                    value: Some(grid_active),
                    on_value_change: move |v: String| grid_active.set(v),
                    label: "2D Grid Command Palette".to_string(),
                    columns: Some(3),
                    should_filter: false,
                    loop_focus: true,
                    vim_bindings: true,
                    style: Some(CMD_ROOT_STYLE.to_string()),

                    CommandInput {
                        id: Some("sw4b-cmd-grid-input".to_string()),
                        value: Some(manual_query),
                        on_value_change: move |q: String| manual_query.set(q),
                        placeholder: Some("Manual mode (items stay visible)...".to_string()),
                        style: Some(CMD_INPUT_STYLE.to_string()),
                    }
                    CommandList {
                        id: Some("sw4b-cmd-grid-list".to_string()),
                        CommandViewport {
                            id: Some("sw4b-cmd-grid-viewport".to_string()),
                            style: Some("display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 0.375rem; padding: 0.5rem;".to_string()),
                            for (val, label) in tiles {
                                CommandItem {
                                    key: "{val}",
                                    id: Some(format!("sw4b-cmd-grid-{val}")),
                                    value: val.to_string(),
                                    on_select: move |_| grid_selected.set(val.to_string()),
                                    style: Some("margin: 0; justify-content: center; border: 1px solid #e2e8f0;".to_string()),
                                    "{label}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
