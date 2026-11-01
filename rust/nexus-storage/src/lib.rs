//! Simple JSON-backed config store (in-memory + serialize).

use nexus_domain::ModuleConfig;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub modules: BTreeMap<String, ModuleConfig>,
    pub diagnostic_mode: bool,
}

impl AppConfig {
    pub fn new() -> Self {
        Self {
            diagnostic_mode: true,
            ..Default::default()
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_domain::{ModuleConfig, RiskLevel};

    #[test]
    fn roundtrip_json() {
        let mut cfg = AppConfig::new();
        cfg.modules.insert(
            "gms".into(),
            ModuleConfig {
                name: "GMS Guard".into(),
                enabled: true,
                risk: RiskLevel::Medium,
                description: "protect gms".into(),
            },
        );
        let json = cfg.to_json().unwrap();
        let back = AppConfig::from_json(&json).unwrap();
        assert_eq!(back.modules.len(), 1);
        assert!(back.diagnostic_mode);
    }
}