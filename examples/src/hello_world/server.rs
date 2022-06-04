use std::io;

use tokio::net::TcpListener;

use rusty_rpc_lib::start_server;
use rusty_rpc_macro::{interface_file, service_server_impl};

interface_file!("examples/src/hello_world/hello_world.protocol");

#[derive(Default)]
struct MyServiceServer;
#[service_server_impl]
impl MyService for MyServiceServer {
    async fn foo(&mut self) -> io::Result<i32> {
        Ok(123)
    }
    async fn bar(&mut self, arg: i32) -> io::Result<i32> {
        Ok(arg)
    }
    async fn baz(&mut self, arg1: i32, arg2: Foo) -> io::Result<Foo> {
        let val = arg1 + arg2.x + arg2.y.z;
        Ok(Foo {
            x: val,
            y: Bar { z: val },
        })
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Failed to bind to port to start server.");
    start_server::<MyServiceServer>(listener)
        .await
        .expect("Server top-level crashed.")
}
