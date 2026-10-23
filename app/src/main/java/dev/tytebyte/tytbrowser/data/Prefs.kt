package dev.tytebyte.tytbrowser.data

import android.content.Context
import android.content.SharedPreferences

/** Настройки браузера (SharedPreferences). Значения по умолчанию соответствуют макету Aura Lite. */
class Prefs(ctx: Context) {
    private val sp: SharedPreferences = ctx.getSharedPreferences("tytbrowser", Context.MODE_PRIVATE)

    /** 0 — светлая, 1 — тёмная, 2 — AMOLED. */
    var theme: Int
        get() = sp.getInt("theme", 1)
        set(v) = sp.edit().putInt("theme", v).apply()

    /** duckduckgo | google | yandex | bing */
    var searchEngine: String
        get() = sp.getString("engine", "duckduckgo") ?: "duckduckgo"
        set(v) = sp.edit().putString("engine", v).apply()

    var addressBarBottom: Boolean
        get() = sp.getBoolean("bar_bottom", false)
        set(v) = sp.edit().putBoolean("bar_bottom", v).apply()

    var blockAds: Boolean
        get() = sp.getBoolean("block_ads", true)
        set(v) = sp.edit().putBoolean("block_ads", v).apply()

    var blockTrackers: Boolean
        get() = sp.getBoolean("block_trackers", true)
        set(v) = sp.edit().putBoolean("block_trackers", v).apply()

    var blockImages: Boolean
        get() = sp.getBoolean("block_images", false)
        set(v) = sp.edit().putBoolean("block_images", v).apply()

    var javaScript: Boolean
        get() = sp.getBoolean("js", true)
        set(v) = sp.edit().putBoolean("js", v).apply()

    var desktopMode: Boolean
        get() = sp.getBoolean("desktop", false)
        set(v) = sp.edit().putBoolean("desktop", v).apply()

    var darkPages: Boolean
        get() = sp.getBoolean("dark_pages", false)
        set(v) = sp.edit().putBoolean("dark_pages", v).apply()

    var clearOnExit: Boolean
        get() = sp.getBoolean("clear_exit", false)
        set(v) = sp.edit().putBoolean("clear_exit", v).apply()

    var blockThirdPartyCookies: Boolean
        get() = sp.getBoolean("block_3p_cookies", true)
        set(v) = sp.edit().putBoolean("block_3p_cookies", v).apply()

    var stripTracking: Boolean
        get() = sp.getBoolean("strip_tracking", true)
        set(v) = sp.edit().putBoolean("strip_tracking", v).apply()

    /** Размер шрифта страниц в процентах (textZoom). */
    var textZoom: Int
        get() = sp.getInt("text_zoom", 100)
        set(v) = sp.edit().putInt("text_zoom", v).apply()

    var readerFontPx: Int
        get() = sp.getInt("reader_px", 17)
        set(v) = sp.edit().putInt("reader_px", v).apply()

    /** 0 — светлая бумага, 1 — сепия, 2 — тёмная, 3 — чёрная. */
    var readerTheme: Int
        get() = sp.getInt("reader_theme", -1)
        set(v) = sp.edit().putInt("reader_theme", v).apply()

    var readerSerif: Boolean
        get() = sp.getBoolean("reader_serif", true)
        set(v) = sp.edit().putBoolean("reader_serif", v).apply()

    /** Открытые вкладки между запусками: URL-ы через \n. */
    var savedTabs: String
        get() = sp.getString("tabs", "") ?: ""
        set(v) = sp.edit().putString("tabs", v).apply()

    var savedActive: Int
        get() = sp.getInt("tabs_active", 0)
        set(v) = sp.edit().putInt("tabs_active", v).apply()

    var blockedTotal: Long
        get() = sp.getLong("blocked_total", 0)
        set(v) = sp.edit().putLong("blocked_total", v).apply()

    /** Встроенный прокси: включён ли, ссылка выбранного профиля и все сохранённые ссылки (по одной на строку). */
    var proxyEnabled: Boolean
        get() = sp.getBoolean("proxy_on", false)
        set(v) = sp.edit().putBoolean("proxy_on", v).apply()

    var proxyActiveLink: String
        get() = sp.getString("proxy_active", "") ?: ""
        set(v) = sp.edit().putString("proxy_active", v).apply()

    var proxyLinks: String
        get() = sp.getString("proxy_links", "") ?: ""
        set(v) = sp.edit().putString("proxy_links", v).apply()

    /** Цепочка прокси: ссылки звеньев по одной в строке (от входа к выходу) и признак «использовать цепочку». */
    var proxyChain: String
        get() = sp.getString("proxy_chain", "") ?: ""
        set(v) = sp.edit().putString("proxy_chain", v).apply()

    var proxyUseChain: Boolean
        get() = sp.getBoolean("proxy_use_chain", false)
        set(v) = sp.edit().putBoolean("proxy_use_chain", v).apply()

    /** Правила маршрутизации (JSON, см. RouteRules) и действие для трафика, не подошедшего ни под одно правило. */
    var proxyRules: String
        get() = sp.getString("proxy_rules", "[]") ?: "[]"
        set(v) = sp.edit().putString("proxy_rules", v).apply()

    var proxyDefaultDirect: Boolean
        get() = sp.getBoolean("proxy_default_direct", false)
        set(v) = sp.edit().putBoolean("proxy_default_direct", v).apply()

    fun reset() {
        val keep = sp.getString("tabs", "")
        val links = proxyLinks
        val chain = proxyChain
        val rules = proxyRules
        sp.edit().clear().putString("tabs", keep).putString("proxy_links", links)
            .putString("proxy_chain", chain).putString("proxy_rules", rules).apply()
    }
}
