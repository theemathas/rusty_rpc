use std::io;

use rusty_rpc_lib::{start_client, start_server, RustyRpcServiceClient, ServiceRef};
use rusty_rpc_macro::{interface_file, service_server_impl};
use tokio::net::{TcpListener, TcpSocket};

interface_file!("rusty_rpc_macro/tests/simple_interface_file.interface");

#[test]
#[allow(unreachable_code)]
fn test_types() {
    // Just test that stuff compiles.
    if false {
        let bar = Bar { z: 1 };
        let foo = Foo { x: 2, y: bar };

        struct DummyService;
        #[service_server_impl]
        impl MyService for DummyService {
            fn foo(&self) -> io::Result<i32> {
                Ok(123)
            }
            fn bar(&self, _a: i32, _b: Foo) -> io::Result<Foo> {
                unimplemented!()
            }
            fn baz(&self) -> io::Result<ServiceRef<dyn MyService>> {
                Ok(ServiceRef::new(unimplemented!() as Box<dyn MyService>))
            }
        }

        let service = DummyService;
        let _: Foo = service.bar(3, foo.clone()).unwrap();

        // Test that types have the right traits.
        fn need_rpc_struct(_: impl rusty_rpc_lib::internal_for_macro::RustyRpcStruct) {}
        need_rpc_struct(foo.clone());

        fn need_rpc_service_server(
            _: impl rusty_rpc_lib::internal_for_macro::RustyRpcServiceServer,
        ) {
        }
        need_rpc_service_server(service);

        fn need_my_service(
            _: impl MyService + rusty_rpc_lib::internal_for_macro::RustyRpcServiceProxy<dyn MyService>,
        ) {
        }
        need_my_service(unimplemented!() as <dyn MyService as RustyRpcServiceClient>::ServiceProxy);

        fn need_rpc_service_client<
            T: rusty_rpc_lib::internal_for_macro::RustyRpcServiceClient + ?Sized,
        >(
            _: &T,
        ) {
        }
        need_rpc_service_client(unimplemented!() as &dyn MyService);
    }
}

// TODO remove this ignore.
#[ignore]
#[tokio::test]
async fn simple_usage() {
    #[derive(Default)]
    struct DummyService;
    #[service_server_impl]
    impl MyService for DummyService {
        fn foo(&self) -> io::Result<i32> {
            Ok(123)
        }
        fn bar(&self, _a: i32, _b: Foo) -> io::Result<Foo> {
            unimplemented!()
        }
        fn baz(&self) -> io::Result<ServiceRef<dyn MyService>> {
            unimplemented!()
        }
    }

    // Port 0 means the OS chooses a port.
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server_handle =
        tokio::spawn(async { start_server::<DummyService>(listener).await.unwrap() });
    let client_handle = tokio::spawn(async move {
        let socket = TcpSocket::new_v4().unwrap().connect(addr).await.unwrap();
        let service = start_client::<dyn MyService, _>(socket).await;
        let foo_output = service.foo().unwrap();
        assert_eq!(123, foo_output);
    });
    client_handle.await.expect("Client crashed.");
    server_handle.abort();
    let server_error = server_handle
        .await
        .expect_err("Server somehow terminated on its own without crashing.");
    assert!(server_error.is_cancelled(), "Server crashed.");
}
