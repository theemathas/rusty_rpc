use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::{
    traits::RustyRpcServiceServerWithKnownClientType, RustyRpcServiceClient, RustyRpcServiceServer,
};

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
pub enum ReturnValue {
    Data(Vec<u8>),
    Service(ServiceId),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MethodId(pub u64);

/// The message that the client sends to the server in order to call an RPC.
#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    DropService(ServiceId),
    CallMethod(ServiceId, MethodId, MethodArgs),
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
pub struct MethodArgs(pub Vec<u8>);

enum InnerServiceRefMut<'a, T: RustyRpcServiceClient + ?Sized + 'a> {
    RemoteServiceRefMut(T::ServiceProxy, PhantomData<&'a T>),
    OwnedLocalService(Box<dyn RustyRpcServiceServer<'a>>, PhantomData<T>),
}

/// Either an owned server-side service, or a client's reference to such a
/// service. (If the fromer, cannot be dereferenced. If the latter, acts like
/// `&T` and can be dereferenced.)
///
/// When the ServiceRefMut's for a certain service is dropped on the client side,
/// the associated resources, are dropped on the server side. If the type `T` is
/// an implementation of a certain service, then `Response<T>` will implement
/// the corresponding service trait.
///
/// The type `T` should be something like `dyn MyService` (bare unsized dyn
/// trait).
pub struct ServiceRefMut<'a, T: RustyRpcServiceClient + ?Sized + 'a>(
    /// Do enum inside struct to get private enum variants.
    InnerServiceRefMut<'a, T>,
);
impl<'a, T: RustyRpcServiceClient + ?Sized + 'a> ServiceRefMut<'a, T> {
    /// Used on the server side.
    pub fn new<S: RustyRpcServiceServerWithKnownClientType<'a, T>>(inner: S) -> Self {
        ServiceRefMut(InnerServiceRefMut::OwnedLocalService(
            Box::new(inner),
            PhantomData,
        ))
    }
}
/// Used only on the client side.
impl<'a, T: RustyRpcServiceClient + ?Sized + 'a> Deref for ServiceRefMut<'a, T> {
    type Target = T::ServiceProxy;
    fn deref(&self) -> &T::ServiceProxy {
        match &self.0 {
            InnerServiceRefMut::RemoteServiceRefMut(x, _) => x,
            InnerServiceRefMut::OwnedLocalService(..) => {
                panic!("Tried to deref() a ServiceRefMut on server side.")
            }
        }
    }
}
impl<'a, T: RustyRpcServiceClient + ?Sized + 'a> DerefMut for ServiceRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.0 {
            InnerServiceRefMut::RemoteServiceRefMut(x, _) => x,
            InnerServiceRefMut::OwnedLocalService(..) => {
                panic!("Tried to deref_mut() a ServiceRefMut on server side.")
            }
        }
    }
}

/// For macro and internal use only.
pub fn service_ref_from_service_proxy<'a, T: RustyRpcServiceClient + ?Sized + 'a>(
    service_proxy: T::ServiceProxy,
) -> ServiceRefMut<'a, T> {
    ServiceRefMut(InnerServiceRefMut::RemoteServiceRefMut(
        service_proxy,
        PhantomData,
    ))
}

/// For macro use only.
pub fn local_service_from_service_ref<'a, T: RustyRpcServiceClient + ?Sized + 'a>(
    service_ref: ServiceRefMut<'a, T>,
) -> Option<Box<dyn RustyRpcServiceServer<'a>>> {
    match service_ref.0 {
        InnerServiceRefMut::RemoteServiceRefMut(..) => None,
        InnerServiceRefMut::OwnedLocalService(x, _) => Some(x),
    }
}
