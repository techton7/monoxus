//! [KEEP-MONOXUS-BRAND] Monoxus/Radix aspect-ratio wrapper attribute shim.
//! Thin compatibility shim over [`ui_sdk::aspect_ratio`] preserving the legacy
//! `data_radix_aspect_ratio_wrapper` attribute field during Strangler Fig waves.

use ui_sdk::foundation::compose::{Slottable, project_attrs_as_child};

pub use ui_sdk::aspect_ratio::{
    ASPECT_RATIO_ROOT_STYLE, ASPECT_RATIO_WRAPPER_STYLE, normalize_aspect_ratio, root_style,
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct AspectRatio(ui_sdk::aspect_ratio::AspectRatio);

impl Default for AspectRatio {
    fn default() -> Self {
        Self(ui_sdk::aspect_ratio::AspectRatio::default())
    }
}

impl AspectRatio {
    pub const fn new(ratio: f64) -> Self {
        Self(ui_sdk::aspect_ratio::AspectRatio::new(ratio))
    }

    pub const fn ratio(&self) -> f64 {
        self.0.ratio()
    }

    pub fn padding_bottom_percent(&self) -> f64 {
        self.0.padding_bottom_percent()
    }

    pub fn attributes(&self) -> AspectRatioAttributes {
        let base = self.0.attributes();
        AspectRatioAttributes {
            wrapper_style: base.wrapper_style,
            root_style: base.root_style,
            data_radix_aspect_ratio_wrapper: "",
            data_aspect_ratio_wrapper: base.data_aspect_ratio_wrapper,
            data_aspect_ratio_root: base.data_aspect_ratio_root,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[derive(serde::Serialize)]
pub struct AspectRatioAttributes {
    pub wrapper_style: String,
    pub root_style: String,
    pub data_radix_aspect_ratio_wrapper: &'static str,
    pub data_aspect_ratio_wrapper: &'static str,
    pub data_aspect_ratio_root: &'static str,
}

pub fn render_aspect_ratio_child<T, C, R>(
    state: &AspectRatio,
    target: T,
    content: Slottable<C>,
    render_child: impl FnOnce(T, AspectRatioAttributes, C) -> R,
) -> R {
    let (target, attributes, content) = project_attrs_as_child(&state.attributes(), target, content);
    render_child(target, attributes, content)
}
