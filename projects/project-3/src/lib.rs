//! # kvs
//!
//! `kvs` is a key-value store

mod de;
mod ser;
mod kvs_client;
mod kvs_server;
mod kvs_engine;

pub use kvs_client::KvsClient;
pub use kvs_server::KvsServer;
pub use kvs_engine::{KvsEngine, KvStore, SledKvsEngine};

pub use anyhow::{anyhow, Error, Result};

use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Serialize, Deserialize, Debug)]
pub enum Command {
    Set { key: String, value: String },
    Get { key: String },
    Rm { key: String },
}

pub enum Response {
    Success(String),
    Fail(String),
}

/*
Set(key1, value1)
=3,'3,Set,'4,key1,'6,value1,
Success(value1)
=2,'7,Success,'6,value1,
Fail(msg)
=2,'4,Fail,'3,msg,
*/
