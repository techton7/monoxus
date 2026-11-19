use crate::foundation::shared::ScopeHandle;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AvatarRelationships {
    root_id: String,
    image_id: String,
    fallback_id: String,
}

impl AvatarRelationships {
    pub fn new(scope: &ScopeHandle) -> Self {
        let root_id = format!("{}-root", scope.token());
        Self {
            image_id: format!("{root_id}-image"),
            fallback_id: format!("{root_id}-fallback"),
            root_id,
        }
    }

    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    pub fn image_id(&self) -> &str {
        &self.image_id
    }

    pub fn fallback_id(&self) -> &str {
        &self.fallback_id
    }
}
