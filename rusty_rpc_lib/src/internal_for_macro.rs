#![doc(hidden)]
//! This module is for internal usage by the rusty_rpc_macro crate only.
//!
//! Contains various exports that macros need access to.

pub use crate::messages::{
    response_from_service_id, MethodAndArgs, ServerMessage, ServerResponse, ServerResult,
};
pub use crate::service_collection::ServiceCollection;
pub use crate::traits::{RustyRpcService, RustyRpcStruct};
