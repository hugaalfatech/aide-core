use hkdf::Hkdf;
use sha2::Sha256;

use crate::models::CryptoHeader;
use crate::AideError;

impl CryptoHeader {
    pub const ENCODED_LEN: usize = 6;

    pub fn encode(&self) -> [u8; Self::ENCODED_LEN] {
        [
            self.version,
            self.hook_alg,
            self.payload_alg,
            self.ze_type,
            self.ze_version,
            self.flags,
        ]
    }

    pub fn decode(bytes: &[u8]) -> Result<CryptoHeader, AideError> {
        if bytes.len() != Self::ENCODED_LEN {
            return Err(AideError::InvalidHeaderLength);
        }
        Ok(CryptoHeader {
            version: bytes[0],
            hook_alg: bytes[1],
            payload_alg: bytes[2],
            ze_type: bytes[3],
            ze_version: bytes[4],
            flags: bytes[5],
        })
    }
}

pub fn derive_isolated_key(
    master_key: &[u8],
    salt: &[u8],
    purpose: &str,
    length: usize,
) -> Result<Vec<u8>, AideError> {
    let hk = Hkdf::<Sha256>::new(Some(salt), master_key);
    let mut okm = vec![0u8; length];
    hk.expand(purpose.as_bytes(), &mut okm)
        .map_err(|_| AideError::HkdfError)?;
    Ok(okm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crypto_header_roundtrip() {
        let header = CryptoHeader {
            version: 1,
            hook_alg: 2,
            payload_alg: 3,
            ze_type: 4,
            ze_version: 5,
            flags: 6,
        };
        let encoded = header.encode();
        let decoded = CryptoHeader::decode(&encoded).expect("decode failed");
        assert_eq!(header, decoded);
    }

    #[test]
    fn crypto_header_invalid_length() {
        let bytes = [1u8, 2, 3];
        let err = CryptoHeader::decode(&bytes).err().expect("expected error");
        assert!(matches!(err, AideError::InvalidHeaderLength));
    }

    #[test]
    fn hkdf_isolates_purposes() {
        let master = b"master-key";
        let salt = b"salt";
        let k1 = derive_isolated_key(master, salt, "hook", 32).expect("k1");
        let k2 = derive_isolated_key(master, salt, "payload", 32).expect("k2");
        assert_ne!(k1, k2);
    }
}

