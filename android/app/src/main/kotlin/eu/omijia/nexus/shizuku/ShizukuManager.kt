package eu.omijia.nexus.shizuku

import android.content.Context
import android.content.pm.PackageManager

class ShizukuManager(private val ctx: Context) {
    fun isShizukuInstalled(): Boolean =
        try {
            ctx.packageManager.getPackageInfo("moe.shizuku.privileged.api", 0)
            true
        } catch (e: PackageManager.NameNotFoundException) {
            false
        }

    fun statusLabel(): String =
        if (isShizukuInstalled()) "Shizuku installed (runtime check pending)" else "Shizuku not installed"
}