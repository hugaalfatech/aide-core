pub mod crypto;
pub mod dim;
pub mod models;
pub mod protocols;
pub mod registry;

pub use crate::crypto::derive_isolated_key;
pub use crate::dim::{resolve_overlays, DimContext, DimOverlay};
pub use crate::models::{CryptoHeader, Fiid};
pub use crate::protocols::{build_sul_entry, parse_fiid_uri, FiidProtocol, QidRequest, SulEntry};
pub use crate::registry::SridRegistry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AideError {
    InvalidCryptoHeader,
    InvalidHeaderLength,
    InvalidUri,
    UnsupportedProtocol,
    MissingAlias,
    HkdfError,
}

