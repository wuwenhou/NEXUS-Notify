package eu.omijia.nexus

object NativeBridge {
    init {
        System.loadLibrary("nexus_bridge")
    }
    external fun nativeGreet(): String
    external fun nativeRunDemo(): String
}