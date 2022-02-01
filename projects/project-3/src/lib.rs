//! # kvs
//!
//! `kvs` is a persistent key/value store server and client with synchronous networking over a custom protocol.

mod de;
mod error;
mod kvs_client;
mod kvs_engine;
mod kvs_server;
mod ser;

pub use error::{Error, Result};
pub use kvs_client::KvsClient;
pub use kvs_engine::{KvStore, KvsEngine, SledKvsEngine};
pub use kvs_server::KvsServer;

use serde::{Deserialize, Serialize};
use structopt::StructOpt;

/// A type that represents either set ([`Set`]), get ([`Get`]) or rm ([`Rm`]).
///
/// [`Set`]: Command::Set
/// [`Get`]: Command::Get
/// [`Rm`]: Command::Rm
#[derive(Clone, StructOpt, Serialize, Deserialize, Debug)]
pub enum Command {
    /// Contains the key and value
    Set { key: String, value: String },
    /// Contains the key
    Get { key: String },
    /// Contains the key
    Rm { key: String },
}

/// A type that represents the possible response, which may be either success ([`SuccessSet`], [`SuccessGet`], [`SuccessRm`]) or failure ([`Fail`])
///
/// [`SuccessSet`]: Response::SuccessSet
/// [`SuccessGet`]: Response::SuccessGet
/// [`SuccessRm`]: Response::SuccessRm
/// [`Fail`]: Response::Fail
#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    SuccessSet(),
    /// Contains the success value for get-command, which is None if the key is not found
    SuccessGet(Option<String>),
    SuccessRm(),
    /// Contains the error info
    Fail(String),
}
