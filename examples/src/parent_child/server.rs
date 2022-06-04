use std::io;

use tokio::net::TcpListener;

use rusty_rpc_lib::{start_server, ServiceRefMut};
use rusty_rpc_macro::{interface_file, service_server_impl};

interface_file!("examples/src/parent_child/parent_child.protocol");

struct ParentServer(i32);
impl Default for ParentServer {
    fn default() -> ParentServer {
        ParentServer(123)
    }
}

struct ChildServer<'a>(&'a mut i32);

#[service_server_impl]
impl ParentService for ParentServer {
    async fn child<'a>(&'a mut self) -> io::Result<ServiceRefMut<dyn ChildService + 'a>> {
        Ok(ServiceRefMut::new(ChildServer(&mut self.0)))
    }
    async fn get(&mut self) -> io::Result<i32> {
        Ok(self.0)
    }
}

#[service_server_impl]
impl<'a> ChildService for ChildServer<'a> {
    async fn set(&mut self, new_value: i32) -> io::Result<i32> {
        *self.0 = new_value;
        Ok(new_value)
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Failed to bind to port to start server.");
    start_server::<ParentServer>(listener)
        .await
        .expect("Server top-level crashed.")
}
