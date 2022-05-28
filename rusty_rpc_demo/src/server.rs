mod messages;
mod stream_sink;

use futures::{SinkExt, StreamExt};
use messages::{ClientMessage, ServerMessage};
use std::io;
use stream_sink::to_stream_sink;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut stream_sink = to_stream_sink::<ClientMessage, ServerMessage, _>(socket);

            stream_sink.send(ServerMessage { x: 1 }).await?;
            println!("Server sent ServerMessage 1 successfully.");

            let msg = stream_sink
                .next()
                .await
                .expect("TcpStream unexpectedly closed by client.")?;
            println!("Received message from client: {msg:?}");

            Ok::<(), io::Error>(())
        });
    }
}
