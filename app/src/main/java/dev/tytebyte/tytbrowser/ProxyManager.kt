package dev.tytebyte.tytbrowser

import android.content.Context
import android.widget.Toast
import androidx.webkit.ProxyConfig
import androidx.webkit.ProxyController
import androidx.webkit.WebViewFeature
import dev.tytebyte.tytbrowser.core.Core
import dev.tytebyte.tytbrowser.data.Prefs
import java.util.concurrent.Executor

/**
 * Встроенный прокси: ядро Xray-core (Go) слушает 127.0.0.1 внутри приложения,
 * а весь трафик WebView направляется в него через ProxyController.
 * Если прокси включён, но не работает, прямое соединение НЕ используется (трафик не утекает).
 */
object ProxyManager {
    const val PORT = 10808
    private val direct = Executor { it.run() }

    val supported: Boolean get() = Core.loaded && dev.tytebyte.tytbrowser.core.XrayNative.loaded && WebViewFeature.isFeatureSupported(WebViewFeature.PROXY_OVERRIDE)

    fun isRunning(): Boolean = Core.proxyPort() > 0

    /** Применяет сохранённое состояние: запускает ядро и включает маршрутизацию WebView либо выключает её. */
    fun apply(ctx: Context, prefs: Prefs) {
        val link = prefs.proxyActiveLink
        if (!prefs.proxyEnabled || link.isEmpty()) {
            disable()
            return
        }
        if (!supported) {
            prefs.proxyEnabled = false
            disable()
            Toast.makeText(ctx, "Прокси недоступен: нужен свежий Android System WebView", Toast.LENGTH_LONG).show()
            return
        }
        val err = Core.proxyStart(link, PORT)
        if (err != null) {
            prefs.proxyEnabled = false
            disable()
            Toast.makeText(ctx, "Прокси не запущен: $err", Toast.LENGTH_LONG).show()
            return
        }
        val cfg = ProxyConfig.Builder().addProxyRule("127.0.0.1:$PORT").build()
        ProxyController.getInstance().setProxyOverride(cfg, direct) {}
    }

    private fun disable() {
        if (WebViewFeature.isFeatureSupported(WebViewFeature.PROXY_OVERRIDE)) {
            ProxyController.getInstance().clearProxyOverride(direct) {}
        }
        Core.proxyStop()
    }

    fun shutdown() = Core.proxyStop()
}
