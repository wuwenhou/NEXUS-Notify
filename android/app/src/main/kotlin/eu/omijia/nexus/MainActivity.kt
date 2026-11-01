package eu.omijia.nexus

import android.os.Bundle
import android.widget.TextView
import android.app.Activity

class MainActivity : Activity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val tv = TextView(this)
        val greet = try { NativeBridge.nativeGreet() } catch (e: Exception) { "ERR: ${e.message}" }
        val demo = try { NativeBridge.nativeRunDemo() } catch (e: Exception) { "ERR: ${e.message}" }
        tv.text = "NEXUS Notify\n$greet\n$demo"
        tv.textSize = 16f
        setContentView(tv)
    }
}