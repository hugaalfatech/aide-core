#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CryptoHeader {
    pub version: u8,
    pub hook_alg: u8,
    pub payload_alg: u8,
    pub ze_type: u8,
    pub ze_version: u8,
    pub flags: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fiid {
    pub srid_alias: String,
    pub crypto_header: CryptoHeader,
    pub qpid_payload: Vec<u8>,
}

