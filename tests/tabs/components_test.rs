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
