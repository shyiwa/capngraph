extern crate capnpc;

fn main() {
    ::capnpc::compile("schemas", &["schemas/graph.capnp"]).unwrap();
}
