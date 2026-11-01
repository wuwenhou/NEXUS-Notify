package eu.omijia.nexus.system

/**
 * SystemAdapter executes typed operations via Shizuku's ADB-shell identity.
 * No raw shell strings. Operations are allowlisted by the Rust PolicyEngine.
 */
class SystemAdapter {
    fun isShizukuAvailable(): Boolean = false // TODO: bind to Shizuku.pingBinder()
    fun hasShellPermission(): Boolean = false   // TODO: check via Shizuku
    fun execute(operation: String): String = "NOT_IMPLEMENTED: $operation"
}