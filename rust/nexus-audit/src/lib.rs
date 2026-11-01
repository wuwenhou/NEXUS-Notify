//! In-memory audit log (persistence handled by nexus-storage).

use nexus_domain::AuditEntry;

#[derive(Debug, Default)]
pub struct AuditLog {
    entries: Vec<AuditEntry>,
    capacity: usize,
}

impl AuditLog {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: Vec::new(),
            capacity: capacity.max(1),
        }
    }

    pub fn record(&mut self, entry: AuditEntry) {
        if self.entries.len() >= self.capacity {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }

    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }

    pub fn last(&self) -> Option<&AuditEntry> {
        self.entries.last()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_domain::{ActionResult, SystemAction};

    #[test]
    fn records_and_evicts() {
        let mut log = AuditLog::new(2);
        for i in 0..3 {
            log.record(nexus_domain::AuditEntry::new(
                SystemAction::ReadDeviceInfo,
                ActionResult::ok("op", format!("msg {}", i)),
                false,
            ));
        }
        assert_eq!(log.entries().len(), 2);
    }

    #[test]
    fn clear_works() {
        let mut log = AuditLog::new(10);
        log.record(nexus_domain::AuditEntry::new(
            SystemAction::ReadDeviceInfo,
            ActionResult::ok("op", "m"),
            false,
        ));
        log.clear();
        assert!(log.entries().is_empty());
    }
}