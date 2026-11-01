//! Core domain types for NEXUS Notify.
//!
//! This crate contains pure data structures and enums shared across all other
//! crates. It has no dependency on Android, Shizuku, or any vendor API.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;

/// Result of a capability/privilege probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrivilegeLevel {
    /// No privileged backend available.
    None,
    /// Public Android APIs only.
    AndroidOnly,
    /// Shizuku UserService (ADB-shell identity) available.
    Shizuku,
    /// Root (not used by NEXUS; reserved).
    Root,
}

/// Capability state reported by an OEM adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityState {
    /// Operation supported and permitted.
    Supported,
    /// Operation not supported on this build.
    Unsupported,
    /// Operation requires a privilege the app does not have.
    PermissionDenied,
    /// State could not be determined; do not mutate.
    Unknown,
    /// Operation attempted and failed.
    Failed,
}

/// Protection state for a managed component (e.g. GMS).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtectionState {
    Protected,
    NotProtected,
    Unsupported,
    PermissionDenied,
    Unknown,
}

impl ProtectionState {
    pub fn is_actionable(&self) -> bool {
        matches!(self, ProtectionState::NotProtected)
    }
}

/// Risk level of a planned system action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// A typed system action. Never a raw shell string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemAction {
    ReadDeviceInfo,
    ReadShizukuStatus,
    ReadGmsProtection,
    EnsureGmsProtection,
    RemoveGmsProtection,
    InspectFcmConnection,
    RequestFcmReconnect,
    InspectAppPolicy { package_name: String },
    RestoreAppPolicy { package_name: String },
    InspectGreezerState,
    InspectPowerKeeperState,
    InspectAurogonState,
    ReadAuditLog,
}

/// Outcome of executing a system action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionResult {
    pub success: bool,
    pub operation: String,
    pub message: String,
    pub state_before: Option<String>,
    pub state_after: Option<String>,
}

impl ActionResult {
    pub fn ok(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            success: true,
            operation: operation.into(),
            message: message.into(),
            state_before: None,
            state_after: None,
        }
    }

    pub fn fail(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            success: false,
            operation: operation.into(),
            message: message.into(),
            state_before: None,
            state_after: None,
        }
    }
}

/// Device profile detected at runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceProfile {
    pub manufacturer: String,
    pub model: String,
    pub android_version: String,
    pub sdk_int: i32,
    pub hyperos_version: Option<String>,
    pub build_fingerprint: Option<String>,
    pub is_xiaomi_family: bool,
    pub is_china_rom: bool,
}

impl DeviceProfile {
    pub fn supported_for_mvp(&self) -> bool {
        self.is_xiaomi_family && self.is_china_rom
    }
}

/// FCM connection diagnostic result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FcmDiagnostic {
    pub ports_checked: Vec<u16>,
    pub connection_observed: CapabilityState,
    pub last_reconnect_request: Option<String>,
    pub note: String,
}

static ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// A single audit log entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: u64,
    pub timestamp: i64,
    pub action: SystemAction,
    pub result: ActionResult,
    pub reversible: bool,
    pub rollback_data: Option<String>,
}

impl AuditEntry {
    pub fn new(action: SystemAction, result: ActionResult, reversible: bool) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        Self {
            id: ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            timestamp: ts,
            action,
            result,
            reversible,
            rollback_data: None,
        }
    }
}

/// Shizuku service status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShizukuStatus {
    NotInstalled,
    NotRunning,
    PermissionDenied,
    Available,
}

impl ShizukuStatus {
    pub fn is_available(&self) -> bool {
        matches!(self, ShizukuStatus::Available)
    }
}

/// Configuration for a protection module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleConfig {
    pub name: String,
    pub enabled: bool,
    pub risk: RiskLevel,
    pub description: String,
}

/// Automation rule (basic).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutomationRule {
    pub id: u64,
    pub name: String,
    pub trigger: String,
    pub action: SystemAction,
    pub enabled: bool,
}

/// Errors produced by domain logic.
#[derive(Debug, Error)]
pub enum NexusError {
    #[error("device not supported: {0}")]
    UnsupportedDevice(String),
    #[error("privilege required: {0}")]
    PrivilegeRequired(String),
    #[error("capability unknown: {0}")]
    CapabilityUnknown(String),
    #[error("invalid package name: {0}")]
    InvalidPackageName(String),
    #[error("state unknown, refusing to mutate")]
    StateUnknown,
    #[error("verification failed: {0}")]
    VerificationFailed(String),
    #[error("internal error: {0}")]
    Internal(String),
}

/// Validate an Android package name (basic).
pub fn validate_package_name(name: &str) -> Result<(), NexusError> {
    if name.is_empty() {
        return Err(NexusError::InvalidPackageName("empty".into()));
    }
    if !name.contains('.') {
        return Err(NexusError::InvalidPackageName("missing dot".into()));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_')
    {
        return Err(NexusError::InvalidPackageName("invalid characters".into()));
    }
    Ok(())
}

impl Default for CapabilityState {
    fn default() -> Self {
        CapabilityState::Unknown
    }
}

impl Default for ProtectionState {
    fn default() -> Self {
        ProtectionState::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protection_state_actionable_only_when_not_protected() {
        assert!(ProtectionState::NotProtected.is_actionable());
        assert!(!ProtectionState::Protected.is_actionable());
        assert!(!ProtectionState::Unknown.is_actionable());
    }

    #[test]
    fn validate_package_name_accepts_valid() {
        assert!(validate_package_name("com.google.android.gms").is_ok());
    }

    #[test]
    fn validate_package_name_rejects_invalid() {
        assert!(validate_package_name("").is_err());
        assert!(validate_package_name("nodothere").is_err());
        assert!(validate_package_name("bad name!").is_err());
    }

    #[test]
    fn device_profile_supported_requires_xiaomi_china() {
        let p = DeviceProfile {
            manufacturer: "Xiaomi".into(),
            model: "14".into(),
            android_version: "14".into(),
            sdk_int: 34,
            hyperos_version: Some("HyperOS 2".into()),
            build_fingerprint: None,
            is_xiaomi_family: true,
            is_china_rom: true,
        };
        assert!(p.supported_for_mvp());
        let p2 = DeviceProfile {
            is_china_rom: false,
            ..p
        };
        assert!(!p2.supported_for_mvp());
    }
}