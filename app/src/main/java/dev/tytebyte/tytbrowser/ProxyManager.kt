package dev.tytebyte.tytbrowser

import android.content.Context
import android.widget.Toast
import androidx.webkit.ProxyConfig
import androidx.webkit.ProxyController
import androidx.webkit.WebViewFeature
import dev.tytebyte.tytbrowser.core.Core
import dev.tytebyte.tytbrowser.data.Prefs
import dev.tytebyte.tytbrowser.data.RouteRules
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

    /** Ссылки звеньев в порядке «вход → выход»: цепочка либо один выбранный профиль. */
    fun activeLinks(prefs: Prefs): String {
        val chain = prefs.proxyChain.lines().map(String::trim).filter(String::isNotEmpty)
        return if (prefs.proxyUseChain && chain.isNotEmpty()) chain.joinToString("\n") else prefs.proxyActiveLink
    }

    /** Применяет сохранённое состояние: запускает ядро и включает маршрутизацию WebView либо выключает её. */
    fun apply(ctx: Context, prefs: Prefs) {
        val links = activeLinks(prefs)
        if (!prefs.proxyEnabled || links.isEmpty()) {
            disable()
            return
        }
        if (!supported) {
            prefs.proxyEnabled = false
            disable()
            Toast.makeText(ctx, "Прокси недоступен: нужен свежий Android System WebView", Toast.LENGTH_LONG).show()
            return
        }
        val rules = RouteRules.coreText(RouteRules.decode(prefs.proxyRules))
        val need = Core.proxyNeedGeo(rules)
        val missing = (need and 1 != 0 && !GeoData.has(ctx, "geoip.dat")) || (need and 2 != 0 && !GeoData.has(ctx, "geosite.dat"))
        if (missing) {
            prefs.proxyEnabled = false
            disable()
            Toast.makeText(ctx, "Правила используют geoip/geosite: скачайте базы в «Правила маршрутизации»", Toast.LENGTH_LONG).show()
            return
        }
        val err = Core.proxyStart(links, rules, if (prefs.proxyDefaultDirect) "direct" else "proxy", PORT, GeoData.dir(ctx).absolutePath)
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
