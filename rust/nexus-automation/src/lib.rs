//! Basic automation rule management (in-memory).

use nexus_domain::{AutomationRule, SystemAction};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Default)]
pub struct AutomationEngine {
    rules: Vec<AutomationRule>,
}

static RULE_COUNTER: AtomicU64 = AtomicU64::new(1);

impl AutomationEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_rule(&mut self, name: impl Into<String>, trigger: impl Into<String>, action: SystemAction) {
        self.rules.push(AutomationRule {
            id: RULE_COUNTER.fetch_add(1, Ordering::Relaxed),
            name: name.into(),
            trigger: trigger.into(),
            action,
            enabled: true,
        });
    }

    pub fn rules(&self) -> &[AutomationRule] {
        &self.rules
    }

    pub fn set_enabled(&mut self, id: u64, enabled: bool) -> bool {
        if let Some(r) = self.rules.iter_mut().find(|r| r.id == id) {
            r.enabled = enabled;
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, id: u64) -> bool {
        let before = self.rules.len();
        self.rules.retain(|r| r.id != id);
        self.rules.len() != before
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_domain::SystemAction;

    #[test]
    fn add_and_list_rules() {
        let mut e = AutomationEngine::new();
        e.add_rule("night", "boot_completed", SystemAction::ReadGmsProtection);
        assert_eq!(e.rules().len(), 1);
        assert!(e.rules()[0].enabled);
    }

    #[test]
    fn toggle_and_remove() {
        let mut e = AutomationEngine::new();
        e.add_rule("r", "t", SystemAction::ReadDeviceInfo);
        let id = e.rules()[0].id;
        assert!(e.set_enabled(id, false));
        assert!(!e.rules()[0].enabled);
        assert!(e.remove(id));
        assert!(e.rules().is_empty());
    }
}