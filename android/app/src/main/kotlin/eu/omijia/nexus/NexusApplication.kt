package eu.omijia.nexus

import android.app.Application

class NexusApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        // TODO: init Rust bridge, Shizuku
    }
}