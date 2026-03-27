//! Client methods which support Bitcoin Core version 28.0.

use bitcoin::BlockHash;
use corepc_types::bitcoin;
use corepc_types::model::{GetBlockHeaderVerbose, GetBlockVerboseOne, GetBlockchainInfo};
use corepc_types::v28;

use super::Client;
use super::json;
use crate::Error;
use crate::Rpc::*;

impl Client {
    /// `getblockchaininfo`
    pub fn get_blockchain_info(&self) -> Result<GetBlockchainInfo, Error> {
        let res: v28::GetBlockchainInfo = self.call(GetBlockchainInfo, &[])?;
        Ok(res.into_model().unwrap())
    }

    /// `getblockheader` (verbose)
    pub fn get_block_header_verbose(
        &self,
        hash: &BlockHash,
    ) -> Result<GetBlockHeaderVerbose, Error> {
        let res: v28::GetBlockHeaderVerbose = self.call(GetBlockHeader, &[json!(hash)])?;
        Ok(res.into_model().unwrap())
    }

    /// `getblock` (verbosity = 1).
    pub fn get_block_verbose(&self, hash: &BlockHash) -> Result<GetBlockVerboseOne, Error> {
        let res: v28::GetBlockVerboseOne = self.call(GetBlock, &[json!(hash), json!(1)])?;
        Ok(res.into_model().unwrap())
    }

    /// `getdescriptorinfo`
    pub fn get_descriptor_info(&self, descriptor: &str) -> Result<v28::GetDescriptorInfo, Error> {
        self.call(GetDescriptorInfo, &[json!(descriptor)])
    }
}
