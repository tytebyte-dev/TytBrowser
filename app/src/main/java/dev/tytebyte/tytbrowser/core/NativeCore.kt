package dev.tytebyte.tytbrowser.core

import org.json.JSONObject

/** Тонкая JNI-обёртка над Rust-библиотекой libtytcore.so. Сигнатуры соответствуют core/src/jni_api.rs. */
object NativeCore {
    @JvmStatic external fun version(): String
    @JvmStatic external fun resolveInput(input: String, engine: String): String
    @JvmStatic external fun cleanUrl(url: String): String
    @JvmStatic external fun blockerConfigure(ads: Boolean, trackers: Boolean): Int
    @JvmStatic external fun blockerAddRules(category: Int, text: String): Int
    @JvmStatic external fun blockerCheck(url: String, pageUrl: String): Int
    @JvmStatic external fun extractReadable(html: String, baseUrl: String): String
    @JvmStatic external fun suggest(query: String, entries: String, limit: Int, now: Long): String
}

enum class InputKind { INTERNAL, URL, SEARCH }

data class Resolved(val kind: InputKind, val url: String, val query: String)

enum class Verdict { ALLOW, ADS, TRACKERS }

const val START_URL = "tyt://start"

/**
 * Безопасный фасад: если нативная библиотека почему-то не загрузилась (редкая ABI, повреждённый APK),
 * браузер продолжает работать на упрощённых запасных реализациях, а не падает.
 */
object Core {
    val loaded: Boolean = try {
        System.loadLibrary("tytcore")
        true
    } catch (_: Throwable) {
        false
    }

    fun version(): String = if (loaded) safe("?") { NativeCore.version() } else "kotlin-fallback"

    fun resolve(input: String, engine: String): Resolved {
        if (loaded) {
            val raw = safe(null) { NativeCore.resolveInput(input, engine) }
            if (raw != null) {
                try {
                    val o = JSONObject(raw)
                    val kind = when (o.optString("kind")) {
                        "url" -> InputKind.URL
                        "search" -> InputKind.SEARCH
                        else -> InputKind.INTERNAL
                    }
                    return Resolved(kind, o.optString("url", START_URL), o.optString("query"))
                } catch (_: Throwable) {
                }
            }
        }
        return fallbackResolve(input, engine)
    }

    private fun fallbackResolve(input: String, engine: String): Resolved {
        val s = input.trim()
        if (s.isEmpty() || s == "start") return Resolved(InputKind.INTERNAL, START_URL, "")
        if ((s.startsWith("https://") || s.startsWith("http://")) && !s.contains(' ')) {
            return Resolved(InputKind.URL, s, "")
        }
        if (!s.contains(' ') && s.contains('.') && !s.contains("://") && !s.contains(':')) {
            return Resolved(InputKind.URL, "https://$s", "")
        }
        val q = java.net.URLEncoder.encode(s, "UTF-8")
        val base = when (engine) {
            "google" -> "https://www.google.com/search?q="
            "yandex" -> "https://yandex.ru/search/?text="
            "bing" -> "https://www.bing.com/search?q="
            else -> "https://duckduckgo.com/?q="
        }
        return Resolved(InputKind.SEARCH, base + q, s)
    }

    fun cleanUrl(url: String): String = if (loaded) safe(url) { NativeCore.cleanUrl(url) } else url

    fun configureBlocker(ads: Boolean, trackers: Boolean): Int =
        if (loaded) safe(0) { NativeCore.blockerConfigure(ads, trackers) } else 0

    fun addRules(trackers: Boolean, text: String): Int =
        if (loaded) safe(0) { NativeCore.blockerAddRules(if (trackers) 1 else 0, text) } else 0

    fun check(url: String, pageUrl: String): Verdict {
        if (!loaded) return Verdict.ALLOW
        return when (safe(0) { NativeCore.blockerCheck(url, pageUrl) }) {
            1 -> Verdict.ADS
            2 -> Verdict.TRACKERS
            else -> Verdict.ALLOW
        }
    }

    /** JSON статьи или null, если на странице нет читаемого текста. */
    fun extractReadable(html: String, baseUrl: String): JSONObject? {
        if (!loaded) return null
        val raw = safe(null) { NativeCore.extractReadable(html, baseUrl) } ?: return null
        return try {
            JSONObject(raw).takeIf { it.optBoolean("ok") }
        } catch (_: Throwable) {
            null
        }
    }

    fun suggest(query: String, entries: String, limit: Int): List<Pair<String, String>> {
        if (!loaded) return emptyList()
        val raw = safe(null) { NativeCore.suggest(query, entries, limit, System.currentTimeMillis() / 1000) } ?: return emptyList()
        return try {
            val a = org.json.JSONArray(raw)
            (0 until a.length()).map { a.getJSONObject(it).let { o -> o.optString("title") to o.optString("url") } }
        } catch (_: Throwable) {
            emptyList()
        }
    }

    private inline fun <T> safe(default: T, block: () -> T): T = try {
        block()
    } catch (_: Throwable) {
        default
    }
}
