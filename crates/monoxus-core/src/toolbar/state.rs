use crate::{
    foundation::{
        roving_focus::RovingFocusConfig,
        shared::{Direction, Orientation, ScopeHandle, data_disabled_attr},
    },
    separator::{Separator, SeparatorOrientation},
    toggle_group::{ToggleGroup, ToggleGroupOrientation},
};

use super::{
    attrs::{
        ToolbarButtonAttributes, ToolbarLinkAttributes, ToolbarRootAttributes,
        ToolbarSeparatorAttributes, ToolbarToggleGroupAttributes, ToolbarToggleItemAttributes,
    },
    relationships::ToolbarRelationships,
    types::{TOOLBAR_PARTS, ToolbarPart, toolbar_orientation_as_str},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Toolbar {
    relationships: ToolbarRelationships,
    orientation: Orientation,
    dir: Direction,
    loop_focus: bool,
}

impl Toolbar {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            relationships: ToolbarRelationships::new(scope),
            orientation: Orientation::Horizontal,
            dir: Direction::Ltr,
            loop_focus: true,
        }
    }

    pub const fn parts() -> &'static [ToolbarPart] {
        &TOOLBAR_PARTS
    }

    pub fn relationships(&self) -> &ToolbarRelationships {
        &self.relationships
    }

    pub const fn orientation(&self) -> Orientation {
        self.orientation
    }

    pub const fn dir(&self) -> Direction {
        self.dir
    }

    pub const fn loop_focus(&self) -> bool {
        self.loop_focus
    }

    pub const fn with_orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub const fn with_direction(mut self, dir: Direction) -> Self {
        self.dir = dir;
        self
    }

    pub const fn with_loop_focus(mut self, loop_focus: bool) -> Self {
        self.loop_focus = loop_focus;
        self
    }

    pub const fn normalized_orientation(&self) -> Orientation {
        match self.orientation {
            Orientation::Vertical => Orientation::Vertical,
            Orientation::Horizontal | Orientation::Both => Orientation::Horizontal,
        }
    }

    pub const fn roving_config(&self) -> RovingFocusConfig {
        RovingFocusConfig::new()
            .with_orientation(self.normalized_orientation())
            .with_direction(self.dir)
            .with_loop_focus(self.loop_focus)
            .with_enabled(true)
    }

    pub const fn inverted_separator_orientation(&self) -> SeparatorOrientation {
        match self.normalized_orientation() {
            Orientation::Horizontal | Orientation::Both => SeparatorOrientation::Vertical,
            Orientation::Vertical => SeparatorOrientation::Horizontal,
        }
    }

    pub const fn toggle_group_orientation(&self) -> ToggleGroupOrientation {
        match self.normalized_orientation() {
            Orientation::Vertical => ToggleGroupOrientation::Vertical,
            Orientation::Horizontal | Orientation::Both => ToggleGroupOrientation::Horizontal,
        }
    }

    pub const fn separator_state(&self, decorative: bool) -> Separator {
        Separator::new()
            .with_orientation(self.inverted_separator_orientation())
            .with_decorative(decorative)
    }

    /// Evaluates whether a keydown on `ToolbarLink` should synthetically activate the link.
    ///
    /// Enforces the `#3232` guard: only activates on `Space` when the link itself (not a nested
    /// descendant such as an `<input>`) holds active focus.
    pub fn should_activate_link_on_key(key: &str, link_is_focused_target: bool) -> bool {
        matches!(key, " " | "Spacebar") && link_is_focused_target
    }

    pub fn root_attributes(&self) -> ToolbarRootAttributes {
        let orientation_str = toolbar_orientation_as_str(self.normalized_orientation());
        ToolbarRootAttributes {
            id: self.relationships.root_id().to_owned(),
            role: "toolbar",
            aria_orientation: orientation_str,
            dir: self.dir.as_str(),
            data_orientation: orientation_str,
            data_toolbar_root: "",
        }
    }

    pub fn button_attributes(
        &self,
        id: String,
        disabled: bool,
        tabindex: i32,
    ) -> ToolbarButtonAttributes {
        ToolbarButtonAttributes {
            id,
            r#type: "button",
            tabindex: if disabled { -1 } else { tabindex },
            disabled,
            data_disabled: data_disabled_attr(disabled),
            data_orientation: toolbar_orientation_as_str(self.normalized_orientation()),
            data_toolbar_button: "",
            data_toolbar_item: "",
        }
    }

    pub fn link_attributes(
        &self,
        id: String,
        href: Option<String>,
        tabindex: i32,
    ) -> ToolbarLinkAttributes {
        ToolbarLinkAttributes {
            id,
            href,
            tabindex,
            data_orientation: toolbar_orientation_as_str(self.normalized_orientation()),
            data_toolbar_link: "",
            data_toolbar_item: "",
        }
    }

    pub fn separator_attributes(
        &self,
        id: Option<String>,
        decorative: bool,
    ) -> ToolbarSeparatorAttributes {
        ToolbarSeparatorAttributes {
            id,
            separator: self.separator_state(decorative).attributes(),
            data_toolbar_separator: "",
        }
    }

    pub fn toggle_group_attributes(
        &self,
        toggle_group: &ToggleGroup,
        custom_role: Option<&'static str>,
    ) -> ToolbarToggleGroupAttributes {
        let base = toggle_group.root_attributes(custom_role);
        ToolbarToggleGroupAttributes {
            id: base.id,
            role: base.role,
            dir: base.dir,
            data_orientation: base.data_orientation,
            data_disabled: base.data_disabled,
            data_toolbar_group: "",
        }
    }

    pub fn toggle_item_attributes(
        &self,
        toggle_group: &ToggleGroup,
        value: &str,
        item_id: Option<String>,
        item_disabled: bool,
        tabindex: i32,
    ) -> ToolbarToggleItemAttributes {
        let base = toggle_group.item_attributes(value, item_id, item_disabled, None);
        ToolbarToggleItemAttributes {
            id: base.id,
            r#type: "button",
            role: base.role,
            aria_checked: base.aria_checked,
            aria_pressed: base.aria_pressed,
            tabindex: if base.disabled { -1 } else { tabindex },
            disabled: base.disabled,
            data_disabled: base.data_disabled,
            data_state: base.data_state,
            data_value: base.data_value,
            data_orientation: toolbar_orientation_as_str(self.normalized_orientation()),
            data_toolbar_group_item: "",
            data_toolbar_item: "",
        }
    }
}
