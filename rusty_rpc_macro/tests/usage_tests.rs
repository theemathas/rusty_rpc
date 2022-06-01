use rusty_rpc_lib::{ServerResponse, ServerResult};
use rusty_rpc_macro::{interface_file, service_impl};

interface_file!("rusty_rpc_macro/tests/simple_interface_file.interface");

#[test]
#[allow(unreachable_code)]
fn test_types() {
    // Just test that stuff compiles.
    if false {
        let bar = Bar { z: 1 };
        let foo = Foo { x: 2, y: bar };

        struct DummyService;
        #[service_impl]
        impl MyService for DummyService {
            fn foo(&self) -> ServerResult<i32> {
                Ok(ServerResponse::new(123))
            }
            fn bar(&self, _a: i32, _b: Foo) -> ServerResult<Foo> {
                unimplemented!()
            }
        }

        let service = DummyService;
        let _: Foo = service.bar(3, foo.clone()).unwrap().into_inner();

        // Test that types have the right traits.
        fn need_rpc_struct(_: impl rusty_rpc_lib::internal_for_macro::RustyRpcStruct) {}
        need_rpc_struct(foo.clone());
        fn need_rpc_service(_: impl rusty_rpc_lib::internal_for_macro::RustyRpcService) {}
        need_rpc_service(service);
        fn need_my_service(_: impl MyService) {}
        need_my_service(unimplemented!() as ServerResponse<&dyn MyService>);
    }
}
