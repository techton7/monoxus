use dioxus::prelude::*;
use monoxus::command::*;
use monoxus::foundation::{compose::Slottable, shared::ScopeHandle};

#[test]
fn test_command_parts_and_relationships_contract() {
    assert_eq!(
        COMMAND_PARTS
            .iter()
            .map(CommandPart::as_str)
            .collect::<Vec<_>>(),
        [
            "root",
            "input-label",
            "input",
            "list",
            "viewport",
            "empty",
            "loading",
            "group",
            "group-heading",
            "group-items",
            "item",
            "separator",
        ]
    );

    let scope = ScopeHandle::root("command").child("palette");
    let rels = CommandRelationships::new(scope.clone());
    assert_eq!(rels.root_id(), scope.token());
    assert_eq!(rels.input_id(), scope.qualify("input"));
    assert_eq!(rels.label_id(), scope.qualify("label"));
    assert_eq!(rels.list_id(), scope.qualify("list"));
    assert_eq!(rels.viewport_id(), scope.qualify("viewport"));
    assert_eq!(rels.group_id("suggestions"), scope.qualify("group-suggestions"));
    assert_eq!(
        rels.group_heading_id("suggestions"),
        scope.qualify("group-heading-suggestions")
    );
    assert_eq!(rels.item_id("calendar"), scope.qualify("item-calendar"));
}

#[test]
fn test_command_scoring_filtering_and_empty_group_hiding() {
    assert_eq!(compute_command_score("Calendar", "Calendar", &[]), 1.0);
    assert!(compute_command_score("Calendar", "cal", &[]) > 0.0);
    assert_eq!(compute_command_score("Calendar", "zzz", &[]), 0.0);
    assert!(
        compute_command_score("Calculator", "math", &["math".to_string()])
            > compute_command_score("Calculator", "math", &[])
    );

    let mut reg = CommandRegistry::new();
    reg.register_group(
        "suggestions".into(),
        "g-sugg".into(),
        Some("h-sugg".into()),
        false,
    );
    reg.register_group(
        "settings".into(),
        "g-sett".into(),
        Some("h-sett".into()),
        false,
    );
    reg.register_item(
        "Calendar".into(),
        "i-cal".into(),
        Some("suggestions".into()),
        false,
        false,
        vec!["date".into()],
    );
    reg.register_item(
        "Calculator".into(),
        "i-calc".into(),
        Some("suggestions".into()),
        false,
        false,
        vec!["math".into()],
    );
    reg.register_item(
        "Billing".into(),
        "i-bill".into(),
        Some("settings".into()),
        false,
        false,
        vec!["invoice".into()],
    );

    reg.recompute_filtered_default("math", true);
    assert_eq!(reg.filtered().count, 1);
    assert!(reg.is_group_visible("suggestions"));
    assert!(!reg.is_group_visible("settings"));
    assert_eq!(
        reg.valid_items("math", true)
            .into_iter()
            .map(|i| i.value.as_str())
            .collect::<Vec<_>>(),
        vec!["Calculator"]
    );

    // Non-matching query hides all groups and triggers empty state once settled
    reg.recompute_filtered_default("nonexistent", true);
    assert_eq!(reg.filtered().count, 0);
    assert!(!reg.is_group_visible("suggestions"));
    assert!(!reg.is_group_visible("settings"));
    assert!(should_render_empty(reg.filtered().count, true, false));
    assert!(!should_render_separator("nonexistent", false));
    assert!(should_render_separator("nonexistent", true));
}

#[test]
fn test_command_keyboard_navigation_1d_vim_group_jump_and_2d_grid() {
    let mut reg = CommandRegistry::new();
    reg.register_group("g1".into(), "g1".into(), None, false);
    reg.register_group("g2".into(), "g2".into(), None, false);
    reg.register_item("one".into(), "i1".into(), Some("g1".into()), false, false, vec![]);
    reg.register_item("two".into(), "i2".into(), Some("g1".into()), true, false, vec![]);
    reg.register_item("three".into(), "i3".into(), Some("g1".into()), false, false, vec![]);
    reg.register_item("four".into(), "i4".into(), Some("g2".into()), false, false, vec![]);
    reg.recompute_filtered_default("", true);

    let plain = CommandNavModifiers::default();
    let ctrl = CommandNavModifiers::new(false, false, true);
    let alt = CommandNavModifiers::new(false, true, false);

    // ArrowDown skips disabled "two"
    assert_eq!(
        resolve_command_navigation(&reg, "", true, "one", CommandNavKey::ArrowDown, plain, None, false, true).as_deref(),
        Some("three")
    );
    // Vim Ctrl+n / Ctrl+p
    assert_eq!(
        resolve_command_navigation(&reg, "", true, "one", CommandNavKey::VimNext, ctrl, None, false, true).as_deref(),
        Some("three")
    );
    assert_eq!(
        resolve_command_navigation(&reg, "", true, "three", CommandNavKey::VimPrev, ctrl, None, false, true).as_deref(),
        Some("one")
    );
    // Alt + ArrowDown jumps to next group
    assert_eq!(
        resolve_command_navigation(&reg, "", true, "one", CommandNavKey::ArrowDown, alt, None, false, true).as_deref(),
        Some("four")
    );
    // Home / End
    assert_eq!(
        resolve_command_navigation(&reg, "", true, "four", CommandNavKey::Home, plain, None, false, true).as_deref(),
        Some("one")
    );
    assert_eq!(
        resolve_command_navigation(&reg, "", true, "one", CommandNavKey::End, plain, None, false, true).as_deref(),
        Some("four")
    );

    // 2D grid navigation (columns = 2)
    let mut grid_reg = CommandRegistry::new();
    for i in 1..=4 {
        grid_reg.register_item(format!("c{i}"), format!("id-{i}"), None, false, false, vec![]);
    }
    grid_reg.recompute_filtered_default("", true);
    assert_eq!(
        resolve_command_navigation(&grid_reg, "", true, "c1", CommandNavKey::ArrowRight, plain, Some(2), false, true).as_deref(),
        Some("c2")
    );
    assert_eq!(
        resolve_command_navigation(&grid_reg, "", true, "c1", CommandNavKey::ArrowDown, plain, Some(2), false, true).as_deref(),
        Some("c3")
    );
    assert_eq!(
        resolve_command_navigation(&grid_reg, "", true, "c3", CommandNavKey::ArrowUp, plain, Some(2), false, true).as_deref(),
        Some("c1")
    );
}

#[test]
fn test_command_attrs_boolean_omission_and_as_child_projection() {
    let item_idle = CommandItemAttrs::new("it-1", "Calendar", Some("sugg".into()), false, false);
    assert_eq!(item_idle.role, "option");
    assert_eq!(item_idle.aria_disabled, "false");
    assert_eq!(item_idle.aria_selected, "false");
    assert_eq!(item_idle.data_disabled, None);
    assert_eq!(item_idle.data_selected, None);

    let item_active = CommandItemAttrs::new("it-2", "Billing", None, true, true);
    assert_eq!(item_active.aria_disabled, "true");
    assert_eq!(item_active.aria_selected, "true");
    assert_eq!(item_active.data_disabled, Some(""));
    assert_eq!(item_active.data_selected, Some(""));

    let loading = CommandLoadingAttrs::new("load-1", 42.0, None);
    assert_eq!(loading.role, "progressbar");
    assert_eq!(loading.aria_valuenow, 42.0);
    assert_eq!(loading.aria_valuemin, "0");
    assert_eq!(loading.aria_valuemax, "100");
    assert_eq!(loading.aria_label, "Loading...");

    let (_, projected, _) = project_command_part_as_child(&item_idle, "el", Slottable::new("ok"));
    assert_eq!(projected, item_idle);
}

#[component]
fn CommandAllPartsVdomApp() -> Element {
    rsx! {
        Command {
            id: "vdom-cmd",
            label: "Command Menu",
            CommandInput { id: "vdom-cmd-input", placeholder: "Search..." }
            CommandList { id: "vdom-cmd-list",
                CommandViewport { id: "vdom-cmd-viewport",
                    CommandEmpty { id: "vdom-cmd-empty", "No results." }
                    CommandLoading { id: "vdom-cmd-loading", progress: 50.0, "Loading..." }
                    CommandGroup { id: "vdom-cmd-g1", value: "Suggestions",
                        CommandGroupHeading { id: "vdom-cmd-h1", "Suggestions" }
                        CommandGroupItems { id: "vdom-cmd-gi1",
                            CommandItem { id: "vdom-cmd-cal", value: "Calendar", "Calendar" }
                            CommandItem { id: "vdom-cmd-dis", value: "Disabled", disabled: true, "Disabled" }
                            CommandLinkItem {
                                id: "vdom-cmd-link",
                                value: "Docs",
                                href: "https://example.com/docs",
                                "Docs"
                            }
                        }
                    }
                    CommandSeparator { id: "vdom-cmd-sep" }
                    CommandGroup { id: "vdom-cmd-g2", value: "Settings",
                        CommandGroupHeading { id: "vdom-cmd-h2", "Settings" }
                        CommandGroupItems { id: "vdom-cmd-gi2",
                            CommandItem { id: "vdom-cmd-prof", value: "Profile", "Profile" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CommandDialogVdomApp() -> Element {
    rsx! {
        CommandDialog {
            id: "vdom-cmd-dialog",
            default_open: Some(true),
            CommandInput { id: "vdom-dlg-input", placeholder: "Type a command..." }
            CommandList {
                CommandViewport {
                    CommandItem { id: "vdom-dlg-item", value: "Run", "Run" }
                }
            }
        }
    }
}

#[test]
fn test_command_vdom_renders_all_parts_and_dialog_cleanly() {
    let mut dom = VirtualDom::new(CommandAllPartsVdomApp);
    dom.rebuild_in_place();

    let mut dlg_dom = VirtualDom::new(CommandDialogVdomApp);
    dlg_dom.rebuild_in_place();
}

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use crate::common::native_fixture::*;
    use keyboard_types::{Code, Key, Modifiers, NamedKey};

    #[component]
    fn NativeCommandScenario() -> Element {
        let mut selected_log = use_signal(String::new);
        let mut show_dynamic_item = use_signal(|| true);
        let mut search_val = use_signal(String::new);

        rsx! {
            div {
                button {
                    id: "toggle-dynamic",
                    onclick: move |_| show_dynamic_item.toggle(),
                    "Toggle"
                }
                button {
                    id: "set-search-prof",
                    onclick: move |_| search_val.set("Profile".to_string()),
                    "Filter Profile"
                }
                button {
                    id: "set-search-empty",
                    onclick: move |_| search_val.set("zzzzzz".to_string()),
                    "Filter Empty"
                }
                button {
                    id: "clear-search",
                    onclick: move |_| search_val.set(String::new()),
                    "Clear"
                }
                div { id: "selected-log", "data-selected-log": "{selected_log}", "{selected_log}" }
                CommandRoot {
                    id: "native-cmd",
                    label: "Palette",
                    loop_focus: true,
                    vim_bindings: true,
                    CommandInput {
                        id: "native-cmd-input",
                        value: search_val,
                        placeholder: "Type...",
                    }
                    CommandList { id: "native-cmd-list",
                        CommandViewport { id: "native-cmd-viewport",
                            CommandEmpty { id: "native-cmd-empty", "Nothing found" }
                            CommandGroup { id: "native-g1", value: "Suggestions",
                                CommandGroupHeading { id: "native-h1", "Suggestions" }
                                CommandGroupItems { id: "native-gi1",
                                    CommandItem {
                                        id: "item-cal",
                                        value: "Calendar",
                                        on_select: move |_| selected_log.set("Calendar".to_string()),
                                        "Calendar"
                                    }
                                    CommandItem {
                                        id: "item-dis",
                                        value: "Disabled",
                                        disabled: true,
                                        on_select: move |_| selected_log.set("Disabled".to_string()),
                                        "Disabled"
                                    }
                                    if show_dynamic_item() {
                                        CommandItem {
                                            id: "item-dyn",
                                            value: "Dynamic",
                                            on_select: move |_| selected_log.set("Dynamic".to_string()),
                                            "Dynamic"
                                        }
                                    }
                                }
                            }
                            CommandSeparator { id: "native-sep" }
                            CommandGroup { id: "native-g2", value: "Settings",
                                CommandGroupHeading { id: "native-h2", "Settings" }
                                CommandGroupItems { id: "native-gi2",
                                    CommandItem {
                                        id: "item-prof",
                                        value: "Profile",
                                        on_select: move |_| selected_log.set("Profile".to_string()),
                                        "Profile"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_native_command_virtual_focus_keyboard_filtering_and_drop_cleanup() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeCommandScenario);

        // Input role & ARIA attributes
        assert_eq!(
            harness.attr("#native-cmd-input", "role").as_deref(),
            Some("combobox")
        );
        assert_eq!(
            harness.attr("#native-cmd-input", "aria-expanded").as_deref(),
            Some("true")
        );
        assert_eq!(
            harness.attr("#native-cmd-input", "aria-controls").as_deref(),
            Some("native-cmd-viewport")
        );
        assert_eq!(
            harness.attr("#native-cmd-input", "aria-activedescendant").as_deref(),
            Some("item-cal")
        );
        assert_eq!(
            harness.attr("#native-gi1", "aria-labelledby").as_deref(),
            Some("native-h1")
        );

        // Focus input and navigate with ArrowDown (skips disabled item-dis -> item-dyn)
        focus_node(&mut harness, "#native-cmd-input");
        assert_native_focus_id(&harness, "native-cmd-input");

        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
        assert_eq!(
            harness.attr("#native-cmd-input", "aria-activedescendant").as_deref(),
            Some("item-dyn")
        );
        assert_eq!(
            harness.attr("#item-dyn", "data-selected").as_deref(),
            Some("")
        );
        assert_eq!(harness.attr("#item-cal", "data-selected"), None);
        assert_native_focus_id(&harness, "native-cmd-input");

        // Vim Ctrl+n moves to item-prof
        send_key_and_pump_with(
            &mut harness,
            Key::Character("n".into()),
            Code::KeyN,
            Modifiers::CONTROL,
        );
        assert_eq!(
            harness.attr("#native-cmd-input", "aria-activedescendant").as_deref(),
            Some("item-prof")
        );

        // Enter selects active item ("Profile")
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
        assert_eq!(
            harness.attr("#selected-log", "data-selected-log").as_deref(),
            Some("Profile")
        );

        // Alt+ArrowUp jumps to previous group ("Suggestions" -> item-cal)
        send_key_and_pump_with(
            &mut harness,
            Key::Named(NamedKey::ArrowUp),
            Code::ArrowUp,
            Modifiers::ALT,
        );
        assert_eq!(
            harness.attr("#native-cmd-input", "aria-activedescendant").as_deref(),
            Some("item-cal")
        );

        // Dynamic unmount (use_drop cleanup): move to item-dyn then unmount it
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
        assert_eq!(
            harness.attr("#native-cmd-input", "aria-activedescendant").as_deref(),
            Some("item-dyn")
        );
        click_and_pump(&mut harness, "#toggle-dynamic");
        assert!(harness.query("#item-dyn").is_none());
        assert_eq!(
            harness.attr("#native-cmd-input", "aria-activedescendant").as_deref(),
            Some("item-cal")
        );

        // Filtering hides empty group and separator, keeps matching group
        click_and_pump(&mut harness, "#set-search-prof");
        assert!(harness.query("#native-sep").is_none());
        assert!(harness.attr("#native-g1", "hidden").is_some());
        assert!(harness.attr("#native-g2", "hidden").is_none());
        assert_eq!(
            harness.attr("#native-cmd-input", "aria-activedescendant").as_deref(),
            Some("item-prof")
        );

        // Non-matching search shows CommandEmpty
        click_and_pump(&mut harness, "#set-search-empty");
        assert!(harness.query("#native-cmd-empty").is_some());
    }

    #[component]
    fn NativeCommandDialogScenario() -> Element {
        let mut open = use_signal(|| false);
        rsx! {
            div {
                button {
                    id: "open-cmd-dlg",
                    onclick: move |_| open.set(true),
                    "Open"
                }
                CommandDialog {
                    id: "native-cmd-dlg",
                    content_id: "native-cmd-dlg-content",
                    open: Some(open),
                    on_open_change: move |next| open.set(next),
                    CommandInput { id: "native-dlg-input", autofocus: true }
                    CommandList {
                        CommandViewport {
                            CommandItem { id: "native-dlg-item", value: "Action", "Action" }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_native_command_dialog_open_and_escape_close() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeCommandDialogScenario);

        assert!(harness.query("#native-cmd-dlg-content").is_none());
        click_and_pump(&mut harness, "#open-cmd-dlg");
        assert!(harness.query("#native-cmd-dlg-content").is_some());
        assert_eq!(
            harness.attr("#native-cmd-dlg-content", "role").as_deref(),
            Some("dialog")
        );
        assert!(harness.query("[data-command-dialog]").is_some());

        focus_node(&mut harness, "#native-dlg-input");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        advance_frame_and_pump(&mut harness, 0.30);
        assert!(harness.query("#native-cmd-dlg-content").is_none());
    }
}
