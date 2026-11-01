package eu.omijia.nexus

object NativeBridge {
    init {
        System.loadLibrary("nexus_bridge")
    }
    external fun nativeGreet(): String
    external fun nativeEvaluate(gms: Int, fcm: Int, shizuku: Int, deviceSupported: Int): String
    external fun nativeReport(gms: Int, fcm: Int, shizuku: Int, deviceSupported: Int): String
    external fun nativeGmsDiagnose(freeze: Int, protection: Int): String
    external fun nativeFcmDiagnose(observed: Int): String
    external fun nativeRecoveryPlan(reason: Int): String
}