#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DimOverlay {
    pub id: String,
    pub priority: u8,
    pub policy: Option<String>,
    pub time_slot: Option<u64>,
    pub location: Option<String>,
    pub expires_at: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DimContext {
    pub policy: Option<String>,
    pub time_slot: Option<u64>,
    pub location: Option<String>,
}

impl DimContext {
    pub fn empty() -> Self {
        DimContext {
            policy: None,
            time_slot: None,
            location: None,
        }
    }
}

pub fn resolve_overlays(mut overlays: Vec<DimOverlay>, now: u64) -> DimContext {
    overlays.retain(|o| match o.expires_at {
        Some(exp) => exp > now,
        None => true,
    });
    overlays.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then_with(|| a.id.cmp(&b.id))
    });
    let mut ctx = DimContext::empty();
    for o in overlays {
        if let Some(ref p) = o.policy {
            ctx.policy = Some(p.clone());
        }
        if let Some(t) = o.time_slot {
            ctx.time_slot = Some(t);
        }
        if let Some(ref loc) = o.location {
            ctx.location = Some(loc.clone());
        }
    }
    ctx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_overlays_applies_priority_and_pruning() {
        let now = 100;
        let overlays = vec![
            DimOverlay {
                id: "low".to_string(),
                priority: 10,
                policy: Some("allow".to_string()),
                time_slot: Some(1),
                location: None,
                expires_at: Some(200),
            },
            DimOverlay {
                id: "high".to_string(),
                priority: 1,
                policy: Some("deny".to_string()),
                time_slot: None,
                location: Some("HCM".to_string()),
                expires_at: Some(200),
            },
            DimOverlay {
                id: "expired".to_string(),
                priority: 0,
                policy: Some("deny".to_string()),
                time_slot: None,
                location: None,
                expires_at: Some(50),
            },
        ];
        let ctx = resolve_overlays(overlays, now);
        assert_eq!(ctx.policy.as_deref(), Some("allow"));
        assert_eq!(ctx.time_slot, Some(1));
        assert_eq!(ctx.location.as_deref(), Some("HCM"));
    }
}

