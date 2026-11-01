//! Policy engine: evaluates current system state against desired config
//! and produces a typed action plan.

use nexus_domain::{
    CapabilityState, NexusError, ProtectionState, SystemAction,
};

/// Desired protection policy for GMS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GmsPolicy {
    pub desired_protected: bool,
}

impl Default for GmsPolicy {
    fn default() -> Self {
        Self {
            desired_protected: true,
        }
    }
}

/// Inputs describing current system state (read by adapters).
#[derive(Debug, Clone, Default)]
pub struct SystemSnapshot {
    pub gms_protection: ProtectionState,
    pub fcm_connection: CapabilityState,
    pub shizuku_available: bool,
    pub device_supported: bool,
}

/// A planned action with rationale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedAction {
    pub action: SystemAction,
    pub reason: String,
    pub risk: nexus_domain::RiskLevel,
    pub requires_confirmation: bool,
}

/// Result of policy evaluation.
#[derive(Debug, Clone, Default)]
pub struct EvaluationResult {
    pub healthy: bool,
    pub issues: Vec<String>,
    pub actions: Vec<PlannedAction>,
}

/// Core policy engine.
pub struct PolicyEngine {
    gms_policy: GmsPolicy,
}

impl PolicyEngine {
    pub fn new() -> Self {
        Self {
            gms_policy: GmsPolicy::default(),
        }
    }

    pub fn with_gms_policy(gms_policy: GmsPolicy) -> Self {
        Self { gms_policy }
    }

    /// Evaluate the snapshot and produce an action plan.
    ///
    /// Rule: never mutate when state is Unknown. Only plan EnsureGmsProtection
    /// when state is explicitly NotProtected.
    pub fn evaluate(&self, snap: &SystemSnapshot) -> Result<EvaluationResult, NexusError> {
        let mut result = EvaluationResult::default();

        if !snap.device_supported {
            result.issues.push("Device not supported for MVP".into());
            return Ok(result);
        }

        if !snap.shizuku_available {
            result.issues.push("Shizuku not available; diagnostic only".into());
        }

        // GMS protection evaluation
        match snap.gms_protection {
            ProtectionState::Protected => {
                // healthy
            }
            ProtectionState::NotProtected => {
                if self.gms_policy.desired_protected {
                    if snap.shizuku_available {
                        result.actions.push(PlannedAction {
                            action: SystemAction::EnsureGmsProtection,
                            reason: "GMS is not protected".into(),
                            risk: nexus_domain::RiskLevel::Medium,
                            requires_confirmation: true,
                        });
                    } else {
                        result.issues.push(
                            "GMS not protected but Shizuku unavailable".into(),
                        );
                    }
                }
            }
            ProtectionState::Unknown => {
                // Do NOT plan mutation; report unknown.
                result.issues.push("GMS protection state unknown; refusing to mutate".into());
            }
            ProtectionState::Unsupported => {
                result.issues.push("GMS protection unsupported on this build".into());
            }
            ProtectionState::PermissionDenied => {
                result.issues.push("Permission denied for GMS protection".into());
            }
        }

        // FCM connection evaluation
        match snap.fcm_connection {
            CapabilityState::Supported => {}
            CapabilityState::Unknown => {
                result.issues.push("FCM connection state unknown".into());
            }
            CapabilityState::Unsupported => {
                result.issues.push("FCM diagnostics unsupported".into());
            }
            CapabilityState::PermissionDenied => {
                result.issues.push("Permission denied for FCM diagnostics".into());
            }
            CapabilityState::Failed => {
                result.issues.push("FCM diagnostics failed".into());
            }
        }

        result.healthy = result.issues.is_empty() && result.actions.is_empty();
        Ok(result)
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_domain::{CapabilityState, ProtectionState};

    fn base_snap() -> SystemSnapshot {
        SystemSnapshot {
            gms_protection: ProtectionState::Protected,
            fcm_connection: CapabilityState::Supported,
            shizuku_available: true,
            device_supported: true,
        }
    }

    #[test]
    fn healthy_when_protected() {
        let engine = PolicyEngine::new();
        let res = engine.evaluate(&base_snap()).unwrap();
        assert!(res.healthy);
        assert!(res.actions.is_empty());
    }

    #[test]
    fn plans_ensure_when_not_protected() {
        let engine = PolicyEngine::new();
        let mut snap = base_snap();
        snap.gms_protection = ProtectionState::NotProtected;
        let res = engine.evaluate(&snap).unwrap();
        assert!(!res.healthy);
        assert_eq!(res.actions.len(), 1);
        assert_eq!(res.actions[0].action, SystemAction::EnsureGmsProtection);
    }

    #[test]
    fn does_not_mutate_when_unknown() {
        let engine = PolicyEngine::new();
        let mut snap = base_snap();
        snap.gms_protection = ProtectionState::Unknown;
        let res = engine.evaluate(&snap).unwrap();
        assert!(res.actions.is_empty());
        assert!(res.issues.iter().any(|i| i.contains("unknown")));
    }

    #[test]
    fn no_action_without_shizuku() {
        let engine = PolicyEngine::new();
        let mut snap = base_snap();
        snap.gms_protection = ProtectionState::NotProtected;
        snap.shizuku_available = false;
        let res = engine.evaluate(&snap).unwrap();
        assert!(res.actions.is_empty());
    }

    #[test]
    fn unsupported_device_returns_no_actions() {
        let engine = PolicyEngine::new();
        let mut snap = base_snap();
        snap.device_supported = false;
        let res = engine.evaluate(&snap).unwrap();
        assert!(res.actions.is_empty());
        assert!(res.issues.iter().any(|i| i.contains("not supported")));
    }
}