//! Orchestrator wiring domain, policy, oem, fcm, audit, storage.

use nexus_audit::AuditLog;
use nexus_domain::DeviceProfile;
use nexus_fcm as fcm;
use nexus_policy::SystemSnapshot;
use nexus_oem_xiaomi as oem;
use nexus_policy::{PolicyEngine, SystemSnapshot as PolicySnapshot};
use nexus_storage::AppConfig;

pub struct NexusCore {
    pub device: Option<DeviceProfile>,
    pub policy: PolicyEngine,
    pub audit: AuditLog,
    pub config: AppConfig,
}

impl NexusCore {
    pub fn new() -> Self {
        Self {
            device: None,
            policy: PolicyEngine::new(),
            audit: AuditLog::new(500),
            config: AppConfig::new(),
        }
    }

    pub fn set_device(&mut self, profile: DeviceProfile) {
        self.device = Some(profile);
    }

    pub fn is_supported(&self) -> bool {
        self.device
            .as_ref()
            .map(|p| p.supported_for_mvp())
            .unwrap_or(false)
    }

    pub fn evaluate(&self, snap: &SystemSnapshot) -> nexus_policy::EvaluationResult {
        let ps = PolicySnapshot {
            gms_protection: snap.gms_protection,
            fcm_connection: snap.fcm_connection,
            shizuku_available: snap.shizuku_available,
            device_supported: snap.device_supported,
        };
        self.policy.evaluate(&ps).unwrap_or_default()
    }

    pub fn fcm_diagnostic_unknown() -> fcm::FcmDiagnostic {
        fcm::build_diagnostic(nexus_domain::CapabilityState::Unknown, "not yet probed")
    }

    pub fn oem_require_supported(profile: &DeviceProfile) -> Result<(), nexus_domain::NexusError> {
        oem::require_supported(profile)
    }
}

impl Default for NexusCore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_domain::{CapabilityState, DeviceProfile, ProtectionState};

    #[test]
    fn core_supported_flag() {
        let mut core = NexusCore::new();
        assert!(!core.is_supported());
        core.set_device(DeviceProfile {
            manufacturer: "Xiaomi".into(),
            model: "14".into(),
            android_version: "14".into(),
            sdk_int: 34,
            hyperos_version: Some("HyperOS 2".into()),
            build_fingerprint: None,
            is_xiaomi_family: true,
            is_china_rom: true,
        });
        assert!(core.is_supported());
    }

    #[test]
    fn core_evaluate_healthy() {
        let core = NexusCore::new();
        let snap = SystemSnapshot {
            gms_protection: ProtectionState::Protected,
            fcm_connection: CapabilityState::Supported,
            shizuku_available: true,
            device_supported: true,
        };
        let res = core.evaluate(&snap);
        assert!(res.healthy);
    }
}