# rusty_rpc

Class project for the CS244b "Distributed System" class at Stanford during the Spring 2022 quarter.

An RPC implemented for the Rust programming language and designed to better use Rust's unique features.

TODO: Write some more stuff here.

For usage examples, see the `examples` directory. I recommend looking in the order: `hello_world`, then `parent_child`, then `tree`.

To run the `hello_world` example, run `cargo run --bin hello_world_server`.
Then, in another terminal, you may run `cargo run --bin hello_world_client` any
number of times. The client will run, verify that it worked properly, print
`Client done successfully!`, then terminate. You'll need to manually terminate
the server with ctrl-C.

Follow similar steps for the `parent_child` and `tree` examples.