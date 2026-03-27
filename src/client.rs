use alloc::string::ToString;
use core::future::Future;
use core::sync::atomic::{AtomicUsize, Ordering};

use jsonrpc::serde_json;
use jsonrpc::{Request, Response};
use serde::Deserialize;
use serde_json::{
    json,
    value::{RawValue, Value},
};

use crate::Error;
use crate::Rpc;

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
        let method = rpc.to_string();
        let raw_value = if params.is_empty() {
            None
        } else {
            Some(serde_json::value::to_raw_value(params)?)
        };
        let request = self.request(&method, raw_value.as_deref());
        let request_id = request.id.clone();
        let response = send_fn(request).map_err(Error::transport)?;
        if response.id != request_id {
            return Err(Error::IdMismatch);
        }
        Ok(response.result()?)
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
        let method = rpc.to_string();
        let raw_value = if params.is_empty() {
            None
        } else {
            Some(serde_json::value::to_raw_value(params)?)
        };
        let request = self.request(&method, raw_value.as_deref());
        let request_id = request.id.clone();
        let value = serde_json::to_value(request)?;
        let response = send_fn(value).await.map_err(Error::transport)?;
        if response.id != request_id {
            return Err(Error::IdMismatch);
        }
        Ok(response.result()?)
    }

    /// Forms the [`Request`] and increments the internal request id id.
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
