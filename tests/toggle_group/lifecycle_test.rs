use monoxus::foundation::compose::project_attrs_as_child;
use dioxus::prelude::*;
use monoxus::{
    foundation::{compose::Slottable, shared::{CollectionRegistry, ScopeHandle}},
    toggle_group::{
        ToggleGroup, ToggleGroupItem, ToggleGroupItemRegistration, ToggleGroupMode,
        ToggleGroupRoot,
    },
};

#[test]
fn test_tab_stop_priority_resolution() {
    let mut reg = CollectionRegistry::new();
    reg.register(
        "item-1".to_string(),
        ToggleGroupItemRegistration {
            value: "item-1".to_string(),
            id: "id-1".to_string(),
            disabled: true, // disabled
        },
    );
    reg.register(
        "item-2".to_string(),
        ToggleGroupItemRegistration {
            value: "item-2".to_string(),
            id: "id-2".to_string(),
            disabled: false,
        },
    );
    reg.register(
        "item-3".to_string(),
        ToggleGroupItemRegistration {
            value: "item-3".to_string(),
            id: "id-3".to_string(),
            disabled: false,
        },
    );

    let scope = ScopeHandle::root("tg-lifecycle");
    let group = ToggleGroup::new(scope.clone(), ToggleGroupMode::Single);

    // Initial resolution with no current tab stop and none pressed: picks first registered enabled (item-2)
    let initial_tab_stop = group.resolve_tab_stop(&reg);
    assert_eq!(initial_tab_stop, Some("item-2".to_string()));

    // When item-3 is pressed, priority 2 picks the pressed enabled item (item-3)
    let pressed_group = group.clone().with_single_value("item-3");
    let pressed_tab_stop = pressed_group.resolve_tab_stop(&reg);
    assert_eq!(pressed_tab_stop, Some("item-3".to_string()));

    // When current_tab_stop is explicitly set to an enabled item (item-2), priority 1 keeps item-2
    let active_group = pressed_group.with_current_tab_stop(Some("item-2".to_string()));
    let active_tab_stop = active_group.resolve_tab_stop(&reg);
    assert_eq!(active_tab_stop, Some("item-2".to_string()));

    // If active tab stop item becomes disabled, fallback picks pressed enabled item
    let mut modified_reg = reg.clone();
    modified_reg.register(
        "item-2".to_string(),
        ToggleGroupItemRegistration {
            value: "item-2".to_string(),
            id: "id-2".to_string(),
            disabled: true,
        },
    );
    let fallback_tab_stop = active_group.resolve_tab_stop(&modified_reg);
    assert_eq!(fallback_tab_stop, Some("item-3".to_string()));
}

#[test]
fn test_unmount_unregistration_and_tab_stop_recovery() {
    let mut reg = CollectionRegistry::new();
    reg.register(
        "a".to_string(),
        ToggleGroupItemRegistration {
            value: "a".to_string(),
            id: "id-a".to_string(),
            disabled: false,
        },
    );
    reg.register(
        "b".to_string(),
        ToggleGroupItemRegistration {
            value: "b".to_string(),
            id: "id-b".to_string(),
            disabled: false,
        },
    );

    let scope = ScopeHandle::root("tg-unmount");
    let mut group = ToggleGroup::new(scope, ToggleGroupMode::Single)
        .with_current_tab_stop(Some("a".to_string()));

    // Unregister "a" (as when component unmounts)
    reg.unregister(&"a".to_string());
    if group.current_tab_stop() == Some("a") {
        group.set_current_tab_stop(None);
    }

    // Now tab stop smoothly recovers to "b" without error
    let recovered = group.resolve_tab_stop(&reg);
    assert_eq!(recovered, Some("b".to_string()));
}

#[test]
fn test_reorder_by_keys() {
    let mut reg = CollectionRegistry::new();
    reg.register(
        "first".to_string(),
        ToggleGroupItemRegistration {
            value: "first".to_string(),
            id: "id-1".to_string(),
            disabled: false,
        },
    );
    reg.register(
        "second".to_string(),
        ToggleGroupItemRegistration {
            value: "second".to_string(),
            id: "id-2".to_string(),
            disabled: false,
        },
    );

    // Reorder: ["second", "first"]
    reg.reorder_by_keys(&["second".to_string(), "first".to_string()]);

    let keys: Vec<_> = reg.iter().map(|e| e.key().clone()).collect();
    assert_eq!(keys, vec!["second".to_string(), "first".to_string()]);
}

#[test]
fn test_as_child_slot_projection_helpers() {
    let scope = ScopeHandle::root("tg-as-child");
    let group = ToggleGroup::new(scope, ToggleGroupMode::Single);

    let root_attrs = group.root_attributes(None);
    let slottable = Slottable::new("Inner group content");
    let (target, projected_root, content) =
        project_attrs_as_child(&root_attrs, 100_u32, slottable);
    assert_eq!(target, 100);
    assert_eq!(content, "Inner group content");
    assert_eq!(projected_root.role, "radiogroup");

    let item_attrs = group.item_attributes("item-val", None, false, Some("item-val"));
    let item_slottable = Slottable::new("Item label");
    let (target_item, projected_item, item_content) =
        project_attrs_as_child(&item_attrs, 200_u32, item_slottable);
    assert_eq!(target_item, 200);
    assert_eq!(item_content, "Item label");
    assert_eq!(projected_item.data_value, "item-val");
    assert_eq!(projected_item.tabindex, 0);
}

#[component]
fn DynamicItemLifecycleApp(show_first: std::rc::Rc<std::cell::Cell<bool>>) -> Element {
    rsx! {
        ToggleGroupRoot {
            r#type: ToggleGroupMode::Single,
            if show_first.get() {
                ToggleGroupItem { value: "item-1".to_string(), "Item 1" }
            }
            ToggleGroupItem { value: "item-2".to_string(), "Item 2" }
        }
    }
}

#[test]
fn test_dynamic_item_conditional_mount_vdom() {
    let show_first = std::rc::Rc::new(std::cell::Cell::new(true));
    let mut dom = VirtualDom::new_with_props(
        DynamicItemLifecycleApp,
        DynamicItemLifecycleAppProps {
            show_first: show_first.clone(),
        },
    );
    dom.rebuild_in_place();

    // Re-render with first item unmounted
    show_first.set(false);
    dom.mark_dirty(dioxus::core::ScopeId::ROOT);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
}
