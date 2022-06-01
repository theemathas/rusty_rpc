pub mod internal_for_macro;

pub use messages::ServiceRef;
pub use traits::{RustyRpcServiceClient, RustyRpcServiceProxy, RustyRpcServiceServer};

mod messages;
mod service_collection;
mod traits;
mod util;

use std::io;

use bytes::BytesMut;
use futures::{SinkExt, StreamExt};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpListener;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use messages::{ClientMessage, ServerMessage};
use service_collection::ServiceCollection;
use util::string_io_error;

use crate::util::other_io_error;

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

    // This implements Stream<Item=BytesMut> and Sink<Bytes>.
    // So we can send and receive "packets" of byte blocks of arbitrary size.
    let mut bytes_stream_sink = Framed::new(read_write, LengthDelimitedCodec::new());

    while let Some(received_bytes_result) = bytes_stream_sink.next().await {
        let received_bytes = received_bytes_result?; // Handle I/O errors.
        let client_message =
            ClientMessage::try_from(received_bytes.freeze()).map_err(other_io_error)?;
        let ClientMessage {
            service_id,
            method_and_args,
        } = client_message;

        let message_to_send: ServerMessage = {
            let service_arc = service_collection
                .get_service_arc(service_id)
                .ok_or_else(|| string_io_error(format!("Invalid service ID: {}", service_id.0)))?;
            let mut service_guard = service_arc
                .try_lock()
                .expect("Service somehow in use while trying to call a method on it.");
            service_guard.parse_and_call_method_locally(method_and_args, service_collection)?
        };

        let bytes_to_send: BytesMut = message_to_send.into();
        bytes_stream_sink.send(bytes_to_send.freeze()).await?;
    }

    Ok(())
}

pub async fn start_client<T: RustyRpcServiceClient>(_rw: impl AsyncRead + AsyncWrite) {
    todo!()
}
