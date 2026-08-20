//! [`Batch`]

use alloc::borrow::Cow;
use alloc::vec::Vec;
use jsonrpc::serde_json::Value;

/// A group of one or more RPC calls to be sent as a single batch.
#[derive(Debug, Clone)]
pub struct Batch {
    /// List of method names and associated params
    pub(crate) calls: Vec<(Cow<'static, str>, Vec<Value>)>,
}

impl Batch {
    /// Create a new [`Batch`] containing the first call.
    ///
    /// `method` accepts a variant of [`Rpc`](crate::Rpc) or a `'static str`. To use a
    /// non-`'static` `&str`, convert it to an owned `String` first.
    pub fn new(method: impl Into<Cow<'static, str>>, params: Vec<Value>) -> Self {
        Self {
            calls: vec![(method.into(), params)],
        }
    }

    /// Create a new [`Batch`] from an iterator of `(method, params)`
    pub fn from_calls<M>(calls: impl IntoIterator<Item = (M, Vec<Value>)>) -> Option<Self>
    where
        M: Into<Cow<'static, str>>,
    {
        let mut iter = calls.into_iter();
        let (method, params) = iter.next()?;
        let mut batch = Self::new(method, params);
        for (method, params) in iter {
            batch.push(method, params);
        }
        Some(batch)
    }

    /// Add a call to this batch
    pub fn push(&mut self, method: impl Into<Cow<'static, str>>, params: Vec<Value>) {
        self.calls.push((method.into(), params));
    }

    /// Returns the count of calls in this batch
    #[allow(clippy::len_without_is_empty)] // Batches can't be empty
    pub fn len(&self) -> usize {
        self.calls.len()
    }

    /// Iterate over calls in this batch
    pub fn calls(&self) -> impl Iterator<Item = (&str, &[Value])> {
        self.calls
            .iter()
            .map(|(method, params)| (method.as_ref(), params.as_slice()))
    }
}
