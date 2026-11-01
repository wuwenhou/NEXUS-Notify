//! Orchestrator wiring domain, policy, oem, fcm, audit, storage.

use nexus_audit::AuditLog;
use nexus_domain::{DeviceProfile, ShizukuStatus};
use nexus_fcm as fcm;
use nexus_policy::SystemSnapshot;
use nexus_oem_xiaomi as oem;
use nexus_policy::{PolicyEngine, SystemSnapshot as PolicySnapshot};
use nexus_storage::AppConfig;

#[derive(Debug, Clone)]
pub struct DiagnosticReport {
    pub device_supported: bool,
    pub shizuku: ShizukuStatus,
    pub gms_protection: nexus_domain::ProtectionState,
    pub fcm_connection: nexus_domain::CapabilityState,
    pub healthy: bool,
    pub issues: Vec<String>,
    pub action_count: usize,
    pub summary: String,
}

pub struct NexusCore {
    pub device: Option<DeviceProfile>,
    pub policy: PolicyEngine,
    pub audit: AuditLog,
    pub config: AppConfig,
    pub shizuku: ShizukuStatus,
}

impl NexusCore {
    pub fn new() -> Self {
        Self {
            device: None,
            policy: PolicyEngine::new(),
            audit: AuditLog::new(500),
            config: AppConfig::new(),
            shizuku: ShizukuStatus::NotInstalled,
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

    pub fn generate_report(&self, snap: &SystemSnapshot) -> DiagnosticReport {
        let res = self.evaluate(snap);
        let device_supported = self.is_supported();
        let summary = if res.healthy {
            "No issues detected.".to_string()
        } else {
            format!("{} issue(s), {} action(s) planned.", res.issues.len(), res.actions.len())
        };
        DiagnosticReport {
            device_supported,
            shizuku: self.shizuku,
            gms_protection: snap.gms_protection,
            fcm_connection: snap.fcm_connection,
            healthy: res.healthy,
            issues: res.issues,
            action_count: res.actions.len(),
            summary,
        }
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

    #[test]
    fn core_generate_report() {
        let core = NexusCore::new();
        let snap = SystemSnapshot {
            gms_protection: ProtectionState::NotProtected,
            fcm_connection: CapabilityState::Unknown,
            shizuku_available: false,
            device_supported: true,
        };
        let r = core.generate_report(&snap);
        assert!(!r.healthy);
        assert!(!r.summary.is_empty());
    }
}