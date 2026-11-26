use super::types::AvatarImageLoadingStatus;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AvatarRootAttributes {
    pub data_status: &'static str,
    pub data_avatar_root: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AvatarImageAttributes {
    pub data_status: &'static str,
    pub data_avatar_image: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AvatarFallbackAttributes {
    pub data_status: &'static str,
    pub data_avatar_fallback: &'static str,
}

impl AvatarRootAttributes {
    pub fn new(status: AvatarImageLoadingStatus) -> Self {
        Self {
            data_status: status.as_str(),
            data_avatar_root: "",
        }
    }
}

impl AvatarImageAttributes {
    pub fn new(status: AvatarImageLoadingStatus) -> Self {
        Self {
            data_status: status.as_str(),
            data_avatar_image: "",
        }
    }
}

impl AvatarFallbackAttributes {
    pub fn new(status: AvatarImageLoadingStatus) -> Self {
        Self {
            data_status: status.as_str(),
            data_avatar_fallback: "",
        }
    }
}

