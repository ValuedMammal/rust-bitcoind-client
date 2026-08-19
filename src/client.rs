//! [`Client`]

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::future::Future;
use core::sync::atomic::{AtomicUsize, Ordering};

use jsonrpc::serde_json;
use jsonrpc::{Request, Response};
use serde::Deserialize;
use serde_json::{
    json,
    value::{RawValue, Value},
};

use crate::{Batch, Error, Rpc};

/// JSONRPC protocol version.
const JSONRPC: &str = "2.0";

/// Client
#[derive(Debug)]
pub struct Client {
    /// Unique ID of the request, increments atomically for each new request.
    id: AtomicUsize,
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

impl Client {
    /// New.
    pub fn new() -> Self {
        Self {
            id: AtomicUsize::new(0),
        }
    }

    /// Execute the RPC.
    pub fn call<T, E>(
        &self,
        rpc: Rpc,
        params: &[Value],
        send_fn: impl Fn(Request) -> Result<Response, E>,
    ) -> Result<T, Error>
    where
        T: for<'de> Deserialize<'de>,
        E: core::error::Error + Send + Sync + 'static,
    {
        let method = rpc.as_str();
        let raw_value = if params.is_empty() {
            None
        } else {
            Some(serde_json::value::to_raw_value(params)?)
        };
        let request = self.request(method, raw_value.as_deref());
        let request_id = request.id.clone();
        let response = send_fn(request).map_err(Error::transport)?;
        if response.id != request_id {
            return Err(Error::IdMismatch);
        }
        Ok(response.result()?)
    }

    /// Execute a [`Batch`] of RPCs.
    pub fn batch_call<E>(
        &self,
        batch: &Batch,
        send_fn: impl Fn(&[Request]) -> Result<Vec<Response>, E>,
    ) -> Result<Vec<Response>, Error>
    where
        E: core::error::Error + Send + Sync + 'static,
    {
        // Create raw params
        let raw_values: Vec<Option<Box<RawValue>>> = batch
            .calls()
            .map(|(_, params)| {
                if params.is_empty() {
                    Ok(None)
                } else {
                    serde_json::value::to_raw_value(params).map(Some)
                }
            })
            .collect::<Result<_, _>>()?;

        // Create requests
        let requests: Vec<Request> = batch
            .calls()
            .zip(&raw_values)
            .map(|((rpc, _), raw)| self.request(rpc.as_str(), raw.as_deref()))
            .collect();

        // Send batch
        let responses = send_fn(&requests).map_err(Error::transport)?;

        // Reorder responses
        reorder(&requests, responses)
    }

    /// Execute the RPC asynchronously.
    pub async fn call_async<T, E, F, Fut>(
        &self,
        rpc: Rpc,
        params: &[Value],
        send_fn: F,
    ) -> Result<T, Error>
    where
        T: for<'de> Deserialize<'de>,
        E: core::error::Error + Send + Sync + 'static,
        F: Fn(Value) -> Fut,
        Fut: Future<Output = Result<Response, E>>,
    {
        let method = rpc.as_str();
        let raw_value = if params.is_empty() {
            None
        } else {
            Some(serde_json::value::to_raw_value(params)?)
        };
        let request = self.request(method, raw_value.as_deref());
        let request_id = request.id.clone();
        let value = serde_json::to_value(request)?;
        let response = send_fn(value).await.map_err(Error::transport)?;
        if response.id != request_id {
            return Err(Error::IdMismatch);
        }
        Ok(response.result()?)
    }

    /// Execute a [`Batch`] of RPCs asynchronously
    pub async fn batch_call_async<E>(
        &self,
        batch: &Batch,
        send_fn: impl AsyncFn(Value) -> Result<Vec<Response>, E>,
    ) -> Result<Vec<Response>, Error>
    where
        E: core::error::Error + Send + Sync + 'static,
    {
        // Create raw params
        let raw_values: Vec<Option<Box<RawValue>>> = batch
            .calls()
            .map(|(_, params)| {
                if params.is_empty() {
                    Ok(None)
                } else {
                    serde_json::value::to_raw_value(params).map(Some)
                }
            })
            .collect::<Result<_, _>>()?;

        // Create requests
        let requests: Vec<Request> = batch
            .calls()
            .zip(&raw_values)
            .map(|((rpc, _), raw)| self.request(rpc.as_str(), raw.as_deref()))
            .collect();

        // Send batch
        let value = serde_json::to_value(&requests)?;
        let responses = send_fn(value).await.map_err(Error::transport)?;

        // Reorder responses
        reorder(&requests, responses)
    }

    /// Forms the [`Request`] and increments the internal request id.
    fn request<'a>(&self, method: &'a str, params: Option<&'a RawValue>) -> Request<'a> {
        let id = self.id.fetch_add(1, Ordering::Relaxed);
        Request {
            method,
            params,
            id: json!(id),
            jsonrpc: Some(JSONRPC),
        }
    }
}

/// Reorders the responses to match the order of the given requests
///
/// # Errors
///
/// - If requests and responses are of mismatched length
/// - If a response returns an invalid, duplicate, or missing id
fn reorder(requests: &[Request], responses: Vec<Response>) -> Result<Vec<Response>, Error> {
    use alloc::collections::BTreeMap;

    // Check for mismatched lengths
    if responses.len() != requests.len() {
        return Err(Error::JsonRpc(jsonrpc::Error::WrongBatchResponseSize));
    }

    // Responses are already in the correct order
    if requests.iter().zip(&responses).all(|(req, resp)| req.id == resp.id) {
        return Ok(responses);
    }

    let mut map = BTreeMap::new();

    for response in responses {
        let key = response.id.as_u64().ok_or_else(|| {
            Error::JsonRpc(jsonrpc::Error::WrongBatchResponseId(response.id.clone()))
        })?;
        // Check for duplicate response ids
        if let Some(dup) = map.insert(key, response) {
            return Err(Error::JsonRpc(jsonrpc::Error::BatchDuplicateResponseId(dup.id)));
        }
    }

    requests
        .iter()
        .map(|request| {
            let key = request
                .id
                .as_u64()
                .ok_or(Error::JsonRpc(jsonrpc::Error::NonceMismatch))?;
            // Check for missing request id
            map.remove(&key).ok_or(Error::JsonRpc(jsonrpc::Error::NonceMismatch))
        })
        .collect()
}
