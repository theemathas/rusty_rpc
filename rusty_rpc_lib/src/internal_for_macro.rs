#![doc(hidden)]
//! This module is for internal usage by the rusty_rpc_macro crate only.
//!
//! Contains various exports that macros need access to.

pub use crate::messages::{
    service_ref_from_service_proxy, MethodAndArgs, ServerMessage, ServiceRef,
};
pub use crate::service_collection::ServiceCollection;
pub use crate::traits::{
    RustyRpcServiceClient, RustyRpcServiceProxy, RustyRpcServiceServer, RustyRpcStruct,
};
