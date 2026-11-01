# Shizuku Integration

NEXUS uses Shizuku's ADB-shell identity. No root.

## Flow

1. App checks `Shizuku.pingBinder()`.
2. If not running, prompt user to start Shizuku.
3. Request permission via `Shizuku.requestPermission()`.
4. On grant, bind UserService (ADB shell).
5. UserService executes typed operations only.

## Safety

- Never assemble shell strings from user input.
- Operations are an allowlisted enum (`SystemAction`).
- Package names validated before use.
- Every mutation verified by read-back.
- Rollback data stored in audit log.

## Capabilities by Android version

ADB-shell permissions vary by Android release. Each operation must be capability-probed at runtime and reported as `SUPPORTED / UNSUPPORTED / PERMISSION_DENIED / UNKNOWN / FAILED`.