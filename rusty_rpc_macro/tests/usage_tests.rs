use rusty_rpc_macro::interface_file;

interface_file!("rusty_rpc_macro/tests/simple_interface_file.interface");

#[test]
fn test_types() {
    // Just test that stuff compiles.
    if false {
        let bar = Bar { z: 1 };
        let foo = Foo { x: 2, y: bar };

        struct DummyService;
        impl MyService for DummyService {
            fn foo(&self) -> i32 {
                unimplemented!()
            }
            fn bar(&self, _a: i32, _b: Foo) -> Foo {
                unimplemented!()
            }
        }

        let service = DummyService;
        let _: Foo = service.bar(3, foo);
    }
}
