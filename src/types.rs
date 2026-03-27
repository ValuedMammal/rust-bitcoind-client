use alloc::string::String;
use alloc::vec::Vec;

use bitcoin::bip158;
use corepc_types::bitcoin;
use corepc_types::v19::GetBlockFilterError;
use serde::{Deserialize, Serialize};

/// Response to `getblockfilter`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(crate) struct GetBlockFilterResponse {
    /// `bip158` block filter (hex).
    pub filter: String,
    /// `bip158` filter header.
    pub header: bip158::FilterHeader,
}

impl GetBlockFilterResponse {
    /// Into model.
    pub fn into_model(self) -> Result<GetBlockFilter, GetBlockFilterError> {
        use bitcoin::hex::FromHex;
        let GetBlockFilterResponse { filter, header } = self;
        let data = <Vec<u8> as FromHex>::from_hex(&filter).map_err(GetBlockFilterError::Filter)?;
        let filter = bip158::BlockFilter::new(&data);

        Ok(GetBlockFilter { filter, header })
    }
}

/// Response to `getblockfilter`.
#[derive(Debug, Clone, PartialEq)]
pub struct GetBlockFilter {
    /// `bip158` block filter.
    pub filter: bip158::BlockFilter,
    /// `bip158` filter header.
    pub header: bip158::FilterHeader,
}

/// Request of `importdescriptors`.
#[derive(Debug, Clone, Serialize, Default)]
pub struct ImportDescriptorsRequest {
    /// (required) Descriptor to import.
    pub desc: String,
    /// Set this descriptor to be the active descriptor for the corresponding output type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// If a ranged descriptor is used, this specifies the range (in the form \[begin,end\]) to import.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<(usize, usize)>,
    /// The next index from which to generate addresses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_index: Option<usize>,
    /// (required) Time from which to start rescanning the blockchain for this descriptor, in UNIX epoch time.
    // Use the string "now" to substitute the current synced blockchain time.
    pub timestamp: u64,
    /// Whether matching outputs should be treated as not incoming payments (e.g. change).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal: Option<bool>,
    /// Label to assign to the address, only allowed with `internal = false`. Disabled for ranged descriptors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Response of `importdescriptors`.
#[derive(Debug, Clone, Deserialize)]
pub struct ImportDescriptorsResponse {
    /// Whether the import was successful.
    pub success: bool,
    /// Warnings.
    #[serde(default)]
    pub warnings: Option<Vec<String>>,
    /// Error.
    pub error: Option<ImportDescriptorsError>,
}

/// Error for `importdescriptors`.
#[derive(Debug, Clone, Deserialize)]
pub struct ImportDescriptorsError {
    /// error code.
    pub code: i32,
    /// error message.
    pub message: String,
}
