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
        assert_eq!(
            harness
                .attr("#native-cmd-dlg-content", "data-command-dialog")
                .as_deref(),
            Some("")
        );
        assert!(
            harness
                .query("#native-cmd-dlg-content [data-command-dialog]")
                .is_none(),
            "data-command-dialog must be attached directly on DialogContent without a separate inner wrapper"
        );

        focus_node(&mut harness, "#native-dlg-input");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Escape), Code::Escape);
        advance_frame_and_pump(&mut harness, 0.30);
        assert!(harness.query("#native-cmd-dlg-content").is_none());
    }

    #[component]
    fn NativeCommandFocusOwnershipScenario() -> Element {
        let mut selected = use_signal(String::new);
        rsx! {
            div {
                button { id: "outside-btn", r#type: "button", "Outside" }
                div { id: "focus-log", "data-val": "{selected}", "{selected}" }
                CommandRoot {
                    id: "fm-cmd-focus",
                    label: "Focus Test",
                    loop_focus: true,
                    CommandInput { id: "fm-cmd-focus-input" }
                    CommandList {
                        CommandViewport {
                            CommandItem {
                                id: "focus-item-1",
                                value: "item-1",
                                on_select: move |_| selected.set("item-1".to_string()),
                                "Item 1"
                            }
                            CommandItem {
                                id: "focus-item-2",
                                value: "item-2",
                                on_select: move |_| selected.set("item-2".to_string()),
                                "Item 2"
                            }
                            CommandItem {
                                id: "focus-item-3",
                                value: "item-3",
                                on_select: move |_| selected.set("item-3".to_string()),
                                "Item 3"
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_native_command_outside_click_and_inside_input_focus_ownership() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeCommandFocusOwnershipScenario);

        // FM-CMD-1: Outside focus -> click item #2 -> CommandRoot gains focus -> ArrowDown moves to item #3
        focus_node(&mut harness, "#outside-btn");
        assert_native_focus_id(&harness, "outside-btn");

        click_and_pump(&mut harness, "#focus-item-2");
        assert_native_focus_id(&harness, "fm-cmd-focus");
        assert_eq!(
            harness.attr("#fm-cmd-focus-input", "aria-activedescendant").as_deref(),
            Some("focus-item-2")
        );

        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
        assert_eq!(
            harness.attr("#fm-cmd-focus-input", "aria-activedescendant").as_deref(),
            Some("focus-item-3")
        );
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowUp), Code::ArrowUp);
        assert_eq!(
            harness.attr("#fm-cmd-focus-input", "aria-activedescendant").as_deref(),
            Some("focus-item-2")
        );

        // FM-CMD-2: Focus inside CommandInput -> click item #3 -> focus stays on CommandInput
        focus_node(&mut harness, "#fm-cmd-focus-input");
        assert_native_focus_id(&harness, "fm-cmd-focus-input");

        click_and_pump(&mut harness, "#focus-item-3");
        assert_native_focus_id(&harness, "fm-cmd-focus-input");
        assert_eq!(
            harness.attr("#fm-cmd-focus-input", "aria-activedescendant").as_deref(),
            Some("focus-item-3")
        );
    }

    #[component]
    fn NativeCommandVisualOrderAndManualScenario() -> Element {
        let mut search_val = use_signal(String::new);
        let mut manual_search = use_signal(String::new);
        rsx! {
            div {
                button { id: "q-d", onclick: move |_| search_val.set("d".to_string()), "Search d" }
                button { id: "q-clear", onclick: move |_| search_val.set(String::new()), "Clear" }
                button { id: "m-a", onclick: move |_| manual_search.set("a".to_string()), "Manual a" }

                CommandRoot {
                    id: "fm-cmd-order",
                    label: "Order Test",
                    CommandInput { id: "fm-cmd-order-input", value: search_val }
                    CommandList {
                        CommandViewport { id: "fm-cmd-order-vp",
                            CommandGroup {
                                id: "fm-g-docs",
                                value: "docs",
                                style: "display: flex; flex-direction: column;",
                                CommandGroupHeading { "Docs" }
                                CommandGroupItems {
                                    CommandItem { id: "fm-i-intro", value: "Introduction", "Introduction" }
                                    CommandItem { id: "fm-i-del", value: "Delegation", "Delegation" }
                                }
                            }
                            CommandGroup {
                                id: "fm-g-other",
                                value: "other",
                                style: "display: flex; flex-direction: column;",
                                CommandGroupHeading { "Other" }
                                CommandGroupItems {
                                    CommandItem { id: "fm-i-prof", value: "Profile", "Profile" }
                                }
                            }
                        }
                    }
                }

                CommandRoot {
                    id: "fm-cmd-manual",
                    label: "Manual Test",
                    should_filter: false,
                    CommandInput { id: "fm-cmd-manual-input", value: manual_search }
                    CommandList {
                        CommandViewport {
                            CommandItem { id: "fm-m1", value: "m1", "M1" }
                            CommandItem { id: "fm-m2", value: "m2", "M2" }
                            CommandItem { id: "fm-m3", value: "m3", "M3" }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_native_command_visual_order_force_select_first_and_group_hidden() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeCommandVisualOrderAndManualScenario);

        // Initial: Introduction precedes Delegation
        assert_eq!(
            harness.attr("#fm-cmd-order-input", "aria-activedescendant").as_deref(),
            Some("fm-i-intro")
        );
        assert!(harness.layout_rect("#fm-i-intro").y < harness.layout_rect("#fm-i-del").y);

        // FM-CMD-3 & FM-CMD-6: Search "d" -> Delegation outranks Introduction visually & by keyboard; Other group hidden with height 0
        click_and_pump(&mut harness, "#q-d");
        assert_eq!(
            harness.attr("#fm-cmd-order-input", "aria-activedescendant").as_deref(),
            Some("fm-i-del")
        );
        assert!(
            harness.layout_rect("#fm-i-del").y < harness.layout_rect("#fm-i-intro").y,
            "Delegation must render above Introduction when ranked higher"
        );
        assert!(harness.attr("#fm-g-other", "hidden").is_some());
        assert_eq!(
            harness.attr("#fm-g-other", "style").as_deref(),
            Some("display: none;")
        );
        assert_eq!(harness.layout_rect("#fm-g-other").height, 0.0);

        focus_node(&mut harness, "#fm-cmd-order-input");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
        assert_eq!(
            harness.attr("#fm-cmd-order-input", "aria-activedescendant").as_deref(),
            Some("fm-i-intro")
        );

        // Clear search -> resets selection to Introduction and restores visual order
        click_and_pump(&mut harness, "#q-clear");
        assert_eq!(
            harness.attr("#fm-cmd-order-input", "aria-activedescendant").as_deref(),
            Some("fm-i-intro")
        );
        assert!(harness.layout_rect("#fm-i-intro").y < harness.layout_rect("#fm-i-del").y);

        // FM-CMD-4: should_filter: false resets to first item on search change while keeping items mounted
        focus_node(&mut harness, "#fm-cmd-manual-input");
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
        send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);
        assert_eq!(
            harness.attr("#fm-cmd-manual-input", "aria-activedescendant").as_deref(),
            Some("fm-m3")
        );
        click_and_pump(&mut harness, "#m-a");
        assert_eq!(
            harness.attr("#fm-cmd-manual-input", "aria-activedescendant").as_deref(),
            Some("fm-m1")
        );
        assert!(harness.query("#fm-m3").is_some());
    }

    #[component]
    fn NativeCommandLinkAndImeScenario() -> Element {
        let mut select_count = use_signal(|| 0usize);
        let mut click_count = use_signal(|| 0usize);
        rsx! {
            div {
                div {
                    id: "link-counts",
                    "data-select": "{select_count}",
                    "data-click": "{click_count}",
                }
                CommandRoot {
                    id: "fm-cmd-link",
                    label: "Link Test",
                    CommandInput { id: "fm-cmd-link-input" }
                    CommandList {
                        CommandViewport {
                            CommandLinkItem {
                                id: "fm-link-docs",
                                value: "docs",
                                href: "#docs",
                                on_select: move |_| select_count += 1,
                                on_click: move |_| click_count += 1,
                                "Docs"
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_native_command_link_item_enter_and_ime_composing_guard() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeCommandLinkAndImeScenario);

        focus_node(&mut harness, "#fm-cmd-link-input");

        // IME composing Enter must NOT activate item
        let composing_down = blitz_traits::events::BlitzKeyEvent {
            key: Key::Named(NamedKey::Enter),
            code: Code::Enter,
            modifiers: Modifiers::default(),
            location: keyboard_types::Location::Standard,
            is_auto_repeating: false,
            is_composing: true,
            state: blitz_traits::events::KeyState::Pressed,
            text: None,
        };
        harness.dispatch(blitz_traits::events::UiEvent::KeyDown(composing_down));
        drain_polls_and_resolve(&mut harness);
        assert_eq!(
            harness.attr("#link-counts", "data-select").as_deref(),
            Some("0")
        );

        // FM-CMD-5: Non-composing Enter dispatches DOM click on <a> and fires on_select exactly once
        send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
        assert_eq!(
            harness.attr("#link-counts", "data-select").as_deref(),
            Some("1")
        );
        assert_eq!(
            harness.attr("#link-counts", "data-click").as_deref(),
            Some("1")
        );
    }

    #[component]
    fn NativeCommandDialogCenteredScenario() -> Element {
        let mut open = use_signal(|| false);
        rsx! {
            div {
                style {
                    "@keyframes dialog-scale-in {{ from {{ transform: scale(0.95); opacity: 0; }} to {{ transform: scale(1); opacity: 1; }} }}
                     [role='dialog'][data-state='open'] {{ animation: dialog-scale-in 200ms cubic-bezier(0.16, 1, 0.3, 1); }}"
                }
                button {
                    id: "open-centered-dlg",
                    onclick: move |_| open.set(true),
                    "Open"
                }
                CommandDialog {
                    id: "fm-centered-dlg",
                    content_id: "fm-centered-dlg-content",
                    content_style: "width: 400px; height: 240px; background-color: #ffffff;",
                    open: Some(open),
                    on_open_change: move |next| open.set(next),
                    // No autofocus prop: proves library-owned CommandDialog auto_focus_input
                    CommandInput { id: "fm-centered-dlg-input" }
                    CommandList {
                        CommandViewport {
                            CommandItem { id: "fm-centered-dlg-item", value: "Run", "Run" }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_native_command_dialog_viewport_center_and_auto_focus() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeCommandDialogCenteredScenario);

        click_and_pump(&mut harness, "#open-centered-dlg");
        advance_frame_and_pump(&mut harness, 0.30);

        // FM-CMD-7: Auto-focused CommandInput without consumer autofocus prop
        assert_native_focus_id(&harness, "fm-centered-dlg-input");

        // DEF-CMD-04 / 4B D-3: DialogContent itself carries both role="dialog" and data-command-dialog=""
        assert_eq!(
            harness.attr("#fm-centered-dlg-content", "role").as_deref(),
            Some("dialog")
        );
        assert_eq!(
            harness
                .attr("#fm-centered-dlg-content", "data-command-dialog")
                .as_deref(),
            Some("")
        );
        assert!(
            harness
                .query("#fm-centered-dlg-content [data-command-dialog]")
                .is_none(),
            "data-command-dialog must be attached directly on DialogContent without a separate inner wrapper"
        );

        let pos_rect = harness.layout_rect("[data-command-dialog-positioner]");
        let dlg_rect = harness.layout_rect("#fm-centered-dlg-content");
        let pos_cx = pos_rect.x + pos_rect.width * 0.5;
        let pos_cy = pos_rect.y + pos_rect.height * 0.5;
        let dlg_cx = dlg_rect.x + dlg_rect.width * 0.5;
        let dlg_cy = dlg_rect.y + dlg_rect.height * 0.5;
        assert!(
            (dlg_cx - pos_cx).abs() <= 2.0,
            "Dialog X center {dlg_cx} diverged from positioner center {pos_cx}"
        );
        assert!(
            (dlg_cy - pos_cy).abs() <= 2.0,
            "Dialog Y center {dlg_cy} diverged from positioner center {pos_cy}"
        );
    }

    #[component]
    fn NativeCommandItemHandlerToggleScenario() -> Element {
        let mut step = use_signal(|| 0u8);
        let mut cb1_hits = use_signal(|| 0usize);
        let mut cb2_hits = use_signal(|| 0usize);
        let on_select: Option<EventHandler<()>> = match step() {
            0 => None,
            1 => Some(EventHandler::new(move |()| cb1_hits += 1)),
            _ => Some(EventHandler::new(move |()| cb2_hits += 1)),
        };
        let on_pointer_move: Option<EventHandler<PointerEvent>> = match step() {
            0 => None,
            1 => Some(EventHandler::new(move |_| {})),
            _ => Some(EventHandler::new(move |_| {})),
        };

        rsx! {
            div {
                button {
                    id: "fm-cmd-advance",
                    onclick: move |_| step += 1,
                    "Advance"
                }
                div {
                    id: "fm-cmd-log",
                    "data-cb1": "{cb1_hits}",
                    "data-cb2": "{cb2_hits}",
                }
                CommandRoot {
                    id: "fm-cmd-toggle-root",
                    CommandInput { id: "fm-cmd-toggle-input" }
                    CommandList {
                        CommandViewport {
                            CommandItem {
                                id: "fm-cmd-toggle-item",
                                value: "ToggleItem",
                                on_select: on_select,
                                on_pointer_move: on_pointer_move,
                                "ToggleItem"
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_native_command_item_handler_toggle_invokes_latest_cb() {
        let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut harness = mount_native_scenario(NativeCommandItemHandlerToggleScenario);

        // FM-API-4: None -> Some(cb1) -> Some(cb2) across 3 renders without ValueDroppedError.
        click_and_pump(&mut harness, "#fm-cmd-advance");
        click_and_pump(&mut harness, "#fm-cmd-advance");
        click_and_pump(&mut harness, "#fm-cmd-toggle-item");

        assert_eq!(harness.attr("#fm-cmd-log", "data-cb1").as_deref(), Some("0"));
        assert_eq!(harness.attr("#fm-cmd-log", "data-cb2").as_deref(), Some("1"));
    }
}

#[test]
fn test_command_item_optional_handler_toggle_vdom() {
    #[component]
    fn ToggleHost(holder: std::rc::Rc<std::cell::RefCell<Option<Signal<u8>>>>) -> Element {
        let step = use_signal(|| 0u8);
        *holder.borrow_mut() = Some(step);
        let on_select: Option<EventHandler<()>> = match step() {
            0 => None,
            1 => Some(EventHandler::new(move |()| {})),
            _ => Some(EventHandler::new(move |()| {})),
        };
        rsx! {
            CommandRoot {
                id: "vdom-fm-api-4-cmd",
                CommandInput {}
                CommandList {
                    CommandViewport {
                        CommandItem {
                            value: "One",
                            on_select: on_select,
                            "One"
                        }
                    }
                }
            }
        }
    }

    let holder = std::rc::Rc::new(std::cell::RefCell::new(None));
    let mut dom = VirtualDom::new_with_props(
        ToggleHost,
        ToggleHostProps {
            holder: holder.clone(),
        },
    );
    dom.rebuild_in_place();
    let mut step = holder.borrow().expect("step signal captured");
    dom.in_scope(ScopeId::APP, || step.set(1));
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    dom.in_scope(ScopeId::APP, || step.set(2));
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
}
