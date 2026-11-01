package eu.omijia.nexus.shizuku

import android.content.Context
import android.content.pm.PackageManager
import android.util.Log

class ShizukuManager(private val ctx: Context) {
    private val tag = "ShizukuManager"

    fun isInstalled(): Boolean =
        try {
            ctx.packageManager.getPackageInfo("moe.shizuku.privileged.api", 0)
            true
        } catch (e: PackageManager.NameNotFoundException) {
            false
        }

    fun pingBinder(): Boolean {
        return try {
            val clazz = Class.forName("rikka.shizuku.Shizuku")
            val method = clazz.getMethod("pingBinder")
            method.invoke(null) as Boolean
        } catch (e: Exception) {
            Log.w(tag, "pingBinder failed: ${e.message}")
            false
        }
    }

    fun isAvailable(): Boolean = isInstalled() && pingBinder()

    fun statusLabel(): String = when {
        !isInstalled() -> "Shizuku not installed"
        !pingBinder() -> "Shizuku installed but not running"
        else -> "Shizuku available (ADB-shell identity)"
    }
}