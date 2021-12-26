//! # kvs
//!
//! `kvs` is a key-value store

mod kvs_client;
mod kvs_server;
mod kvs_engine;

pub use kvs_client::KvsClient;
pub use kvs_server::KvsServer;
pub use kvs_engine::{KvsEngine, KvStore, SledKvsEngine};

pub use anyhow::{anyhow, Result};

use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Serialize, Deserialize)]
pub enum Command {
    Set { key: String, value: String },
    Get { key: String },
    Rm { key: String },
}
