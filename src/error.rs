use alloc::boxed::Box;

use jsonrpc::serde_json;

/// Error
#[derive(Debug)]
pub enum Error {
    /// mismatched IDs
    IdMismatch,
    /// Invalid cookie file
    InvalidCookieFile,
    /// `std::io`
    #[cfg(feature = "std")]
    Io(std::io::Error),
    /// `jsonrpc`
    JsonRpc(jsonrpc::Error),
    /// error modeling a `corepc` type
    #[cfg(feature = "simple-http")]
    Model(Box<dyn core::error::Error + Send + Sync + 'static>),
    /// `serde_json`
    SerdeJson(serde_json::Error),
    /// Error returned from RPC
    Returned(alloc::string::String),
    /// Bitcoin address parsing error
    #[cfg(feature = "simple-http")]
    BitcoinAddressParse(corepc_types::bitcoin::address::ParseError),
    /// Bitcoin hex parsing error
    #[cfg(feature = "simple-http")]
    BitcoinHexParse(corepc_types::bitcoin::hex::HexToArrayError),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::IdMismatch => write!(f, "request id mismatch"),
            Self::InvalidCookieFile => write!(f, "invaild cookie file"),
            #[cfg(feature = "std")]
            Self::Io(e) => write!(f, "{e}"),
            #[cfg(feature = "simple-http")]
            Self::Model(e) => write!(f, "{e}"),
            Self::JsonRpc(e) => write!(f, "{e}"),
            Self::SerdeJson(e) => write!(f, "{e}"),
            Self::Returned(s) => write!(f, "{s}"),
            #[cfg(feature = "simple-http")]
            Self::BitcoinAddressParse(e) => write!(f, "{e}"),
            #[cfg(feature = "simple-http")]
            Self::BitcoinHexParse(e) => write!(f, "{e}"),
        }
    }
}

impl core::error::Error for Error {}

impl Error {
    /// Convert `e` to a [`Error::Model`] error.
    #[cfg(feature = "simple-http")]
    pub(crate) fn model<E>(e: E) -> Self
    where
        E: core::error::Error + Send + Sync + 'static,
    {
        Self::Model(Box::new(e))
    }

    /// Convert `e` to a [`jsonrpc::Error::Transport`] error.
    pub(crate) fn transport<E>(e: E) -> Self
    where
        E: core::error::Error + Send + Sync + 'static,
    {
        Self::JsonRpc(jsonrpc::Error::Transport(Box::new(e)))
    }
}

macro_rules! impl_error_from {
    ( $to:ident, $from:ty ) => {
        impl From<$from> for Error {
            fn from(e: $from) -> Self {
                Self::$to(e)
            }
        }
    };
}

impl_error_from!(SerdeJson, serde_json::Error);
impl_error_from!(JsonRpc, jsonrpc::Error);
#[cfg(feature = "std")]
impl_error_from!(Io, std::io::Error);
#[cfg(feature = "simple-http")]
impl_error_from!(
    BitcoinAddressParse,
    corepc_types::bitcoin::address::ParseError
);
#[cfg(feature = "simple-http")]
impl_error_from!(BitcoinHexParse, corepc_types::bitcoin::hex::HexToArrayError);
