//! RPC methods

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

// All implemented RPCs
impl_rpc_methods!(
    GetBestBlockHash,
    GetBlockchainInfo,
    GetBlockHash,
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
