//! Bridge surface for Kotlin/Android integration.
//!
//! UniFFI bindings are enabled via the `uniffi` feature.

use nexus_core::NexusCore;
use nexus_domain::DeviceProfile;
use nexus_policy::SystemSnapshot;
use std::sync::Mutex;

#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct NexusEngine {
    core: Mutex<NexusCore>,
}

impl NexusEngine {
    pub fn new() -> Self {
        Self {
            core: Mutex::new(NexusCore::new()),
        }
    }

    pub fn set_device(&self, profile: DeviceProfile) {
        self.core.lock().unwrap().set_device(profile);
    }

    pub fn is_supported(&self) -> bool {
        self.core.lock().unwrap().is_supported()
    }

    pub fn evaluate(&self, snap: &SystemSnapshot) -> nexus_policy::EvaluationResult {
        self.core.lock().unwrap().evaluate(snap)
    }

    pub fn generate_report(&self, snap: &SystemSnapshot) -> nexus_core::DiagnosticReport {
        self.core.lock().unwrap().generate_report(snap)
    }
}

impl Default for NexusEngine {
    fn default() -> Self {
        Self::new()
    }
}

pub use nexus_policy::EvaluationResult;

// ---------- UniFFI surface (feature-gated) ----------

#[cfg(feature = "uniffi")]
mod uniffi_surface {
    use super::NexusEngine;
    use nexus_domain::{CapabilityState, DeviceProfile, ProtectionState};
    use nexus_policy::SystemSnapshot;
    use std::sync::Arc;

    #[derive(uniffi::Enum)]
    pub enum UProtectionState {
        Protected,
        NotProtected,
        Unsupported,
        PermissionDenied,
        Unknown,
    }

    impl From<ProtectionState> for UProtectionState {
        fn from(v: ProtectionState) -> Self {
            match v {
                ProtectionState::Protected => Self::Protected,
                ProtectionState::NotProtected => Self::NotProtected,
                ProtectionState::Unsupported => Self::Unsupported,
                ProtectionState::PermissionDenied => Self::PermissionDenied,
                ProtectionState::Unknown => Self::Unknown,
            }
        }
    }

    impl From<UProtectionState> for ProtectionState {
        fn from(v: UProtectionState) -> Self {
            match v {
                UProtectionState::Protected => Self::Protected,
                UProtectionState::NotProtected => Self::NotProtected,
                UProtectionState::Unsupported => Self::Unsupported,
                UProtectionState::PermissionDenied => Self::PermissionDenied,
                UProtectionState::Unknown => Self::Unknown,
            }
        }
    }

    #[derive(uniffi::Enum)]
    pub enum UCapabilityState {
        Supported,
        Unsupported,
        PermissionDenied,
        Unknown,
        Failed,
    }

    impl From<CapabilityState> for UCapabilityState {
        fn from(v: CapabilityState) -> Self {
            match v {
                CapabilityState::Supported => Self::Supported,
                CapabilityState::Unsupported => Self::Unsupported,
                CapabilityState::PermissionDenied => Self::PermissionDenied,
                CapabilityState::Unknown => Self::Unknown,
                CapabilityState::Failed => Self::Failed,
            }
        }
    }

    impl From<UCapabilityState> for CapabilityState {
        fn from(v: UCapabilityState) -> Self {
            match v {
                UCapabilityState::Supported => Self::Supported,
                UCapabilityState::Unsupported => Self::Unsupported,
                UCapabilityState::PermissionDenied => Self::PermissionDenied,
                UCapabilityState::Unknown => Self::Unknown,
                UCapabilityState::Failed => Self::Failed,
            }
        }
    }

    #[derive(uniffi::Record)]
    pub struct UDeviceInfo {
        pub manufacturer: String,
        pub model: String,
        pub android_version: String,
        pub sdk_int: i32,
        pub hyperos_version: Option<String>,
        pub is_china_rom: bool,
    }

    #[derive(uniffi::Record)]
    pub struct UEvaluation {
        pub healthy: bool,
        pub issues: Vec<String>,
        pub action_count: i32,
    }

    #[derive(uniffi::Record)]
    pub struct UDiagnosticReport {
        pub device_supported: bool,
        pub healthy: bool,
        pub issues: Vec<String>,
        pub action_count: i32,
        pub summary: String,
    }

    #[uniffi::export]
    impl NexusEngine {
        #[uniffi::constructor]
        pub fn new_engine() -> Arc<Self> {
            Arc::new(NexusEngine::new())
        }

        pub fn set_device_info(&self, info: UDeviceInfo) {
            let profile = DeviceProfile {
                manufacturer: info.manufacturer,
                model: info.model,
                android_version: info.android_version,
                sdk_int: info.sdk_int,
                hyperos_version: info.hyperos_version,
                build_fingerprint: None,
                is_xiaomi_family: true,
                is_china_rom: info.is_china_rom,
            };
            self.set_device(profile);
        }

        pub fn is_supported_device(&self) -> bool {
            self.is_supported()
        }

        pub fn evaluate_state(
            &self,
            gms: UProtectionState,
            fcm: UCapabilityState,
            shizuku: bool,
            device_supported: bool,
        ) -> UEvaluation {
            let snap = SystemSnapshot {
                gms_protection: gms.into(),
                fcm_connection: fcm.into(),
                shizuku_available: shizuku,
                device_supported,
            };
            let res = self.evaluate(&snap);
            UEvaluation {
                healthy: res.healthy,
                issues: res.issues,
                action_count: res.actions.len() as i32,
            }
        }

        pub fn diagnostic_report(
            &self,
            gms: UProtectionState,
            fcm: UCapabilityState,
            shizuku: bool,
            device_supported: bool,
        ) -> UDiagnosticReport {
            let snap = SystemSnapshot {
                gms_protection: gms.into(),
                fcm_connection: fcm.into(),
                shizuku_available: shizuku,
                device_supported,
            };
            let r = self.generate_report(&snap);
            UDiagnosticReport {
                device_supported: r.device_supported,
                healthy: r.healthy,
                issues: r.issues,
                action_count: r.action_count as i32,
                summary: r.summary,
            }
        }
    }
}

#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_domain::{CapabilityState, DeviceProfile, ProtectionState};

    #[test]
    fn engine_supported_after_device_set() {
        let e = NexusEngine::new();
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