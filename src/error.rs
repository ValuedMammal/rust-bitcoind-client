use alloc::boxed::Box;

use jsonrpc::serde_json;

/// Error
#[derive(Debug)]
pub enum Error {
    /// Bitcoin hex parsing error
    #[cfg(feature = "simple-http")]
    DecodeHex(corepc_types::bitcoin::hex::HexToArrayError),
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
    /// parse integer
    #[cfg(feature = "simple-http")]
    ParseInt(core::num::ParseIntError),
    /// Error returned in the response
    Response(alloc::string::String),
    /// `serde_json`
    SerdeJson(serde_json::Error),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            #[cfg(feature = "simple-http")]
            Self::DecodeHex(e) => write!(f, "{e}"),
            Self::IdMismatch => write!(f, "request id mismatch"),
            Self::InvalidCookieFile => write!(f, "invaild cookie file"),
            #[cfg(feature = "std")]
            Self::Io(e) => write!(f, "{e}"),
            #[cfg(feature = "simple-http")]
            Self::Model(e) => write!(f, "{e}"),
            #[cfg(feature = "simple-http")]
            Self::ParseInt(e) => write!(f, "{e}"),
            Self::JsonRpc(e) => write!(f, "{e}"),
            Self::SerdeJson(e) => write!(f, "{e}"),
            Self::Response(s) => write!(f, "{s}"),
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

#[cfg(feature = "simple-http")]
impl_error_from!(DecodeHex, corepc_types::bitcoin::hex::HexToArrayError);
#[cfg(feature = "std")]
impl_error_from!(Io, std::io::Error);
impl_error_from!(JsonRpc, jsonrpc::Error);
#[cfg(feature = "simple-http")]
impl_error_from!(ParseInt, core::num::ParseIntError);
impl_error_from!(SerdeJson, serde_json::Error);
