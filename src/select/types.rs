use super::runtime::SelectRuntime;

pub use monoxus_core::select::types::*;

#[derive(Clone)]
pub struct SelectContext {
    pub runtime: SelectRuntime,
    pub name: Option<String>,
}
