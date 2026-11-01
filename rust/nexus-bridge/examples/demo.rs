use nexus_bridge::NexusEngine;
use nexus_policy::SystemSnapshot;
use nexus_domain::{CapabilityState, ProtectionState};
use nexus_policy::EvaluationResult;

fn main() {
    let engine = NexusEngine::new();
    let snap = SystemSnapshot {
        gms_protection: ProtectionState::NotProtected,
        fcm_connection: CapabilityState::Unknown,
        shizuku_available: false,
        device_supported: true,
    };
    let res: EvaluationResult = engine.evaluate(&snap);
    println!("NEXUS Notify demo");
    println!("healthy: {}", res.healthy);
    println!("issues: {:?}", res.issues);
    println!("actions: {}", res.actions.len());
    let report = engine.generate_report(&snap);
    println!("report: {}", report.summary);
}
