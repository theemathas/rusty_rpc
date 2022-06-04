use std::io;

use tokio::net::TcpListener;

use rusty_rpc_lib::{start_server, ServiceRefMut};
use rusty_rpc_macro::{interface_file, service_server_impl};

interface_file!("examples/src/tree/tree.protocol");

struct Node {
    value: i32,
    children: Vec<Node>,
}

struct TreeServer(Node);
#[service_server_impl]
impl TreeService for TreeServer {
    async fn root(&mut self) -> io::Result<ServiceRefMut<dyn NodeService>> {
        Ok(ServiceRefMut::new(NodeServer(&mut self.0)))
    }
}
impl Default for TreeServer {
    fn default() -> Self {
        TreeServer(Node {
            value: 0,
            children: vec![
                Node {
                    value: 1,
                    children: vec![],
                },
                Node {
                    value: 2,
                    children: vec![],
                },
            ],
        })
    }
}

struct NodeServer<'a>(&'a mut Node);
#[service_server_impl]
impl<'a> NodeService for NodeServer<'a> {
    async fn nth_child(&mut self, n: i32) -> io::Result<ServiceRefMut<dyn NodeService>> {
        // Crash if invalid n.
        let child_node = self.0.children.get_mut(n as usize).expect("Invalid n");
        Ok(ServiceRefMut::new(NodeServer(child_node)))
    }

    async fn get_value(&mut self) -> io::Result<i32> {
        Ok(self.0.value)
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Failed to bind to port to start server.");
    start_server::<TreeServer>(listener)
        .await
        .expect("Server top-level crashed.")
}
