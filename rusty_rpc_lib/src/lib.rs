pub mod internal_for_macro;

pub use messages::ServiceRef;
pub use traits::{
    RustyRpcServiceClient, RustyRpcServiceProxy, RustyRpcServiceServer,
    RustyRpcServiceServerWithKnownClientType,
};

mod messages;
mod service_collection;
mod traits;
mod util;

use std::io;
use std::sync::Arc;

use bytes::{Bytes, BytesMut};
use futures::{SinkExt, StreamExt};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use messages::{service_ref_from_service_proxy, ClientMessage, ServerMessage, ServiceId};
use service_collection::ServiceCollection;
use traits::ClientStreamSink;
use util::{other_io_error, string_io_error};

/// Starts a server, accepting new connections in an infinite loop.
///
/// `T` is the type of the initial service to be used as the starting point of
/// each connection. For each connection, a new value of that type will be
/// created using the `Default` trait.
///
/// To implement [RustyRpcServiceServer], use the `#[service_server_impl]`
/// attribute in the `rusty_rpc_macro` crate.
pub async fn start_server<T: RustyRpcServiceServer + Default>(
    listener: TcpListener,
) -> std::io::Result<()> {
    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = handle_connection::<T, _>(&mut ServiceCollection::new(), socket).await {
                eprintln!("Connection handler terminated due to error: {}", e);
            };
        });
    }
}

async fn handle_connection<
    T: RustyRpcServiceServer + Default,
    RW: AsyncRead + AsyncWrite + Unpin,
>(
    service_collection: &mut ServiceCollection,
    read_write: RW,
) -> io::Result<()> {
    // Add initial service.
    let initial_service_id = service_collection.register_service(Box::new(T::default()));
    assert_eq!(initial_service_id.0, 0);

    // This implements Stream<Item=io::Result<BytesMut>> and Sink<Bytes>.
    // So we can send and receive "packets" of byte blocks of arbitrary size.
    let mut bytes_stream_sink = Framed::new(read_write, LengthDelimitedCodec::new());

    while let Some(received_bytes_result) = bytes_stream_sink.next().await {
        let received_bytes = received_bytes_result?; // Handle I/O errors.
        let client_message =
            ClientMessage::try_from(received_bytes.freeze()).map_err(other_io_error)?;
        let message_to_send: ServerMessage = match client_message {
            ClientMessage::DropService(service_id) => {
                let service_arc = service_collection
                    .remove_service_arc(service_id)
                    .ok_or_else(|| {
                        string_io_error(format!("Invalid service ID: {}", service_id.0))
                    })?;

                let service_mutex = Arc::try_unwrap(service_arc)
                    .ok() // Needed because the Err field doesn't impl Debug.
                    .expect("Client attempted to drop a service that is still in use.");
                std::mem::drop(service_mutex.into_inner());

                ServerMessage::DropServiceDone
            }
            ClientMessage::CallMethod(service_id, method_id, method_args) => {
                let service_arc =
                    service_collection
                        .get_service_arc(service_id)
                        .ok_or_else(|| {
                            string_io_error(format!("Invalid service ID: {}", service_id.0))
                        })?;
                let mut service_guard = service_arc
                    .try_lock()
                    .expect("Service somehow in use while trying to call a method on it.");
                service_guard
                    .parse_and_call_method_locally(method_id, method_args, service_collection)
                    .await?
            }
        };

        bytes_stream_sink.send(Bytes::from(message_to_send)).await?;
    }

    Ok(())
}

/// Start a client connection with the specified initial service.
pub async fn start_client<
    T: RustyRpcServiceClient + ?Sized,
    RW: AsyncRead + AsyncWrite + 'static + Send + Unpin,
>(
    read_write: RW,
) -> ServiceRef<T> {
    let initial_service_id = ServiceId(0);
    let bytes_stream_sink = Framed::new(read_write, LengthDelimitedCodec::new());
    let client_stream_sink = bytes_stream_sink
        .map(
            |in_bytes: io::Result<BytesMut>| -> io::Result<ServerMessage> {
                in_bytes.and_then(|x| ServerMessage::try_from(x.freeze()).map_err(other_io_error))
            },
        )
        .with(|out_message: ClientMessage| {
            futures::future::ready(io::Result::Ok(Bytes::from(out_message)))
        });
    let wrapped: Arc<Mutex<dyn ClientStreamSink>> = Arc::new(Mutex::new(client_stream_sink));
    let proxy = T::ServiceProxy::from_service_id(initial_service_id, wrapped as _);
    service_ref_from_service_proxy(proxy)
}
