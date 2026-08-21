//! `bitcoind_client`.

#![no_std]

#[macro_use]
#[cfg(feature = "std")]
extern crate std;

#[macro_use]
#[allow(unused_imports)]
extern crate alloc;

mod batch;
mod client;
mod error;
#[cfg(feature = "simple-http")]
pub mod http;
mod rpc;
pub use batch::*;
pub use client::*;
pub use error::*;
pub use rpc::*;
#[cfg(feature = "simple-http")]
pub mod types;

pub use corepc_types;
pub use jsonrpc;
