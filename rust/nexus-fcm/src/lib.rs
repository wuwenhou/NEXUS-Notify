//! FCM connection diagnostics.

pub use nexus_domain::FcmDiagnostic;
use nexus_domain::CapabilityState;

pub const FCM_PORTS: [u16; 3] = [5228, 5229, 5230];

/// Build a diagnostic result. Actual socket inspection is performed by the
/// Android adapter; this crate only models the result.
pub fn build_diagnostic(observed: CapabilityState, note: impl Into<String>) -> FcmDiagnostic {
    nexus_domain::FcmDiagnostic {
        ports_checked: FCM_PORTS.to_vec(),
        connection_observed: observed,
        last_reconnect_request: None,
        note: note.into(),
    }
}

pub fn should_request_reconnect(d: &FcmDiagnostic) -> bool {
    matches!(d.connection_observed, CapabilityState::Unknown | CapabilityState::Failed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_domain::CapabilityState;

    #[test]
    fn ports_are_fcm_standard() {
        assert_eq!(FCM_PORTS, [5228, 5229, 5230]);
    }

    #[test]
    fn reconnect_requested_when_unknown() {
        let d = build_diagnostic(CapabilityState::Unknown, "no data");
        assert!(should_request_reconnect(&d));
    }

    #[test]
    fn no_reconnect_when_supported() {
        let d = build_diagnostic(CapabilityState::Supported, "ok");
        assert!(!should_request_reconnect(&d));
    }
}