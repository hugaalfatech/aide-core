use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct SridRegistry {
    map: HashMap<String, String>,
}

impl SridRegistry {
    pub fn new() -> SridRegistry {
        SridRegistry {
            map: HashMap::new(),
        }
    }

    pub fn with_defaults() -> SridRegistry {
        let mut reg = SridRegistry::new();
        reg.insert(
            "0xDA12".to_string(),
            "SHARD_12.DA_PLANE.ROCKBASE.VN.v1".to_string(),
        );
        reg
    }

    pub fn insert(&mut self, alias: String, descriptor: String) {
        self.map.insert(alias, descriptor);
    }

    pub fn resolve(&self, alias: &str) -> Option<String> {
        self.map.get(alias).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_default_alias() {
        let reg = SridRegistry::with_defaults();
        let v = reg.resolve("0xDA12").expect("alias missing");
        assert!(v.starts_with("SHARD_12."));
    }
}

