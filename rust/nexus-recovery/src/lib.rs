//! Recovery planning after reboot / permission loss.

use nexus_domain::{SystemAction, AuditEntry};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryReason {
    BootCompleted,
    ShizukuReconnected,
    PermissionLost,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct RecoveryPlan {
    pub reason: RecoveryReason,
    pub actions: Vec<SystemAction>,
}

impl RecoveryPlan {
    pub fn from_reason(reason: RecoveryReason) -> Self {
        let actions = match reason {
            RecoveryReason::BootCompleted | RecoveryReason::ShizukuReconnected => {
                vec![SystemAction::ReadGmsProtection]
            }
            RecoveryReason::PermissionLost => vec![SystemAction::ReadShizukuStatus],
            RecoveryReason::Unknown => vec![],
        };
        Self { reason, actions }
    }
}

pub fn needs_recovery(entries: &[AuditEntry]) -> bool {
    // Simple heuristic: if last action failed, suggest recovery.
    entries.last().map(|e| !e.result.success).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_domain::{ActionResult, SystemAction};

    #[test]
    fn boot_plan_reads_gms() {
        let p = RecoveryPlan::from_reason(RecoveryReason::BootCompleted);
        assert_eq!(p.actions, vec![SystemAction::ReadGmsProtection]);
    }

    #[test]
    fn needs_recovery_on_failure() {
        let e = nexus_domain::AuditEntry::new(
            SystemAction::ReadGmsProtection,
            ActionResult::fail("x", "boom"),
            false,
        );
        assert!(needs_recovery(&[e]));
    }
}