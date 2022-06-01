use std::io;

use rusty_rpc_lib::{RustyRpcServiceClient, ServiceRef};
use rusty_rpc_macro::{interface_file, service_server_impl};

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
