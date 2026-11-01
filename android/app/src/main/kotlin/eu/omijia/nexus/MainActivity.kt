package eu.omijia.nexus

import android.app.Activity
import android.os.Bundle
import android.widget.TextView

class MainActivity : Activity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val tv = TextView(this)
        val sb = StringBuilder()
        sb.append("NEXUS Notify\n\n")
        sb.append("Greet: ").append(safe { NativeBridge.nativeGreet() }).append("\n\n")
        // Scenario: GMS not protected, FCM unknown, no Shizuku, device supported
        sb.append("Evaluate (GMS=notprotected, FCM=unknown, shizuku=no, dev=yes):\n")
        sb.append(safe { NativeBridge.nativeEvaluate(1, 3, 0, 1) }).append("\n\n")
        sb.append("Report:\n")
        sb.append(safe { NativeBridge.nativeReport(1, 3, 0, 1) }).append("\n\n")
        // Scenario: healthy
        sb.append("Evaluate (GMS=protected, FCM=supported, shizuku=yes, dev=yes):\n")
        sb.append(safe { NativeBridge.nativeEvaluate(0, 0, 1, 1) }).append("\n")
        tv.text = sb.toString()
        tv.textSize = 13f
        setContentView(tv)
    }

    private fun safe(block: () -> String): String =
        try { block() } catch (e: Exception) { "ERR: ${e.message}" }
}