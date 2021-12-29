//! # kvs
//!
//! `kvs` is a key-value store

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

#[derive(Clone, StructOpt, Serialize, Deserialize, Debug)]
pub enum Command {
    Set { key: String, value: String },
    Get { key: String },
    Rm { key: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Success(String),
    Fail(String),
}
