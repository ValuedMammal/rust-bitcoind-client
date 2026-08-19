//! [`Batch`]

use alloc::vec::Vec;
use jsonrpc::serde_json::Value;

use crate::Rpc;

/// A group of RPC calls to be sent as a single batch
#[derive(Debug, Clone)]
pub struct Batch {
    /// List of RPCs and associated params
    pub(crate) calls: Vec<(Rpc, Vec<Value>)>,
}

impl Batch {
    /// Create a new [`Batch`] containing the first call
    pub fn new(rpc: Rpc, params: Vec<Value>) -> Self {
        Self {
            calls: vec![(rpc, params)],
        }
    }

    /// Create a new [`Batch`] from an iterator of `(Rpc, params)`
    pub fn from_calls(calls: impl IntoIterator<Item = (Rpc, Vec<Value>)>) -> Option<Self> {
        let mut iter = calls.into_iter();
        let (rpc, params) = iter.next()?;
        let mut batch = Self::new(rpc, params);
        for (rpc, params) in iter {
            batch.push(rpc, params);
        }
        Some(batch)
    }

    /// Add a call to this batch
    pub fn push(&mut self, rpc: Rpc, params: Vec<Value>) {
        self.calls.push((rpc, params));
    }

    /// Returns the count of calls in this batch
    #[allow(clippy::len_without_is_empty)] // Batches can't be empty
    pub fn len(&self) -> usize {
        self.calls.len()
    }

    /// Iterate over calls in this batch
    pub fn calls(&self) -> impl Iterator<Item = &(Rpc, Vec<Value>)> {
        self.calls.iter()
    }
}
