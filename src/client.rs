//! [`Client`]

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};

use jsonrpc::serde_json;
use jsonrpc::{Request, Response};
use serde::Deserialize;
use serde_json::{
    json,
    value::{RawValue, Value, to_raw_value},
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
    pub fn send<T, E>(
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
            Some(to_raw_value(params)?)
        };
        let request = self.request(method, raw_value.as_deref());
        let request_id = request.id.clone();
        let response = send_fn(request).map_err(Error::transport)?;
        if response.jsonrpc != Some(JSONRPC.into()) {
            return Err(Error::JsonRpc(jsonrpc::Error::VersionMismatch));
        }
        if response.id != request_id {
            return Err(Error::JsonRpc(jsonrpc::Error::NonceMismatch));
        }
        Ok(response.result()?)
    }

    /// Execute a [`Batch`] of RPCs.
    pub fn send_batch<E>(
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
                    to_raw_value(params).map(Some)
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
    pub async fn send_async<T, E>(
        &self,
        rpc: Rpc,
        params: &[Value],
        send_fn: impl AsyncFn(&Request) -> Result<Response, E>,
    ) -> Result<T, Error>
    where
        T: for<'de> Deserialize<'de>,
        E: core::error::Error + Send + Sync + 'static,
    {
        let method = rpc.as_str();
        let raw_value = if params.is_empty() {
            None
        } else {
            Some(to_raw_value(params)?)
        };
        let request = self.request(method, raw_value.as_deref());
        let request_id = request.id.clone();
        let response = send_fn(&request).await.map_err(Error::transport)?;
        if response.jsonrpc != Some(JSONRPC.into()) {
            return Err(Error::JsonRpc(jsonrpc::Error::VersionMismatch));
        }
        if response.id != request_id {
            return Err(Error::JsonRpc(jsonrpc::Error::NonceMismatch));
        }
        Ok(response.result()?)
    }

    /// Execute a [`Batch`] of RPCs asynchronously
    pub async fn send_batch_async<E>(
        &self,
        batch: &Batch,
        send_fn: impl AsyncFn(&[Request]) -> Result<Vec<Response>, E>,
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
                    to_raw_value(params).map(Some)
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
        let responses = send_fn(&requests).await.map_err(Error::transport)?;

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
        if response.jsonrpc != Some(JSONRPC.into()) {
            return Err(Error::JsonRpc(jsonrpc::Error::VersionMismatch));
        }
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

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use super::*;

    fn req(id: u64) -> Request<'static> {
        Request {
            method: "test",
            params: None,
            id: json!(id),
            jsonrpc: Some(JSONRPC),
        }
    }

    fn resp(id: Value) -> Response {
        Response {
            result: Some(to_raw_value(&true).unwrap()),
            error: None,
            id,
            jsonrpc: Some(JSONRPC.to_string()),
        }
    }

    #[test]
    fn in_order_responses_return_unchanged() {
        let requests = vec![req(0), req(1), req(2)];
        // Ids already line up positionally, so they should be returned unchanged.
        let responses = vec![resp(json!(0)), resp(json!(1)), resp(json!(2))];
        let result = reorder(&requests, responses).unwrap();
        let ids: Vec<_> = result.iter().map(|r| r.id.as_u64().unwrap()).collect();
        assert_eq!(ids, [0, 1, 2]);
    }

    #[test]
    fn responses_are_reordered() {
        let requests = vec![req(0), req(1), req(2)];
        let responses = vec![resp(json!(2)), resp(json!(0)), resp(json!(1))];
        let result = reorder(&requests, responses).unwrap();
        let ids: Vec<_> = result.iter().map(|r| r.id.as_u64().unwrap()).collect();
        assert_eq!(ids, [0, 1, 2]);
    }

    #[test]
    fn null_response_id_errors() {
        let requests = vec![req(0), req(1)];
        let responses = vec![resp(json!(1)), resp(Value::Null)];
        let err = reorder(&requests, responses).unwrap_err();
        assert!(matches!(
            err,
            Error::JsonRpc(jsonrpc::Error::WrongBatchResponseId(id))
            if id == Value::Null
        ));
    }

    #[test]
    fn string_response_id_errors() {
        let requests = vec![req(0), req(1)];
        let responses = vec![resp(json!(1)), resp(json!("0"))];
        let err = reorder(&requests, responses).unwrap_err();
        assert!(matches!(
            err,
            Error::JsonRpc(jsonrpc::Error::WrongBatchResponseId(id))
            if id == json!("0")
        ));
    }

    #[test]
    fn duplicate_response_ids_error() {
        let requests = vec![req(0), req(1)];
        let responses = vec![resp(json!(0)), resp(json!(0))];
        let err = reorder(&requests, responses).unwrap_err();
        assert!(matches!(
            err,
            Error::JsonRpc(jsonrpc::Error::BatchDuplicateResponseId(dup_id))
            if dup_id == json!(0)
        ));
    }

    #[test]
    fn unrequested_response_ids_error() {
        let requests = vec![req(0), req(1)];
        // Correct length and no duplicates, but neither id was ever requested.
        let responses = vec![resp(json!(7)), resp(json!(8))];
        let err = reorder(&requests, responses).unwrap_err();
        assert!(matches!(err, Error::JsonRpc(jsonrpc::Error::NonceMismatch)));
    }

    #[test]
    fn mismatched_response_count_errors() {
        let requests = vec![req(0), req(1)];
        let responses = vec![resp(json!(0))];
        let err = reorder(&requests, responses).unwrap_err();
        assert!(matches!(err, Error::JsonRpc(jsonrpc::Error::WrongBatchResponseSize)));
    }
}
