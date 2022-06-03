#![doc(hidden)]
//! This module is for internal usage by the rusty_rpc_macro crate only.
//!
//! Contains various exports that macros need access to.

pub use crate::messages::{
    ClientMessage, MethodArgs, MethodId, ReturnValue, ServerMessage, ServiceId, ServiceRef,
};
pub use crate::service_collection::ServiceCollection;
pub use crate::traits::{
    ClientStreamSink, RustyRpcServiceClient, RustyRpcServiceProxy, RustyRpcServiceServer,
    RustyRpcStruct,
};
pub use crate::util::string_io_error;

pub use async_trait::async_trait;
pub use bytes::Bytes;
pub use futures::{SinkExt, StreamExt};
pub use rmp_serde;
pub use serde::{Deserialize, Serialize};
pub use tokio::sync::Mutex;
