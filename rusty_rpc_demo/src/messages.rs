use serde::{Deserialize, Serialize};

/// A message sent from the server to the client
#[derive(Debug, Deserialize, Serialize)]
pub struct ServerMessage {
    pub x: i32,
}

/// A message sent from the client to the server
#[derive(Debug, Deserialize, Serialize)]
pub struct ClientMessage {
    pub y: i32,
}
