use dioxus::prelude::*;

use crate::{
    foundation::{
        roving_focus::{RovingFocusRuntime, use_roving_focus_group},
        shared::{Direction, Orientation, ScopeHandle},
    },
    toggle_group::{
        ToggleGroup, ToggleGroupMode, ToggleGroupMultipleChangeHandler, ToggleGroupOrientation,
        ToggleGroupSingleChangeHandler,
    },
};

use super::{
    attrs::{ToolbarRootAttributes, ToolbarToggleGroupAttributes, ToolbarToggleItemAttributes},
    state::Toolbar,
};

#[derive(Clone)]
pub struct ToolbarRuntime {
    pub roving: RovingFocusRuntime,
    pub orientation: Signal<Orientation>,
    pub dir: Signal<Direction>,
    pub loop_focus: Signal<bool>,
    pub root_id: Signal<String>,
    pub root_dom_id: Signal<String>,
}

pub fn use_toolbar_runtime(toolbar: Toolbar, root_dom_id: String) -> ToolbarRuntime {
    let roving = use_roving_focus_group(toolbar.roving_config(), None, root_dom_id);
    ToolbarRuntime {
        roving,
        orientation: use_signal(|| toolbar.normalized_orientation()),
        dir: use_signal(|| toolbar.dir()),
        loop_focus: use_signal(|| toolbar.loop_focus()),
        root_id: use_signal(|| toolbar.relationships().root_id().to_owned()),
        root_dom_id: roving.root_dom_id,
    }
}

impl ToolbarRuntime {
    pub fn state(&self) -> Toolbar {
        let scope = ScopeHandle::root("toolbar").child((self.root_id)());
        Toolbar::new(scope)
            .with_orientation((self.orientation)())
            .with_direction((self.dir)())
            .with_loop_focus((self.loop_focus)())
    }

    pub fn sync_props(&self, orientation: Orientation, dir: Direction, loop_focus: bool, root_dom_id: &str) {
        let normalized = match orientation {
            Orientation::Vertical => Orientation::Vertical,
            Orientation::Horizontal | Orientation::Both => Orientation::Horizontal,
        };
        let mut ori_sig = self.orientation;
        if *ori_sig.peek() != normalized {
            ori_sig.set(normalized);
        }
        let mut dir_sig = self.dir;
        if *dir_sig.peek() != dir {
            dir_sig.set(dir);
        }
        let mut loop_sig = self.loop_focus;
        if *loop_sig.peek() != loop_focus {
            loop_sig.set(loop_focus);
        }
        self.roving.set_root_dom_id(root_dom_id);
        self.roving.set_config(
            self.state().roving_config(),
        );
    }

    pub fn toggle_group_orientation(&self) -> ToggleGroupOrientation {
        self.state().toggle_group_orientation()
    }

    pub fn root_attributes(&self) -> ToolbarRootAttributes {
        self.state().root_attributes()
    }
}

#[derive(Clone)]
pub struct ToolbarToggleGroupRuntime {
    pub mode: Signal<ToggleGroupMode>,
    pub single_value: Signal<String>,
    pub multiple_values: Signal<Vec<String>>,
    pub single_controlled: Signal<bool>,
    pub multiple_controlled: Signal<bool>,
    pub disabled: Signal<bool>,
    pub group_id: Signal<String>,
    pub group_dom_id: Signal<String>,
    pub on_single_change: Option<ToggleGroupSingleChangeHandler>,
    pub on_multiple_change: Option<ToggleGroupMultipleChangeHandler>,
}

impl ToolbarToggleGroupRuntime {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        mode: ToggleGroupMode,
        initial_single: String,
        initial_multiple: Vec<String>,
        single_controlled: bool,
        multiple_controlled: bool,
        disabled: bool,
        group_id: String,
        group_dom_id: String,
        on_single_change: Option<ToggleGroupSingleChangeHandler>,
        on_multiple_change: Option<ToggleGroupMultipleChangeHandler>,
    ) -> Self {
        Self {
            mode: use_signal(|| mode),
            single_value: use_signal(|| initial_single),
            multiple_values: use_signal(|| initial_multiple),
            single_controlled: use_signal(|| single_controlled),
            multiple_controlled: use_signal(|| multiple_controlled),
            disabled: use_signal(|| disabled),
            group_id: use_signal(|| group_id),
            group_dom_id: use_signal(|| group_dom_id),
            on_single_change,
            on_multiple_change,
        }
    }

    pub fn group_dom_id(&self) -> String {
        (self.group_dom_id)()
    }

    pub fn state(&self, orientation: ToggleGroupOrientation, dir: Direction) -> ToggleGroup {
        let scope = ScopeHandle::root("toolbar-toggle-group").child((self.group_id)());
        ToggleGroup::new(scope, (self.mode)())
            .with_orientation(orientation)
            .with_direction(dir)
            .with_roving_focus(false)
            .with_loop_focus(false)
            .with_disabled((self.disabled)())
            .with_single_value((self.single_value)())
            .with_multiple_values((self.multiple_values)())
    }

    pub fn is_item_pressed(&self, value: &str) -> bool {
        match (self.mode)() {
            ToggleGroupMode::Single => {
                let current = (self.single_value)();
                !current.is_empty() && current == value
            }
            ToggleGroupMode::Multiple => (self.multiple_values)().iter().any(|v| v == value),
        }
    }

    pub fn is_effective_disabled(&self, item_disabled: bool) -> bool {
        (self.disabled)() || item_disabled
    }

    pub fn toggle_item(&self, value: &str) {
        match (self.mode)() {
            ToggleGroupMode::Single => {
                let current = (self.single_value)();
                let next = if current == value {
                    String::new()
                } else {
                    value.to_string()
                };
                if !*(self.single_controlled).peek() {
                    let mut sig = self.single_value;
                    sig.set(next.clone());
                }
                if let Some(ref handler) = self.on_single_change {
                    handler(next);
                }
            }
            ToggleGroupMode::Multiple => {
                let mut list = (self.multiple_values)();
                if let Some(pos) = list.iter().position(|v| v == value) {
                    list.remove(pos);
                } else {
                    list.push(value.to_string());
                }
                if !*(self.multiple_controlled).peek() {
                    let mut sig = self.multiple_values;
                    sig.set(list.clone());
                }
                if let Some(ref handler) = self.on_multiple_change {
                    handler(list);
                }
            }
        }
    }

    pub fn root_attributes(
        &self,
        toolbar: &Toolbar,
        custom_role: Option<&'static str>,
    ) -> ToolbarToggleGroupAttributes {
        let tg = self.state(toolbar.toggle_group_orientation(), toolbar.dir());
        toolbar.toggle_group_attributes(&tg, custom_role)
    }

    pub fn item_attributes(
        &self,
        toolbar: &Toolbar,
        value: &str,
        item_id: Option<String>,
        item_disabled: bool,
        tabindex: i32,
    ) -> ToolbarToggleItemAttributes {
        let tg = self.state(toolbar.toggle_group_orientation(), toolbar.dir());
        toolbar.toggle_item_attributes(&tg, value, item_id, item_disabled, tabindex)
    }
}
