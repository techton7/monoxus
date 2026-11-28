//! Zero-bloat Dioxus agent runtime, declarative [`AgentBoundary`] scoping,
//! [`AgentBridgeHandle`] execution bridge, and `wasm32` WebMCP adapter.

pub mod boundary;
pub mod runtime;
pub mod webmcp;

pub use ui_sdk::agent::*;

pub use self::{boundary::*, runtime::*, webmcp::*};
