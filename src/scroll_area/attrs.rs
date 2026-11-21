use crate::foundation::shared::{Direction, Orientation, data_disabled_attr};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScrollAreaRootAttributes {
    pub(crate) id: String,
    pub(crate) dir: Option<Direction>,
    pub(crate) disabled: bool,
}

impl ScrollAreaRootAttributes {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            dir: None,
            disabled: false,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn dir(&self) -> Option<Direction> {
        self.dir
    }

    pub fn dir_str(&self) -> Option<&'static str> {
        self.dir.map(|d| match d {
            Direction::Ltr => "ltr",
            Direction::Rtl => "rtl",
        })
    }

    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub const fn with_dir(mut self, dir: Option<Direction>) -> Self {
        self.dir = dir;
        self
    }

    pub const fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub const fn data_disabled(&self) -> Option<&'static str> {
        data_disabled_attr(self.disabled)
    }

    pub const fn data_monoxus_scroll_area_root(&self) -> &'static str {
        ""
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScrollAreaViewportAttributes {
    pub(crate) id: String,
    pub(crate) disabled: bool,
}

impl ScrollAreaViewportAttributes {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            disabled: false,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn tabindex(&self) -> i32 {
        0
    }

    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub const fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub const fn data_disabled(&self) -> Option<&'static str> {
        data_disabled_attr(self.disabled)
    }

    pub const fn data_monoxus_scroll_area_viewport(&self) -> &'static str {
        ""
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScrollAreaContentAttributes {
    pub(crate) id: String,
}

impl ScrollAreaContentAttributes {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn role(&self) -> &'static str {
        "presentation"
    }

    pub const fn data_monoxus_scroll_area_content(&self) -> &'static str {
        ""
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollAreaScrollbarAttributes {
    pub(crate) id: String,
    pub(crate) orientation: Orientation,
    pub(crate) aria_controls: String,
    pub(crate) aria_valuenow: Option<f64>,
    pub(crate) aria_valuemin: Option<f64>,
    pub(crate) aria_valuemax: Option<f64>,
    pub(crate) visible: bool,
    pub(crate) disabled: bool,
}

impl ScrollAreaScrollbarAttributes {
    pub fn new(
        id: impl Into<String>,
        orientation: Orientation,
        aria_controls: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            orientation,
            aria_controls: aria_controls.into(),
            aria_valuenow: None,
            aria_valuemin: None,
            aria_valuemax: None,
            visible: true,
            disabled: false,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn role(&self) -> &'static str {
        "scrollbar"
    }

    pub const fn orientation(&self) -> Orientation {
        self.orientation
    }

    pub fn aria_controls(&self) -> &str {
        &self.aria_controls
    }

    pub const fn aria_orientation(&self) -> &'static str {
        match self.orientation {
            Orientation::Horizontal => "horizontal",
            Orientation::Vertical | Orientation::Both => "vertical",
        }
    }

    pub const fn aria_valuenow(&self) -> Option<f64> {
        self.aria_valuenow
    }

    pub const fn aria_valuemin(&self) -> Option<f64> {
        self.aria_valuemin
    }

    pub const fn aria_valuemax(&self) -> Option<f64> {
        self.aria_valuemax
    }

    pub const fn data_orientation(&self) -> &'static str {
        self.aria_orientation()
    }

    pub const fn data_state(&self) -> &'static str {
        if self.visible { "visible" } else { "hidden" }
    }

    pub const fn is_visible(&self) -> bool {
        self.visible
    }

    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub const fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub const fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub const fn with_values(
        mut self,
        now: Option<f64>,
        min: Option<f64>,
        max: Option<f64>,
    ) -> Self {
        self.aria_valuenow = now;
        self.aria_valuemin = min;
        self.aria_valuemax = max;
        self
    }

    pub const fn data_disabled(&self) -> Option<&'static str> {
        data_disabled_attr(self.disabled)
    }

    pub const fn data_monoxus_scroll_area_scrollbar(&self) -> &'static str {
        ""
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScrollAreaThumbAttributes {
    pub(crate) id: String,
    pub(crate) visible: bool,
    pub(crate) disabled: bool,
}

impl ScrollAreaThumbAttributes {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            visible: true,
            disabled: false,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn data_state(&self) -> &'static str {
        if self.visible { "visible" } else { "hidden" }
    }

    pub const fn is_visible(&self) -> bool {
        self.visible
    }

    pub const fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub const fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub const fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub const fn data_disabled(&self) -> Option<&'static str> {
        data_disabled_attr(self.disabled)
    }

    pub const fn data_monoxus_scroll_area_thumb(&self) -> &'static str {
        ""
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScrollAreaCornerAttributes {
    pub(crate) id: String,
}

impl ScrollAreaCornerAttributes {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub const fn data_state(&self) -> &'static str {
        "visible"
    }

    pub const fn data_monoxus_scroll_area_corner(&self) -> &'static str {
        ""
    }
}
