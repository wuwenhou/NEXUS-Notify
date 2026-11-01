//! Bridge surface for Kotlin/Android integration.
//!
//! UniFFI bindings are enabled via the `uniffi` feature. The core API is
//! always available as plain Rust for host-side testing.

use nexus_core::NexusCore;
use nexus_domain::DeviceProfile;
use nexus_policy::SystemSnapshot;

pub struct NexusEngine {
    core: NexusCore,
}

impl NexusEngine {
    pub fn new() -> Self {
        Self {
            core: NexusCore::new(),
        }
    }

    pub fn set_device(&mut self, profile: DeviceProfile) {
        self.core.set_device(profile);
    }

    pub fn is_supported(&self) -> bool {
        self.core.is_supported()
    }

    pub fn evaluate(&self, snap: &SystemSnapshot) -> nexus_policy::EvaluationResult {
        self.core.evaluate(snap)
    }
}

impl Default for NexusEngine {
    fn default() -> Self {
        Self::new()
    }
}

// Re-export policy type for convenience.
pub use nexus_policy::EvaluationResult;

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_domain::{CapabilityState, DeviceProfile, ProtectionState};

    #[test]
    fn engine_supported_after_device_set() {
        let mut e = NexusEngine::new();
        e.set_device(DeviceProfile {
            manufacturer: "Xiaomi".into(),
            model: "14".into(),
            android_version: "14".into(),
            sdk_int: 34,
            hyperos_version: Some("HyperOS 2".into()),
            build_fingerprint: None,
            is_xiaomi_family: true,
            is_china_rom: true,
        });
        assert!(e.is_supported());
    }

    #[test]
    fn engine_evaluate() {
        let e = NexusEngine::new();
        let snap = SystemSnapshot {
            gms_protection: ProtectionState::Protected,
            fcm_connection: CapabilityState::Supported,
            shizuku_available: true,
            device_supported: true,
        };
        let res = e.evaluate(&snap);
        assert!(res.healthy);
    }
}