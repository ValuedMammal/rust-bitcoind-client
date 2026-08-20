//! `simple_http` [`Client`].

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::string::{String, ToString};
use std::vec::Vec;

use bitcoin::{Address, Amount, Block, BlockHash, FeeRate, Transaction, Txid, block::Header};
use corepc_types::bitcoin;
use corepc_types::model::{self, GetBlockHeaderVerbose, GetBlockVerboseOne, MempoolEntry};
use corepc_types::v31;
use jsonrpc::{Response, Transport};
use jsonrpc::{serde, serde_json};
use serde::Deserialize;
use serde_json::json;

use crate::Rpc::*;
use crate::types::{GetBlockFilter, ImportDescriptorsRequest, ImportDescriptorsResponse};
use crate::{Batch, Error};

/// RPC Client.
#[derive(Debug)]
pub struct Client {
    /// The inner JSON-RPC client.
    inner: crate::Client,
    /// Simple HTTP transport
    tp: jsonrpc::simple_http::SimpleHttpTransport,
}

/// The way of authenticating to the JSON-RPC server.
#[derive(Debug, Clone)]
pub enum Auth {
    /// User and password
    UserPass(String, String),
    /// Path to cookie file
    CookieFile(PathBuf),
}

impl Auth {
    /// Get the user:pass credentials from this [`Auth`].
    fn get_user_pass(self) -> Result<(String, String), Error> {
        match self {
            Auth::UserPass(user, pass) => Ok((user, pass)),
            Auth::CookieFile(path) => {
                let line = BufReader::new(File::open(path)?)
                    .lines()
                    .next()
                    .ok_or(Error::InvalidCookieFile)??;
                let colon = line.find(':').ok_or(Error::InvalidCookieFile)?;

                Ok((line[..colon].to_string(), line[colon + 1..].to_string()))
            }
        }
    }
}

impl Client {
    /// Creates a `simple_http` client with `url` and `auth`.
    ///
    /// This can fail if we are unable to read the configured [`Auth::CookieFile`].
    pub fn new(url: &str, auth: Auth) -> Result<Self, Error> {
        let (user, pass) = auth.get_user_pass()?;
        Ok(Self::new_user_pass(url, user, Some(pass)))
    }

    /// Creates a `simple_http` client with `user` and `pass`.
    pub fn new_user_pass(url: &str, user: String, pass: Option<String>) -> Self {
        let tp = jsonrpc::simple_http::Builder::new()
            .url(url)
            .expect("URL check failed")
            .timeout(std::time::Duration::from_secs(15))
            .auth(user, pass)
            .build();

        Self {
            inner: crate::Client::new(),
            tp,
        }
    }

    /// Creates a `simple_http` client with `cookie` authentication.
    pub fn new_cookie_auth(url: &str, cookie: String) -> Self {
        let tp = jsonrpc::simple_http::Builder::new()
            .url(url)
            .expect("URL check failed")
            .timeout(std::time::Duration::from_secs(15))
            .cookie_auth(cookie)
            .build();

        Self {
            inner: crate::Client::new(),
            tp,
        }
    }

    /// Execute the RPC.
    ///
    /// Accepts an [`Rpc`](crate::Rpc) variant as the `method`, or any method name string.
    pub fn send<T>(&self, method: impl AsRef<str>, params: &[serde_json::Value]) -> Result<T, Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.inner
            .send(method, params, |request| self.tp.send_request(request))
    }

    /// Sends a heterogeneous batch of RPCs
    pub fn send_batch(&self, batch: &Batch) -> Result<Vec<Response>, Error> {
        self.inner.send_batch(batch, |request| self.tp.send_batch(request))
    }

    /// Sends multiple requests of varying parameters to the given [`Rpc`] method.
    ///
    /// For each request returns the result of deserializing into the specified type `T`.
    /// To send a heterogeneous batch of RPCs, see [`send_batch`](Client::send_batch).
    ///
    /// This is typically used with RPCs that require parameters, for example requesting
    /// block hashes over an array of heights.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use jsonrpc::serde_json::json;
    /// # use bitcoind_client::Rpc;
    /// # use bitcoind_client::simple_http::Client;
    /// use corepc_types::v31::GetBlockHash;
    /// # let client = Client::new_user_pass("", String::new(), None);
    ///
    /// let heights = [1u32, 2, 3];
    /// let params = heights.iter().map(|h| vec![json!(h)]);
    /// for result in client.send_many::<GetBlockHash>(Rpc::GetBlockHash, params)? {
    ///     assert!(matches!(result, Ok(GetBlockHash(..))));
    /// }
    /// # <Result<_, bitcoind_client::Error>>::Ok(())
    /// ```
    pub fn send_many<T>(
        &self,
        method: impl Into<Cow<'static, str>>,
        params: impl IntoIterator<Item = Vec<serde_json::Value>>,
    ) -> Result<Vec<Result<T, Error>>, Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        let batch = Batch::from_calls(core::iter::repeat(method.into()).zip(params))
            .ok_or(Error::JsonRpc(jsonrpc::Error::EmptyBatch))?;
        Ok(self
            .send_batch(&batch)?
            .iter()
            .map(|resp| resp.result::<T>().map_err(Error::JsonRpc))
            .collect())
    }
}

// `bitcoind` RPC methods
impl Client {
    /// `getblockchaininfo`.
    pub fn get_blockchain_info(&self) -> Result<model::GetBlockchainInfo, Error> {
        let res: v31::GetBlockchainInfo = self.send(GetBlockchainInfo, &[])?;
        res.into_model().map_err(Error::model)
    }

    /// `getdescriptorinfo`
    pub fn get_descriptor_info(&self, descriptor: &str) -> Result<v31::GetDescriptorInfo, Error> {
        self.send(GetDescriptorInfo, &[json!(descriptor)])
    }

    /// `getblockcount`
    pub fn get_block_count(&self) -> Result<u32, Error> {
        self.send(GetBlockCount, &[])
    }

    /// `getbestblockhash`
    pub fn get_best_block_hash(&self) -> Result<BlockHash, Error> {
        Ok(self.send::<String>(GetBestBlockHash, &[])?.parse()?)
    }

    /// `getblockhash`
    pub fn get_block_hash(&self, height: u32) -> Result<BlockHash, Error> {
        let res: String = self.send(GetBlockHash, &[json!(height)])?;
        Ok(res.parse()?)
    }

    /// `getblockheader`
    pub fn get_block_header(&self, hash: &BlockHash) -> Result<Header, Error> {
        let res: v31::GetBlockHeader = self.send(GetBlockHeader, &[json!(hash), json!(false)])?;
        Ok(res.into_model().map_err(Error::model)?.0)
    }

    /// `getblockheader` (verbose)
    pub fn get_block_header_verbose(
        &self,
        hash: &BlockHash,
    ) -> Result<GetBlockHeaderVerbose, Error> {
        let res: v31::GetBlockHeaderVerbose = self.send(GetBlockHeader, &[json!(hash)])?;
        res.into_model().map_err(Error::model)
    }

    /// `getblockfilter`
    pub fn get_block_filter(&self, hash: &BlockHash) -> Result<GetBlockFilter, Error> {
        use crate::types::GetBlockFilterResponse;
        let res: GetBlockFilterResponse = self.send(GetBlockFilter, &[json!(hash)])?;
        res.into_model().map_err(Error::model)
    }

    /// `getblock` (raw)
    pub fn get_block_raw(&self, hash: &BlockHash) -> Result<String, Error> {
        let res: v31::GetBlockVerboseZero = self.send(GetBlock, &[json!(hash), json!(0)])?;
        Ok(res.0)
    }

    /// `getblock`
    pub fn get_block(&self, hash: &BlockHash) -> Result<Block, Error> {
        let res: v31::GetBlockVerboseZero = self.send(GetBlock, &[json!(hash), json!(0)])?;
        res.block().map_err(Error::model)
    }

    /// `getrawmempool`
    pub fn get_raw_mempool(&self) -> Result<Vec<Txid>, Error> {
        let res: v31::GetRawMempool = self.send(GetRawMempool, &[])?;
        Ok(res.into_model().map_err(Error::model)?.0)
    }

    /// `sendtoaddress`
    pub fn send_to_address(&self, address: &Address, amount: Amount) -> Result<Txid, Error> {
        let res: v31::SendToAddress =
            self.send(SendToAddress, &[json!(address), json!(amount.to_btc())])?;
        Ok(res.txid()?)
    }

    /// `getrawtransaction`
    pub fn get_raw_transaction(&self, txid: &Txid) -> Result<Transaction, Error> {
        let res: v31::GetRawTransaction = self.send(GetRawTransaction, &[json!(txid)])?;
        Ok(res.into_model().map_err(Error::model)?.0)
    }

    /// `importdescriptors`
    pub fn import_descriptors(
        &self,
        requests: &[ImportDescriptorsRequest],
    ) -> Result<Vec<ImportDescriptorsResponse>, Error> {
        self.send(ImportDescriptors, &[json!(requests)])
    }

    /// `estimatesmartfee`
    pub fn estimate_smart_fee(&self, blocks: u32) -> Result<FeeRate, Error> {
        let res: v31::EstimateSmartFee = self.send(EstimateSmartFee, &[json!(blocks)])?;
        if let Some(e) = res.errors.and_then(|v| v.first().cloned()) {
            return Err(Error::Response(e));
        }
        let btc_kvb = res
            .fee_rate
            .ok_or(Error::Response("estimatesmartfee returned no fee_rate".to_string()))?;
        // Reject infinite and negative values
        if !btc_kvb.is_finite() || btc_kvb <= 0.0 {
            return Err(Error::Response(format!("invalid feerate: {btc_kvb} BTC/kvB")));
        }
        // No transaction can pay more BTC/kvB as a fee than the total supply
        if btc_kvb > Amount::MAX_MONEY.to_btc() {
            return Err(Error::Response(format!("invalid feerate: {btc_kvb} BTC/kvB")));
        }
        // 1 sat/vb = 0.00001000 btc/kvb * 10^8 sat/btc * 0.25 wu/sat = 250 sat/kwu
        let sat_kwu = (btc_kvb * 25_000_000.0).round() as u64;

        Ok(FeeRate::from_sat_per_kwu(sat_kwu))
    }
}

// --- v31 compatible APIs

#[cfg(feature = "31_0")]
impl Client {
    /// `getblock` (verbose = 1)
    pub fn get_block_verbose(&self, hash: &BlockHash) -> Result<GetBlockVerboseOne, Error> {
        let res: v31::GetBlockVerboseOne = self.send(GetBlock, &[json!(hash), json!(1)])?;
        res.into_model().map_err(Error::model)
    }

    /// `getrawmempool` (verbose = true)
    pub fn get_raw_mempool_verbose(&self) -> Result<BTreeMap<Txid, MempoolEntry>, Error> {
        let res: v31::GetRawMempoolVerbose = self.send(GetRawMempool, &[json!(true)])?;
        Ok(res.into_model().map_err(Error::model)?.0)
    }
}

// --- v30 compatible APIs

#[cfg(not(feature = "31_0"))]
use corepc_types::v30;

#[cfg(not(feature = "31_0"))]
impl Client {
    /// `getblock` (verbose = 1)
    pub fn get_block_verbose(&self, hash: &BlockHash) -> Result<GetBlockVerboseOne, Error> {
        let res: v30::GetBlockVerboseOne = self.send(GetBlock, &[json!(hash), json!(1)])?;
        res.into_model().map_err(Error::model)
    }

    /// `getrawmempool` (verbose = true)
    pub fn get_raw_mempool_verbose(&self) -> Result<BTreeMap<Txid, MempoolEntry>, Error> {
        let res: v30::GetRawMempoolVerbose = self.send(GetRawMempool, &[json!(true)])?;
        Ok(res.into_model().map_err(Error::model)?.0)
    }
}
