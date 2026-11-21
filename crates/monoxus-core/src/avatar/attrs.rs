use crate::foundation::compose::{Slottable, project_as_child};

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

pub fn project_avatar_root_as_child<T, C>(
    attrs: &AvatarRootAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, AvatarRootAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

pub fn project_avatar_image_as_child<T, C>(
    attrs: &AvatarImageAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, AvatarImageAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}

pub fn project_avatar_fallback_as_child<T, C>(
    attrs: &AvatarFallbackAttributes,
    target: T,
    content: Slottable<C>,
) -> (T, AvatarFallbackAttributes, C) {
    let (target, content) = project_as_child(target, content);
    (target, attrs.clone(), content)
}
