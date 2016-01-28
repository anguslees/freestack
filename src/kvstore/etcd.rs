use std::fmt;

use rustc_serialize::base64::{ToBase64,FromBase64};
use etcd;
use log;

use kvstore::{KvStore,Error,Result,ErrorKind};

impl From<etcd::Error> for Error {
    fn from(err: etcd::Error) -> Error {
        match err {
            etcd::Error::Io(_) => Error {
                kind: ErrorKind::Retryable,
                description: "IO error".to_owned(),
                cause: Some(Box::new(err)),
            },
            _ => Error {
                kind: ErrorKind::Permanent,
                description: "Etcd error".to_owned(),
                cause: Some(Box::new(err)),
            }
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct NotFound<'a>{
    pub key: &'a str,
}

impl<'a> fmt::Display for NotFound<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> ::std::result::Result<(), fmt::Error> {
        write!(f, "Key \"{}\" not found", self.key)
    }
}

impl<'a> ::std::error::Error for NotFound<'a> {
    fn description(&self) -> &str {
        "Not found"
    }
}

impl<'a> From<NotFound<'a>> for Error {
    fn from(err: NotFound<'a>) -> Error {
        Error {
            kind: ErrorKind::Permanent,
            description: "Not found",
            cause: Some(Box::new(err)),
        }
    }
}

pub struct Etcd {
    etcd_client: etcd::Client,
}

impl Etcd {
    pub fn new(root_url: &str) -> Result<Self> {
        let client = try!(etcd::Client::new(root_url));
        let version = try!(client.version());
        info!("Using etcd cluster at {} - version {}/{}",
              root_url,
              version.etcdserver.unwrap_or("(Unknown)".to_owned()),
              version.etcdcluster.unwrap_or("(Unknown)".to_owned()));
        Etcd { etcd_client: client }
    }
}

fn fetch_seq(node: &etcd::Node) -> Result<u64> {
    node.modifiedIndex
        .ok_or(
            Error {
                kind: ErrorKind::Permanent,
                description: "No modifiedIndex in get response".to_owned(),
                cause: None,
            })
}

impl KvStore for Etcd {
    type Sequencer = u64;

    fn get(&self, key: &str)
           -> Result<(Self::Sequencer, Vec<u8>)>
    {
        let info = try!(self.etcd_client.get(key, false, false));
        let seq = try!(fetch_seq(info.node));
        let data = try!(info.node.value
                        .ok_or(NotFound{key: key}));
        Ok((seq, data.from_base64()))
    }

    fn set(&mut self, key: &str, value: &[u8], current_seq: Option<Self::Sequencer>)
           -> Result<Self::Sequencer>
    {
        let data = value.to_base64();
        let info = try!(self.etcd_client.set(key, &data, None));
        fetch_seq(info.node)
    }

    fn delete(&mut self, key: &str, current_seq: Option<Self::Sequencer>)
              -> Result<()>
    {
        try!(self.etcd_client.delete(key, false));
        Ok(())
    }
}
