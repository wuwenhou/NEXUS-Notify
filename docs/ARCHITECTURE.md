# NEXUS Notify — Architecture Specification v1.0

## Product

NEXUS Notify diagnoses and improves push-notification reliability on Xiaomi/Redmi/POCO China ROM. It addresses:

1. GMS frozen/restricted by Xiaomi background services.
2. Unstable FCM connection.
3. HyperOS background restrictions per app.
4. Policy drift after reboot/ROM update.

## Layers

1. Presentation — Jetpack Compose (Dashboard, Settings, Logs, Automation).
2. Android Integration — Kotlin, Shizuku, BroadcastReceiver, WorkManager, JNI.
3. Rust Core — Policy, FCM diagnostics, OEM adapter, automation, recovery, audit, storage.
4. System Adapter — Shizuku UserService, ADB shell, Android framework.
5. Android/HyperOS — PowerKeeper, Greezer, Aurogon, GMS, AppOps.

## Rust crates

- `nexus-domain` — types, errors, validation.
- `nexus-policy` — evaluation and action planning.
- `nexus-oem-xiaomi` — device detection, capabilities.
- `nexus-fcm` — FCM diagnostics model.
- `nexus-automation` — rules.
- `nexus-recovery` — recovery planning.
- `nexus-audit` — audit log.
- `nexus-storage` — config persistence.
- `nexus-core` — orchestrator.
- `nexus-bridge` — Kotlin/Android surface (UniFFI planned).

## Key invariants

- Rust core never calls Android APIs directly; all IO goes through adapters.
- `Unknown` state never becomes a mutation.
- Default mode is Diagnostic.
- Every mutation is verified by read-back and audited.

## MVP (v0.1.0)

- Device detection + Shizuku status.
- GMS protection read/ensure.
- Greezer diagnostics.
- FCM connection diagnostics.
- Recovery after reboot.
- Audit log.

Out of MVP: Aurogon guard, auto-unstop, multi-app management, automation rules beyond basics.