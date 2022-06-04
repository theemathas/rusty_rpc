use tokio::net::TcpStream;

use rusty_rpc_lib::start_client;
use rusty_rpc_macro::interface_file;

interface_file!("examples/src/parent_child/parent_child.protocol");

#[tokio::main]
async fn main() {
    let stream = TcpStream::connect("127.0.0.1:8080")
        .await
        .expect("Failed to connect to server");
    let mut parent_service = start_client::<dyn ParentService, _>(stream).await;

    assert_eq!(123, parent_service.get().await.unwrap());

    let mut child_service_1 = parent_service.child().await.unwrap();
    child_service_1.set(456).await.unwrap();
    child_service_1.close().await.unwrap();
    drop(child_service_1);
    // Compilation will fail if the above line is omitted.
    // Compilation will also fail if child_service_1 is used after this line.

    assert_eq!(456, parent_service.get().await.unwrap());

    let mut child_service_2 = parent_service.child().await.unwrap();
    child_service_2.set(789).await.unwrap();
    child_service_2.close().await.unwrap();
    drop(child_service_2);
    // Compilation will fail if the above line is omitted.
    // Compilation will also fail if child_service_2 is used after this line.

    assert_eq!(789, parent_service.get().await.unwrap());

    parent_service.close().await.unwrap();

    println!("Client done successfully!");
}
