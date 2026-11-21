use crate::foundation::shared::{Orientation, ScopeHandle};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScrollAreaRelationships {
    scope: ScopeHandle,
    root_id: String,
    viewport_id: String,
    content_id: String,
    scrollbar_x_id: String,
    scrollbar_y_id: String,
    thumb_x_id: String,
    thumb_y_id: String,
    corner_id: String,
}

impl ScrollAreaRelationships {
    pub fn new(scope: ScopeHandle) -> Self {
        Self {
            root_id: scope.token(),
            viewport_id: scope.qualify("viewport"),
            content_id: scope.qualify("content"),
            scrollbar_x_id: scope.qualify("scrollbar-x"),
            scrollbar_y_id: scope.qualify("scrollbar-y"),
            thumb_x_id: scope.qualify("thumb-x"),
            thumb_y_id: scope.qualify("thumb-y"),
            corner_id: scope.qualify("corner"),
            scope,
        }
    }

    pub fn with_root_id(mut self, root_id: impl Into<String>) -> Self {
        self.root_id = root_id.into();
        self
    }

    pub fn with_viewport_id(mut self, viewport_id: impl Into<String>) -> Self {
        self.viewport_id = viewport_id.into();
        self
    }

    pub fn with_content_id(mut self, content_id: impl Into<String>) -> Self {
        self.content_id = content_id.into();
        self
    }

    pub fn with_scrollbar_x_id(mut self, scrollbar_x_id: impl Into<String>) -> Self {
        self.scrollbar_x_id = scrollbar_x_id.into();
        self
    }

    pub fn with_scrollbar_y_id(mut self, scrollbar_y_id: impl Into<String>) -> Self {
        self.scrollbar_y_id = scrollbar_y_id.into();
        self
    }

    pub fn with_thumb_x_id(mut self, thumb_x_id: impl Into<String>) -> Self {
        self.thumb_x_id = thumb_x_id.into();
        self
    }

    pub fn with_thumb_y_id(mut self, thumb_y_id: impl Into<String>) -> Self {
        self.thumb_y_id = thumb_y_id.into();
        self
    }

    pub fn with_corner_id(mut self, corner_id: impl Into<String>) -> Self {
        self.corner_id = corner_id.into();
        self
    }

    pub fn scope(&self) -> &ScopeHandle {
        &self.scope
    }

    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    pub fn viewport_id(&self) -> &str {
        &self.viewport_id
    }

    pub fn content_id(&self) -> &str {
        &self.content_id
    }

    pub fn scrollbar_x_id(&self) -> &str {
        &self.scrollbar_x_id
    }

    pub fn scrollbar_y_id(&self) -> &str {
        &self.scrollbar_y_id
    }

    pub fn thumb_x_id(&self) -> &str {
        &self.thumb_x_id
    }

    pub fn thumb_y_id(&self) -> &str {
        &self.thumb_y_id
    }

    pub fn corner_id(&self) -> &str {
        &self.corner_id
    }

    pub fn scrollbar_id(&self, orientation: Orientation) -> &str {
        match orientation {
            Orientation::Horizontal => &self.scrollbar_x_id,
            Orientation::Vertical | Orientation::Both => &self.scrollbar_y_id,
        }
    }

    pub fn thumb_id(&self, orientation: Orientation) -> &str {
        match orientation {
            Orientation::Horizontal => &self.thumb_x_id,
            Orientation::Vertical | Orientation::Both => &self.thumb_y_id,
        }
    }
}
