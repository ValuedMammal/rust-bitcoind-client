//! `simple_http` [`Client`].

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::string::{String, ToString};
use std::vec::Vec;

use bitcoin::{Address, Amount, Block, BlockHash, FeeRate, Transaction, Txid};

use corepc_types::bitcoin;
use corepc_types::model::MempoolEntry;
#[cfg(not(feature = "28_0"))]
use corepc_types::model::{GetBlockHeaderVerbose, GetBlockVerboseOne, GetBlockchainInfo};
use corepc_types::v29;
use jsonrpc::Transport;
use jsonrpc::{serde, serde_json};
use serde::Deserialize;
use serde_json::json;

use crate::Error;
use crate::Rpc::{self, *};
use crate::types::{GetBlockFilter, ImportDescriptorsRequest, ImportDescriptorsResponse};

#[cfg(feature = "28_0")]
pub mod v28;

// RPC Client.
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

    /// Execute the RPC
    fn call<T>(&self, rpc: Rpc, params: &[serde_json::Value]) -> Result<T, Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.inner.call(rpc, params, |request| self.tp.send_request(request))
    }
}

// `bitcoind` RPC methods
impl Client {
    /// `getblockcount`
    pub fn get_block_count(&self) -> Result<u32, Error> {
        self.call(GetBlockCount, &[])
    }

    /// `getbestblockhash`
    pub fn get_best_block_hash(&self) -> Result<BlockHash, Error> {
        Ok(self.call::<String>(GetBestBlockHash, &[])?.parse()?)
    }

    /// `getblockhash`
    pub fn get_block_hash(&self, height: u32) -> Result<BlockHash, Error> {
        let res: String = self.call(GetBlockHash, &[json!(height)])?;
        Ok(res.parse()?)
    }

    /// `getblockfilter`
    pub fn get_block_filter(&self, hash: &BlockHash) -> Result<GetBlockFilter, Error> {
        use crate::types::GetBlockFilterResponse;
        let res: GetBlockFilterResponse = self.call(GetBlockFilter, &[json!(hash)])?;
        res.into_model().map_err(Error::model)
    }

    /// `getblock` (raw)
    pub fn get_block_raw(&self, hash: &BlockHash) -> Result<String, Error> {
        let res: v29::GetBlockVerboseZero = self.call(GetBlock, &[json!(hash), json!(0)])?;
        Ok(res.0)
    }

    /// `getblock`
    pub fn get_block(&self, hash: &BlockHash) -> Result<Block, Error> {
        let res: v29::GetBlockVerboseZero = self.call(GetBlock, &[json!(hash), json!(0)])?;
        res.block().map_err(Error::model)
    }

    /// `getrawmempool`
    pub fn get_raw_mempool(&self) -> Result<Vec<Txid>, Error> {
        let res: v29::GetRawMempool = self.call(GetRawMempool, &[])?;
        Ok(res.into_model().map_err(Error::model)?.0)
    }

    /// `getrawmempool` (verbose = true)
    pub fn get_raw_mempool_verbose(&self) -> Result<BTreeMap<Txid, MempoolEntry>, Error> {
        let res: v29::GetRawMempoolVerbose = self.call(GetRawMempool, &[json!(true)])?;
        Ok(res.into_model().map_err(Error::model)?.0)
    }

    /// `sendtoaddress`
    pub fn send_to_address(&self, address: &Address, amount: Amount) -> Result<Txid, Error> {
        let res: v29::SendToAddress =
            self.call(SendToAddress, &[json!(address), json!(amount.to_btc())])?;
        Ok(res.txid()?)
    }

    /// `getrawtransaction`
    pub fn get_raw_transaction(&self, txid: &Txid) -> Result<Transaction, Error> {
        let res: v29::GetRawTransaction = self.call(GetRawTransaction, &[json!(txid)])?;
        Ok(res.into_model().map_err(Error::model)?.0)
    }

    /// `importdescriptors`
    pub fn import_descriptors(
        &self,
        requests: &[ImportDescriptorsRequest],
    ) -> Result<Vec<ImportDescriptorsResponse>, Error> {
        self.call(ImportDescriptors, &[json!(requests)])
    }

    /// `estimatesmartfee`
    pub fn estimate_smart_fee(&self, blocks: u32) -> Result<FeeRate, Error> {
        let res: v29::EstimateSmartFee = self.call(EstimateSmartFee, &[json!(blocks)])?;
        if let Some(e) = res.errors.and_then(|v| v.first().cloned()) {
            return Err(Error::Response(e));
        }
        let btc_kvb = res
            .fee_rate
            .ok_or(Error::Response("estimatesmartfee returned no feerate".to_string()))?;
        // This is a conservative upper bound on the maximum feerate that is valid by consensus,
        // since there cannot be more than 21M BTC in fees per 1Mb block.
        if btc_kvb > Amount::MAX_MONEY.to_btc() / 1000.0 {
            return Err(Error::Response(format!("invalid feerate: {btc_kvb} BTC/kvB")));
        }
        // 1 sat/vb = 0.00001000 btc/kvb * 10^8 sat/btc * 0.25 wu/sat = 250 sat/kwu
        let sat_kwu = (btc_kvb * 25_000_000.0).round() as u64;

        Ok(FeeRate::from_sat_per_kwu(sat_kwu))
    }
}

#[cfg(not(feature = "28_0"))]
impl Client {
    /// `getblockchaininfo`.
    pub fn get_blockchain_info(&self) -> Result<GetBlockchainInfo, Error> {
        let res: v29::GetBlockchainInfo = self.call(GetBlockchainInfo, &[])?;
        res.into_model().map_err(Error::model)
    }

    /// `getblockheader` (verbose)
    pub fn get_block_header_verbose(
        &self,
        hash: &BlockHash,
    ) -> Result<GetBlockHeaderVerbose, Error> {
        let res: v29::GetBlockHeaderVerbose = self.call(GetBlockHeader, &[json!(hash)])?;
        res.into_model().map_err(Error::model)
    }

    /// `getblock`
    pub fn get_block_verbose(&self, hash: &BlockHash) -> Result<GetBlockVerboseOne, Error> {
        let res: v29::GetBlockVerboseOne = self.call(GetBlock, &[json!(hash), json!(1)])?;
        res.into_model().map_err(Error::model)
    }

    /// `getdescriptorinfo`
    pub fn get_descriptor_info(&self, descriptor: &str) -> Result<v29::GetDescriptorInfo, Error> {
        self.call(GetDescriptorInfo, &[json!(descriptor)])
    }
}
