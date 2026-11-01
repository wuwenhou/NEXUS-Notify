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

// ---------- Android JNI entry points (compiled only for Android targets) ----------
#[cfg(target_os = "android")]
mod android_jni {
    use std::os::raw::{c_char, c_void};
    use nexus_core::NexusCore;
    use nexus_domain::{CapabilityState, ProtectionState};
    use nexus_fcm;
    use nexus_policy::SystemSnapshot;
    use nexus_recovery::{RecoveryPlan, RecoveryReason};

    type JniEnv = *mut c_void;
    type JClass = *mut c_void;
    type JString = *mut c_void;

    const NEW_STRING_UTF_IDX: usize = 167;

    unsafe fn new_string_utf(env: JniEnv, s: *const c_char) -> JString {
        let table_ptr = *(env as *mut *const usize);
        let fn_ptr = *table_ptr.add(NEW_STRING_UTF_IDX);
        let f: unsafe extern "system" fn(JniEnv, *const c_char) -> JString =
            std::mem::transmute(fn_ptr);
        f(env, s)
    }

    fn to_protected(v: i32) -> ProtectionState {
        match v {
            0 => ProtectionState::Protected,
            1 => ProtectionState::NotProtected,
            2 => ProtectionState::Unsupported,
            3 => ProtectionState::PermissionDenied,
            _ => ProtectionState::Unknown,
        }
    }

    fn to_capability(v: i32) -> CapabilityState {
        match v {
            0 => CapabilityState::Supported,
            1 => CapabilityState::Unsupported,
            2 => CapabilityState::PermissionDenied,
            3 => CapabilityState::Unknown,
            _ => CapabilityState::Failed,
        }
    }

    #[no_mangle]
    pub extern "system" fn Java_eu_omijia_nexus_NativeBridge_nativeGreet(
        env: JniEnv,
        _class: JClass,
    ) -> JString {
        let msg = "NEXUS Notify Rust core OK\0";
        unsafe { new_string_utf(env, msg.as_ptr() as *const c_char) }
    }

    #[no_mangle]
    pub extern "system" fn Java_eu_omijia_nexus_NativeBridge_nativeEvaluate(
        env: JniEnv,
        _class: JClass,
        gms: i32,
        fcm: i32,
        shizuku: i32,
        device_supported: i32,
    ) -> JString {
        let core = NexusCore::new();
        let snap = SystemSnapshot {
            gms_protection: to_protected(gms),
            fcm_connection: to_capability(fcm),
            shizuku_available: shizuku != 0,
            device_supported: device_supported != 0,
        };
        let res = core.evaluate(&snap);
        let json = serde_json::json!({
            "healthy": res.healthy,
            "issues": res.issues,
            "action_count": res.actions.len(),
        });
        let s = format!("{}\0", json);
        unsafe { new_string_utf(env, s.as_ptr() as *const c_char) }
    }

    #[no_mangle]
    pub extern "system" fn Java_eu_omijia_nexus_NativeBridge_nativeGmsDiagnose(
        env: JniEnv,
        _class: JClass,
        freeze: i32,
        protection: i32,
    ) -> JString {
        let core = NexusCore::new();
        let freeze_state = match freeze {
            0 => nexus_domain::GmsFreezeState::NotFrozen,
            1 => nexus_domain::GmsFreezeState::Frozen,
            _ => nexus_domain::GmsFreezeState::Unknown,
        };
        let prot = to_protected(protection);
        let d = core.diagnose_gms(freeze_state, prot);
        let json = serde_json::json!({
            "freeze_state": format!("{:?}", d.freeze_state),
            "protection": format!("{:?}", d.protection),
            "needs_protection": d.needs_protection,
            "summary": d.summary,
        });
        let s = format!("{}\0", json);
        unsafe { new_string_utf(env, s.as_ptr() as *const c_char) }
    }

    #[no_mangle]
    pub extern "system" fn Java_eu_omijia_nexus_NativeBridge_nativeFcmDiagnose(
        env: JniEnv,
        _class: JClass,
        observed: i32,
    ) -> JString {
        let state = match observed {
            0 => nexus_domain::CapabilityState::Supported,
            1 => nexus_domain::CapabilityState::Unsupported,
            2 => nexus_domain::CapabilityState::PermissionDenied,
            3 => nexus_domain::CapabilityState::Unknown,
            _ => nexus_domain::CapabilityState::Failed,
        };
        let d = nexus_fcm::build_diagnostic(state, "probed");
        let reconnect = nexus_fcm::should_request_reconnect(&d);
        let json = serde_json::json!({
            "ports": d.ports_checked,
            "connection_observed": format!("{:?}", d.connection_observed),
            "should_reconnect": reconnect,
        });
        let s = format!("{}\0", json);
        unsafe { new_string_utf(env, s.as_ptr() as *const c_char) }
    }

    #[no_mangle]
    pub extern "system" fn Java_eu_omijia_nexus_NativeBridge_nativeRecoveryPlan(
        env: JniEnv,
        _class: JClass,
        reason: i32,
    ) -> JString {
        let r = match reason {
            0 => RecoveryReason::BootCompleted,
            1 => RecoveryReason::ShizukuReconnected,
            2 => RecoveryReason::PermissionLost,
            _ => RecoveryReason::Unknown,
        };
        let plan = RecoveryPlan::from_reason(r);
        let actions: Vec<String> = plan.actions.iter().map(|a| format!("{:?}", a)).collect();
        let json = serde_json::json!({
            "reason": format!("{:?}", plan.reason),
            "actions": actions,
        });
        let s = format!("{}\0", json);
        unsafe { new_string_utf(env, s.as_ptr() as *const c_char) }
    }

    #[no_mangle]
    pub extern "system" fn Java_eu_omijia_nexus_NativeBridge_nativeReport(
        env: JniEnv,
        _class: JClass,
        gms: i32,
        fcm: i32,
        shizuku: i32,
        device_supported: i32,
    ) -> JString {
        let core = NexusCore::new();
        let snap = SystemSnapshot {
            gms_protection: to_protected(gms),
            fcm_connection: to_capability(fcm),
            shizuku_available: shizuku != 0,
            device_supported: device_supported != 0,
        };
        let r = core.generate_report(&snap);
        let json = serde_json::json!({
            "device_supported": r.device_supported,
            "healthy": r.healthy,
            "issues": r.issues,
            "action_count": r.action_count,
            "summary": r.summary,
            "shizuku": format!("{:?}", r.shizuku),
        });
        let s = format!("{}\0", json);
        unsafe { new_string_utf(env, s.as_ptr() as *const c_char) }
    }
}

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