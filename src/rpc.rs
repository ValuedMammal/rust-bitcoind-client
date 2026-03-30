//! [`Rpc`] methods

macro_rules! impl_rpc_methods {
    ( $($name:ident,)+ ) => {
        /// RPCs
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[allow(missing_docs)]
        pub enum Rpc {
            $(
                $name,
            )+
        }

        impl core::fmt::Display for Rpc {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                let s = match self {
                    $(
                        Self::$name => stringify!($name).to_lowercase(),
                    )+
                };

                f.write_str(&s)
            }
        }
    }
}

// RPC methods go here. These names MUST match the name of the RPC method (when converted to lowercase).
// See <https://bitcoincore.org/en/doc/> for details.
impl_rpc_methods!(
    GetBestBlockHash,
    GetBlockchainInfo,
    GetBlockHash,
    GetBlockCount,
    GetBlock,
    GetBlockHeader,
    GetBlockFilter,
    GetDescriptorInfo,
    GetRawMempool,
    SendToAddress,
    GetRawTransaction,
    ImportDescriptors,
    EstimateSmartFee,
);
