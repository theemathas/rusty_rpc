use std::ops::Deref;

use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::RustyRpcServiceClient;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServiceId(pub u64);
impl ServiceId {
    pub fn increment(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }
}

/// The message that the server responds to the client, giving back the RPC return value.
#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    DropServiceDone,
    MethodReturned(ReturnValue),
}
impl TryFrom<Bytes> for ServerMessage {
    type Error = rmp_serde::decode::Error;
    fn try_from(bytes: Bytes) -> Result<ServerMessage, Self::Error> {
        rmp_serde::decode::from_slice(&bytes)
    }
}
impl From<ServerMessage> for Bytes {
    fn from(msg: ServerMessage) -> Bytes {
        rmp_serde::to_vec(&msg)
            .expect("Serialization of ServerMessage somehow failed.")
            .into()
    }
}

/// Represents the return value of an RPC call, as written on the wire.
#[derive(Serialize, Deserialize)]
pub struct ReturnValue {
    // TODO
}

/// The message that the client sends to the server in order to call an RPC.
#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    DropService(ServiceId),
    CallMethod(ServiceId, MethodAndArgs),
}
impl TryFrom<Bytes> for ClientMessage {
    type Error = rmp_serde::decode::Error;

    fn try_from(bytes: Bytes) -> Result<ClientMessage, Self::Error> {
        rmp_serde::decode::from_slice(&bytes)
    }
}
impl From<ClientMessage> for Bytes {
    fn from(msg: ClientMessage) -> Bytes {
        rmp_serde::to_vec(&msg)
            .expect("Serialization of ClientMessage somehow failed.")
            .into()
    }
}

/// Represents the data used to specify the method and arguments for a given RPC
/// call, as written on the wire.
#[derive(Serialize, Deserialize)]
pub struct MethodAndArgs {
    // TODO
}

enum InnerServiceRef<T: RustyRpcServiceClient + ?Sized> {
    RemoteServiceRef(T::ServiceProxy),
    OwnedLocalService(Box<T>),
}

/// Either an owned server-side service, or a client's reference to such a
/// service. (If the fromer, cannot be dereferenced. If the latter, acts like
/// `&T` and can be dereferenced.)
///
/// When all ServiceRef's for a certain service is dropped on the client side,
/// the associated resources, are dropped on the server side. If the type `T` is
/// an implementation of a certain service, then `Response<T>` will implement
/// the corresponding service trait.
///
/// The type `T` should be something like `dyn MyService` (bare unsized dyn
/// trait).
pub struct ServiceRef<T: RustyRpcServiceClient + ?Sized>(
    /// Do enum inside struct to get private enum variants.
    InnerServiceRef<T>,
);
impl<T: RustyRpcServiceClient + ?Sized> ServiceRef<T> {
    /// Used on the server side.
    pub fn new(inner: Box<T>) -> Self {
        ServiceRef(InnerServiceRef::OwnedLocalService(inner))
    }
}
/// Used only on the client side.
impl<T: RustyRpcServiceClient + ?Sized> Deref for ServiceRef<T> {
    type Target = T::ServiceProxy;
    fn deref(&self) -> &T::ServiceProxy {
        match &self.0 {
            InnerServiceRef::RemoteServiceRef(x) => x,
            InnerServiceRef::OwnedLocalService(_) => {
                panic!("Tried to dereference a ServiceRef on server side.")
            }
        }
    }
}

pub fn service_ref_from_service_proxy<T: RustyRpcServiceClient + ?Sized>(
    service_proxy: T::ServiceProxy,
) -> ServiceRef<T> {
    ServiceRef(InnerServiceRef::RemoteServiceRef(service_proxy))
}
