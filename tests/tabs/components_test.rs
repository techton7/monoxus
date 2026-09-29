use dioxus::prelude::*;
use monoxus::tabs::{TabsContent, TabsList, TabsRoot, TabsTrigger};

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
