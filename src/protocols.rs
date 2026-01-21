use crate::dim::{resolve_overlays, DimContext, DimOverlay};
use crate::models::CryptoHeader;
use crate::AideError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QidRequest {
    pub srid_alias: String,
    pub payload: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FiidProtocol {
    Qid(QidRequest),
    Qia(String),
    Qed(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SulEntry {
    pub srid_alias: String,
    pub crypto_header: CryptoHeader,
    pub dim: DimContext,
}

pub fn parse_fiid_uri(uri: &str) -> Result<FiidProtocol, AideError> {
    if let Some(rest) = uri.strip_prefix("qid://qca-rb/") {
        let mut iter = rest.splitn(2, '/');
        let alias = iter
            .next()
            .ok_or(AideError::MissingAlias)?
            .to_string();
        let payload = iter.next().unwrap_or("").to_string();
        return Ok(FiidProtocol::Qid(QidRequest { srid_alias: alias, payload }));
    }
    if let Some(rest) = uri.strip_prefix("qia://") {
        return Ok(FiidProtocol::Qia(rest.to_string()));
    }
    if let Some(rest) = uri.strip_prefix("qed://") {
        return Ok(FiidProtocol::Qed(rest.to_string()));
    }
    Err(AideError::InvalidUri)
}

pub fn build_sul_entry(
    uri: &str,
    header_bytes: &[u8],
    overlays: Vec<DimOverlay>,
    now: u64,
) -> Result<SulEntry, AideError> {
    let proto = parse_fiid_uri(uri)?;
    let alias = match proto {
        FiidProtocol::Qid(q) => q.srid_alias,
        _ => return Err(AideError::UnsupportedProtocol),
    };
    let header = CryptoHeader::decode(header_bytes)?;
    let dim_ctx = resolve_overlays(overlays, now);
    Ok(SulEntry {
        srid_alias: alias,
        crypto_header: header,
        dim: dim_ctx,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_qid_uri_extracts_alias_and_payload() {
        let uri = "qid://qca-rb/0xDA12/1.AF01.1700000000.abcd.deadbeef.sigxyz";
        let proto = parse_fiid_uri(uri).expect("parse failed");
        match proto {
            FiidProtocol::Qid(q) => {
                assert_eq!(q.srid_alias, "0xDA12");
                assert!(q.payload.starts_with("1.AF01."));
            }
            _ => panic!("expected qid protocol"),
        }
    }

    #[test]
    fn build_sul_entry_combines_header_and_dim() {
        let uri = "qid://qca-rb/0xDA12/1.AF01.1700000000.abcd.deadbeef.sigxyz";
        let header = CryptoHeader {
            version: 1,
            hook_alg: 1,
            payload_alg: 1,
            ze_type: 1,
            ze_version: 1,
            flags: 0,
        };
        let header_bytes = header.encode();
        let overlays = vec![
            DimOverlay {
                id: "p1".to_string(),
                priority: 5,
                policy: Some("allow".to_string()),
                time_slot: None,
                location: Some("HCM".to_string()),
                expires_at: None,
            },
            DimOverlay {
                id: "p2".to_string(),
                priority: 10,
                policy: Some("deny".to_string()),
                time_slot: Some(2),
                location: None,
                expires_at: None,
            },
        ];
        let entry =
            build_sul_entry(uri, &header_bytes, overlays, 0).expect("build failed");
        assert_eq!(entry.srid_alias, "0xDA12");
        assert_eq!(entry.crypto_header, header);
        assert_eq!(entry.dim.policy.as_deref(), Some("deny"));
        assert_eq!(entry.dim.time_slot, Some(2));
        assert_eq!(entry.dim.location.as_deref(), Some("HCM"));
    }
}

