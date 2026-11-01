**Document:** `NEXUS_NOTIFY_SYSTEM_ANALYSIS.md`  
**Version:** 0.1-draft  
**Date:** 2026-11-01  
**Status:** Phase 0 research baseline  
**Product scope:** Xiaomi / Redmi / POCO running Xiaomi HyperOS China ROM  
**Privilege model:** Android APIs + Shizuku (ADB-shell identity); no root  
**Distribution:** Open source; proposed project license Apache-2.0, subject to clean-room implementation and dependency review

---

## 1. Executive summary

NEXUS Notify is a proposed Android application for diagnosing and improving push-notification reliability on Xiaomi China ROM devices. The initial engineering hypothesis is that delayed notifications can arise from several distinct layers:

1. Google Play services (GMS) is frozen or restricted by Xiaomi background-management services.
2. The persistent Firebase Cloud Messaging (FCM) connection is lost or not re-established promptly.
3. GMS receives a push, but Xiaomi broadcast/background policy prevents delivery or process start for the target application.
4. The target application is stopped, lacks autostart/background permission, or is otherwise restricted.
5. Android Doze, network conditions, Google services state, or app-specific behavior causes delay independently of Xiaomi's vendor services.

These are separate failure modes. NEXUS must diagnose them separately rather than applying a blanket â€œdisable battery optimizationâ€ fix.

**MVP objective:** identify the device/build and Shizuku capability; read and report relevant state; protect GMS only through a verified, reversible operation; record before/after state; and provide FCM diagnostics without claiming that a healthy socket guarantees end-to-end notification delivery.

**Non-goal:** bypass Android security boundaries, exploit hidden vulnerabilities, modify firmware, disable all power management, or promise instant notifications.

---

## 2. Scope and product boundaries

### 2.1 Supported target

- Xiaomi, Redmi and POCO devices running Xiaomi HyperOS China ROM.
- Owner/user profile only in MVP.
- Android versions and HyperOS builds are capability-detected, not assumed compatible from marketing version alone.
- Global ROM, MIUI legacy builds, XSpace/clone profiles and HyperOS 4 experimental behavior are outside MVP support until independently tested.

### 2.2 Privilege model

NEXUS uses a layered privilege abstraction:

| Backend | Intended use | Requirement |
|---|---|---|
| Public Android APIs | Device/build information, app state visible to the app, scheduling, local database | Normal app permissions |
| Shizuku UserService | Privileged read/write operations supported by ADB-shell identity | Shizuku running and user approval |
| Manual Settings deep links | Guide user to OEM settings that cannot safely be changed programmatically | User action |
| Unsupported | Operation exceeds ADB-shell permission or relies on an unstable/unknown API | No attempt; explain limitation |

Shizuku does not grant root. ADB-shell permissions vary by Android release and operation. Every operation must be capability-probed and permission-checked at runtime.

### 2.3 Product safety principles

- Diagnostic-only mode is the default.
- No shell command strings assembled from untrusted user input.
- Use a typed allowlist of operations and validated package names.
- Never replace an entire vendor allowlist when only one entry needs changing.
- Preserve unrelated entries and original state.
- Require confirmation for system mutations.
- Verify every mutation by reading state back.
- Provide rollback where the prior state is known.
- If state is unknown, do not mutate.
- Never disable Doze, App Standby, App Freezer, or other power controls globally as a default remedy.
- Do not collect notification content, account credentials, FCM registration tokens, or message payloads.
- No analytics or network telemetry by default.

---

## 3. Reference-project analysis

This section describes documented behavior in public repository READMEs. It is not a source-code audit. Before implementation, inspect the exact commit, source files, LICENSE, and third-party notices of each reference project.

### 3.1 HyperOS FCM Fix â€” dingwen07

Repository: https://github.com/dingwen07/hyperos-fcm-fix

Documented capabilities:
- Uses Shizuku's ADB-shell identity; no root.
- Keeps `com.google.android.gms` in Xiaomi's hidden `Settings.System.MILLET_NO_RESTRICT_APP` no-restrictions list.
- Repairs the entry if PowerKeeper rewrites the list.
- Applies documented Greezer operations to remove a known GMS limiter and restore GMS to a runtime allowlist.
- Optional FCM connection protection checks for established GMS connections on ports 5228â€“5230 and may request a GMS reconnect.
- Per-app Aurogon FCM protection, Auto unstop (Android 16+), HyperOS Autostart and AOSP battery optimization controls.
- Recovery after reboot, app update or Shizuku restart; diagnostic logs.
- README states it requires a Xiaomi HyperOS device and the Xiaomi `com.miui.system` shared library.

Important constraints:
- Polling is not exact: Android may suspend the UserService during Doze.
- WorkManager periodic work is inexact and can be delayed by Doze or other system constraints.
- The README describes HyperOS 4 nighttime protection as experimental and off by default; it may increase battery consumption.
- Socket presence is a diagnostic signal, not proof that a message reached an application.
- A reconnect broadcast is a workaround, not a public FCM contract guaranteed across all GMS builds.
- Internal Xiaomi service interfaces can change between builds.

Design lesson: this is the strongest reference for a Shizuku-based, Xiaomi-specific system adapter and reversible per-feature controls. Reproduce behavior only after validating it on a controlled test device; do not copy implementation code without license review.

### 3.2 Noti Keeper â€” quangtrang1111

Repository: https://github.com/quangtrang1111/noti-keeper

Documented capabilities:
- Xiaomi-specific build observes changes to the hidden no-freeze list and re-adds GMS.
- Uses Android scheduling/content-observer mechanisms rather than a continuously running service, according to its README.
- Sends selected GMS reconnect/heartbeat broadcasts as a connection recovery mechanism.
- Provides a universal build with adaptive checks for other Chinese ROM families.
- The project claims low idle resource use and no Internet permission.

Important constraints:
- Its Xiaomi build depends on legacy target-SDK behavior to write a non-public vendor setting as a normal app; the README notes Android 14+ installation restrictions for apps targeting old SDKs and varying ROM behavior.
- NEXUS should not adopt a legacy target-SDK installation workaround as its architecture. Prefer modern target SDK + Shizuku for privileged operations.
- The README states that Noti Keeper does not fix every case where the target app itself is prevented from waking.
- Same application ID is used for its Xiaomi and Universal builds; this is a packaging detail, not a design pattern NEXUS needs to copy.

License note:
- Repository README states GPL-3.0 with an additional attribution condition requiring derivative products to remain open source under the same license and display attribution. Treat this as a strong copyleft/attribution obligation; obtain qualified license review before using any code. Clean-room reimplementation from independently written specifications is preferred.

Design lesson: event-driven observation of the freeze-list change is more efficient than aggressive polling when the platform exposes a reliable observer. NEXUS should use events where validated, with a conservative recovery check as fallback.

### 3.3 NotiGuard â€” Liuchijang

Repository: https://github.com/Liuchijang/NotiGuard

Documented capabilities:
- FCM Guard watches `MILLET_NO_RESTRICT_APP`, preserves existing entries, re-adds GMS if missing, and sends a heartbeat after repair.
- README states FCM Guard and the no-restrict list editor can operate without Shizuku under its legacy target-SDK approach.
- Additional system tweaks use Shizuku, including App Freezer, Doze, adaptive battery, App Standby and wakelock-related settings.
- App scanner and per-app checks cover notification permission, Autostart, background run, battery optimization, standby bucket and background data.
- README says changes are verified by reading the value back.
- Repository is marked MIT, and its README credits a port from FCMGuard-HyperOS (MIT, Â© Yijie Gao), with third-party notices.

Important constraints:
- Broad toggles such as disabling Doze/App Standby/App Freezer can have significant battery and system-wide consequences. NEXUS must not expose these as default â€œfixâ€ buttons.
- A read-only diagnostic should be separate from mutation.
- Verify the exact source and third-party license notices before reusing any component.

Design lesson: a catalog-driven diagnostics and settings UI is useful. NEXUS can adopt the concept of describing each check/action as metadata, while applying stricter risk classification and rollback requirements.

### 3.4 Feature comparison

| Capability | HyperOS FCM Fix | Noti Keeper | NotiGuard | NEXUS MVP |
|---|---:|---:|---:|---:|
| Xiaomi GMS no-restrict protection | Yes | Yes | Yes | Yes, after device validation |
| Greezer-specific handling | Yes | Xiaomi build focus | Not its central documented feature | Diagnostic first; mutation gated |
| FCM reconnect/heartbeat | Optional reconnect | Yes | Heartbeat after repair | Diagnostics first; reconnect experimental |
| Aurogon per-app handling | Yes | No | Not central | Later phase |
| App policy diagnostics | Yes | Limited | Yes | Basic read-only |
| Shizuku integration | Yes | No for Xiaomi app's core path | For system tweaks | Yes |
| Auto unstop | Yes, Android 16+ | No | Not documented | Later phase |
| Broad power-policy toggles | Per-app controls | No | Yes | Excluded from MVP |
| Audit/rollback model | Logs and cleanup guidance | Minimal | Read-back verification | Mandatory typed audit/rollback |

---

## 4. Technical model of notification delivery

### 4.1 Logical path

`Application server â†’ FCM backend â†’ GMS persistent connection â†’ Android broadcast/service delivery â†’ target application â†’ notification posting`

A failure at any stage can produce a delayed or missing notification.

### 4.2 Diagnostic layers

**Layer A â€” Device and Google services**
- Is `com.google.android.gms` installed and enabled for the owner profile?
- Is the package version recorded?
- Is Google Play services in a restricted/stopped state visible through supported diagnostics?
- Is the device online and DNS/network access functioning?

**Layer B â€” Xiaomi GMS protection**
- Can NEXUS read `MILLET_NO_RESTRICT_APP` on this build?
- Is the value format understood and safely parseable?
- Is `com.google.android.gms` present?
- Is the relevant Greezer capability exposed and does the operation return a verifiable result?
- Has PowerKeeper rewritten the value since last observation?

**Layer C â€” FCM connection**
- Can the current GMS build expose reliable connection state through a supported diagnostic?
- If socket inspection is used, distinguish `ESTABLISHED` from other TCP states.
- Ports 5228â€“5230 are documented by the reference project, but should be treated as observed transport endpoints, not a universal guarantee.
- Record timestamp, address family, state and source of observation.
- Do not capture payloads or tokens.
- A missing/unknown socket state must not automatically trigger repeated reconnect loops.

**Layer D â€” Target app**
- Is the app installed and enabled?
- Does it declare an FCM/GCM receive component that can be discovered under Android package-visibility rules?
- Is notification permission granted where required?
- Is the app stopped?
- Are Autostart/AppOps/background restrictions discoverable on this ROM?
- Does the app have its own server/network issue?

**Layer E â€” End-to-end verification**
- Use a test application and a controlled test server/account.
- Record send time, receipt callback time and notification-post time.
- Test with screen on/off, Wi-Fi/mobile data, idle/charging and reboot.
- Never infer end-to-end health from a single GMS socket observation.

---

## 5. Xiaomi-specific research matrix

All vendor-specific operations are capability-gated. The names below identify research areas, not a promise that a stable public API exists.

| Area | Research question | MVP handling |
|---|---|---|
| `MILLET_NO_RESTRICT_APP` | Can it be read safely? What is its serialization format? Does it preserve other packages? | Read-only first; then narrowly add GMS while preserving entries |
| PowerKeeper | Which event rewrites the list? Can change be observed without polling? | Detect changes; do not modify PowerKeeper |
| Greezer | Is the documented dump/shell entry point present on this build? | Probe capability; no-op if unsupported; verify result |
| Aurogon | What app-level broadcast policy applies? | Defer mutation to post-MVP |
| Autostart AppOps | Which operation codes/states exist on each build? | Read-only inventory in MVP |
| Package stopped state | Can state be read and safely restored? | Defer Auto unstop |
| Battery optimization | Which public APIs and settings are available? | Report only; do not force Unrestricted |
| XSpace/clone profile | Are there separate GMS UIDs and app instances? | Out of MVP |

### 5.1 Build capability profile

Do not use a single `HyperOS 2` or `HyperOS 3` boolean. Build a profile from:
- Manufacturer and device model.
- Build fingerprint and incremental build.
- Android API level and security patch.
- HyperOS version properties where available.
- Presence of Xiaomi shared library `com.miui.system`.
- Presence and version of relevant system packages.
- Shizuku server UID and permission state.
- Per-operation probe results.

Store capability results with a timestamp and build fingerprint. Invalidate them after OTA/system package updates.

---

## 6. Privilege and operation model

### 6.1 Privilege abstraction

```text
PrivilegeBackend
â”œâ”€â”€ PublicAndroidBackend
â”œâ”€â”€ ShizukuAdbShellBackend
â””â”€â”€ ManualUserActionBackend
```

Each operation declares:
- Required backend.
- Required permission/capability.
- Read or write classification.
- Risk level.
- Precondition.
- Expected result.
- Verification query.
- Rollback strategy.
- Minimum supported build/API.
- Timeout and output-size limit.

### 6.2 Operation risk levels

| Level | Meaning | Policy |
|---|---|---|
| R0 | Read-only diagnostics | May run automatically |
| R1 | Narrow, reversible package-level change | User enables module; show preview and rollback |
| R2 | Vendor-specific or broad system policy change | Disabled by default; explicit confirmation and build allowlist |
| R3 | Destructive, security-reducing or poorly understood change | Not implemented |

### 6.3 Typed action contract

Rust must not return arbitrary shell text. It returns a typed action request, for example:

```rust
pub enum SystemOperation {
    ReadNoRestrictPackages,
    AddPackageToNoRestrict {
        package_name: ValidatedPackageName,
    },
    InspectGreezerCapability,
    InspectFcmTransport,
    ReadAppPolicy {
        package_name: ValidatedPackageName,
    },
}
```

Kotlin maps each variant to a fixed, reviewed implementation. User input never becomes a shell fragment. If an implementation must call a shell interface, arguments are individually validated and passed without shell concatenation.

---

## 7. Rust and Kotlin architecture

### 7.1 Rust responsibilities

- Pure policy evaluation.
- Device capability model and compatibility rules.
- Parsing normalized diagnostic output.
- Comparing desired state with observed state.
- Generating typed action plans.
- Automation rule evaluation.
- Audit event normalization.
- Unit and property-based tests.

Rust does **not** own Android lifecycle, Binder connections, Shizuku permission prompts, WorkManager scheduling, or UI.

### 7.2 Kotlin responsibilities

- Jetpack Compose UI and Android lifecycle.
- Shizuku API and UserService lifecycle.
- Public Android API access.
- Fixed system-operation adapters.
- WorkManager scheduling and boot/update recovery.
- Database access/migrations, or a clearly defined storage adapter.
- UniFFI/JNI bridge and coroutine orchestration.

### 7.3 Core crates

```text
rust/
â”œâ”€â”€ nexus-domain/       # Core types, states, IDs, risk classes
â”œâ”€â”€ nexus-policy/       # Desired-vs-observed state evaluation
â”œâ”€â”€ nexus-oem-xiaomi/   # Xiaomi build profile and capability adapters
â”œâ”€â”€ nexus-fcm/          # FCM diagnostic interpretation
â”œâ”€â”€ nexus-automation/  # Deterministic rule evaluation
â”œâ”€â”€ nexus-recovery/     # Recovery plans and rollback logic
â”œâ”€â”€ nexus-audit/        # Structured audit events
â”œâ”€â”€ nexus-storage/      # Persistence interfaces/models
â””â”€â”€ nexus-bridge/       # UniFFI exported API
```

Start with fewer crates if build complexity becomes a burden. `nexus-domain`, `nexus-policy`, `nexus-oem-xiaomi`, `nexus-bridge` are sufficient for the first vertical slice.

### 7.4 Android modules

```text
android/app/src/main/kotlin/eu/omijia/nexus/
â”œâ”€â”€ app/
â”œâ”€â”€ ui/
â”‚   â”œâ”€â”€ dashboard/
â”‚   â”œâ”€â”€ diagnostics/
â”‚   â”œâ”€â”€ protection/
â”‚   â”œâ”€â”€ apps/
â”‚   â”œâ”€â”€ automation/
â”‚   â””â”€â”€ settings/
â”œâ”€â”€ privilege/
â”‚   â”œâ”€â”€ PrivilegeBackend.kt
â”‚   â”œâ”€â”€ PublicAndroidBackend.kt
â”‚   â”œâ”€â”€ ShizukuBackend.kt
â”‚   â””â”€â”€ ManualActionBackend.kt
â”œâ”€â”€ shizuku/
â”‚   â”œâ”€â”€ ShizukuManager.kt
â”‚   â””â”€â”€ NexusUserService.kt
â”œâ”€â”€ system/
â”‚   â”œâ”€â”€ DeviceProfileDetector.kt
â”‚   â”œâ”€â”€ XiaomiSystemAdapter.kt
â”‚   â””â”€â”€ OperationRegistry.kt
â”œâ”€â”€ bridge/
â”œâ”€â”€ workers/
â”œâ”€â”€ receivers/
â”œâ”€â”€ data/
â””â”€â”€ security/
```

### 7.5 Runtime flow

```text
Compose UI
  â†’ ViewModel
    â†’ Kotlin UseCase
      â†’ Rust Policy Engine (read/evaluate)
        â†’ typed ActionPlan
      â†’ Kotlin Permission/Capability Validator
        â†’ user confirmation if write
        â†’ Shizuku UserService / public Android API
      â†’ read-back verification
      â†’ Rust Audit/Policy update
    â†’ UI state
```

---

## 8. FCM and background scheduling policy

### 8.1 Do not promise exact periodic execution

WorkManager periodic work has a minimum interval of 15 minutes and is inexact. Doze, constraints and OEM scheduling may delay execution. It must not be used as a real-time heartbeat or a guarantee that the app will repair a state at a precise second.

### 8.2 Scheduling hierarchy

1. Event-driven observation where a documented and validated event is available.
2. One-time work after boot, app update, Shizuku reconnect or a user-requested Apply action.
3. Conservative periodic health check as a fallback.
4. Manual diagnostics on demand.

Avoid frequent wakeups. Track battery cost and false-positive rate. If the observer is unsupported, show that limitation instead of silently increasing polling frequency.

### 8.3 FCM reconnect policy

- Treat reconnect/heartbeat broadcasts as experimental compatibility behavior.
- Enable only after validating on the specific GMS version/build.
- Debounce repeated requests.
- Do not send reconnect on every polling cycle.
- Never treat â€œbroadcast sentâ€ as â€œFCM reconnectedâ€.
- Verify by observing a later connection state where possible.
- If state is unavailable, report `UNKNOWN`, not `HEALTHY`.

---

## 9. Data model and audit

### 9.1 Core states

```rust
pub enum ObservedState {
    Enabled,
    Disabled,
    Present,
    Missing,
    Restricted,
    Unrestricted,
    Unsupported,
    PermissionDenied,
    Unknown,
    Error,
}

pub enum OperationOutcome {
    AppliedAndVerified,
    AppliedButUnverified,
    NoChangeRequired,
    Unsupported,
    PermissionDenied,
    Failed,
    RolledBack,
}
```

### 9.2 Audit event fields

- `event_id`
- `timestamp_utc`
- `device_build_fingerprint_hash` (or local build identifier; avoid unnecessary device identifiers)
- `module_id`
- `operation_id`
- `risk_level`
- `state_before`
- `requested_change`
- `state_after`
- `verification_result`
- `rollback_available`
- `error_code`

Logs remain local. Redact package names when exporting diagnostics unless the user explicitly includes them.

### 9.3 Rollback model

Before an R1/R2 mutation:
1. Read current value.
2. Parse and validate it.
3. Save a local snapshot with build fingerprint and timestamp.
4. Compute a minimal delta.
5. Show the intended change.
6. Execute only after approval.
7. Read back and compare.
8. If verification fails, attempt rollback only if the saved state is still applicable.
9. If the build changed or state is ambiguous, stop and ask for manual review.

---

## 10. Security and privacy threat model

| Threat | Mitigation |
|---|---|
| Malformed shell argument | Typed operations; no shell string concatenation |
| Excessive Shizuku privilege use | Capability checks and least privilege |
| Vendor API changed after OTA | Build allowlist, runtime probes, fail closed |
| Incorrect list rewrite removes other packages | Parse, preserve, minimal delta, read-back |
| App reports false success | Separate execution from verification |
| Log leaks package/user information | Local-only logs, redaction, explicit export |
| Background loop drains battery | Event-driven first; bounded periodic fallback |
| App is removed while exemptions remain | Cleanup workflow and uninstall checklist |
| Rollback restores stale policy | Build fingerprint/version guard |
| Supply-chain dependency risk | Pin versions, audit dependencies, SBOM/release provenance |

No reverse-engineering step should rely on bypassing authentication, exploiting vulnerabilities, or modifying Xiaomi system binaries. The research target is observable behavior and documented/accessible interfaces under the user's authorized ADB-shell context.

---

## 11. Clean-room and open-source licensing plan

Proposed NEXUS license: Apache-2.0 **only if** all NEXUS implementation code is independently written and dependencies are compatible.

Reference-project notes:
- Noti Keeper README states GPL-3.0 plus an attribution condition requiring derivative products to remain under the same license and display attribution. Do not copy its source into an Apache-2.0 project without a proper legal review and license decision.
- NotiGuard repository is marked MIT; its README identifies third-party FCM Guard code and includes `THIRD_PARTY_NOTICES.md`. Preserve required notices if any permitted code is reused.
- HyperOS FCM Fix: inspect the repository's actual LICENSE and notices at the exact commit before reuse. Do not infer its license from the README.
- Shizuku and UniFFI have their own licenses; verify versions and transitive dependencies before release.

Preferred approach:
1. Read public documentation and observe behavior on a test device.
2. Write NEXUS requirements and interface contracts independently.
3. Implement new code from those specifications.
4. Keep a source-attribution and dependency ledger.
5. Run license scanning before each release.
6. Include `THIRD_PARTY_NOTICES.md`, SBOM and reproducible build metadata.

This is an engineering risk-control plan, not legal advice.

---

## 12. MVP acceptance criteria

### A. Device identification
- Detect manufacturer/model/build/API level.
- Confirm Xiaomi HyperOS environment using multiple signals.
- Identify unsupported or ambiguous builds without attempting writes.

### B. Privilege
- Detect whether Shizuku is installed/running/authorized.
- Query Shizuku service UID and required permission availability.
- Show missing privilege as a clear state.
- Continue read-only diagnostics where possible.

### C. GMS protection
- Read GMS package state.
- Read and parse the no-restrict list only when capability is verified.
- Preserve all existing entries.
- Add only the GMS package if absent.
- Verify by reading back.
- Record prior state and rollback information.
- If the setting format is unknown or write permission is unavailable, do not mutate.

### D. FCM diagnostics
- Report connection observation source and timestamp.
- Distinguish connected, disconnected, inaccessible and unknown.
- Do not claim end-to-end delivery based only on socket state.
- Reconnect feature remains behind an experimental flag until verified.

### E. Reliability
- No crash if Xiaomi service is absent.
- No write without permission and user consent.
- No false `Protected` state after unverified execution.
- Recovery after app process restart and device reboot is tested.
- No network telemetry or notification content collection.

### F. Battery
- No persistent high-frequency polling in MVP.
- Record worker execution duration and count.
- Compare idle battery impact against baseline during test sessions.

---

## 13. Test plan

### 13.1 Test matrix

Use at least two physical Xiaomi China ROM devices if available, and record exact build fingerprints. Do not generalize results from one phone to all HyperOS devices.

| Scenario | Expected behavior |
|---|---|
| Shizuku absent | Read-only mode; no writes |
| Shizuku unauthorized | Explain permission state |
| Unsupported Xiaomi build | Capability marked unsupported |
| GMS absent/disabled | Report; do not attempt protection |
| GMS already protected | No-op; audit `NoChangeRequired` |
| GMS missing from list | Preview minimal delta; require approval |
| List contains other apps | Preserve all unrelated entries |
| PowerKeeper rewrites list | Detect only if observer is validated; otherwise report at next check |
| Greezer interface absent | Mark unsupported; do not crash |
| FCM socket inaccessible | Report unknown, not disconnected |
| FCM socket established | Report transport observation, not end-to-end success |
| Device reboot | Re-probe capabilities and restore only approved policies |
| OTA/build fingerprint changes | Invalidate cached capability profile |
| Rollback | Restore only when snapshot matches current build/context |

### 13.2 End-to-end test application

Create a separate minimal test app under `tools/fcm-test-app/`:
- Receives a controlled test push.
- Records server send timestamp, receive callback timestamp and notification-post timestamp.
- Uses a test account and non-sensitive payload.
- Does not inspect other apps' notification content.

Compare notification latency distributions (median, p95, missed-message rate) before and after changes. Run multiple trials in each device state.

### 13.3 Test report

Every test report must include:
- Device model.
- Build fingerprint.
- Android API level.
- GMS version.
- Shizuku version and UID.
- Network type.
- Screen/idle/charging state.
- Feature toggles.
- Sample count.
- Median and p95 latency.
- Failures and limitations.

---

## 14. Phase 0 deliverables and next actions

### Deliverables
- [x] Product scope fixed: Xiaomi China ROM only.
- [x] Privilege model fixed: Shizuku + ADB-shell, no root.
- [x] Open-source intent fixed.
- [x] Reference-project behavior compared from public documentation.
- [x] Major risk areas and license constraints identified.
- [x] MVP acceptance criteria drafted.
- [ ] Inspect exact source commits and LICENSE files.
- [ ] Build a device/build compatibility inventory.
- [ ] Validate read access to `MILLET_NO_RESTRICT_APP` on a physical test device.
- [ ] Validate Shizuku operation permissions on target Android builds.
- [ ] Produce `API_SPECIFICATION.md` and a first read-only prototype.

### Immediate next implementation slice

Do not start with automatic repair. Start with a read-only prototype:

1. Create the Android project and Rust workspace.
2. Integrate Shizuku and report its state.
3. Detect Xiaomi/HyperOS build profile.
4. Read GMS package metadata.
5. Probe whether the no-restrict list can be read and parsed.
6. Display capability results and export a redacted diagnostic report.
7. Add tests for parser behavior using captured, sanitized fixtures.

Only after this slice passes on a real device should the project implement a reversible GMS list update.

---

## 15. Primary references

1. HyperOS FCM Fix â€” https://github.com/dingwen07/hyperos-fcm-fix
2. HyperOS FCM Fix releases â€” https://github.com/dingwen07/hyperos-fcm-fix/releases
3. Noti Keeper â€” https://github.com/quangtrang1111/noti-keeper
4. NotiGuard â€” https://github.com/Liuchijang/NotiGuard
5. Shizuku â€” https://github.com/RikkaApps/Shizuku
6. Shizuku API â€” https://github.com/RikkaApps/Shizuku-API
7. Android WorkManager periodic work â€” https://developer.android.com/reference/androidx/work/PeriodicWorkRequest
8. Android package visibility â€” https://developer.android.com/training/package-visibility/declaring
9. UniFFI â€” https://github.com/mozilla/uniffi-rs
10. Android NDK â€” https://developer.android.com/ndk

**Research caveat:** Repository READMEs are author-maintained descriptions, not independent verification. HyperOS internals are undocumented/unstable. Every write operation and compatibility claim must be validated against the exact device build before being enabled.
