use super::runtime::SelectRuntime;

pub use crate::compat::select::types::*;

#[derive(Clone)]
pub struct SelectContext {
    pub runtime: SelectRuntime,
    pub name: Option<String>,
}
