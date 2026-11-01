package eu.omijia.nexus

import android.app.Activity
import android.os.Bundle
import android.widget.TextView
import eu.omijia.nexus.shizuku.ShizukuManager

class MainActivity : Activity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val tv = TextView(this)
        val sb = StringBuilder()
        sb.append("NEXUS Notify\n\n")
        sb.append("Greet: ").append(safe { NativeBridge.nativeGreet() }).append("\n\n")
        sb.append("GMS Diagnose (frozen, notprotected):\n")
        sb.append(safe { NativeBridge.nativeGmsDiagnose(1, 1) }).append("\n\n")
        sb.append("Evaluate (GMS=notprotected, FCM=unknown, shizuku=no, dev=yes):\n")
        sb.append(safe { NativeBridge.nativeEvaluate(1, 3, 0, 1) }).append("\n\n")
        sb.append("Report:\n")
        sb.append(safe { NativeBridge.nativeReport(1, 3, 0, 1) }).append("\n\n")
        sb.append("FCM Diagnose (unknown):\n")
        sb.append(safe { NativeBridge.nativeFcmDiagnose(3) }).append("\n\n")
        sb.append("Recovery (boot):\n")
        sb.append(safe { NativeBridge.nativeRecoveryPlan(0) }).append("\n\n")
        sb.append("Evaluate (healthy scenario):\n")
        sb.append(safe { NativeBridge.nativeEvaluate(0, 0, 1, 1) }).append("\n")
        sb.append("\nShizuku: ").append(ShizukuManager(this).statusLabel())
        tv.text = sb.toString()
        tv.textSize = 12f
        setContentView(tv)
    }

    private fun safe(block: () -> String): String =
        try { block() } catch (e: Exception) { "ERR: ${e.message}" }
}