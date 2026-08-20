//! [`Rpc`] methods

use alloc::borrow::Cow;
use core::fmt::{self, Display};

// RPC methods go here. These names MUST match the name of the RPC method (when converted to lowercase).
// See <https://bitcoincore.org/en/doc/> for details.
/// RPCs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Rpc {
    /// `getbestblockhash`
    GetBestBlockHash,
    /// `getblockchaininfo`
    GetBlockchainInfo,
    /// `getblockhash`
    GetBlockHash,
    /// `getblockcount`
    GetBlockCount,
    /// `getblock`
    GetBlock,
    /// `getblockheader`
    GetBlockHeader,
    /// `getblockfilter`
    GetBlockFilter,
    /// `getdescriptorinfo`
    GetDescriptorInfo,
    /// `getrawmempool`
    GetRawMempool,
    /// `sendtoaddress`
    SendToAddress,
    /// `getrawtransaction`
    GetRawTransaction,
    /// `importdescriptors`
    ImportDescriptors,
    /// `estimatesmartfee`
    EstimateSmartFee,
}

impl Rpc {
    /// Returns the RPC method name string
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::GetBestBlockHash => "getbestblockhash",
            Self::GetBlockchainInfo => "getblockchaininfo",
            Self::GetBlockHash => "getblockhash",
            Self::GetBlockCount => "getblockcount",
            Self::GetBlock => "getblock",
            Self::GetBlockHeader => "getblockheader",
            Self::GetBlockFilter => "getblockfilter",
            Self::GetDescriptorInfo => "getdescriptorinfo",
            Self::GetRawMempool => "getrawmempool",
            Self::SendToAddress => "sendtoaddress",
            Self::GetRawTransaction => "getrawtransaction",
            Self::ImportDescriptors => "importdescriptors",
            Self::EstimateSmartFee => "estimatesmartfee",
        }
    }
}

impl Display for Rpc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AsRef<str> for Rpc {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<Rpc> for Cow<'static, str> {
    // Borrowed, so converting a known `Rpc` variant never allocates.
    fn from(rpc: Rpc) -> Self {
        Cow::Borrowed(rpc.as_str())
    }
}

#[cfg(test)]
mod test {
    use super::Rpc;
    use alloc::string::ToString;

    #[test]
    fn test_rpc_method_names() {
        for (rpc, name) in [
            (Rpc::GetBestBlockHash, "getbestblockhash"),
            (Rpc::GetBlockchainInfo, "getblockchaininfo"),
            (Rpc::GetBlockHash, "getblockhash"),
            (Rpc::GetBlockCount, "getblockcount"),
            (Rpc::GetBlock, "getblock"),
            (Rpc::GetBlockHeader, "getblockheader"),
            (Rpc::GetBlockFilter, "getblockfilter"),
            (Rpc::GetDescriptorInfo, "getdescriptorinfo"),
            (Rpc::GetRawMempool, "getrawmempool"),
            (Rpc::SendToAddress, "sendtoaddress"),
            (Rpc::GetRawTransaction, "getrawtransaction"),
            (Rpc::ImportDescriptors, "importdescriptors"),
            (Rpc::EstimateSmartFee, "estimatesmartfee"),
        ] {
            let result = rpc.to_string();
            assert_eq!(result, name, "expected Rpc name {name}, got {result}");
        }
    }
}
