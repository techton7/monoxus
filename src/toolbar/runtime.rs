use dioxus::prelude::*;
use ui_sdk::kernel::{ChangeContext, ChangeEvent};

use crate::{
    foundation::{
        roving_focus::{RovingFocusRuntime, use_roving_focus_group},
        shared::{Direction, Orientation, ScopeHandle},
        state::{ChangeOutcome, sync_signal},
    },
    toggle_group::{
        ToggleGroup, ToggleGroupMode, ToggleGroupOrientation,
        runtime::{
            ToggleGroupRuntime, use_toggle_group_runtime, use_toggle_group_runtime_with_details,
        },
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

    pub fn sync_props(
        &self,
        orientation: Orientation,
        dir: Direction,
        loop_focus: bool,
        root_dom_id: &str,
    ) {
        let normalized = match orientation {
            Orientation::Vertical => Orientation::Vertical,
            Orientation::Horizontal | Orientation::Both => Orientation::Horizontal,
        };
        let mut ori_sig = self.orientation;
        sync_signal(&mut ori_sig, normalized);
        let mut dir_sig = self.dir;
        sync_signal(&mut dir_sig, dir);
        let mut loop_sig = self.loop_focus;
        sync_signal(&mut loop_sig, loop_focus);
        self.roving.set_root_dom_id(root_dom_id);
        self.roving.set_config(self.state().roving_config());
    }

    pub fn toggle_group_orientation(&self) -> ToggleGroupOrientation {
        self.state().toggle_group_orientation()
    }

    pub fn root_attributes(&self) -> ToolbarRootAttributes {
        self.state().root_attributes()
    }
}

/// Toolbar toggle-group runtime delegating value/mode state to [`ToggleGroupRuntime`] with
/// `roving_focus: false` so parent [`ToolbarRuntime::roving`] retains single-tab-stop ownership.
#[derive(Clone)]
pub struct ToolbarToggleGroupRuntime {
    pub inner: ToggleGroupRuntime,
    pub group_id: Signal<String>,
    pub group_dom_id: Signal<String>,
}

#[allow(clippy::too_many_arguments)]
pub fn use_toolbar_toggle_group_runtime(
    mode: ToggleGroupMode,
    value: Option<String>,
    default_value: Option<String>,
    on_value_change: Option<EventHandler<String>>,
    values: Option<Vec<String>>,
    default_values: Option<Vec<String>>,
    on_values_change: Option<EventHandler<Vec<String>>>,
    orientation: ToggleGroupOrientation,
    dir: Direction,
    disabled: bool,
    group_id: String,
    group_dom_id: String,
) -> ToolbarToggleGroupRuntime {
    let inner = use_toggle_group_runtime(
        mode,
        value,
        default_value,
        on_value_change,
        values,
        default_values,
        on_values_change,
        orientation,
        dir,
        false,
        false,
        disabled,
        group_id.clone(),
        group_dom_id.clone(),
    );

    let mut group_id_sig = use_signal(|| group_id.clone());
    sync_signal(&mut group_id_sig, group_id);
    let mut group_dom_id_sig = use_signal(|| group_dom_id.clone());
    sync_signal(&mut group_dom_id_sig, group_dom_id);

    ToolbarToggleGroupRuntime {
        inner,
        group_id: group_id_sig,
        group_dom_id: group_dom_id_sig,
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn use_toolbar_toggle_group_runtime_with_details(
    mode: ToggleGroupMode,
    value: Option<String>,
    default_value: Option<String>,
    on_value_change: Option<EventHandler<String>>,
    on_value_change_details: Option<EventHandler<ChangeEvent<String>>>,
    values: Option<Vec<String>>,
    default_values: Option<Vec<String>>,
    on_values_change: Option<EventHandler<Vec<String>>>,
    on_values_change_details: Option<EventHandler<ChangeEvent<Vec<String>>>>,
    orientation: ToggleGroupOrientation,
    dir: Direction,
    disabled: bool,
    group_id: String,
    group_dom_id: String,
) -> ToolbarToggleGroupRuntime {
    let inner = use_toggle_group_runtime_with_details(
        mode,
        value,
        default_value,
        on_value_change,
        on_value_change_details,
        values,
        default_values,
        on_values_change,
        on_values_change_details,
        orientation,
        dir,
        false,
        false,
        disabled,
        group_id.clone(),
        group_dom_id.clone(),
    );

    let mut group_id_sig = use_signal(|| group_id.clone());
    sync_signal(&mut group_id_sig, group_id);
    let mut group_dom_id_sig = use_signal(|| group_dom_id.clone());
    sync_signal(&mut group_dom_id_sig, group_dom_id);

    ToolbarToggleGroupRuntime {
        inner,
        group_id: group_id_sig,
        group_dom_id: group_dom_id_sig,
    }
}

impl ToolbarToggleGroupRuntime {
    pub fn group_dom_id(&self) -> String {
        (self.group_dom_id)()
    }

    pub fn state(&self, orientation: ToggleGroupOrientation, dir: Direction) -> ToggleGroup {
        let scope = ScopeHandle::root("toolbar-toggle-group").child((self.group_id)());
        ToggleGroup::new(scope, (self.inner.mode)())
            .with_orientation(orientation)
            .with_direction(dir)
            .with_roving_focus(false)
            .with_loop_focus(false)
            .with_disabled((self.inner.disabled)())
            .with_single_value((self.inner.single_value)())
            .with_multiple_values((self.inner.multiple_values)())
    }

    pub fn is_item_pressed(&self, value: &str) -> bool {
        self.inner.is_item_pressed(value)
    }

    pub fn is_effective_disabled(&self, item_disabled: bool) -> bool {
        self.inner.is_effective_disabled(item_disabled)
    }

    pub fn toggle_item(&self, value: &str) {
        self.inner.toggle_item(value);
    }

    pub fn toggle_item_with_context(&self, value: &str, ctx: ChangeContext) -> ChangeOutcome {
        self.inner.toggle_item_with_context(value, ctx)
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
