package dev.tytebyte.tytbrowser

import android.webkit.WebView
import dev.tytebyte.tytbrowser.core.START_URL
import java.util.concurrent.atomic.AtomicInteger

/** Вкладка. WebView создаётся лениво и может быть выгружен для экономии памяти (адрес при этом сохраняется). */
class Tab(val id: Long, val priv: Boolean) {
    var url: String = START_URL
    var title: String = ""
    var web: WebView? = null
    var home: Boolean = true
    var progress: Int = 0
    var loading: Boolean = false
    var parentId: Long = -1L
    var lastUsed: Long = System.currentTimeMillis()

    /** Адрес основной страницы для блокировщика (читается из потока загрузки ресурсов). */
    @Volatile
    var pageUrl: String = ""
    val blocked = AtomicInteger(0)

    val displayTitle: String
        get() = when {
            title.isNotBlank() -> title
            home -> if (priv) "Приватная вкладка" else "Быстрый доступ"
            else -> url
        }
}
