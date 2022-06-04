use tokio::net::TcpStream;

use rusty_rpc_lib::start_client;
use rusty_rpc_macro::interface_file;

interface_file!("examples/src/hello_world/hello_world.protocol");

#[tokio::main]
async fn main() {
    let stream = TcpStream::connect("127.0.0.1:8080")
        .await
        .expect("Failed to connect to server");
    let mut service = start_client::<dyn MyService, _>(stream).await;

    let foo_result = service.foo().await.unwrap();
    assert_eq!(123, foo_result);

    let bar_output = service.bar(2).await.unwrap();
    assert_eq!(2, bar_output);

    let baz_output = service
        .baz(
            900,
            Foo {
                x: 80,
                y: Bar { z: 7 },
            },
        )
        .await
        .unwrap();
    assert_eq!(987, baz_output.x);
    assert_eq!(987, baz_output.y.z);

    service.close().await.unwrap();

    println!("Client done successfully!");
}
