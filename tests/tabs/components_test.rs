use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;
use monoxus::tabs::{TabsActivationMode, TabsContent, TabsContext, TabsList, TabsRoot, TabsTrigger};

#[component]
fn TabsCompoundSample() -> Element {
    rsx! {
        TabsRoot {
            default_value: "tab1".to_string(),
            TabsList {
                TabsTrigger { value: "tab1".to_string(), "Tab 1" }
                TabsTrigger { value: "tab2".to_string(), "Tab 2" }
            }
            TabsContent { value: "tab1".to_string(), "Panel 1" }
            TabsContent { value: "tab2".to_string(), "Panel 2" }
        }
    }
}

#[test]
fn test_tabs_compound_components_render() {
    let mut dom = VirtualDom::new(TabsCompoundSample);
    dom.rebuild_in_place();
}

#[component]
fn TabsManualModeSample(active_tab: Rc<RefCell<String>>) -> Element {
    let active_tab_clone = active_tab.clone();
    rsx! {
        TabsRoot {
            default_value: "tab1".to_string(),
            activation_mode: TabsActivationMode::Manual,
            TabsList {
                TabsTrigger { id: "trig-1".to_string(), value: "tab1".to_string(), "Tab 1" }
                TabsTrigger { id: "trig-2".to_string(), value: "tab2".to_string(), "Tab 2" }
                TabsTrigger { id: "trig-disabled".to_string(), value: "tab3".to_string(), disabled: true, "Tab 3 Disabled" }
            }
            TabsContent {
                value: "tab1".to_string(),
                input { id: "unguarded-input", r#type: "text" }
            }
            TabsContent { value: "tab2".to_string(), "Panel 2" }
            TabsContent { value: "tab3".to_string(), "Panel 3" }
            TabsActiveProbe { active_tab: active_tab_clone }
        }
    }
}

#[component]
fn TabsActiveProbe(active_tab: Rc<RefCell<String>>) -> Element {
    let ctx = use_context::<TabsContext>();
    *active_tab.borrow_mut() = ctx.runtime.tabs().active_value().to_string();
    rsx! {}
}

#[test]
fn test_tabs_trigger_cascaded_disabled_and_keyboard_activation() {
    let active = Rc::new(RefCell::new(String::new()));
    let active_capture = active.clone();
    let mut dom = VirtualDom::new_with_props(TabsManualModeSample, TabsManualModeSampleProps {
        active_tab: active_capture,
    });
    dom.rebuild_in_place();

    assert_eq!(*active.borrow(), "tab1");
}

#[cfg(feature = "native")]
#[test]
fn native_tabs_post_mount_props_and_custom_content_id_aria_controls() {
    use crate::common::native_fixture::{
        NATIVE_TEST_LOCK, assert_native_focus_id, click_and_pump, focus_node,
        mount_native_scenario, send_key_and_pump,
    };
    use keyboard_types::{Code, Key, NamedKey};
    use monoxus::tabs::TabsOrientation;

    #[component]
    fn PostMountTabsScenario() -> Element {
        let mut orientation = use_signal(|| TabsOrientation::Horizontal);
        let mut activation_mode = use_signal(|| TabsActivationMode::Automatic);

        rsx! {
            div {
                button {
                    id: "tabs-set-vertical",
                    style: "width: 120px; height: 24px; display: block;",
                    onclick: move |_| orientation.set(TabsOrientation::Vertical),
                    "Vertical"
                }
                button {
                    id: "tabs-set-manual",
                    style: "width: 120px; height: 24px; display: block;",
                    onclick: move |_| activation_mode.set(TabsActivationMode::Manual),
                    "Manual"
                }
                TabsRoot {
                    id: "fm-w2-tabs-root",
                    class: "fm-w2-tabs-root".to_string(),
                    default_value: "tab1".to_string(),
                    orientation: orientation(),
                    activation_mode: activation_mode(),
                    TabsList {
                        id: "fm-w2-tabs-list",
                        TabsTrigger {
                            id: "fm-w2-tab-1",
                            value: "tab1".to_string(),
                            style: "width: 80px; height: 24px;",
                            "Tab 1"
                        }
                        TabsTrigger {
                            id: "fm-w2-tab-2",
                            value: "tab2".to_string(),
                            style: "width: 80px; height: 24px;",
                            "Tab 2"
                        }
                    }
                    TabsContent {
                        id: "custom-panel".to_string(),
                        value: "tab1".to_string(),
                        "Panel 1"
                    }
                    TabsContent {
                        id: "custom-panel-2".to_string(),
                        value: "tab2".to_string(),
                        "Panel 2"
                    }
                }
            }
        }
    }

    let _lock = NATIVE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut harness = mount_native_scenario(PostMountTabsScenario);

    // H-9b: custom TabsContent id must be published in TabsTrigger aria-controls.
    assert_eq!(
        harness.attr("#fm-w2-tab-1", "aria-controls").as_deref(),
        Some("custom-panel"),
        "FM-W2-TAB: TabsTrigger aria-controls must match custom TabsContent id"
    );
    assert_eq!(
        harness.attr("#fm-w2-tabs-list", "aria-orientation").as_deref(),
        Some("horizontal")
    );

    // H-9a: Post-mount orientation mutation propagates to TabsList, TabsTrigger, TabsContent, and roving focus.
    click_and_pump(&mut harness, "#tabs-set-vertical");
    assert_eq!(
        harness.attr(".fm-w2-tabs-root", "data-orientation").as_deref(),
        Some("vertical")
    );
    assert_eq!(
        harness.attr("#fm-w2-tabs-list", "aria-orientation").as_deref(),
        Some("vertical"),
        "FM-W2-TAB: post-mount orientation update must reach TabsList"
    );
    assert_eq!(
        harness.attr("#fm-w2-tab-1", "data-orientation").as_deref(),
        Some("vertical"),
        "FM-W2-TAB: post-mount orientation update must reach TabsTrigger"
    );
    assert_eq!(
        harness.attr("#custom-panel", "data-orientation").as_deref(),
        Some("vertical"),
        "FM-W2-TAB: post-mount orientation update must reach TabsContent"
    );

    // H-9a: Post-mount activation_mode mutation switches to Manual without clobbering active tab.
    click_and_pump(&mut harness, "#tabs-set-manual");
    focus_node(&mut harness, "#fm-w2-tab-1");
    send_key_and_pump(&mut harness, Key::Named(NamedKey::ArrowDown), Code::ArrowDown);

    assert_native_focus_id(&harness, "#fm-w2-tab-2");
    assert_eq!(
        harness.attr("#fm-w2-tab-1", "aria-selected").as_deref(),
        Some("true"),
        "FM-W2-TAB: after switching to Manual mode post-mount, ArrowDown must move focus without selecting tab2"
    );
    assert_eq!(
        harness.attr("#fm-w2-tab-2", "aria-selected").as_deref(),
        Some("false")
    );

    // Pressing Enter activates focused tab2 and links aria-controls to custom-panel-2.
    send_key_and_pump(&mut harness, Key::Named(NamedKey::Enter), Code::Enter);
    assert_eq!(
        harness.attr("#fm-w2-tab-2", "aria-selected").as_deref(),
        Some("true")
    );
    assert_eq!(
        harness.attr("#fm-w2-tab-2", "aria-controls").as_deref(),
        Some("custom-panel-2"),
        "FM-W2-TAB: activated tab2 trigger must point aria-controls to custom-panel-2"
    );
}
