pub use monoxus_core::avatar::types::*;

pub fn image_load_status_to_avatar(
    status: oxidase::dom::observer::ImageLoadStatus,
) -> AvatarImageLoadingStatus {
    match status {
        oxidase::dom::observer::ImageLoadStatus::Loading => AvatarImageLoadingStatus::Loading,
        oxidase::dom::observer::ImageLoadStatus::Loaded => AvatarImageLoadingStatus::Loaded,
        oxidase::dom::observer::ImageLoadStatus::Error => AvatarImageLoadingStatus::Error,
    }
}
