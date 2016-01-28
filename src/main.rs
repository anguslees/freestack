extern crate capnp;
extern crate capnp_rpc;

use std::error::Error;
use std::io;

use capnp_rpc::{RpcSystem,twoparty,rpc_twoparty_capnp};
use capnp_rpc::capability::{InitRequest,LocalClient,WaitForContent};
use gj::{EventLoop,Promise,TaskReaper,TaskSet};
use gj::io::tcp;

use freestack::identity3;


pub fn accept_loop(listener: tcp::Listener,
                   mut task_set: TaskSet<(), Box<Error>>,
                   client: Client,,
                   ) -> Promise<(), io::Error>
{
    listener.accept().lift().then(move |(listener, stream)| {
        let (reader, writer) = stream.split();
        let mut network =
            twoparty::VatNetwork::new(reader, writer,
                                      rpc_twoparty_capnp::Side::Server, Default::default());
        let disconnect_promise = network.on_disconnect();

        let rpc_system = RpcSystem::new(Box::new(network), Some(client.clone().client));

        task_set.add(disconnect_promise.attach(rpc_system).lift());
        accept_loop(listener, task_set, client)
    })
}

struct Reaper;
impl TaskReaper<(), Box<Error>> for Reaper {
    fn task_failed(&mut self, error: Box<Error>) {
        // FIXME: log instead
        println!("Task failed: {}", error);
    }
}

fn main() {
    println!("Starting up");

    let bind_addr = "localhost:1234";
    let etcd_url = "http://localhost:2379";

    let store = Rc::new(RefCell::new(
        kvstore::etcd::Etcd::new(etcd_url)
            .expect("Error connecting to etcd")));
    let identity3_server = identity3::bootstrap_interface(store);

    EventLoop::top_level(move |wait_scope| {
        use std::net::ToSocketAddrs;
        let addr = try!(bind_addr.to_socket_addrs()).next().expect("could ot parse address");
        let listener = try!(tcp::Listener::bind(addr));

        let task_set = TaskSet::new(Box::new(Reaper));
        try!(accept_loop(listener, task_set, identity3_server).wait(wait_scope));

        Ok(())
    }).expect("top level error");
}
