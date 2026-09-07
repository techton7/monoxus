#![allow(dead_code)]

use monoxus::{
    foundation::shared::ScopeHandle,
    select::{Select, SelectItemData},
};

pub fn make_fruit_items() -> Vec<SelectItemData> {
    vec![
        SelectItemData::new("apple", "Apple", false),
        SelectItemData::new("banana", "Banana", false),
        SelectItemData::new("blueberry", "Blueberry", false),
        SelectItemData::new("grapes", "Grapes", true), // disabled
        SelectItemData::new("pineapple", "Pineapple", false),
    ]
}

pub fn make_test_select(name: &'static str) -> Select {
    let scope = ScopeHandle::root("select-test").child(name);
    Select::new(scope).with_items(make_fruit_items())
}
