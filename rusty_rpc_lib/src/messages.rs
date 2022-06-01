use std::io;

use bytes::{Bytes, BytesMut};
use simple_error::SimpleError;

use crate::traits::{RustyRpcService, RustyRpcStruct};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ServiceId(pub u64);
impl ServiceId {
    pub fn increment(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }
}

/// The message that the server responds to the client, giving back the RPC return value.
pub struct ServerMessage {}
impl From<ServerMessage> for BytesMut {
    fn from(_: ServerMessage) -> Self {
        todo!()
    }
}

/// The message that the client sends to the server in order to call an RPC.
pub struct ClientMessage {
    // TODO implement dropping a service
    /// The service that the client wants to call a method on.
    pub service_id: ServiceId,
    pub method_and_args: MethodAndArgs,
}
impl TryFrom<Bytes> for ClientMessage {
    type Error = SimpleError;

    fn try_from(_: Bytes) -> Result<ClientMessage, SimpleError> {
        todo!()
    }
}

/// Represents the data used to specify the method and arguments for a given RPC
/// call.
pub struct MethodAndArgs {
    // TODO
}

pub type ServerResult<T> = io::Result<ServerResponse<T>>;

enum InnerServerResponse<T> {
    RemoteService(ServiceId), // TODO reference to connection somehow
    DataOrLocalService(T),
}

/// Wrapper around an RPC return value.
///
/// When this wrapper is dropped on the client side, the associated resources,
/// if any, are dropped on the server side. If the type `T` is an implementation
/// of a certain service, then `Response<T>` will implement the corresponding
/// service trait.
pub struct ServerResponse<T>(
    /// Do enum inside struct in order to get private enum variants and to make
    /// implementing Drop easier.
    ///
    /// This thing is None only after being dropped.
    Option<InnerServerResponse<T>>,
);
impl<T> ServerResponse<T> {
    /// Used on the server side to create a new Response.
    pub fn new(inner: T) -> Self {
        ServerResponse(Some(InnerServerResponse::DataOrLocalService(inner)))
    }
}
impl<T: RustyRpcStruct> ServerResponse<T> {
    /// Unwraps the Response.
    pub fn into_inner(mut self) -> T {
        match self.0.take().expect("Tried to into_inner on dropped Response") {
            InnerServerResponse::RemoteService(_) =>
                panic!("Somehow had a RemoteService variant on an impl RustyRpcStruct, which should not be possible."),
            InnerServerResponse::DataOrLocalService(x) => x,
        }
    }
}
impl<T> Drop for ServerResponse<T> {
    fn drop(&mut self) {
        let temp = self.0.take().expect("Tried to drop Response twice");
        match temp {
            InnerServerResponse::RemoteService(_service_id) => todo!(),
            InnerServerResponse::DataOrLocalService(_) => (),
        }
    }
}

pub fn response_from_service_id<T: RustyRpcService>(service_id: ServiceId) -> ServerResponse<T> {
    ServerResponse(Some(InnerServerResponse::RemoteService(service_id)))
}
