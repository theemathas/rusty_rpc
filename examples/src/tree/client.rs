use tokio::net::TcpStream;

use rusty_rpc_lib::start_client;
use rusty_rpc_macro::interface_file;

interface_file!("examples/src/tree/tree.protocol");

#[tokio::main]
async fn main() {
    let stream = TcpStream::connect("127.0.0.1:8080")
        .await
        .expect("Failed to connect to server");
    let mut tree_service = start_client::<dyn TreeService, _>(stream).await;

    {
        let mut node_0_service = tree_service.root().await.unwrap();
        assert_eq!(0, node_0_service.get_value().await.unwrap());

        {
            let mut node_1_service = node_0_service.nth_child(0).await.unwrap();
            assert_eq!(1, node_1_service.get_value().await.unwrap());
            node_1_service.close().await.unwrap();
        }

        {
            let mut node_2_service = node_0_service.nth_child(1).await.unwrap();
            assert_eq!(2, node_2_service.get_value().await.unwrap());
            node_2_service.close().await.unwrap();
        }

        node_0_service.close().await.unwrap();
    }

    tree_service.close().await.unwrap();

    println!("Client done successfully!");
}
