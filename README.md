# NEXUS Notify

HyperOS Notification Reliability & Automation Engine for Xiaomi / Redmi / POCO China ROM.

- **Platform:** Android
- **OEM:** Xiaomi only (MVP)
- **Core:** Rust
- **UI:** Kotlin + Jetpack Compose
- **Privilege:** Shizuku (ADB-shell identity) + ADB; no root
- **License:** Apache-2.0 (proposed)

## Status

Phase 1 (Rust Core) in progress. Host-side crates build and tests pass.

## Repository layout

```
NEXUS-Notify/
├── rust/          # Cargo workspace (domain, policy, oem-xiaomi, fcm, ...)
├── android/       # Android app (Kotlin + Compose) — scaffold
├── docs/          # Architecture, system analysis, security model
└── scripts/       # Build helpers
```

## Build (Rust core)

```sh
cd rust
cargo build
cargo test
```

## Principles

- Diagnostic mode by default.
- No root, no bootloader unlock, no firmware changes.
- Typed operations only; no raw shell strings from untrusted input.
- Unknown state is never treated as "not protected".

See `docs/NEXUS_NOTIFY_SYSTEM_ANALYSIS.md` and `docs/ARCHITECTURE.md`.