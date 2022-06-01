use std::io;
use std::sync::{Arc, Mutex};

use futures::{Sink, Stream};

use crate::messages::{ClientMessage, MethodAndArgs, ServerMessage, ServiceId};
use crate::ServiceCollection;

/// For any given service trait `MyService` which came from the
/// `interface_file!` macro in the `rusty_rpc_macro` crate, the type `dyn
/// MyService` (unsized naked dyn trait) implements [RustyRpcServiceClient].
///
/// Users should not manually implement this trait.
///
/// For some reason the Send + Sync + 'static bound is needed for
/// `tokio::spawn`.
pub trait RustyRpcServiceClient {
    /// A proxy that client uses to reference such a service. This proxy type
    /// will implement the service trait (e.g. MyService).
    ///
    /// When the ServiceRef for a certain service is dropped on the client
    /// side, the associated resources, are dropped on the server side. If the
    /// type `T` is an implementation of a certain service, then `Response<T>`
    /// will implement the corresponding service trait.
    type ServiceProxy: RustyRpcServiceProxy<Self>;
}

/// Used with [RustyRpcServiceClient]. Something that implements
/// `RustyRpcServiceProxy<dyn T>` will also always be generated so it implements
/// `T`. This type is a proxy that deallocates server-side resources when
/// dropped on the client side.
#[allow(drop_bounds)]
pub trait RustyRpcServiceProxy<T: RustyRpcServiceClient + ?Sized>: Drop {
    #[doc(hidden)]
    fn from_service_id(
        service_id: ServiceId,
        stream_sink: Arc<Mutex<dyn ClientStreamSink>>,
    ) -> Self;
}

/// Alias for `Stream + Sink`, so we can use it as a dyn trait. Represents the
/// communication channel endpoint on the client's side
pub trait ClientStreamSink:
    Stream<Item = io::Result<ServerMessage>> + Sink<ClientMessage, Error = io::Error>
{
}
impl<T: Stream<Item = io::Result<ServerMessage>> + Sink<ClientMessage, Error = io::Error>>
    ClientStreamSink for T
{
}

/// This trait will be automatically implemented by any user type marked with
/// the `#[service_server_impl]` attribute in the `rusty_rpc_macro` crate. Users
/// should not manually implement this trait.
///
/// Client-side access to services (via [RustyRpcServiceClient::ServiceRef]) CANNOT
/// use this trait.
///
/// For some reason the Send + Sync + 'static bound is needed for
/// `tokio::spawn`.
pub trait RustyRpcServiceServer: Send + Sync + 'static {
    #[doc(hidden)]
    fn parse_and_call_method_locally(
        &mut self,
        method_and_args: MethodAndArgs,
        connection: &mut ServiceCollection,
    ) -> io::Result<ServerMessage>;
}

/// This trait will be automatically implemented by struct types generated by
/// the `interface_file!` macro in the `rusty_rpc_macro` crate. Users should not
/// manually implement this trait.
pub trait RustyRpcStruct {
    // TODO add things needed here.
}
/// i32 is treated like a struct in this library.
impl RustyRpcStruct for i32 {}
