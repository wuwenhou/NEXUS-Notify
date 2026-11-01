# Security Model

## Principles
- No root, no bootloader unlock, no firmware modification.
- No Accessibility service.
- No collection of notification content, tokens, or credentials.
- No analytics/telemetry by default.
- Diagnostic mode is the default; mutations require confirmation.

## Input safety
- No raw shell strings from untrusted input.
- All operations are a typed enum (SystemAction).
- Package names validated before use.

## State safety
- `Unknown` state never becomes a mutation.
- Every mutation verified by read-back.
- Rollback data stored in audit log where prior state is known.

## Privilege
- Shizuku provides ADB-shell identity, not root.
- Capabilities probed at runtime and reported as SUPPORTED/UNSUPPORTED/PERMISSION_DENIED/UNKNOWN/FAILED.