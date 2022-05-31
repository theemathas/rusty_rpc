use bytes::{Bytes, BytesMut};
use simple_error::SimpleError;

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
