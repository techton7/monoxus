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
