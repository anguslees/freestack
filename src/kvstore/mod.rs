use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use std::io::Cursor;
use std::marker::PhantomData;

use capnp::serialize_packed;
use gj::Promise;

pub mod etcd;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub description: String,
    pub cause: Option<Box<::std::error::Error>>,
}

pub fn permanent(desc: String) -> Error {
    Error {kind: ErrorKind::Permanent, description: desc, cause: None}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// It is reasonable to rety, after some backoff.
    Retryable,
    /// Permanent error - retyring the same operations is unlikely to be useful.
    Permanent,
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        self.description.as_str()
    }
    fn cause(&self) -> Option<&::std::error::Error> { self.cause }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> ::std::result::Result<(), fmt::Error> {
        match self.cause {
            None => write!(f, "{}", self.description),
            Some(cause) => write!(f, "{}: {}", self.description, cause),
        }
    }
}

impl From<Error> for ::capnp::Error {
    fn from(err: Error) -> ::capnp::Error {
        ::capnp::Error {
            kind: match err.ErrorKind {
                ErrorKind::Retryable => ::capnp::ErrorKind::Overloaded,
                ErrorKind::Permanent => ::capnp::ErrorKind::Failed,
            },
            description: err.description,
        }
    }
}

pub trait KvStore {
    type Sequencer: Eq;
    fn get(&self, key: &str) -> Result<(Self::Sequencer, Vec<u8>)>;
    fn set(&mut self, key: &str, value: &[u8], current_seq: Option<Self::Sequencer>) -> Result<Self::Sequencer>;
    fn delete(&mut self, key: &str, current_seq: Option<Self::Sequencer>) -> Result<()>;
}

#[derive(Debug)]
pub struct KvEntry<K: KvStore, T: for<'a> ::capnp::traits::Owned<'a>> {
    store: Rc<RefCell<K>>,
    key: String,
    _capnp_type: PhantomData<T>,
}

impl<K: KvStore, T: for<'a> ::capnp::traits::Owned<'a>> KvEntry<K, T> {
    pub fn new(store: Rc<RefCell<K>>, key: String) -> Self {
        KvEntry{store: store, key: key, _capnp_type: PhantomData}
    }

    pub fn inner_store(&self) -> Rc<RefCell<K>> {
        self.store.clone()
    }

    pub fn key(&self) -> &str { &self.key }

    pub fn get_raw(&self) -> Promise<(K::Sequencer, Vec<u8>), Error>
    {
        // FIXME: KvStore should be gj/async
        Promise::ok(pry!(self.store.borrow().get(self.key)))
    }

    pub fn get_then<'a, R, F, E>(&self, cb: F) -> Promise<R, E>
        where F: FnOnce(K::Sequencer, <T as ::capnp::traits::Owned<'a>>::Reader)
                        -> ::std::result::Result<R, E>
    {
        self.get_raw()
            .map(|seq, data| {
                let message_reader = try!(serialize_packed::read_message(
                    &mut Cursor::new(&data), Default::default()));
                let obj = message_reader.get_root::<T::Builder>();
                cb(seq, obj)
            })
    }

    pub fn set_raw(&self, value: &[u8], current_seq: Option<K::Sequencer>)
                   -> Promise<K::Sequencer, Error>
    {
        // FIXME: KvStore should be gj/async
        Promise::ok(pry!(self.store.borrow().set(self.key, value, current_seq)))
    }

    pub fn set<'a>(&self, value: <T as ::capnp::traits::Owned<'a>>::Builder, current_seq: Option<K::Sequencer>)
                   -> Promise<K::Sequencer, Error>
    {
        let mut cursor = Cursor::new(Vec::new());
        try!(serialize_packed::write_message(&mut cursor, value));
        self.set_raw(cursor.get_ref(), current_seq)
    }
}


#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    type MemStore = BTreeMap<String, Vec<u8>>;
    impl KvStore for MemStore {
        type Sequencer = u64;
        fn get(&self, key: &str) -> Result<(Self::Sequencer, Vec<u8>)> {
            match self.get(key) {
                Some(v) => Ok((1, v)),
                None => Err(permanent("not found")),
            }
        }

        fn set(&mut self, key: &str, value: &[u8], current_seq: Option<Self::Sequencer>)
               -> Result<Self::Sequencer>
        {
            self.insert(key, value.clone());
            Ok(current_seq.or_default() + 1)
        }

        fn delete(&mut self, key: &str, current_seq: Option<Self::Sequencer>) -> Result<()> {
            self.delete(key);
            Ok(())
        }
    }

    #[test]
    fn test_kventry() {
        ::gj::EventLoop::top_level(move |wait_scope| {

            let mut memstore = MemStore::new();
            let mut entry = KvEntry::new(Rc::new(RefCell::new(memstore)), "/testkey".to_owned());

            {
                let resp = entry.get_raw().wait(wait_scope);
                assert!(resp.is_err());
                assert_eq!(resp, Err(Error{kind: ErrorKind::Permanent, ..}));
            }

            {
                let data = [42];
                assert_eq!(entry.set_raw(&data, Some(17)).wait(wait_scope),
                           Ok((18, vec![42])));
            }
        }).expect("top level error");
    }
}
