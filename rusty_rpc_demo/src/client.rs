mod messages;
mod stream_sink;

use futures::{SinkExt, StreamExt};
use messages::{ClientMessage, ServerMessage};
use stream_sink::to_stream_sink;
use tokio::net::TcpSocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tcp_stream = TcpSocket::new_v4()?
        .connect("127.0.0.1:8080".parse().unwrap())
        .await?;
    let mut stream_sink = to_stream_sink::<ServerMessage, ClientMessage, _>(tcp_stream);

    let msg = stream_sink
        .next()
        .await
        .expect("TcpStream unexpectedly closed by server.")?;
    println!("Received message from server: {msg:?}");
    stream_sink.send(ClientMessage { y: 2 }).await?;
    println!("Server sent ClientMessage 2 successfully.");

    Ok(())
}
