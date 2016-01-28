extern crate capnp;
extern crate capnp_rpc;
#[macro_use] extern crate gj;
extern crate rustc_serialize;
extern crate etcd;
#[macro_use] extern crate log;

pub mod persistent_capnp {
    include!(concat!(env!("OUT_DIR"), "/persistent_capnp.rs"));
}
pub mod types_capnp {
    include!(concat!(env!("OUT_DIR"), "/types_capnp.rs"));
}
pub mod identity3_capnp {
    include!(concat!(env!("OUT_DIR"), "/identity3_capnp.rs"));
}
pub mod compute2_capnp {
    include!(concat!(env!("OUT_DIR"), "/compute2_capnp.rs"));
}

pub mod identity3;
pub mod kvstore;
