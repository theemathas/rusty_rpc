use std::io;

use rusty_rpc_lib::{start_client, start_server, RustyRpcServiceClient, ServiceRefMut};
use rusty_rpc_macro::{interface_file, service_server_impl};
use tokio::net::{TcpListener, TcpSocket};

interface_file!("rusty_rpc_macro/tests/simple_interface_file.interface");

#[tokio::test]
#[allow(unreachable_code)]
async fn test_types() {
    // Just test that stuff compiles.
    if false {
        let bar = Bar { z: 1 };
        let foo = Foo { x: 2, y: bar };

        struct DummyService;
        #[service_server_impl]
        impl MyService for DummyService {
            async fn foo(&mut self) -> io::Result<i32> {
                Ok(123)
            }
            async fn bar(&mut self, _a: i32) -> io::Result<i32> {
                unimplemented!()
            }
            async fn bar2(&mut self, _a: i32, _b: Foo) -> io::Result<Foo> {
                unimplemented!()
            }
            async fn baz(&mut self) -> io::Result<ServiceRefMut<dyn MyService>> {
                Ok(ServiceRefMut::new(DummyService))
            }
        }

        let mut service = DummyService;
        let _: i32 = service.bar(3).await.unwrap();
        let _: Foo = service.bar2(3, foo.clone()).await.unwrap();

        // Test that types have the right traits.
        fn need_rpc_struct(_: impl rusty_rpc_lib::internal_for_macro::RustyRpcStruct) {}
        need_rpc_struct(foo.clone());

        fn need_rpc_service_server(
            _: impl rusty_rpc_lib::internal_for_macro::RustyRpcServiceServer,
        ) {
        }
        need_rpc_service_server(service);

        fn need_my_service(
            _: impl MyService + rusty_rpc_lib::internal_for_macro::RustyRpcServiceProxy,
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

#[tokio::test]
async fn simple_usage() {
    #[derive(Default)]
    struct DummyService;
    #[service_server_impl]
    impl MyService for DummyService {
        async fn foo(&mut self) -> io::Result<i32> {
            Ok(123)
        }
        async fn bar(&mut self, arg: i32) -> io::Result<i32> {
            Ok(arg)
        }
        async fn bar2(&mut self, arg1: i32, arg2: Foo) -> io::Result<Foo> {
            let val = arg1 + arg2.x + arg2.y.z;
            Ok(Foo {
                x: val,
                y: Bar { z: val },
            })
        }
        async fn baz(&mut self) -> io::Result<ServiceRefMut<dyn MyService>> {
            Ok(ServiceRefMut::new(ConstService(9999)))
        }
    }

    struct ConstService(i32);
    #[service_server_impl]
    impl MyService for ConstService {
        async fn foo(&mut self) -> io::Result<i32> {
            Ok(self.0)
        }
        async fn bar(&mut self, _arg: i32) -> io::Result<i32> {
            unimplemented!()
        }
        async fn bar2(&mut self, _arg1: i32, _arg2: Foo) -> io::Result<Foo> {
            unimplemented!()
        }
        async fn baz(&mut self) -> io::Result<ServiceRefMut<dyn MyService>> {
            unimplemented!()
        }
    }

    // Port 0 means the OS chooses a port.
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server_handle =
        tokio::spawn(async { start_server::<DummyService>(listener).await.unwrap() });

    let client_handle = tokio::spawn(async move {
        let stream = TcpSocket::new_v4().unwrap().connect(addr).await.unwrap();
        let mut service = start_client::<dyn MyService, _>(stream).await;

        let foo_output = service.foo().await.unwrap();
        assert_eq!(123, foo_output);

        let bar_output = service.bar(2).await.unwrap();
        assert_eq!(2, bar_output);

        let bar2_output = service
            .bar2(
                900,
                Foo {
                    x: 80,
                    y: Bar { z: 7 },
                },
            )
            .await
            .unwrap();
        assert_eq!(987, bar2_output.x);
        assert_eq!(987, bar2_output.y.z);

        let mut baz_output_service = service.baz().await.unwrap();
        let baz_foo_output = baz_output_service.foo().await.unwrap();
        assert_eq!(9999, baz_foo_output);
        baz_output_service.close().await.unwrap();

        service.close().await.unwrap();
    });

    client_handle.await.expect("Client crashed.");
    server_handle.abort();
    let server_error = server_handle
        .await
        .expect_err("Server somehow terminated on its own without crashing.");
    assert!(server_error.is_cancelled(), "Server crashed.");
}
