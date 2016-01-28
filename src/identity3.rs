use std::sync::Arc;
use std::convert::From;
use std::collections::BTreeMap;
use std::default::Default;
use std::rc::Rc;
use std::cell::RefCell;
use std::io::Cursor;

use capnp::capability::Server;
use capnp::{serialize_packed,Error};
use gj::Promise;

use kvstore::{KvStore,KvEntry,permanent};
use identity3_capnp::{auth,domain,user};

struct DomainImpl<K: KvStore> {
    store: KvEntry<K, domain::details::Owned>,
}

impl<K: KvStore> DomainImpl<K> {
    pub fn new(store: KvEntry<K, domain::details::Owned>) -> Self {
        DomainImpl { store: store }
    }

    fn get_user(&self, name: &str) -> UserImpl<K> {
        let key = format!("{}/users/{}", self.store.key(), name);
        let entry = KvEntry::new(self.store.inner_store(), key);
        UserImpl::new(entry)
    }
}

impl<K: KvStore> domain::Server for DomainImpl<K> {
    fn details(&mut self, _params: domain::DetailsParams, mut results: domain::DetailsResults)
               -> Promise<(), Error>
    {
        self.store
            .get_then(|seq, details| {
                results.get().set_details(details);
                Ok(())
            })
    }
}

struct UserImpl<K: KvStore> {
    store: KvEntry<K, user::details::Owned>,
}

impl<K: KvStore> UserImpl<K> {
    pub fn new(store: KvEntry<K, user::details::Owned>) -> Self {
        UserImpl { store: store }
    }
}

impl<K: KvStore> user::Server for UserImpl<K> {
    fn details(&mut self, _params: user::DetailsParams, mut results: user::DetailsResults)
               -> Promise<(), Error>
    {
        unimplemented!();
    }
}

struct AuthImpl<K: KvStore> {
    store: Rc<RefCell<K>>,
}

impl<K: KvStore> AuthImpl<K> {
    pub fn new(store: Rc<RefCell<K>>) -> Self {
        AuthImpl { store: store }
    }

    fn get_domain(&self, name: &str) -> DomainImpl<K> {
        let key = format!("v1/domains/{}", name);
        let entry = KvEntry::new(self.store.clone(), key);
        DomainImpl::new(entry)
    }
}

impl<K: KvStore> auth::Server for AuthImpl<K> {
    fn auth(&mut self, params: auth::AuthParams, mut results: auth::AuthResults)
            -> Promise<(), Error>
    {
        let req = pry!(pry!(params.get()).get_req());
        match pry!(req.get_method().which()) {
            auth::auth_request::method::Password(v) => {
                let mut resp = results.get().init_resp();
                let pass = pry!(v);

                let dom = self.get_domain(pry!(pass.get_domain()));
                let dom_check = dom.store
                    .get_then(move |seq, details| {
                        if !details.get_enabled() {
                            return Err(Error::failed("Domain disabled".to_owned()));
                        }
                        Ok(())
                    });

                let user = dom.get_user(pry!(pass.get_user()));
                let user_check = user.store
                    .get_then(move |seq, details| {
                        if try!(details.get_password()) != try!(pass.get_password()) {
                            return Err(Error::failed("Password incorrect".to_owned()));
                        }
                        if !details.get_enabled() {
                            return Err(Error::failed("User disabled".to_owned()));
                        }
                        Ok(())
                    });

                let user_client = user::ToClient::new(user)
                    .from_server::<::capnp_rpc::Server>();
                resp.set_user(user_client);

                let checks = vec![dom_check, user_check];
                Promise::all(checks.into_iter())
                    .map(|_| Ok(()))
            },
            auth::auth_request::method::Token(_) =>
                Promise::err(Error::unimplemented("token unsupported".to_owned())),
        }
    }
}

pub fn bootstrap_interface<K: KvStore>(store_root: Rc<RefCell<K>>) -> Box<Server> {
    let mut auth = AuthImpl::new(store_root);

    Box::new(auth::ServerDispatch {
        server: Box::new(auth),
    })
}

#[test]
fn test_sturdyref() {
    let mut message = ::capnp::message::Builder::new_default();
    {
        let mut domain = message.init_root::<domain::details::Builder>();
        domain.set_name("MyDomain");
        domain.set_description("This is my test domain");
    }

    serialize_packed::write_message(&mut ::std::io::stdout(), &message).unwrap();
}
