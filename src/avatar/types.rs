#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AvatarImageLoadingStatus {
    #[default]
    Idle,
    Loading,
    Loaded,
    Error,
}

impl AvatarImageLoadingStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Loading => "loading",
            Self::Loaded => "loaded",
            Self::Error => "error",
        }
    }

    pub const fn is_loaded(self) -> bool {
        matches!(self, Self::Loaded)
    }
}

pub const AVATAR_PARTS: [AvatarPart; 3] =
    [AvatarPart::Root, AvatarPart::Image, AvatarPart::Fallback];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AvatarPart {
    Root,
    Image,
    Fallback,
}

impl AvatarPart {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Image => "image",
            Self::Fallback => "fallback",
        }
    }
}

impl From<oxidase::dom::observer::ImageLoadStatus> for AvatarImageLoadingStatus {
    fn from(status: oxidase::dom::observer::ImageLoadStatus) -> Self {
        match status {
            oxidase::dom::observer::ImageLoadStatus::Loading => Self::Loading,
            oxidase::dom::observer::ImageLoadStatus::Loaded => Self::Loaded,
            oxidase::dom::observer::ImageLoadStatus::Error => Self::Error,
        }
    }
}
