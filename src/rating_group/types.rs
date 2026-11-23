use dioxus::prelude::Callback;

pub use monoxus_core::rating_group::types::*;

/// Custom `aria-valuetext` specification for [`crate::rating_group::RatingGroupRoot`].
#[derive(Clone, PartialEq)]
pub enum RatingGroupAriaValuetext {
    Static(String),
    Formatter(Callback<(f64, usize), String>),
}

impl From<String> for RatingGroupAriaValuetext {
    fn from(value: String) -> Self {
        Self::Static(value)
    }
}

impl From<&str> for RatingGroupAriaValuetext {
    fn from(value: &str) -> Self {
        Self::Static(value.to_string())
    }
}
