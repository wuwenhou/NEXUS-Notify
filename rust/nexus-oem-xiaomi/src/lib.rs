//! Xiaomi/HyperOS specific adapter.
//!
//! Pure logic here operates on a DeviceProfile and capability probes.
//! No Android/IO performed in this crate.

use nexus_domain::{CapabilityState, DeviceProfile, NexusError};

/// Detect whether a device profile is Xiaomi family.
pub fn is_xiaomi_family(profile: &DeviceProfile) -> bool {
    let m = profile.manufacturer.to_lowercase();
    m.contains("xiaomi") || m.contains("redmi") || m.contains("poco")
}

/// Detect China ROM heuristic from build fingerprint.
pub fn is_china_rom(profile: &DeviceProfile) -> bool {
    if profile.is_china_rom {
        return true;
    }
    profile
        .build_fingerprint
        .as_deref()
        .map(|f| f.contains("CN") || f.contains(".cn."))
        .unwrap_or(false)
}

/// Capabilities available on this build.
#[derive(Debug, Clone, Default)]
pub struct XiaomiCapabilities {
    pub gms_protection: CapabilityState,
    pub greezer_inspect: CapabilityState,
    pub powerkeeper_inspect: CapabilityState,
    pub aurogon_inspect: CapabilityState,
    pub fcm_reconnect: CapabilityState,
}

impl XiaomiCapabilities {
    /// MVP: only GMS protection + greezer inspect are considered; others unknown.
    pub fn for_mvp() -> Self {
        Self {
            gms_protection: CapabilityState::Unknown,
            greezer_inspect: CapabilityState::Unknown,
            powerkeeper_inspect: CapabilityState::Unknown,
            aurogon_inspect: CapabilityState::Unknown,
            fcm_reconnect: CapabilityState::Unknown,
        }
    }
}

/// Validate that a device is supported before any mutation.
pub fn require_supported(profile: &DeviceProfile) -> Result<(), NexusError> {
    if !is_xiaomi_family(profile) {
        return Err(NexusError::UnsupportedDevice("not Xiaomi family".into()));
    }
    if !is_china_rom(profile) {
        return Err(NexusError::UnsupportedDevice("not China ROM".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_domain::DeviceProfile;

    fn xiaomi_cn() -> DeviceProfile {
        DeviceProfile {
            manufacturer: "Xiaomi".into(),
            model: "14".into(),
            android_version: "14".into(),
            sdk_int: 34,
            hyperos_version: Some("HyperOS 2".into()),
            build_fingerprint: Some("xiaomi/aurora_cn/...".into()),
            is_xiaomi_family: true,
            is_china_rom: true,
        }
    }

    #[test]
    fn require_supported_ok_for_xiaomi_cn() {
        assert!(require_supported(&xiaomi_cn()).is_ok());
    }

    #[test]
    fn require_supported_rejects_non_xiaomi() {
        let mut p = xiaomi_cn();
        p.manufacturer = "Google".into();
        p.is_xiaomi_family = false;
        assert!(require_supported(&p).is_err());
    }

    #[test]
    fn china_rom_heuristic_from_fingerprint() {
        let mut p = xiaomi_cn();
        p.is_china_rom = false;
        p.build_fingerprint = Some("foo/bar.CN/baz".into());
        assert!(is_china_rom(&p));
    }
}