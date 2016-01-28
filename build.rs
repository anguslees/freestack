extern crate capnpc;

fn main() {
    let input = [
        // TODO: I think capnpc should follow schema imports, particularly to stdinc dirs.
        "/usr/include/capnp/persistent.capnp",
        "schema/types.capnp",
        "schema/compute2.capnp",
        "schema/identity3.capnp",
        ];
    capnpc::compile("schema", &input).unwrap();
}
